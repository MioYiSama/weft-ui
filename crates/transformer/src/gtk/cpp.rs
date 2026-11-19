//! Indented C++ source and string literals.

use std::fmt::{Display, Write as _};

use crate::code::Indented;

/// C++ source written line by line, with braces on the opening line.
#[derive(Debug)]
pub(crate) struct Code {
    pub(crate) writer: Indented,
}

impl Code {
    pub(crate) fn new() -> Self {
        Self {
            writer: Indented::new("  "),
        }
    }

    /// Writes a line ending with an opening brace, then indents the block.
    pub(crate) fn open(&mut self, line: impl Display) {
        self.writer.line(format_args!("{line} {{"));
        self.writer.indent();
    }

    /// Ends the matching block and writes its closing brace and suffix.
    pub(crate) fn close(&mut self, suffix: impl Display) {
        self.writer.dedent();
        self.writer.line(format_args!("}}{suffix}"));
    }

    /// Writes an access specifier one level out, as clang-format does.
    pub(crate) fn access(&mut self, specifier: impl Display) {
        self.writer.dedent();
        self.writer.line(format_args!(" {specifier}:"));
        self.writer.indent();
    }
}

/// A `std::string_view` literal (`"…"sv`) of UTF-8 text, so embedded NULs
/// keep their place. Control characters are three-digit octal escapes, which
/// never absorb a following digit.
pub(crate) fn string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // `??` followed by some characters is a trigraph before C++17.
            '?' => out.push_str("\\?"),
            c if c.is_ascii_control() => {
                write!(out, "\\{:03o}", u32::from(c)).expect("writing to a String cannot fail");
            }
            c => out.push(c),
        }
    }
    out.push_str("\"sv");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_are_escaped_and_keep_nuls() {
        assert_eq!(
            string("say \"hi\" \\{x}\n\r\t\0\u{7}1\u{7f} ??= 世界\u{2028}"),
            r#""say \"hi\" \\{x}\n\r\t\000\0071\177 \?\?= 世界"#.to_owned() + "\u{2028}\"sv",
        );
    }
}
