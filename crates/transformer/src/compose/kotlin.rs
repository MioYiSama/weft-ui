//! Indented Kotlin source and Kotlin literals.

use std::fmt::{self, Display, Write as _};

use crate::code::Indented;
use crate::color::Color;

/// Kotlin source written line by line, indented by nesting depth.
#[derive(Debug)]
pub(crate) struct Code {
    pub(crate) writer: Indented,
}

impl Code {
    pub(crate) fn new() -> Self {
        Self {
            writer: Indented::new("    "),
        }
    }

    /// Writes `line` and indents the lines after it.
    pub(crate) fn open(&mut self, line: impl Display) {
        self.writer.start_line();
        self.open_after(line);
    }

    /// Ends the indentation of the matching [`Code::open`] and writes `line`.
    pub(crate) fn close(&mut self, line: impl Display) {
        self.writer.dedent();
        self.writer.line(line);
    }

    /// Finishes a line and indents the lines after it.
    pub(crate) fn open_after(&mut self, tail: impl Display) {
        self.writer.end_line(tail);
        self.writer.indent();
    }
}

/// Formats an RGBA color as an ARGB Compose color literal.
pub(super) fn color(color: Color) -> impl Display {
    let Color { r, g, b, a } = color;
    fmt::from_fn(move |f| write!(f, "Color(0x{a:02x}{r:02x}{g:02x}{b:02x}L)"))
}

/// Writes a Kotlin string literal. Escaping `$` disables string templates.
pub(crate) fn string(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    f.write_char('"')?;
    for c in s.chars() {
        match c {
            '\\' => f.write_str("\\\\")?,
            '"' => f.write_str("\\\"")?,
            '$' => f.write_str("\\$")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            '\u{8}' => f.write_str("\\b")?,
            c if c.is_control() => write!(f, "\\u{:04x}", u32::from(c))?,
            c => f.write_char(c)?,
        }
    }
    f.write_char('"')
}
