//! Imperative construction of a template's elements and mount points.

use weft_ui_compiler::ir::Element;
use weft_ui_compiler::view::{Constant, Template, ViewNodeKind};

use crate::cpp::{self, Code};

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
                Element::Button => match event(template, index) {
                    Some(slot) => format!("[this] {{ dispatch({slot}); }}"),
                    None => "nullptr".to_owned(),
                },
                Element::Input => {
                    let slot = controlled(template, index);
                    format!("[this](std::string_view v) {{ return dispatch({slot}, v); }}")
                }
                Element::Checkbox => {
                    let slot = controlled(template, index);
                    format!("[this](bool v) {{ return dispatch({slot}, v); }}")
                }
                Element::Text | Element::Image => String::new(),
            };
            let made = format!("make<weft::{}>({args})", wrapper(*element));
            let member = member(template, index);
            if is_field(template, index) {
                out.line(format!("n{index}_ = &{made};"));
            } else {
                out.line(format!("auto &n{index} = {made};"));
            }
            for prop in props {
                out.line(format!(
                    "{member}{}({});",
                    setter(prop.name),
                    literal(&prop.value)
                ));
            }
            out.line(format!("{region}.add({});", reference(template, index)));
            for &child in children {
                node(out, template, child, Some(index));
            }
        }
        ViewNodeKind::Point(point) => out.line(format!("{region}.point(*this, {point});")),
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

/// Event slot of an element, excluding lifecycle events handled by JS.
fn event(template: &Template, node: u32) -> Option<usize> {
    template.events.iter().position(|e| e.node == node)
}

fn controlled(template: &Template, node: u32) -> usize {
    event(template, node).expect("controlled elements have an onChange event with a value slot")
}

pub(crate) fn wrapper(element: Element) -> &'static str {
    match element {
        Element::Column | Element::Row => "Stack",
        Element::Text => "Text",
        Element::Button => "Button",
        Element::Input => "Input",
        Element::Checkbox => "Checkbox",
        Element::Image => "Image",
    }
}

/// The wrapper method writing property `name`: `set_` and the name in snake case.
pub(crate) fn setter(name: &str) -> String {
    let mut setter = String::from("set");
    for c in name.chars() {
        if c.is_ascii_uppercase() || setter.len() == 3 {
            setter.push('_');
        }
        setter.push(c.to_ascii_lowercase());
    }
    setter
}

fn literal(value: &Constant) -> String {
    match value {
        Constant::Int(v) => v.to_string(),
        Constant::Bool(v) => v.to_string(),
        Constant::String(s) => cpp::string(s),
        Constant::Color(c) => format!(
            "weft::rgba(0x{:02x}{:02x}{:02x}{:02x}u)",
            c.r, c.g, c.b, c.a
        ),
    }
}
