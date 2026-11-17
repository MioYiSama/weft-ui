//! Remote mode: one create function per component for a view layer that only
//! renders data (currently the WeChat mini program, `weft-ui-codegen-wechat`).
//!
//! Every block (component body, `if` branch, `for` row, slot content) owns a
//! plain data object, and the view (the mini program's WXML) reads
//! precomputed values from it. Update functions
//! re-run only the bindings whose dirty bits are set and store values that
//! changed, which the runtime turns into `setData` paths. Event handlers and
//! keys are evaluated in the logic layer when they fire or when rows are
//! aligned, so they never travel to the view.
//!
//! Data keys of a block, by node id `N`:
//!
//! | Key        | Value                                                  |
//! | ---------- | ------------------------------------------------------ |
//! | `n{N}_{i}` | property `i` of an element                             |
//! | `e{N}`     | handler id of an element (`data-h`)                    |
//! | `i{N}`     | index of the active `if` branch, `-1` for none         |
//! | `b{N}`     | data of the active `if` branch, or `null`              |
//! | `l{N}`     | rows of a `for` loop; each row has its key in `k`      |
//! | `c{N}`     | data of a component instance                           |
//! | `y{N}`     | data of slot content; its template name is in `t`      |

use weft_ui_compiler::ir::{
    Component, ComponentId, DepRoot, Element, NodeId, NodeKind, Program, Prop, PropValue,
};

use crate::blocks::{
    Consts, Ctx, Literal, Parts, arg_updates, arg_values, binding_cond, branch_index, handler_body,
    instance_watches, lifecycle, lifecycle_handlers, literal, root_shape, walk_cond, words,
};
use crate::expr::{Env, expr};
use crate::js::{self, Code};

/* ---- element mapping shared with the view generator ---------------------- */

