//! DOM create functions: one per component.
//!
//! Every block (component body, `if` branch, `for` row, slot content) gets an
//! update function that re-runs only the bindings whose dirty bits are set
//! and walks into nested blocks only when bits they depend on are set.

use std::fmt::{self, Display, Write};

use crate::bridge::DomView;
use crate::bridge::blocks::{
    Consts, Ctx, Literal, Parts, binding_cond, element_handler_body, instance_watches, literal,
    words,
};
use crate::bridge::expr::{Env, expr};
use crate::bridge::flow::{Backend, ComponentIr, Flow};
use crate::bridge::js::{self, Code};
use crate::color::Color;
use crate::elements::{Css, CssUnit, Element};
use crate::ir::{Component, ComponentId, NodeId, NodeKind, Program, Prop, PropValue};
use crate::style::{COLUMN_STYLE, ROW_STYLE};

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
    Flow::new(program, comp, consts, Gen).block(&comp.roots, &Ctx::default(), "c", out);
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
    Style(&'static str, CssUnit),
}

fn target(element: Element, prop: &str) -> Target {
    match (element, prop) {
        (Element::Text, "value") => Target::Text,
        (Element::Input, "value") => Target::Value,
        (Element::Checkbox, "checked") => Target::Checked,
        (_, "disabled") => Target::Disabled,
        (_, "placeholder") => Target::Attr("placeholder"),
        (_, "src") => Target::Attr("src"),
        (element, prop) => match element.spec().prop(prop).and_then(|spec| spec.css) {
            Some(Css { property, unit }) => Target::Style(property, unit),
            None => unreachable!("`{}` has no property `{prop}`", element.name()),
        },
    }
}

/// Tag, fixed attributes and base style of an element.
fn tag(element: Element) -> (&'static str, &'static str, &'static str) {
    match element {
        Element::Column => ("div", "", COLUMN_STYLE),
        Element::Row => ("div", "", ROW_STYLE),
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
        Target::Style(name, CssUnit::Px) => format!(
            "{node}.style.setProperty({}, $.px({value}));",
            js::string(name)
        ),
        Target::Style(name, CssUnit::Color) => format!(
            "{node}.style.setProperty({}, $.color({value}));",
            js::string(name)
        ),
        Target::Style(_, CssUnit::Bold) => {
            format!("{node}.style.fontWeight = {value} ? \"bold\" : \"\";")
        }
    }
}

struct Gen;

impl Gen {
    fn html(comp: &Component, id: NodeId, out: &mut String) {
        let NodeKind::Element {
            element,
            props,
            children,
        } = &comp.node(id).kind
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
            let mut add_style = |decl: std::fmt::Arguments<'_>| {
                if !style.is_empty() {
                    style.push(';');
                }
                style
                    .write_fmt(decl)
                    .expect("writing to String is infallible");
            };
            match (target(element, prop.name), value) {
                (Target::Text, Literal::Str(s)) => text = Some(s),
                (Target::Disabled, Literal::Bool(true)) => attrs.push_str(" disabled"),
                (Target::Style(name, CssUnit::Bold), Literal::Bool(true)) => {
                    add_style(format_args!("{name}:bold"));
                }
                (Target::Disabled | Target::Style(_, CssUnit::Bold), _) => {}
                (Target::Attr(name), Literal::Str(s)) => {
                    attrs.push_str(&format!(" {name}=\""));
                    html_attr(s, &mut attrs);
                    attrs.push('"');
                }
                (Target::Style(name, CssUnit::Px), Literal::Int(v)) => {
                    add_style(format_args!("{name}:{v}px"));
                }
                (Target::Style(name, CssUnit::Color), Literal::Str(s)) => add_style(format_args!(
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
            Self::html(comp, child, out);
        }
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
    }

    /// Declares `n{id}` for every template node of `nodes` and their element
    /// children.
    fn refs(
        comp: &Component,
        nodes: &[NodeId],
        parent: &str,
        mut prev: Option<String>,
        out: &mut Code,
    ) {
        for &id in nodes {
            let name = format!("n{}", id.0);
            match &prev {
                Some(prev) => out.line(format!("const {name} = {prev}.nextSibling;")),
                None => out.line(format!("const {name} = {parent}.firstChild;")),
            }
            if let NodeKind::Element { children, .. } = &comp.node(id).kind
                && !children.is_empty()
            {
                Self::refs(comp, children, &name, None, out);
            }
            prev = Some(name);
        }
    }
}

impl Backend for Gen {
    fn start_block(&self, comp: &Component, roots: &[NodeId], consts: &mut Consts, out: &mut Code) {
        // The first node must be stable: dynamic content is inserted before
        // anchors, so a leading anchor gets a marker comment in front.
        let marker = roots
            .first()
            .is_none_or(|&r| !matches!(comp.node(r).kind, NodeKind::Element { .. }));
        let mut html = String::new();
        if marker {
            html.push_str("<!---->");
        }
        for &root in roots {
            Self::html(comp, root, &mut html);
        }
        let template = consts.template(format!("$.template({})", js::string(&html)));
        out.line(format!("const f = {template}();"));
        Self::refs(
            comp,
            roots,
            "f",
            marker.then(|| "f.firstChild".to_owned()),
            out,
        );
    }

    fn init_block(&self, var: &str, out: &mut Code) {
        out.line(format!("{var}.init(f);"));
    }

    fn element<'parent>(
        &self,
        ir: &ComponentIr<'_>,
        id: NodeId,
        parent: &'parent str,
        element: Element,
        props: &[Prop],
        parts: &mut Parts,
    ) -> &'parent str {
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
                    if let Some(cond) = binding_cond(ir.comp, prop.binding) {
                        parts
                            .update
                            .line(format!("if ({cond}) {}", apply(target, &n, &value)));
                    }
                }
                PropValue::Handler(_) if matches!(prop.name, "onMount" | "onUnmount") => {}
                PropValue::Handler(handler) => {
                    listened = true;
                    let body = element_handler_body(ir.program, ir.comp, id, prop, handler, parts);
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
        // The element DOM is in the template; dynamic children use their own anchors.
        parent
    }

    fn anchor<'parent>(&self, id: NodeId, _parent: &'parent str) -> impl Display + 'parent {
        fmt::from_fn(move |f| write!(f, "n{}", id.0))
    }

    fn block_decl(&self, _id: NodeId, _index: usize, out: &mut Code) {
        out.line("const b = new $.Block();");
    }

    fn row_args(&self, _id: NodeId) -> impl Display {
        ""
    }

    fn mount<'args>(
        &self,
        id: NodeId,
        _parent: &'args str,
        child: &'args str,
    ) -> impl Display + 'args {
        fmt::from_fn(move |f| write!(f, "{child}.mount(n{0}.parentNode, n{0});", id.0))
    }

    fn slot_destroy(&self) -> &'static str {
        "destroy(false)"
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
