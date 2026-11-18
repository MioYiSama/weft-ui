//! Indented Kotlin source and Kotlin literals.

use std::fmt::{self, Display, Write as _};

/// Kotlin source written line by line, indented by nesting depth.
#[derive(Debug, Default)]
pub(crate) struct Code {
    text: String,
    depth: usize,
}

impl Code {
    pub(crate) fn line(&mut self, line: impl Display) {
        self.start_line();
        self.end_line(line);
    }

    /// Writes `line` and indents the lines after it.
    pub(crate) fn open(&mut self, line: impl Display) {
        self.start_line();
        self.open_after(line);
    }

    /// Ends the indentation of the matching [`Code::open`] and writes `line`.
    pub(crate) fn close(&mut self, line: impl Display) {
        self.depth = self
            .depth
            .checked_sub(1)
            .expect("every close has a matching open");
        self.line(line);
    }

    /// Starts a line whose fragments are written directly into the buffer.
    pub(crate) fn start_line(&mut self) -> &mut String {
        for _ in 0..self.depth {
            self.text.push_str("    ");
        }
        &mut self.text
    }

    pub(crate) fn end_line(&mut self, tail: impl Display) {
        writeln!(self.text, "{tail}").expect("writing to a String cannot fail");
    }

    /// Finishes a line and indents the lines after it.
    pub(crate) fn open_after(&mut self, tail: impl Display) {
        self.end_line(tail);
        self.depth += 1;
    }

    pub(crate) fn blank(&mut self) {
        self.text.push('\n');
    }

    pub(crate) fn finish(self) -> String {
        assert_eq!(self.depth, 0, "every open has a matching close");
        self.text
    }
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
