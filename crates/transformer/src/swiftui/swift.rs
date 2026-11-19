//! Indented Swift source and Swift literals.

use std::fmt::{self, Display, Write as _};

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
pub(crate) fn string(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    f.write_char('"')?;
    for c in s.chars() {
        match c {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            '\0' => f.write_str("\\0")?,
            c if c.is_control() => write!(f, "\\u{{{:x}}}", u32::from(c))?,
            c => f.write_char(c)?,
        }
    }
    f.write_char('"')
}
