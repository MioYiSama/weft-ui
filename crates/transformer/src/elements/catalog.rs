// Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit.

use super::{Children, Css, CssUnit, ElementSpec, FeatureSpec, PropKind, PropSpec, PropType};

/// Support exceptions for a target platform; omitted entries are supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    No,
    #[allow(dead_code, reason = "the catalog may omit ignored entries")]
    Ignored,
}

/// Built-in elements available in Weft views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Element {
    Column,
    Row,
    Text,
    Button,
    Input,
    Checkbox,
    Image,
}

impl Element {
    /// The element's name in a Weft view.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.spec().name
    }
}

const PROPS_0: &[PropSpec] = &[
    PropSpec {
        name: "padding",
        kind: PropKind::Value(PropType::Int),
        positional: false,
        required: false,
        css: Some(Css {
            property: "padding",
            unit: CssUnit::Px,
        }),
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "gap",
        kind: PropKind::Value(PropType::Int),
        positional: false,
        required: false,
        css: Some(Css {
            property: "gap",
            unit: CssUnit::Px,
        }),
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "width",
        kind: PropKind::Value(PropType::Int),
        positional: false,
        required: false,
        css: Some(Css {
            property: "width",
            unit: CssUnit::Px,
        }),
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "height",
        kind: PropKind::Value(PropType::Int),
        positional: false,
        required: false,
        css: Some(Css {
            property: "height",
            unit: CssUnit::Px,
        }),
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "background",
        kind: PropKind::Value(PropType::Color),
        positional: false,
        required: false,
        css: Some(Css {
            property: "background",
            unit: CssUnit::Color,
        }),
        controlled_by: None,
        support: &[],
    },
];

const PROPS_1: &[PropSpec] = &[
    PropSpec {
        name: "value",
        kind: PropKind::Value(PropType::String),
        positional: true,
        required: true,
        css: None,
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "color",
        kind: PropKind::Value(PropType::Color),
        positional: false,
        required: false,
        css: Some(Css {
            property: "color",
            unit: CssUnit::Color,
        }),
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "size",
        kind: PropKind::Value(PropType::Int),
        positional: false,
        required: false,
        css: Some(Css {
            property: "font-size",
            unit: CssUnit::Px,
        }),
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "bold",
        kind: PropKind::Value(PropType::Bool),
        positional: false,
        required: false,
        css: Some(Css {
            property: "font-weight",
            unit: CssUnit::Bold,
        }),
        controlled_by: None,
        support: &[],
    },
];

const PROPS_2: &[PropSpec] = &[
    PropSpec {
        name: "onClick",
        kind: PropKind::Handler { payload: None },
        positional: false,
        required: false,
        css: None,
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "disabled",
        kind: PropKind::Value(PropType::Bool),
        positional: false,
        required: false,
        css: None,
        controlled_by: None,
        support: &[],
    },
];

const PROPS_3: &[PropSpec] = &[
    PropSpec {
        name: "value",
        kind: PropKind::Value(PropType::String),
        positional: true,
        required: true,
        css: None,
        controlled_by: Some("onChange"),
        support: &[],
    },
    PropSpec {
        name: "onChange",
        kind: PropKind::Handler {
            payload: Some(PropType::String),
        },
        positional: false,
        required: false,
        css: None,
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "placeholder",
        kind: PropKind::Value(PropType::String),
        positional: false,
        required: false,
        css: None,
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "disabled",
        kind: PropKind::Value(PropType::Bool),
        positional: false,
        required: false,
        css: None,
        controlled_by: None,
        support: &[],
    },
];

const PROPS_4: &[PropSpec] = &[
    PropSpec {
        name: "checked",
        kind: PropKind::Value(PropType::Bool),
        positional: true,
        required: true,
        css: None,
        controlled_by: Some("onChange"),
        support: &[],
    },
    PropSpec {
        name: "onChange",
        kind: PropKind::Handler {
            payload: Some(PropType::Bool),
        },
        positional: false,
        required: false,
        css: None,
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "disabled",
        kind: PropKind::Value(PropType::Bool),
        positional: false,
        required: false,
        css: None,
        controlled_by: None,
        support: &[],
    },
];

const PROPS_5: &[PropSpec] = &[
    PropSpec {
        name: "src",
        kind: PropKind::Value(PropType::String),
        positional: true,
        required: true,
        css: None,
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "width",
        kind: PropKind::Value(PropType::Int),
        positional: false,
        required: false,
        css: Some(Css {
            property: "width",
            unit: CssUnit::Px,
        }),
        controlled_by: None,
        support: &[],
    },
    PropSpec {
        name: "height",
        kind: PropKind::Value(PropType::Int),
        positional: false,
        required: false,
        css: Some(Css {
            property: "height",
            unit: CssUnit::Px,
        }),
        controlled_by: None,
        support: &[],
    },
];

pub(crate) const ELEMENTS: [ElementSpec; 7] = [
    ElementSpec {
        element: Element::Column,
        name: "Column",
        props: PROPS_0,
        children: Children::Any,
        support: &[],
    },
    ElementSpec {
        element: Element::Row,
        name: "Row",
        props: PROPS_0,
        children: Children::Any,
        support: &[],
    },
    ElementSpec {
        element: Element::Text,
        name: "Text",
        props: PROPS_1,
        children: Children::None,
        support: &[],
    },
    ElementSpec {
        element: Element::Button,
        name: "Button",
        props: PROPS_2,
        children: Children::Any,
        support: &[],
    },
    ElementSpec {
        element: Element::Input,
        name: "Input",
        props: PROPS_3,
        children: Children::None,
        support: &[],
    },
    ElementSpec {
        element: Element::Checkbox,
        name: "Checkbox",
        props: PROPS_4,
        children: Children::None,
        support: &[],
    },
    ElementSpec {
        element: Element::Image,
        name: "Image",
        props: PROPS_5,
        children: Children::None,
        support: &[],
    },
];

#[expect(dead_code, reason = "feature checks land in L6 and R7")]
pub(crate) const FEATURES: [FeatureSpec; 2] = [
    FeatureSpec {
        name: "theme",
        support: &[],
    },
    FeatureSpec {
        name: "assets",
        support: &[],
    },
];