/// How a property value reaches the view.
#[derive(Debug, Clone, Copy)]
pub enum Target {
    Text,
    /// String attribute.
    Attr(&'static str),
    /// Boolean attribute; WXML treats every non-empty string as true, so
    /// values are always written as expressions.
    Flag(&'static str),
    Style(&'static str, Unit),
}

/// Conversion of a style value to CSS, done in the logic layer.
#[derive(Debug, Clone, Copy)]
pub enum Unit {
    Raw,
    Px,
    Bold,
}

pub fn target(element: Element, prop: &str) -> Target {
    match (element, prop) {
        (Element::Text, "value") => Target::Text,
        (Element::Input, "value") => Target::Attr("value"),
        (Element::Checkbox, "checked") => Target::Flag("checked"),
        (_, "disabled") => Target::Flag("disabled"),
        (_, "placeholder") => Target::Attr("placeholder"),
        (_, "src") => Target::Attr("src"),
        (_, "padding") => Target::Style("padding", Unit::Px),
        (_, "gap") => Target::Style("gap", Unit::Px),
        (_, "width") => Target::Style("width", Unit::Px),
        (_, "height") => Target::Style("height", Unit::Px),
        (_, "size") => Target::Style("font-size", Unit::Px),
        (_, "background") => Target::Style("background", Unit::Raw),
        (_, "color") => Target::Style("color", Unit::Raw),
        (_, "bold") => Target::Style("font-weight", Unit::Bold),
        (element, prop) => unreachable!("`{}` has no property `{prop}`", element.name()),
    }
}

/// Values the user edits in the view; the logic layer restores them to the
/// bound value when the host does not accept an edit.
fn is_controlled(element: Element, prop: &str) -> bool {
    matches!(
        (element, prop),
        (Element::Input, "value") | (Element::Checkbox, "checked")
    )
}

/// Where the view reads a property from.
pub enum Value {
    /// Constant written into the markup.
    Inline(String),
    /// Key in the block's data.
    Data(String),
}

/// Whether a string can appear verbatim in WXML text or attribute values.
/// Anything WXML could interpret (`{{`, entities, quotes) or trim is sent as
/// data instead.
fn markup_safe(s: &str) -> bool {
    s.trim() == s
        && !s
            .chars()
            .any(|c| matches!(c, '<' | '>' | '&' | '"' | '\'' | '{' | '}') || c.is_control())
}

/// How property `index` of element node `node` appears in the view; `None`
/// for handlers.
pub fn prop_value(element: Element, node: NodeId, index: usize, prop: &Prop) -> Option<Value> {
    let PropValue::Expr(value) = &prop.value else {
        return None;
    };
    let target = target(element, prop.name);
    let inline = match (prop.binding, literal(value)) {
        _ if is_controlled(element, prop.name) => None,
        (Some(_), _) | (None, None) => None,
        (None, Some(literal)) => match (target, literal) {
            (Target::Text | Target::Attr(_) | Target::Style(_, Unit::Raw), Literal::Str(s)) => {
                markup_safe(s).then(|| s.to_owned())
            }
            (Target::Flag(_), Literal::Bool(b)) => Some(b.to_string()),
            (Target::Style(_, Unit::Px), Literal::Int(v)) => Some(format!("{v}px")),
            (Target::Style(_, Unit::Bold), Literal::Bool(b)) => {
                Some(if b { "bold" } else { "normal" }.to_owned())
            }
            (target, _) => unreachable!("literal of the wrong type for {target:?}"),
        },
    };
    Some(match inline {
        Some(text) => Value::Inline(text),
        None => Value::Data(format!("n{}_{index}", node.0)),
    })
}

/// The WXML event attribute of a handler property.
pub fn event_attr(element: Element, prop: &str) -> &'static str {
    match (element, prop) {
        (Element::Input, "onChange") => "bindinput",
        (Element::Checkbox, "onChange") => "bindchange",
        (_, "onClick") => "bindtap",
        (element, name) => unreachable!("`{}` has no handler `{name}`", element.name()),
    }
}

/// The handler property of an element, excluding lifecycle events.
pub fn handler_prop(props: &[Prop]) -> Option<&Prop> {
    props.iter().find(|p| {
        matches!(p.value, PropValue::Handler(_)) && !matches!(p.name, "onMount" | "onUnmount")
    })
}

/// The event that reports user edits of a controlled element without a handler.
pub fn controlled_event(element: Element) -> Option<&'static str> {
    match element {
        Element::Input => Some("bindinput"),
        Element::Checkbox => Some("bindchange"),
        _ => None,
    }
}

/// Template rendering the children a component call passes to a slot.
pub fn slot_template(comp: &Component, node: NodeId) -> String {
    format!("{}-s{}", comp.name, node.0)
}

/* ---- create functions ---------------------------------------------------- */

/// Emits `export function {Name}$c(app, args, children, depth)`.
pub(crate) fn component(program: &Program, id: ComponentId, consts: &mut Consts, out: &mut Code) {
    let comp = program.component(id);
    out.open(format!(
        "export function {}$c(app, args, children, depth) {{",
        comp.name
    ));
    out.line(format!(
        "const c = new $.Instance(app, {}, depth, args, children);",
        words(comp.slots.len())
    ));
    instance_watches(program, comp, consts, out);
    let mut generator = Gen {
        program,
        comp,
        consts,
    };
    generator.block(&comp.roots, &Ctx::default(), "c", out);
    out.line("return c;");
    out.close("}");
}

/// JS value stored for a property.
fn data_value(target: Target, value: &str) -> String {
    match target {
        Target::Style(_, Unit::Px) => format!("$.px({value})"),
        Target::Style(_, Unit::Bold) => format!("({value} ? \"bold\" : \"normal\")"),
        _ => value.to_owned(),
    }
}

struct Gen<'a> {
    program: &'a Program,
    comp: &'a Component,
    consts: &'a mut Consts,
}

