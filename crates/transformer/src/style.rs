//! CSS property metadata shared by the Web and WeChat backends.

use crate::protocol::wechat_unit;

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

/// CSS name and value conversion for a Weft style property.
pub(crate) fn css(prop: &str) -> Option<(&'static str, CssUnit)> {
    Some(match prop {
        "padding" => ("padding", CssUnit::Px),
        "gap" => ("gap", CssUnit::Px),
        "width" => ("width", CssUnit::Px),
        "height" => ("height", CssUnit::Px),
        "size" => ("font-size", CssUnit::Px),
        "background" => ("background", CssUnit::Color),
        "color" => ("color", CssUnit::Color),
        "bold" => ("font-weight", CssUnit::Bold),
        _ => return None,
    })
}

/// Base style of a column.
pub(crate) const COLUMN_STYLE: &str = "display:flex;flex-direction:column;align-items:flex-start";
/// Base style of a row.
pub(crate) const ROW_STYLE: &str = "display:flex;flex-direction:row;align-items:flex-start";
