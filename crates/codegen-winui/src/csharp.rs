//! Indented C# source and regular string literals.

use std::fmt::Write as _;

/// C# source written line by line, with Allman braces.
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

    /// Writes a declaration and its opening brace, then indents its body.
    pub(crate) fn open(&mut self, line: impl AsRef<str>) {
        self.line(line);
        self.line("{");
        self.depth += 1;
    }

    /// Ends the matching block and writes its closing brace and suffix.
    pub(crate) fn close(&mut self, suffix: &str) {
        self.depth = self
            .depth
            .checked_sub(1)
            .expect("every close has a matching open");
        self.line(format!("}}{suffix}"));
    }

    pub(crate) fn blank(&mut self) {
        self.text.push('\n');
    }

    pub(crate) fn finish(self) -> String {
        assert_eq!(self.depth, 0, "every open has a matching close");
        self.text
    }
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