impl<'a> Gen<'a> {
    fn node_kind(&self, id: NodeId) -> &'a NodeKind {
        &self.comp.node(id).kind
    }

    /// Builds the nodes `roots` into the block object `var` (already declared).
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

    fn node(&mut self, id: NodeId, ctx: &Ctx, var: &str, parts: &mut Parts) {
        match self.node_kind(id) {
            NodeKind::Element { .. } => self.element(id, ctx, var, parts),
            NodeKind::If { .. } => self.if_node(id, ctx, var, parts),
            NodeKind::For { .. } => self.for_node(id, ctx, var, parts),
            NodeKind::Component { .. } => self.component_node(id, ctx, var, parts),
            NodeKind::Slot { .. } => {
                let y = format!("y{}", id.0);
                parts.create.line(format!(
                    "const {y} = c.children ? c.children.create() : null;"
                ));
                parts
                    .create
                    .line(format!("if ({y}) {y}.mount({var}, \"{y}\");"));
                parts.cleanup.line(format!("if ({y}) {y}.destroy();"));
            }
        }
    }

    fn element(&mut self, id: NodeId, ctx: &Ctx, var: &str, parts: &mut Parts) {
        let NodeKind::Element {
            element,
            props,
            children,
        } = self.node_kind(id)
        else {
            unreachable!("caller checked the node kind");
        };
        let element = *element;
        let mut controlled = None;
        for (index, prop) in props.iter().enumerate() {
            let Some(Value::Data(key)) = prop_value(element, id, index, prop) else {
                continue;
            };
            let PropValue::Expr(value) = &prop.value else {
                unreachable!("only expressions have a view value");
            };
            let value = data_value(target(element, prop.name), &expr(value, Env::default()));
            parts.create.line(format!("{var}.o.{key} = {value};"));
            if let Some(cond) = binding_cond(self.comp, prop.binding) {
                parts
                    .update
                    .line(format!("if ({cond}) {var}.set(\"{key}\", {value});"));
            }
            if is_controlled(element, prop.name) {
                controlled = Some(key);
            }
        }
        let handler = handler_prop(props).map(|prop| {
            let PropValue::Handler(handler) = &prop.value else {
                unreachable!("handler_prop only returns handlers");
            };
            let param = if prop.name == "onClick" { "" } else { "$a" };
            format!("({param}) => {{ {} }}", handler_body(self.program, handler))
        });
        if handler.is_some() || controlled.is_some() {
            parts.create.line(format!(
                "{var}.on(\"e{}\", {}, {});",
                id.0,
                handler.as_deref().unwrap_or("null"),
                controlled
                    .as_deref()
                    .map_or_else(|| "null".to_owned(), js::string)
            ));
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

        let mut create = Code::default();
        create.open(format!(
            "const {name} = new $.Branches({var}, \"{name}\", \"b{}\", (i) => {{",
            id.0
        ));
        create.open("switch (i) {");
        let mut all_roots = Vec::new();
        for (i, branch) in branches.iter().enumerate() {
            create.open(format!("case {i}: {{"));
            create.line("const b = new $.Block();");
            self.block(&branch.children, ctx, "b", &mut create);
            create.line("return b;");
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

        parts.create.open(format!(
            "const {name} = new $.List({var}, \"{name}\", (item) => {{"
        ));
        parts.create.line(format!(
            "const {row} = new $.Row({words}, {}.depth + 1, {shape}, item);",
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

        let slot = if children.is_empty() {
            "null".to_owned()
        } else {
            let slot = format!("s{}", id.0);
            parts.create.open(format!(
                "const {slot} = new $.Slot({}, () => {{",
                js::string(&slot_template(self.comp, id))
            ));
            parts.create.line("const b = new $.Block();");
            let mut body = Code::default();
            self.block(children, ctx, "b", &mut body);
            parts.create.append(body);
            parts.create.line("return b;");
            parts.create.close("});");
            slot
        };

        parts.create.line(format!(
            "const {name} = {}$c(c.app, [{}], {slot}, {}.depth + 1);",
            callee.name,
            arg_values(callee, args).join(", "),
            ctx.scope()
        ));
        parts
            .create
            .line(format!("{name}.mount({var}, \"c{}\");", id.0));

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
