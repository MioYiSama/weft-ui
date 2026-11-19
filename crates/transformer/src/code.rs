//! Shared indentation and line writing for native view source.

use std::fmt::{Display, Write as _};

/// Writes source line by line using the backend's indentation unit.
#[derive(Debug)]
pub(crate) struct Indented {
    text: String,
    depth: usize,
    unit: &'static str,
}

impl Indented {
    pub(crate) fn new(unit: &'static str) -> Self {
        Self {
            text: String::new(),
            depth: 0,
            unit,
        }
    }

    pub(crate) fn line(&mut self, line: impl Display) {
        self.start_line();
        self.end_line(line);
    }

    /// Starts a line whose fragments are written directly into the buffer.
    pub(crate) fn start_line(&mut self) -> &mut String {
        for _ in 0..self.depth {
            self.text.push_str(self.unit);
        }
        &mut self.text
    }

    pub(crate) fn end_line(&mut self, tail: impl Display) {
        writeln!(self.text, "{tail}").expect("writing to a String cannot fail");
    }

    pub(crate) fn indent(&mut self) {
        self.depth += 1;
    }

    /// # Panics
    ///
    /// Panics if there is no indentation level to end.
    pub(crate) fn dedent(&mut self) {
        self.depth = self
            .depth
            .checked_sub(1)
            .expect("every close has a matching open");
    }

    pub(crate) fn blank(&mut self) {
        self.text.push('\n');
    }

    /// # Panics
    ///
    /// Panics if indentation levels remain open.
    pub(crate) fn finish(self) -> String {
        assert_eq!(self.depth, 0, "every open has a matching close");
        self.text
    }
}

#[cfg(test)]
mod tests {
    use super::Indented;

    #[test]
    fn writes_complete_and_partial_lines_without_indenting_blanks() {
        let mut out = Indented::new("  ");
        out.line("root");
        out.indent();
        out.blank();
        out.start_line().push_str("child ");
        out.end_line(1);
        out.indent();
        out.line("leaf");
        out.dedent();
        out.dedent();
        out.line("end");
        assert_eq!(out.finish(), "root\n\n  child 1\n    leaf\nend\n");
    }

    #[test]
    #[should_panic(expected = "every close has a matching open")]
    fn rejects_a_dedent_at_the_root() {
        Indented::new("  ").dedent();
    }

    #[test]
    #[should_panic(expected = "every open has a matching close")]
    fn rejects_unclosed_indentation() {
        let mut out = Indented::new("  ");
        out.indent();
        out.finish();
    }
}
