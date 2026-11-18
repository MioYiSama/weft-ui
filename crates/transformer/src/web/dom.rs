//! DOM create functions: one per component.
//!
//! Every block (component body, `if` branch, `for` row, slot content) gets an
//! update function that re-runs only the bindings whose dirty bits are set
//! and walks into nested blocks only when bits they depend on are set.

use crate::bridge::DomView;
use crate::bridge::blocks::{
    Consts, Ctx, Literal, Parts, arg_updates, arg_values, binding_cond, branch_index, handler_body,
    instance_watches, lifecycle, lifecycle_handlers, literal, root_shape, walk_cond, words,
};
use crate::bridge::expr::{Env, expr};
use crate::bridge::js::{self, Code};
use crate::color::Color;
use crate::ir::{
    Component, ComponentId, DepRoot, Element, NodeId, NodeKind, Program, Prop, PropValue,
};

/// The DOM [`DomView`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct Dom;

impl DomView for Dom {
    fn component(&self, program: &Program, id: ComponentId, consts: &mut Consts, out: &mut Code) {
        component(program, id, consts, out);
    }
}

/// Emits `export function {Name}$c(app, args, children, depth)`.
fn component(program: &Program, id: ComponentId, consts: &mut Consts, out: &mut Code) {
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
    let mut body = Code::default();
    generator.block(&comp.roots, &Ctx::default(), "c", &mut body);
    out.append(body);
    out.line("return c;");
    out.close("}");
}

