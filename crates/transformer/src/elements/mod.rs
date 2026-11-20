//! The built-in element catalog and its accepted properties.

mod catalog;

pub use catalog::Element;
pub(crate) use catalog::{ELEMENTS, FEATURES, Level};

use crate::ir::Type;
use crate::protocol::{Platform, wechat_unit};

#[derive(Debug)]
pub(crate) struct FeatureSpec {
    pub name: &'static str,
    pub support: &'static [(Platform, Level)],
}

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
            PropType::String => Type::String,
            PropType::Color => Type::Color,
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
pub(crate) struct Css {
    pub property: &'static str,
    pub unit: CssUnit,
}

/// Conversion of a property value to CSS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssUnit {
    Px,
    Color,
    Bold,
}

impl CssUnit {
    /// Conversion code used by the WeChat sink.
    pub(crate) fn code(self) -> u8 {
        match self {
            Self::Px => wechat_unit::PX,
            Self::Color => wechat_unit::COLOR,
            Self::Bold => wechat_unit::BOLD,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PropSpec {
    pub name: &'static str,
    pub kind: PropKind,
    pub positional: bool,
    pub required: bool,
    pub css: Option<Css>,
    pub controlled_by: Option<&'static str>,
    pub support: &'static [(Platform, Level)],
}

impl PropSpec {
    /// Whether a style can supply this property as a default.
    pub(crate) fn is_style_property(&self) -> bool {
        !self.positional && matches!(self.kind, PropKind::Value(_))
    }

    /// Declared value type of a built-in element property.
    pub(crate) fn prop_type(&self) -> PropType {
        let PropKind::Value(ty) = self.kind else {
            unreachable!("`{}` is not a value property", self.name);
        };
        ty
    }
}

/// Lifecycle props accepted by every element and component call.
pub(crate) const LIFECYCLE: [PropSpec; 2] = [
    PropSpec {
        name: "onMount",
        kind: PropKind::Lifecycle,
        positional: false,
        required: false,
        css: None,
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "onUnmount",
        kind: PropKind::Lifecycle,
        positional: false,
        required: false,
        css: None,
        controlled_by: None,
        support: &[],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Children {
    Any,
    None,
    #[expect(
        dead_code,
        reason = "no built-in element restricts child element types yet"
    )]
    Only(&'static [Element]),
}

#[derive(Debug)]
pub(crate) struct ElementSpec {
    pub element: Element,
    pub name: &'static str,
    pub props: &'static [PropSpec],
    pub children: Children,
    pub support: &'static [(Platform, Level)],
}

impl ElementSpec {
    pub(crate) fn prop(&self, name: &str) -> Option<&'static PropSpec> {
        self.props.iter().find(|prop| prop.name == name)
    }
}

impl Element {
    pub(crate) fn spec(self) -> &'static ElementSpec {
        // The generator emits both the enum and its specs in schema order.
        &ELEMENTS[self as usize]
    }
}

pub(crate) fn element(name: &str) -> Option<&'static ElementSpec> {
    ELEMENTS.iter().find(|e| e.name == name)
}

pub(crate) fn is_style_property(name: &str) -> bool {
    ELEMENTS.iter().any(|element| {
        element
            .props
            .iter()
            .any(|prop| prop.name == name && prop.is_style_property())
    })
}
