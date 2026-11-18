//! Remote mode: one create function per component that drives the scopes of
//! the [view protocol](crate::view) for a separate view layer on WeChat and
//! the native platforms.
//!
//! Every block (component body, `if` branch, `for` row, slot content) is a
//! scope of its layout template. Create code writes every value slot once;
//! update functions re-run only the bindings whose dirty bits are set and
//! write values that changed, which the runtime (`@weft-ui/runtime/remote`)
//! turns into ops for its sink. Event handlers and keys are evaluated in the
//! logic layer when they fire or when rows are aligned, so they never travel
//! to the view: the view only reports which event slot of which scope fired.

use crate::ir::{Component, ComponentId, DepRoot, NodeId, NodeKind, Program, Prop, PropValue};
use crate::view::{TemplateId, ViewLayout};

use super::blocks::{
    Consts, Ctx, Parts, arg_updates, arg_values, binding_cond, branch_index, handler_body,
    instance_watches, lifecycle, lifecycle_handlers, root_shape, walk_cond, words,
};
use super::expr::{Env, expr};
use super::js::Code;

/// Emits `export function {Name}$c(app, args, children, depth)`.
pub(crate) fn component(
    program: &Program,
    layout: &ViewLayout,
    id: ComponentId,
    consts: &mut Consts,
    out: &mut Code,
) {
    let comp = program.component(id);
    out.open(format!(
        "export function {}$c(app, args, children, depth) {{",
        comp.name
    ));
    out.line(format!(
        "const c = new $.Instance(app, {}, {}, depth, args, children);",
        ViewLayout::body(id).0,
        words(comp.slots.len())
    ));
    instance_watches(program, comp, consts, out);
    let mut generator = Gen {
        program,
        layout,
        id,
        comp,
        consts,
    };
    generator.block(&comp.roots, &Ctx::default(), "c", out);
    out.line("return c;");
    out.close("}");
}

/// The handler property of an element, excluding lifecycle events.
fn handler_prop(props: &[Prop]) -> Option<&Prop> {
    props.iter().find(|p| {
        matches!(p.value, PropValue::Handler(_)) && !matches!(p.name, "onMount" | "onUnmount")
    })
}

struct Gen<'a> {
    program: &'a Program,
    layout: &'a ViewLayout,
    id: ComponentId,
    comp: &'a Component,
    consts: &'a mut Consts,
}

