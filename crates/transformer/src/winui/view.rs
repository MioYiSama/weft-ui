//! Imperative construction of a template's elements and mount points.

use crate::elements::Element;
use crate::view::{Constant, Template, ValueType, ViewNodeKind};

use super::csharp::{self, Code};

/// Constructs the template's roots in pre-order.
pub(crate) fn body(out: &mut Code, template: &Template, has_theme: bool) {
    for &root in &template.roots {
        node(out, template, has_theme, root, None);
    }
}

fn node(out: &mut Code, template: &Template, has_theme: bool, index: u32, parent: Option<u32>) {
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
            let mut slots = template
                .values
                .iter()
                .filter(|v| v.node == index)
                .peekable();
            let declaration = if slots.peek().is_some() { "" } else { "var " };
            let args = match element {
                Element::Column => "vertical: true".to_owned(),
                Element::Row => "vertical: false".to_owned(),
                Element::Button => match template.event(index, "onClick") {
                    Some(slot) => format!("() => Dispatch({slot})"),
                    None => "null".to_owned(),
                },
                Element::Input | Element::Checkbox => {
                    let (slot, _) = template.controlled(index);
                    format!("v => Dispatch({slot}, WeftValue.Of(v))")
                }
                Element::Scroll => props
                    .iter()
                    .find(|prop| prop.name == "direction")
                    .map_or_else(String::new, |prop| {
                        format!("direction: {}", literal(*element, prop.name, &prop.value))
                    }),
                Element::Stack | Element::Spacer | Element::Text | Element::Image => String::new(),
                Element::Span => unreachable!("platform support rejects `Span` before generation"),
            };
            out.writer.line(format_args!(
                "{declaration}n{index} = new {}({args});",
                wrapper(*element)
            ));
            // Fixed colors and apps without a token table never need repainting.
            if has_theme
                && (props.iter().any(|p| matches!(p.value, Constant::Token(_)))
                    || slots.any(|v| v.ty == ValueType::Color))
            {
                out.writer.line(format_args!("TrackColors(n{index});"));
            }
            for prop in props.iter().filter(|p| p.name != "direction") {
                out.writer.line(format_args!(
                    "n{index}.{} = {};",
                    property(prop.name),
                    literal(*element, prop.name, &prop.value)
                ));
            }
            out.writer
                .line(format_args!("{region}.Add(n{index}.Element);"));
            for &child in children {
                node(out, template, has_theme, child, Some(index));
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
        Element::Span => unreachable!("platform support rejects `Span` before generation"),
        Element::Button => "WeftButton",
        Element::Input => "WeftInput",
        Element::Checkbox => "WeftCheckbox",
        Element::Image => "WeftImage",
        Element::Stack => "WeftOverlay",
        Element::Scroll => "WeftScroll",
        Element::Spacer => "WeftSpacer",
    }
}

/// All built-in property names are ASCII lower camel case.
pub(crate) fn property(name: &str) -> String {
    let mut property = name.to_owned();
    property[..1].make_ascii_uppercase();
    property
}

pub(super) fn union(element: Element, name: &str) -> Option<&'static str> {
    match (element, name) {
        (Element::Column | Element::Row | Element::Scroll, "align") => Some("CrossAlign"),
        (Element::Column | Element::Row, "justify") => Some("Justify"),
        (Element::Stack, "alignment") => Some("Alignment"),
        (Element::Scroll, "direction") => Some("Axis"),
        (
            Element::Column
            | Element::Row
            | Element::Stack
            | Element::Scroll
            | Element::Text
            | Element::Image,
            "shadow",
        ) => Some("Shadow"),
        _ => None,
    }
}

fn literal(element: Element, name: &str, value: &Constant) -> String {
    match value {
        Constant::Int(v) => v.to_string(),
        Constant::Float(v) => format!("{v:?}d"),
        Constant::Bool(v) => v.to_string(),
        Constant::String(s) => match union(element, name) {
            Some(ty) => format!("Weft{ty}.{}", property(s)),
            None => csharp::string(s),
        },
        Constant::Token(name) => format!("WeftColor.Token({})", csharp::string(name)),
        Constant::Color(c) => format!("WeftColor.Rgba({})", csharp::rgba(*c)),
    }
}
