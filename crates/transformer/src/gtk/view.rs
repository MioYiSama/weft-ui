//! Imperative construction of a template's elements and mount points.

use crate::elements::Element;
use crate::view::{Constant, Template, ValueType, ViewNodeKind};

use super::cpp::{self, Code};

/// Constructs the template's roots in pre-order.
pub(crate) fn body(out: &mut Code, template: &Template) {
    for &root in &template.roots {
        node(out, template, root, None);
    }
}

fn node(out: &mut Code, template: &Template, index: u32, parent: Option<u32>) {
    let region = match parent {
        Some(parent) => format!("{}children()", member(template, parent)),
        None => "roots()".to_owned(),
    };
    match &template.nodes[index as usize].kind {
        ViewNodeKind::Element {
            element,
            props,
            children,
        } => {
            let args = match element {
                Element::Column => "weft::Axis::Vertical".to_owned(),
                Element::Row => "weft::Axis::Horizontal".to_owned(),
                Element::Button => match template.event(index, "onClick") {
                    Some(slot) => format!("[this] {{ dispatch({slot}); }}"),
                    None => "nullptr".to_owned(),
                },
                Element::Input => {
                    let (slot, _) = template.controlled(index);
                    format!("[this](std::string_view v) {{ return dispatch({slot}, v); }}")
                }
                Element::Checkbox => {
                    let (slot, _) = template.controlled(index);
                    format!("[this](bool v) {{ return dispatch({slot}, v); }}")
                }
                Element::Scroll => props
                    .iter()
                    .find(|prop| prop.name == "direction")
                    .map_or_else(String::new, |prop| {
                        literal(*element, prop.name, &prop.value)
                    }),
                Element::Stack | Element::Spacer | Element::Text | Element::Image => String::new(),
                Element::Span => unreachable!("platform support rejects `Span` before generation"),
            };
            let made = format!("make<weft::{}>({args})", wrapper(*element));
            let member = member(template, index);
            if is_field(template, index) {
                out.writer.line(format_args!("n{index}_ = &{made};"));
            } else {
                out.writer.line(format_args!("auto &n{index} = {made};"));
            }
            for prop in props.iter().filter(|prop| prop.name != "direction") {
                out.writer.line(format_args!(
                    "{member}{}({});",
                    setter(prop.name),
                    literal(*element, prop.name, &prop.value)
                ));
            }
            out.writer.line(format_args!(
                "{region}.add({});",
                reference(template, index)
            ));
            for &child in children {
                node(out, template, child, Some(index));
            }
        }
        ViewNodeKind::Point(point) => out
            .writer
            .line(format_args!("{region}.point(*this, {point});")),
    }
}

/// Whether node `index` is kept in a field `n{index}_` for `apply`.
pub(crate) fn is_field(template: &Template, index: u32) -> bool {
    template.values.iter().any(|v| v.node == index)
}

/// The element of node `index` in the constructor: a local reference, or a field pointer.
fn reference(template: &Template, index: u32) -> String {
    if is_field(template, index) {
        format!("*n{index}_")
    } else {
        format!("n{index}")
    }
}

/// Member access on the element of node `index`, up to the member name.
fn member(template: &Template, index: u32) -> String {
    if is_field(template, index) {
        format!("n{index}_->")
    } else {
        format!("n{index}.")
    }
}

pub(crate) fn wrapper(element: Element) -> &'static str {
    match element {
        Element::Column | Element::Row => "Linear",
        Element::Stack => "Stack",
        Element::Scroll => "Scroll",
        Element::Spacer => "Spacer",
        Element::Text => "Text",
        Element::Span => unreachable!("platform support rejects `Span` before generation"),
        Element::Button => "Button",
        Element::Input => "Input",
        Element::Checkbox => "Checkbox",
        Element::Image => "Image",
    }
}

/// The wrapper method writing property `name`: `set_` and the name in snake case.
pub(crate) fn setter(name: &str) -> String {
    prefixed("set", name)
}

fn prefixed(prefix: &str, name: &str) -> String {
    let mut result = String::from(prefix);
    for (index, c) in name.chars().enumerate() {
        if index == 0 || c.is_ascii_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

fn union_type(element: Element, prop: &str) -> Option<&'static str> {
    match (element, prop) {
        (Element::Column | Element::Row | Element::Scroll, "align") => Some("CrossAlign"),
        (Element::Column | Element::Row, "justify") => Some("Justify"),
        (Element::Stack, "alignment") => Some("Alignment"),
        (Element::Scroll, "direction") => Some("Axis"),
        (_, "shadow") => Some("Shadow"),
        _ => None,
    }
}

pub(crate) fn slot_value(element: Element, prop: &str, ty: ValueType) -> String {
    if let Some(union) = union_type(element, prop) {
        return format!("weft::{}(weft::string(value))", prefixed("parse", union));
    }
    let accessor = match ty {
        ValueType::Int | ValueType::Float => "number",
        ValueType::Bool => "boolean",
        ValueType::String => "string",
        ValueType::Color => "color",
    };
    format!("weft::{accessor}(value)")
}

fn literal(element: Element, prop: &str, value: &Constant) -> String {
    match value {
        Constant::Int(v) => v.to_string(),
        Constant::Float(v) => format!("{v:?}"),
        Constant::Bool(v) => v.to_string(),
        Constant::String(s) => union_type(element, prop).map_or_else(
            || cpp::string(s),
            |union| format!("weft::{union}::{}{}", s[..1].to_ascii_uppercase(), &s[1..]),
        ),
        Constant::Token(name) => format!("weft::Color::token({})", cpp::string(name)),
        Constant::Color(c) => format!("weft::Color::rgba({})", cpp::rgba(*c)),
    }
}