impl<'a> Gen<'a> {
    fn node_kind(&self, id: NodeId) -> &'a NodeKind {
        &self.comp.node(id).kind
    }

    /// Builds the nodes `roots` into the scope `var` (already declared).
    fn block(&mut self, roots: &[NodeId], ctx: &Ctx, var: &str, out: &mut Code) {
        let mut parts = Parts::default();
        for &root in roots {
            self.node(root, ctx, var, &mut parts);
        }
        out.append(parts.create);
        if !parts.update.is_empty() {
            out.open(format!("{var}.u = () => {{"));
            out.append(parts.update);
            out.close("};");
        }
        if !parts.cleanup.is_empty() {
            out.open(format!("{var}.x = () => {{"));
            out.append(parts.cleanup);
            out.close("};");
        }
    }

    /// Declares the scope `b` of `template` for a branch or slot content.
    fn block_scope(&mut self, template: TemplateId, roots: &[NodeId], ctx: &Ctx, out: &mut Code) {
        out.line(format!("const b = new $.Block(c.app, {});", template.0));
        self.block(roots, ctx, "b", out);
        out.line("return b;");
    }

    fn node(&mut self, id: NodeId, ctx: &Ctx, var: &str, parts: &mut Parts) {
        match self.node_kind(id) {
            NodeKind::Element { .. } => self.element(id, ctx, var, parts),
            NodeKind::If { .. } => self.if_node(id, ctx, var, parts),
            NodeKind::For { .. } => self.for_node(id, ctx, var, parts),
            NodeKind::Component { .. } => self.component_node(id, ctx, var, parts),
            NodeKind::Slot { .. } => {
                let point = self.layout.point(self.id, id);
                let y = format!("y{}", id.0);
                parts.create.line(format!(
                    "const {y} = c.children ? c.children.create() : null;"
                ));
                parts
                    .create
                    .line(format!("if ({y}) {var}.attach({point}, {y});"));
                parts.cleanup.line(format!("if ({y}) {y}.destroy();"));
            }
        }
    }

    fn element(&mut self, id: NodeId, ctx: &Ctx, var: &str, parts: &mut Parts) {
        let NodeKind::Element {
            props, children, ..
        } = self.node_kind(id)
        else {
            unreachable!("caller checked the node kind");
        };
        for (index, prop) in props.iter().enumerate() {
            let Some(slot) = self.layout.value_slot(self.id, id, index) else {
                continue;
            };
            let PropValue::Expr(value) = &prop.value else {
                unreachable!("only expressions have value slots");
            };
            let value = expr(value, Env::default());
            parts.create.line(format!("{var}.set({slot}, {value});"));
            if let Some(cond) = binding_cond(self.comp, prop.binding) {
                parts
                    .update
                    .line(format!("if ({cond}) {var}.set({slot}, {value});"));
            }
        }
        if let Some(event) = self.layout.event_slot(self.id, id) {
            let template = self.layout.template(self.layout.location(self.id, id).0);
            let controls = template.events[event as usize]
                .controls
                .map_or_else(|| "-1".to_owned(), |slot| slot.to_string());
            let handler = handler_prop(props).map_or_else(
                || "null".to_owned(),
                |prop| {
                    let PropValue::Handler(handler) = &prop.value else {
                        unreachable!("handler_prop only returns handlers");
                    };
                    let param = if prop.name == "onClick" { "" } else { "$a" };
                    format!("({param}) => {{ {} }}", handler_body(self.program, handler))
                },
            );
            parts
                .create
                .line(format!("{var}.on({event}, {handler}, {controls});"));
        }
        for &child in children {
            self.node(child, ctx, var, parts);
        }
        lifecycle(self.program, lifecycle_handlers(props), parts);
    }

    fn if_node(&mut self, id: NodeId, ctx: &Ctx, var: &str, parts: &mut Parts) {
        let NodeKind::If { branches, binding } = self.node_kind(id) else {
            unreachable!("caller checked the node kind");
        };
        let name = format!("i{}", id.0);
        let index = branch_index(branches);
        let point = self.layout.point(self.id, id);
        let templates = self.layout.child_templates(self.id, id);

        let mut create = Code::default();
        create.open(format!(
            "const {name} = new $.Branches({var}, {point}, (i) => {{"
        ));
        create.open("switch (i) {");
        let mut all_roots = Vec::new();
        for (i, (branch, &template)) in branches.iter().zip(templates).enumerate() {
            create.open(format!("case {i}: {{"));
            self.block_scope(template, &branch.children, ctx, &mut create);
            create.close("}");
            all_roots.extend_from_slice(&branch.children);
        }
        create.line("default:");
        create.line("  return null;");
        create.close("}");
        create.close("});");
        create.line(format!("{name}.set({index});"));
        parts.create.append(create);

        if let Some(cond) = binding_cond(self.comp, *binding) {
            parts
                .update
                .line(format!("if ({cond}) {name}.set({index});"));
        }
        if let Some(cond) = walk_cond(self.comp, &all_roots, ctx) {
            parts.update.line(format!("if ({cond}) {name}.update();"));
        }
        parts.cleanup.line(format!("{name}.destroy();"));
    }

    fn for_node(&mut self, id: NodeId, ctx: &Ctx, var: &str, parts: &mut Parts) {
        let NodeKind::For {
            loop_id,
            list,
            key,
            children,
            list_binding,
            ..
        } = self.node_kind(id)
        else {
            unreachable!("caller checked the node kind");
        };
        let loop_id = *loop_id;
        let (name, row) = (format!("l{}", id.0), format!("r{}", loop_id.0));
        let list = expr(list, Env::default());
        let key = expr(
            key,
            Env {
                key_loop: Some(loop_id),
            },
        );
        let shape = root_shape(self.program, self.comp, self.consts, DepRoot::Item(loop_id));
        let words = words(self.comp.loops[loop_id.index()].slots.len());
        let point = self.layout.point(self.id, id);
        let [template] = self.layout.child_templates(self.id, id) else {
            unreachable!("a loop has one row template");
        };

        parts.create.open(format!(
            "const {name} = new $.List({var}, {point}, (item) => {{"
        ));
        parts.create.line(format!(
            "const {row} = new $.Row(c.app, {}, {words}, {}.depth + 1, {shape}, item);",
            template.0,
            ctx.scope()
        ));
        let mut body = Code::default();
        self.block(children, &ctx.with_loop(loop_id), &row, &mut body);
        parts.create.append(body);
        parts.create.line(format!("return {row};"));
        parts.create.close(format!("}}, ($it) => {key});"));
        parts.create.line(format!("{name}.set({list});"));

        if let Some(cond) = binding_cond(self.comp, *list_binding) {
            parts
                .update
                .line(format!("if ({cond}) {name}.set({list});"));
        }
        if let Some(cond) = walk_cond(self.comp, children, ctx) {
            parts.update.line(format!("if ({cond}) {name}.update();"));
        }
        parts.cleanup.line(format!("{name}.destroy();"));
    }

    fn component_node(&mut self, id: NodeId, ctx: &Ctx, var: &str, parts: &mut Parts) {
        let NodeKind::Component {
            component,
            args,
            props,
            children,
        } = self.node_kind(id)
        else {
            unreachable!("caller checked the node kind");
        };
        let callee = self.program.component(*component);
        let name = format!("k{}", id.0);
        let point = self.layout.point(self.id, id);

        let slot = match self.layout.child_templates(self.id, id) {
            [] => "null".to_owned(),
            [template] => {
                let slot = format!("s{}", id.0);
                parts
                    .create
                    .open(format!("const {slot} = new $.Slot(() => {{"));
                let mut body = Code::default();
                self.block_scope(*template, children, ctx, &mut body);
                parts.create.append(body);
                parts.create.close("});");
                slot
            }
            _ => unreachable!("a component call has at most one slot content"),
        };

        parts.create.line(format!(
            "const {name} = {}$c(c.app, [{}], {slot}, {}.depth + 1);",
            callee.name,
            arg_values(callee, args).join(", "),
            ctx.scope()
        ));
        parts.create.line(format!("{var}.attach({point}, {name});"));

        arg_updates(self.comp, &name, args, &mut parts.update);
        if !children.is_empty()
            && let Some(cond) = walk_cond(self.comp, children, ctx)
        {
            parts
                .update
                .line(format!("if ({cond}) s{}.update();", id.0));
        }
        parts.cleanup.line(format!("{name}.destroy();"));
        lifecycle(self.program, lifecycle_handlers(props), parts);
    }
}
