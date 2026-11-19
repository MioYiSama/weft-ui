//! The built-in element catalog and its accepted properties.

use crate::ir::{Element, Type};

/// Value type of an element property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropType {
    Int,
    Bool,
    String,
    Color,
}

impl PropType {
    pub(crate) fn to_type(self) -> Type {
        match self {
            PropType::Int => Type::Int,
            PropType::Bool => Type::Bool,
            PropType::String | PropType::Color => Type::String,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropKind {
    Value(PropType),
    /// Event handler; the payload is the value the element produces.
    Handler {
        payload: Option<PropType>,
    },
    /// `onMount` / `onUnmount`: accepts events only.
    Lifecycle,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PropSpec {
    pub name: &'static str,
    pub kind: PropKind,
    pub positional: bool,
    pub required: bool,
}

const fn value(name: &'static str, ty: PropType) -> PropSpec {
    PropSpec {
        name,
        kind: PropKind::Value(ty),
        positional: false,
        required: false,
    }
}

const fn main_value(name: &'static str, ty: PropType) -> PropSpec {
    PropSpec {
        name,
        kind: PropKind::Value(ty),
        positional: true,
        required: true,
    }
}

const fn handler(name: &'static str, payload: Option<PropType>) -> PropSpec {
    PropSpec {
        name,
        kind: PropKind::Handler { payload },
        positional: false,
        required: false,
    }
}

/// Lifecycle props accepted by every element and component call.
pub(crate) const LIFECYCLE: [PropSpec; 2] = [
    PropSpec {
        name: "onMount",
        kind: PropKind::Lifecycle,
        positional: false,
        required: false,
    },
    PropSpec {
        name: "onUnmount",
        kind: PropKind::Lifecycle,
        positional: false,
        required: false,
    },
];

const LAYOUT: [PropSpec; 5] = [
    value("padding", PropType::Int),
    value("gap", PropType::Int),
    value("width", PropType::Int),
    value("height", PropType::Int),
    value("background", PropType::Color),
];

pub(crate) struct ElementSpec {
    pub element: Element,
    pub name: &'static str,
    pub props: &'static [PropSpec],
    pub children: bool,
}

pub(crate) const ELEMENTS: [ElementSpec; 7] = [
    ElementSpec {
        element: Element::Column,
        name: "Column",
        props: &LAYOUT,
        children: true,
    },
    ElementSpec {
        element: Element::Row,
        name: "Row",
        props: &LAYOUT,
        children: true,
    },
    ElementSpec {
        element: Element::Text,
        name: "Text",
        props: &[
            main_value("value", PropType::String),
            value("color", PropType::Color),
            value("size", PropType::Int),
            value("bold", PropType::Bool),
        ],
        children: false,
    },
    ElementSpec {
        element: Element::Button,
        name: "Button",
        props: &[handler("onClick", None), value("disabled", PropType::Bool)],
        children: true,
    },
    ElementSpec {
        element: Element::Input,
        name: "Input",
        props: &[
            main_value("value", PropType::String),
            handler("onChange", Some(PropType::String)),
            value("placeholder", PropType::String),
            value("disabled", PropType::Bool),
        ],
        children: false,
    },
    ElementSpec {
        element: Element::Checkbox,
        name: "Checkbox",
        props: &[
            main_value("checked", PropType::Bool),
            handler("onChange", Some(PropType::Bool)),
            value("disabled", PropType::Bool),
        ],
        children: false,
    },
    ElementSpec {
        element: Element::Image,
        name: "Image",
        props: &[
            main_value("src", PropType::String),
            value("width", PropType::Int),
            value("height", PropType::Int),
        ],
        children: false,
    },
];

pub(crate) fn element(name: &str) -> Option<&'static ElementSpec> {
    ELEMENTS.iter().find(|e| e.name == name)
}

/// Declared value type of a built-in element property.
pub(crate) fn prop_type(element: Element, name: &str) -> PropType {
    ELEMENTS
        .iter()
        .find(|spec| spec.element == element)
        .and_then(|spec| spec.props.iter().find(|prop| prop.name == name))
        .and_then(|prop| match prop.kind {
            PropKind::Value(ty) => Some(ty),
            _ => None,
        })
        .expect("every element value property has a declared type")
}

impl Element {
    #[must_use]
    pub fn name(self) -> &'static str {
        ELEMENTS
            .iter()
            .find(|e| e.element == self)
            .map(|e| e.name)
            .expect("every element has a spec")
    }
}
