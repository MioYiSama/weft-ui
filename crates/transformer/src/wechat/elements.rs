//! How elements and their properties appear in WXML and in the view data.
//!
//! Data keys of a scope, by IR node id `N`:
//!
//! | Key        | Value                                                  |
//! | ---------- | ------------------------------------------------------ |
//! | `n{N}_{i}` | property `i` of an element                             |
//! | `e{N}`     | handler id of an element (`data-h`)                    |
//! | `i{N}`     | index of the active `if` branch, `-1` for none         |
//! | `b{N}`     | data of the active `if` branch, or `null`              |
//! | `l{N}`     | rows of a `for` loop; each row has its scope id in `k` |
//! | `c{N}`     | data of a component instance                           |
//! | `y{N}`     | data of slot content; its template name is in `t`      |

use crate::ir::{ComponentId, Element, NodeId, Prop, PropValue};
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
    Style(&'static str, Unit),
}

/// Conversion of a style value to CSS, done by the sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unit {
    Color,
    Px,
    Bold,
}

impl Unit {
    /// Code of the unit in the sink's table.
    pub(crate) fn code(self) -> u8 {
        match self {
            Unit::Color => 3,
            Unit::Px => 1,
            Unit::Bold => 2,
        }
    }
}

pub(crate) fn target(element: Element, prop: &str) -> Target {
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
        (_, "background") => Target::Style("background", Unit::Color),
        (_, "color") => Target::Style("color", Unit::Color),
        (_, "bold") => Target::Style("font-weight", Unit::Bold),
        (element, prop) => unreachable!("`{}` has no property `{prop}`", element.name()),
    }
}

/// The WXML event attribute reporting an element's event slot.
pub(crate) fn event_attr(element: Element) -> &'static str {
    match element {
        Element::Input => "bindinput",
        Element::Checkbox => "bindchange",
        Element::Button => "bindtap",
        element => unreachable!("`{}` has no events", element.name()),
    }
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
        (Target::Style(_, Unit::Color), Constant::Color(color)) => Some(color.to_string()),
        (Target::Flag(_), Constant::Bool(b)) => Some(b.to_string()),
        (Target::Style(_, Unit::Px), Constant::Int(v)) => Some(format!("{v}px")),
        (Target::Style(_, Unit::Bold), Constant::Bool(b)) => {
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