/// How a property value reaches the DOM.
#[derive(Debug, Clone, Copy)]
enum Target {
    Text,
    Value,
    Checked,
    Disabled,
    Attr(&'static str),
    Px(&'static str),
    Color(&'static str),
    Bold,
}

fn target(element: Element, prop: &str) -> Target {
    match (element, prop) {
        (Element::Text, "value") => Target::Text,
        (Element::Input, "value") => Target::Value,
        (Element::Checkbox, "checked") => Target::Checked,
        (_, "disabled") => Target::Disabled,
        (_, "placeholder") => Target::Attr("placeholder"),
        (_, "src") => Target::Attr("src"),
        (_, "padding") => Target::Px("padding"),
        (_, "gap") => Target::Px("gap"),
        (_, "width") => Target::Px("width"),
        (_, "height") => Target::Px("height"),
        (_, "size") => Target::Px("font-size"),
        (_, "background") => Target::Color("background"),
        (_, "color") => Target::Color("color"),
        (_, "bold") => Target::Bold,
        (element, prop) => unreachable!("`{}` has no property `{prop}`", element.name()),
    }
}

/// Tag, fixed attributes and base style of an element.
fn tag(element: Element) -> (&'static str, &'static str, &'static str) {
    match element {
        Element::Column => (
            "div",
            "",
            "display:flex;flex-direction:column;align-items:flex-start",
        ),
        Element::Row => (
            "div",
            "",
            "display:flex;flex-direction:row;align-items:flex-start",
        ),
        Element::Text => ("span", "", ""),
        Element::Button => ("button", " type=\"button\"", ""),
        Element::Input => ("input", "", ""),
        Element::Checkbox => ("input", " type=\"checkbox\"", ""),
        Element::Image => ("img", "", ""),
    }
}

fn is_void(element: Element) -> bool {
    matches!(element, Element::Input | Element::Checkbox | Element::Image)
}

/// Whether a property is written into the template instead of by code.
/// Controlled values are always set by code so the runtime knows them.
fn is_template_prop(element: Element, prop: &Prop) -> bool {
    let PropValue::Expr(value) = &prop.value else {
        return false;
    };
    prop.binding.is_none()
        && literal(value).is_some()
        && !matches!(target(element, prop.name), Target::Value | Target::Checked)
}

fn apply(target: Target, node: &str, value: &str) -> String {
    match target {
        Target::Text => format!("{node}.textContent = {value};"),
        Target::Value => format!("$.value({node}, {value});"),
        Target::Checked => format!("$.checked({node}, {value});"),
        Target::Disabled => format!("{node}.disabled = {value};"),
        Target::Attr(name) => format!("{node}.setAttribute({}, {value});", js::string(name)),
        Target::Px(name) => format!(
            "{node}.style.setProperty({}, $.px({value}));",
            js::string(name)
        ),
        Target::Color(name) => format!(
            "{node}.style.setProperty({}, $.color({value}));",
            js::string(name)
        ),
        Target::Bold => format!("{node}.style.fontWeight = {value} ? \"bold\" : \"\";"),
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

    fn is_element(&self, id: NodeId) -> bool {
        matches!(self.node_kind(id), NodeKind::Element { .. })
    }

    /// Builds the nodes `roots` into the block object `var` (already declared).
    fn block(&mut self, roots: &[NodeId], ctx: &Ctx, var: &str, out: &mut Code) {
        // The first node must be stable: dynamic content is inserted before
        // anchors, so a leading anchor gets a marker comment in front.
        let marker = roots.first().is_none_or(|&r| !self.is_element(r));
        let mut html = String::new();
        if marker {
            html.push_str("<!---->");
        }
        for &root in roots {
            self.html(root, &mut html);
        }
        let template = self
            .consts
            .template(format!("$.template({})", js::string(&html)));
        out.line(format!("const f = {template}();"));
        self.refs(roots, "f", marker.then(|| "f.firstChild".to_owned()), out);

        let mut parts = Parts::default();
        for &root in roots {
            self.node(root, ctx, &mut parts);
        }
        out.append(parts.create);
        out.line(format!("{var}.init(f);"));
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

    fn html(&self, id: NodeId, out: &mut String) {
        let NodeKind::Element {
            element,
            props,
            children,
        } = self.node_kind(id)
        else {
            out.push_str("<!---->");
            return;
        };
        let element = *element;
        let (tag, fixed, base_style) = tag(element);
        let mut style = base_style.to_owned();
        let mut attrs = String::from(fixed);
        let mut text = None;
        for prop in props.iter().filter(|p| is_template_prop(element, p)) {
            let PropValue::Expr(value) = &prop.value else {
                continue;
            };
            let Some(value) = literal(value) else {
                continue;
            };
            let mut add_style = |decl: String| {
                if !style.is_empty() {
                    style.push(';');
                }
                style.push_str(&decl);
            };
            match (target(element, prop.name), value) {
                (Target::Text, Literal::Str(s)) => text = Some(s),
                (Target::Disabled, Literal::Bool(true)) => attrs.push_str(" disabled"),
                (Target::Bold, Literal::Bool(true)) => add_style("font-weight:bold".to_owned()),
                (Target::Disabled | Target::Bold, _) => {}
                (Target::Attr(name), Literal::Str(s)) => {
                    attrs.push_str(&format!(" {name}=\""));
                    html_attr(s, &mut attrs);
                    attrs.push('"');
                }
                (Target::Px(name), Literal::Int(v)) => add_style(format!("{name}:{v}px")),
                (Target::Color(name), Literal::Str(s)) => add_style(format!(
                    "{name}:{}",
                    Color::parse(s)
                        .expect("color literals were validated during semantic analysis")
                )),
                (target, _) => unreachable!("literal of the wrong type for {target:?}"),
            }
        }
        out.push('<');
        out.push_str(tag);
        out.push_str(&attrs);
        if !style.is_empty() {
            out.push_str(" style=\"");
            html_attr(&style, out);
            out.push('"');
        }
        out.push('>');
        if is_void(element) {
            return;
        }
        if let Some(text) = text {
            html_text(text, out);
        }
        for &child in children {
            self.html(child, out);
        }
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
    }

    /// Declares `n{id}` for every template node of `nodes` and their element
    /// children.
    fn refs(&self, nodes: &[NodeId], parent: &str, mut prev: Option<String>, out: &mut Code) {
        for &id in nodes {
            let name = format!("n{}", id.0);
            let init = match &prev {
                Some(prev) => format!("{prev}.nextSibling"),
                None => format!("{parent}.firstChild"),
            };
            out.line(format!("const {name} = {init};"));
            if let NodeKind::Element { children, .. } = self.node_kind(id)
                && !children.is_empty()
            {
                self.refs(children, &name, None, out);
            }
            prev = Some(name);
        }
    }

    fn node(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
        match self.node_kind(id) {
            NodeKind::Element { .. } => self.element(id, ctx, parts),
            NodeKind::If { .. } => self.if_node(id, ctx, parts),
            NodeKind::For { .. } => self.for_node(id, ctx, parts),
            NodeKind::Component { .. } => self.component_node(id, ctx, parts),
            NodeKind::Slot { .. } => {
                let (y, n) = (format!("y{}", id.0), format!("n{}", id.0));
                parts.create.line(format!(
                    "const {y} = c.children ? c.children.create() : null;"
                ));
                parts
                    .create
                    .line(format!("if ({y}) {y}.mount({n}.parentNode, {n});"));
                parts.cleanup.line(format!("if ({y}) {y}.destroy(false);"));
            }
        }
    }

    fn element(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
        let NodeKind::Element {
            element,
            props,
            children,
        } = self.node_kind(id)
        else {
            unreachable!("caller checked the node kind");
        };
        let element = *element;
        let n = format!("n{}", id.0);
        let mut listened = false;
        for prop in props {
            match &prop.value {
                PropValue::Expr(value) => {
                    if is_template_prop(element, prop) {
                        continue;
                    }
                    let target = target(element, prop.name);
                    let value = expr(value, Env::default());
                    parts.create.line(apply(target, &n, &value));
                    if let Some(cond) = binding_cond(self.comp, prop.binding) {
                        parts
                            .update
                            .line(format!("if ({cond}) {}", apply(target, &n, &value)));
                    }
                }
                PropValue::Handler(_) if matches!(prop.name, "onMount" | "onUnmount") => {}
                PropValue::Handler(handler) => {
                    listened = true;
                    let body = handler_body(self.program, handler);
                    parts.create.line(match (element, prop.name) {
                        (Element::Input, "onChange") => {
                            format!("$.input({n}, ($a) => {{ {body} }});")
                        }
                        (Element::Checkbox, "onChange") => {
                            format!("$.checkbox({n}, ($a) => {{ {body} }});")
                        }
                        (_, "onClick") => {
                            format!("{n}.addEventListener(\"click\", () => {{ {body} }});")
                        }
                        (element, name) => {
                            unreachable!("`{}` has no handler `{name}`", element.name())
                        }
                    });
                }
            }
        }
        // Controlled inputs revert user edits the host did not accept.
        if !listened {
            match element {
                Element::Input => parts.create.line(format!("$.input({n}, null);")),
                Element::Checkbox => parts.create.line(format!("$.checkbox({n}, null);")),
                _ => {}
            }
        }
        for &child in children {
            self.node(child, ctx, parts);
        }
        lifecycle(self.program, lifecycle_handlers(props), parts);
    }

    fn if_node(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
        let NodeKind::If { branches, binding } = self.node_kind(id) else {
            unreachable!("caller checked the node kind");
        };
        let (var, n) = (format!("i{}", id.0), format!("n{}", id.0));
        let index = branch_index(branches);

        let mut create = Code::default();
        create.open(format!("const {var} = new $.Branches({n}, (i) => {{"));
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
        create.line(format!("{var}.set({index});"));
        parts.create.append(create);

        if let Some(cond) = binding_cond(self.comp, *binding) {
            parts
                .update
                .line(format!("if ({cond}) {var}.set({index});"));
        }
        if let Some(cond) = walk_cond(self.comp, &all_roots, ctx) {
            parts.update.line(format!("if ({cond}) {var}.update();"));
        }
        parts.cleanup.line(format!("{var}.destroy();"));
    }

    fn for_node(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
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
        let (var, n, row) = (
            format!("l{}", id.0),
            format!("n{}", id.0),
            format!("r{}", loop_id.0),
        );
        let list = expr(list, Env::default());
        let key = expr(
            key,
            Env {
                key_loop: Some(loop_id),
            },
        );
        let shape = root_shape(self.program, self.comp, self.consts, DepRoot::Item(loop_id));
        let words = words(self.comp.loops[loop_id.index()].slots.len());

        parts
            .create
            .open(format!("const {var} = new $.List({n}, (item) => {{"));
        parts.create.line(format!(
            "const {row} = new $.Row({words}, {}.depth + 1, {shape}, item);",
            ctx.scope()
        ));
        let mut body = Code::default();
        self.block(children, &ctx.with_loop(loop_id), &row, &mut body);
        parts.create.append(body);
        parts.create.line(format!("return {row};"));
        parts.create.close(format!("}}, ($it) => {key});"));
        parts.create.line(format!("{var}.set({list});"));

        if let Some(cond) = binding_cond(self.comp, *list_binding) {
            parts.update.line(format!("if ({cond}) {var}.set({list});"));
        }
        if let Some(cond) = walk_cond(self.comp, children, ctx) {
            parts.update.line(format!("if ({cond}) {var}.update();"));
        }
        parts.cleanup.line(format!("{var}.destroy();"));
    }

    fn component_node(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
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
        let (var, n) = (format!("k{}", id.0), format!("n{}", id.0));

        let slot = if children.is_empty() {
            "null".to_owned()
        } else {
            let slot = format!("s{}", id.0);
            parts
                .create
                .open(format!("const {slot} = new $.Slot(() => {{"));
            parts.create.line("const b = new $.Block();");
            let mut body = Code::default();
            self.block(children, ctx, "b", &mut body);
            parts.create.append(body);
            parts.create.line("return b;");
            parts.create.close("});");
            slot
        };

        parts.create.line(format!(
            "const {var} = {}$c(c.app, [{}], {slot}, {}.depth + 1);",
            callee.name,
            arg_values(callee, args).join(", "),
            ctx.scope()
        ));
        parts
            .create
            .line(format!("{var}.mount({n}.parentNode, {n});"));

        arg_updates(self.comp, &var, args, &mut parts.update);
        if !children.is_empty()
            && let Some(cond) = walk_cond(self.comp, children, ctx)
        {
            parts
                .update
                .line(format!("if ({cond}) s{}.update();", id.0));
        }
        parts.cleanup.line(format!("{var}.destroy(false);"));
        lifecycle(self.program, lifecycle_handlers(props), parts);
    }
}

/// Escapes text content for an HTML template.
fn html_text(value: &str, out: &mut String) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
}

/// Escapes a double-quoted HTML attribute value.
fn html_attr(value: &str, out: &mut String) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            c => out.push(c),
        }
    }
}
