//! Indented Swift source and Swift literals.

use std::fmt::Write as _;

/// Swift source written line by line, indented by nesting depth.
#[derive(Debug, Default)]
pub(crate) struct Code {
    text: String,
    depth: usize,
}

impl Code {
    pub(crate) fn line(&mut self, line: impl AsRef<str>) {
        for _ in 0..self.depth {
            self.text.push_str("    ");
        }
        self.text.push_str(line.as_ref());
        self.text.push('\n');
    }

    /// Writes `line` and indents the lines after it.
    pub(crate) fn open(&mut self, line: impl AsRef<str>) {
        self.line(line);
        self.depth += 1;
    }

    /// Ends the indentation of the matching [`Code::open`] and writes `line`.
    pub(crate) fn close(&mut self, line: impl AsRef<str>) {
        self.depth = self
            .depth
            .checked_sub(1)
            .expect("every close has a matching open");
        self.line(line);
    }

    /// Writes `line` one level out, between two indented blocks.
    pub(crate) fn reopen(&mut self, line: impl AsRef<str>) {
        self.close(line);
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
