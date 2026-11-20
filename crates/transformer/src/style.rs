//! Style generation shared by the Web and WeChat backends.

use std::fmt::{self, Write};

use crate::color::Color;

/// Color references in static markup use the same variable as dynamic values.
pub(crate) fn theme_color(name: &str) -> impl fmt::Display + '_ {
    fmt::from_fn(move |out| write!(out, "var(--weft-{name})"))
}

/// CSS variables on the mini program view component, with system dark overrides.
pub(crate) fn theme_styles(colors: &[(String, Color, Color)]) -> String {
    if colors.is_empty() {
        return String::new();
    }
    fn declarations(
        out: &mut String,
        colors: &[(String, Color, Color)],
        color: impl Fn(Color, Color) -> Color,
    ) {
        out.push_str(":host{");
        for (name, light, dark) in colors {
            let _ = write!(out, "--weft-{name}:{};", color(*light, *dark).rgba_hex());
        }
        out.push('}');
    }
    let mut out = String::new();
    declarations(&mut out, colors, |light, _| light);
    out.push_str("\n@media (prefers-color-scheme: dark){");
    declarations(&mut out, colors, |_, dark| dark);
    out.push_str("}\n");
    out
}

/// Base style of a column.
pub(crate) const COLUMN_STYLE: &str = "display:flex;flex-direction:column;align-items:flex-start";
/// Base style of a row.
pub(crate) const ROW_STYLE: &str = "display:flex;flex-direction:row;align-items:flex-start";
