//! Indented C# source and regular string literals.

use std::fmt::{Display, Write as _};

use crate::code::Indented;
use crate::color::Color;

/// C# source written line by line, with Allman braces.
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

    /// Writes a declaration and its opening brace, then indents its body.
    pub(crate) fn open(&mut self, line: impl Display) {
        self.writer.line(line);
        self.writer.line("{");
        self.writer.indent();
    }

    /// Ends the matching block and writes its closing brace and suffix.
    pub(crate) fn close(&mut self, suffix: impl Display) {
        self.writer.dedent();
        self.writer.line(format_args!("}}{suffix}"));
    }
}

/// A C# unsigned integer literal in RGBA channel order.
pub(crate) fn rgba(color: Color) -> impl Display {
    let value = u32::from_be_bytes([color.r, color.g, color.b, color.a]);
    std::fmt::from_fn(move |out| write!(out, "0x{value:08x}u"))
}

/// A regular C# string literal; braces never start interpolation.
pub(crate) fn string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') => {
                write!(out, "\\u{:04x}", u32::from(c)).expect("writing to a String cannot fail");
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_are_escaped_and_never_interpolated() {
        assert_eq!(
            string("say \"hi\" \\{x}\n\r\t\0\u{7}\u{1f}\u{85}\u{2028}\u{2029} 世界"),
            r#""say \"hi\" \\{x}\n\r\t\0\u0007\u001f\u0085\u2028\u2029 世界""#,
        );
    }
}
