//! Signatures of built-in elements and members.

use crate::ir::{Element, Member, Type};

/// Value type of an element property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropType {
    Int,
    Bool,
    String,
}

impl PropType {
    pub(crate) fn to_type(self) -> Type {
        match self {
            PropType::Int => Type::Int,
            PropType::Bool => Type::Bool,
            PropType::String => Type::String,
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
    value("background", PropType::String),
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
            value("color", PropType::String),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemberKind {
    Property,
    Method,
}

pub(crate) struct MemberSig {
    pub member: Member,
    pub kind: MemberKind,
    pub params: Vec<Type>,
    pub ret: Type,
}

/// Looks up a built-in member on a non-optional receiver type.
pub(crate) fn member(recv: &Type, name: &str) -> Option<MemberSig> {
    use MemberKind::{Method, Property};
    let sig = |member, kind, params, ret| MemberSig {
        member,
        kind,
        params,
        ret,
    };
    Some(match (recv, name) {
        (Type::String | Type::Array(_), "length") => {
            sig(Member::Length, Property, vec![], Type::Int)
        }
        (Type::String | Type::Array(_), "isEmpty") => {
            sig(Member::IsEmpty, Property, vec![], Type::Bool)
        }
        (Type::String, "trim") => sig(Member::Trim, Method, vec![], Type::String),
        (Type::String, "uppercase") => sig(Member::Uppercase, Method, vec![], Type::String),
        (Type::String, "lowercase") => sig(Member::Lowercase, Method, vec![], Type::String),
        (Type::String, "contains") => sig(Member::Contains, Method, vec![Type::String], Type::Bool),
        (Type::Array(elem), "contains") if elem.is_scalar() => {
            sig(Member::Contains, Method, vec![(**elem).clone()], Type::Bool)
        }
        (Type::Int | Type::Float | Type::Bool, "toString") => {
            sig(Member::ToString, Method, vec![], Type::String)
        }
        (Type::Float, "toFixed") => sig(Member::ToFixed, Method, vec![Type::Int], Type::String),
        _ => return None,
    })
}
