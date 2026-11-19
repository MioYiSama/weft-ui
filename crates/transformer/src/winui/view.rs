//! Imperative construction of a template's elements and mount points.

use crate::ir::Element;
use crate::view::{Constant, Template, ViewNodeKind};

use super::csharp::{self, Code};

/// Constructs the template's roots in pre-order.
pub(crate) fn body(out: &mut Code, template: &Template) {
    for &root in &template.roots {
        node(out, template, root, None);
    }
}

fn node(out: &mut Code, template: &Template, index: u32, parent: Option<u32>) {
    let region = match parent {
        Some(parent) => format!("n{parent}.Children"),
        None => "Roots".to_owned(),
    };
    match &template.nodes[index as usize].kind {
        ViewNodeKind::Element {
            element,
            props,
            children,
        } => {
            let declaration = if template.values.iter().any(|v| v.node == index) {
                ""
            } else {
                "var "
            };
            let args = match element {
                Element::Column => "vertical: true".to_owned(),
                Element::Row => "vertical: false".to_owned(),
                Element::Button => match template.event(index) {
                    Some((slot, _)) => format!("() => Dispatch({slot})"),
                    None => "null".to_owned(),
                },
                Element::Input | Element::Checkbox => {
                    let (slot, _) = template.controlled(index);
                    format!("v => Dispatch({slot}, WeftValue.Of(v))")
                }
                Element::Text | Element::Image => String::new(),
            };
            out.writer.line(format_args!(
                "{declaration}n{index} = new {}({args});",
                wrapper(*element)
            ));
            for prop in props {
                out.writer.line(format_args!(
                    "n{index}.{} = {};",
                    property(prop.name),
                    literal(&prop.value)
                ));
            }
            out.writer
                .line(format_args!("{region}.Add(n{index}.Element);"));
            for &child in children {
                node(out, template, child, Some(index));
            }
        }
        ViewNodeKind::Point(point) => out
            .writer
            .line(format_args!("{region}.Point(this, {point});")),
    }
}

pub(crate) fn wrapper(element: Element) -> &'static str {
    match element {
        Element::Column | Element::Row => "WeftStack",
        Element::Text => "WeftText",
        Element::Button => "WeftButton",
        Element::Input => "WeftInput",
        Element::Checkbox => "WeftCheckbox",
        Element::Image => "WeftImage",
    }
}

/// All built-in property names are ASCII lower camel case.
pub(crate) fn property(name: &str) -> String {
    let mut property = name.to_owned();
    property[..1].make_ascii_uppercase();
    property
}

fn literal(value: &Constant) -> String {
    match value {
        Constant::Int(v) => v.to_string(),
        Constant::Bool(v) => v.to_string(),
        Constant::String(s) => csharp::string(s),
        Constant::Color(c) => format!(
            "WeftColor.Rgba(0x{:02x}{:02x}{:02x}{:02x}u)",
            c.r, c.g, c.b, c.a
        ),
    }
}
