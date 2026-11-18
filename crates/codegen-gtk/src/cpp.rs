//! Indented C++ source and string literals.

use std::fmt::Write as _;

/// C++ source written line by line, with braces on the opening line.
#[derive(Debug, Default)]
pub(crate) struct Code {
    text: String,
    depth: usize,
}

impl Code {
    pub(crate) fn line(&mut self, line: impl AsRef<str>) {
        for _ in 0..self.depth {
            self.text.push_str("  ");
        }
        self.text.push_str(line.as_ref());
        self.text.push('\n');
    }

    /// Writes a line ending with an opening brace, then indents the block.
    pub(crate) fn open(&mut self, line: impl AsRef<str>) {
        self.line(format!("{} {{", line.as_ref()));
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

    /// Writes an access specifier one level out, as clang-format does.
    pub(crate) fn access(&mut self, specifier: &str) {
        self.depth -= 1;
        self.line(format!(" {specifier}:"));
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
