//! Indented Swift source and Swift literals.

use std::fmt::{Display, Write as _};

use crate::code::Indented;

/// Swift source written line by line, indented by nesting depth.
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
        self.writer.line(line);
        self.writer.indent();
    }

    /// Ends the indentation of the matching [`Code::open`] and writes `line`.
    pub(crate) fn close(&mut self, line: impl Display) {
        self.writer.dedent();
        self.writer.line(line);
    }

    /// Writes `line` one level out, between two indented blocks.
    pub(crate) fn reopen(&mut self, line: impl Display) {
        self.close(line);
        self.writer.indent();
    }
}

/// A Swift string literal of `s`. Backslashes are escaped, so `\(` never
/// starts an interpolation.
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
            c if c.is_control() => {
                write!(out, "\\u{{{:x}}}", u32::from(c)).expect("writing to a String cannot fail");
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
