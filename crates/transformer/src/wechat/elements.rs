//! How elements and their properties appear in WXML and in the view data.
//!
//! Data keys of a scope, by IR node id `N`:
//!
//! | Key        | Value                                                  |
//! | ---------- | ------------------------------------------------------ |
//! | `n{N}_{i}` | property `i` of an element                             |
//! | `e{N}_{k}` | handler id for event `k` of an element (`data-e<type>`) |
//! | `i{N}`     | index of the active `if` branch, `-1` for none         |
//! | `b{N}`     | data of the active `if` branch, or `null`              |
//! | `l{N}`     | rows of a `for` loop; each row has its scope id in `k` |
//! | `c{N}`     | data of a component instance                           |
//! | `y{N}`     | data of slot content; its template name is in `t`      |

use std::fmt;

use crate::elements::{Css, CssUnit, Element};
use crate::ir::{ComponentId, NodeId, Prop, PropValue};
use crate::view::{Constant, ViewLayout, ViewNodeKind};

/// How a property value reaches the view.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Target {
    Text,
    /// String attribute.
    Attr(&'static str),
    /// Boolean attribute; WXML treats every non-empty string as true, so
    /// values are always written as expressions.
    Flag(&'static str),
    Style(&'static str, CssUnit),
}

pub(crate) fn target(element: Element, prop: &str) -> Target {
    match (element, prop) {
        (Element::Text, "value") => Target::Text,
        (Element::Input, "value") => Target::Attr("value"),
        (Element::Checkbox, "checked") => Target::Flag("checked"),
        (_, "disabled") => Target::Flag("disabled"),
        (_, "placeholder") => Target::Attr("placeholder"),
        (_, "src") => Target::Attr("src"),
        (element, prop) => match element.spec().prop(prop).and_then(|spec| spec.css) {
            Some(Css { property, unit }) => Target::Style(property, unit),
            None => unreachable!("`{}` has no property `{prop}`", element.name()),
        },
    }
}

/// The mini program event type bound to an element's handler.
pub(crate) fn event_type(element: Element, event: &str) -> &'static str {
    match (element, event) {
        (Element::Input, "onChange") => "input",
        (Element::Checkbox, "onChange") => "change",
        (Element::Button, "onClick") => "tap",
        _ => unreachable!("`{}` has no handler `{event}`", element.name()),
    }
}

/// Data key of event `index` on element `node`.
pub(crate) fn event_key(node: NodeId, index: usize) -> impl fmt::Display {
    fmt::from_fn(move |f| write!(f, "e{}_{index}", node.0))
}

/// Data key of property `index` of element `node`.
pub(crate) fn value_key(node: NodeId, index: usize) -> String {
    format!("n{}_{index}", node.0)
}

/// Where the view reads a property from.
pub(crate) enum Value {
    /// Constant written into the markup.
    Inline(String),
    /// Key in the scope's data.
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

/// Markup text of a constant, or `None` if it must be sent as data.
pub(crate) fn inline(target: Target, constant: &Constant) -> Option<String> {
    match (target, constant) {
        (Target::Text | Target::Attr(_), Constant::String(s)) => markup_safe(s).then(|| s.clone()),
        (Target::Style(_, CssUnit::Color), Constant::Color(color)) => Some(color.to_string()),
        (Target::Flag(_), Constant::Bool(b)) => Some(b.to_string()),
        (Target::Style(_, CssUnit::Px), Constant::Int(v)) => Some(format!("{v}px")),
        (Target::Style(_, CssUnit::Bold), Constant::Bool(b)) => {
            Some(if *b { "bold" } else { "normal" }.to_owned())
        }
        (target, _) => unreachable!("constant of the wrong type for {target:?}"),
    }
}

/// How property `index` of element node `node` appears in the view; `None`
/// for handlers.
pub(crate) fn prop_value(
    layout: &ViewLayout,
    component: ComponentId,
    element: Element,
    node: NodeId,
    index: usize,
    prop: &Prop,
) -> Option<Value> {
    if let PropValue::Handler(_) = prop.value {
        return None;
    }
    let key = value_key(node, index);
    if layout.value_slot(component, node, index).is_some() {
        return Some(Value::Data(key));
    }
    let (template, local) = layout.location(component, node);
    let ViewNodeKind::Element { props, .. } = &layout.template(template).nodes[local as usize].kind
    else {
        unreachable!("node {} is an element", node.0);
    };
    let constant = &props
        .iter()
        .find(|p| p.name == prop.name)
        .expect("properties without a value slot are static")
        .value;
    Some(match inline(target(element, prop.name), constant) {
        Some(text) => Value::Inline(text),
        None => Value::Data(key),
    })
}
