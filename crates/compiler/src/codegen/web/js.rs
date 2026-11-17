//! JavaScript / TypeScript source helpers shared by the host and DOM generators.

use std::fmt::Write;

/// Lines of generated code with relative indentation.
#[derive(Debug, Default)]
pub(super) struct Code {
    lines: Vec<(usize, String)>,
    indent: usize,
}

impl Code {
    pub(super) fn line(&mut self, text: impl Into<String>) {
        self.lines.push((self.indent, text.into()));
    }

    /// Writes `text` and indents the following lines.
    pub(super) fn open(&mut self, text: impl Into<String>) {
        self.line(text);
        self.indent += 1;
    }

    /// Dedents and writes `text`.
    pub(super) fn close(&mut self, text: impl Into<String>) {
        self.indent -= 1;
        self.line(text);
    }

    /// Appends `other` at the current indentation.
    pub(super) fn append(&mut self, other: Code) {
        let base = self.indent;
        self.lines
            .extend(other.lines.into_iter().map(|(i, l)| (base + i, l)));
    }

    pub(super) fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub(super) fn write_to(&self, out: &mut String) {
        for (indent, line) in &self.lines {
            for _ in 0..*indent {
                out.push_str("  ");
            }
            out.push_str(line);
            out.push('\n');
        }
    }
}

/// Double-quoted JS string literal.
pub(super) fn string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' | '\u{2029}' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c if c.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Words that cannot name a JS binding or a TS type in a module.
const RESERVED: &[&str] = &[
    "Infinity",
    "NaN",
    "any",
    "arguments",
    "await",
    "bigint",
    "boolean",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "never",
    "new",
    "null",
    "number",
    "object",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "string",
    "super",
    "switch",
    "symbol",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "undefined",
    "unknown",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Local binding name for a declaration or parameter; reserved words get a
/// `$` suffix (DSL identifiers never contain `$`).
pub(super) fn binding(name: &str) -> String {
    if RESERVED.contains(&name) {
        format!("{name}$")
    } else {
        name.to_owned()
    }
}

/// `name` or `name as alias` in an import / export list.
pub(super) fn aliased(name: &str, local: &str) -> String {
    if name == local {
        name.to_owned()
    } else {
        format!("{name} as {local}")
    }
}

/// Key of an object literal property; `__proto__` would set the prototype.
pub(super) fn object_key(name: &str) -> String {
    if name == "__proto__" {
        format!("[{}]", string(name))
    } else {
        name.to_owned()
    }
}

/// Name of a class accessor; `constructor` is not allowed as an accessor.
pub(super) fn member_name(name: &str) -> String {
    if name == "constructor" {
        format!("[{}]", string(name))
    } else {
        name.to_owned()
    }
}

/// Handler registration method for an event, e.g. `onTextChanged`.
pub(super) fn handler_method(event: &str) -> String {
    let mut chars = event.chars();
    let first = chars.next().map(|c| c.to_ascii_uppercase());
    format!(
        "on{}{}",
        first.map(String::from).unwrap_or_default(),
        chars.as_str()
    )
}

/// Relative import specifier from the module at `from` to the module at `to`
/// (both normalized, `/`-separated).
pub(super) fn relative_specifier(from: &str, to: &str) -> String {
    let from_dir: Vec<&str> = match from.rfind('/') {
        Some(slash) => from[..slash].split('/').collect(),
        None => Vec::new(),
    };
    let to_parts: Vec<&str> = to.split('/').collect();
    let (to_dir, file) = to_parts.split_at(to_parts.len() - 1);
    let common = from_dir
        .iter()
        .zip(to_dir)
        .take_while(|(a, b)| a == b)
        .count();
    let mut out = String::new();
    if common == from_dir.len() {
        out.push_str("./");
    } else {
        for _ in common..from_dir.len() {
            out.push_str("../");
        }
    }
    for part in &to_dir[common..] {
        out.push_str(part);
        out.push('/');
    }
    out.push_str(file[0]);
    out
}

/// Escapes text content for an HTML template.
pub(super) fn html_text(value: &str, out: &mut String) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
}

/// Escapes a double-quoted HTML attribute value.
pub(super) fn html_attr(value: &str, out: &mut String) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_specifiers() {
        assert_eq!(relative_specifier("ui/a.weft", "ui/b.weft"), "./b.weft");
        assert_eq!(
            relative_specifier("ui/a.weft", "ui/lib/b.weft"),
            "./lib/b.weft"
        );
        assert_eq!(
            relative_specifier("ui/x/a.weft", "ui/y/b.weft"),
            "../y/b.weft"
        );
        assert_eq!(relative_specifier("a.weft", "b.weft"), "./b.weft");
        assert_eq!(
            relative_specifier("/p/ui/a.weft", "/p/models.weft"),
            "../models.weft"
        );
    }

    #[test]
    fn strings_escape_line_terminators() {
        assert_eq!(string("a\"b\\\n\u{2028}"), r#""a\"b\\\n\u2028""#);
    }
}
