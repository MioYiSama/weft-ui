//! JavaScript / TypeScript source helpers shared by the logic module and view
//! generators.

use std::fmt::Write;

use weft_ui_ast::Span;
use weft_ui_ast::span::offset;

use crate::transform::{Mapping, MappingKind};

/// Lines of generated code with relative indentation.
#[derive(Debug, Default)]
pub struct Code {
    lines: Vec<(usize, String)>,
    /// Mapped fragments, by index into `lines`, in line order.
    marks: Vec<(usize, Mark)>,
    indent: usize,
}

/// Fragment of a line that stands for a span of the `.weft` source.
#[derive(Debug, Clone, Copy)]
struct Mark {
    /// Byte offset in the line text, before indentation.
    column: usize,
    len: usize,
    original: Span,
    kind: MappingKind,
}

/// One line built from plain and mapped fragments.
#[derive(Debug, Default)]
#[must_use]
pub struct MappedLine {
    text: String,
    marks: Vec<Mark>,
}

impl MappedLine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(mut self, text: &str) -> Self {
        self.text.push_str(text);
        self
    }

    /// Appends `text`, which stands for `original`.
    pub fn map(mut self, text: &str, original: Span, kind: MappingKind) -> Self {
        self.marks.push(Mark {
            column: self.text.len(),
            len: text.len(),
            original,
            kind,
        });
        self.text.push_str(text);
        self
    }

    /// Appends the [`binding`] of `name`, declared at `original`: verbatim,
    /// or an alias if it is renamed.
    pub fn binding(self, name: &str, original: Span) -> Self {
        let local = binding(name);
        let kind = if local == name {
            MappingKind::Verbatim
        } else {
            MappingKind::Alias
        };
        self.map(&local, original, kind)
    }
}

impl Code {
    pub fn line(&mut self, text: impl Into<String>) {
        self.lines.push((self.indent, text.into()));
    }

    pub fn mapped_line(&mut self, line: MappedLine) {
        let index = self.lines.len();
        self.marks
            .extend(line.marks.into_iter().map(|mark| (index, mark)));
        self.line(line.text);
    }

    /// Writes `text` and indents the following lines.
    pub fn open(&mut self, text: impl Into<String>) {
        self.line(text);
        self.indent += 1;
    }

    /// Writes `line` and indents the following lines.
    pub fn open_mapped(&mut self, line: MappedLine) {
        self.mapped_line(line);
        self.indent += 1;
    }

    /// Dedents and writes `text`.
    pub fn close(&mut self, text: impl Into<String>) {
        self.indent -= 1;
        self.line(text);
    }

    /// Appends `other` at the current indentation.
    pub fn append(&mut self, other: Code) {
        let base = self.indent;
        let first = self.lines.len();
        self.lines
            .extend(other.lines.into_iter().map(|(i, l)| (base + i, l)));
        self.marks
            .extend(other.marks.into_iter().map(|(i, m)| (first + i, m)));
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn write_to(&self, out: &mut String) {
        for (indent, line) in &self.lines {
            Self::write_line(*indent, line, out);
        }
    }

    /// Writes the code like [`Code::write_to`] and returns where its mapped
    /// fragments ended up in `out`, in order.
    pub fn write_mapped(&self, out: &mut String) -> Vec<Mapping> {
        let mut mappings = Vec::with_capacity(self.marks.len());
        let mut marks = self.marks.iter().peekable();
        for (index, (indent, line)) in self.lines.iter().enumerate() {
            let start = out.len() + 2 * indent;
            while let Some((_, mark)) = marks.next_if(|(i, _)| *i == index) {
                mappings.push(Mapping {
                    start: offset(start + mark.column),
                    len: offset(mark.len),
                    original: mark.original,
                    kind: mark.kind,
                });
            }
            Self::write_line(*indent, line, out);
        }
        mappings
    }

    fn write_line(indent: usize, line: &str, out: &mut String) {
        for _ in 0..indent {
            out.push_str("  ");
        }
        out.push_str(line);
        out.push('\n');
    }
}

/// Double-quoted JS string literal.
pub fn string(value: &str) -> String {
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
pub fn binding(name: &str) -> String {
    if RESERVED.contains(&name) {
        format!("{name}$")
    } else {
        name.to_owned()
    }
}

/// `name` or `name as alias` in an import / export list.
pub fn aliased(name: &str, local: &str) -> String {
    if name == local {
        name.to_owned()
    } else {
        format!("{name} as {local}")
    }
}

/// Key of an object literal property; `__proto__` would set the prototype.
pub fn object_key(name: &str) -> String {
    if name == "__proto__" {
        format!("[{}]", string(name))
    } else {
        name.to_owned()
    }
}

/// Name of a class accessor; `constructor` is not allowed as an accessor.
pub fn member_name(name: &str) -> String {
    if name == "constructor" {
        format!("[{}]", string(name))
    } else {
        name.to_owned()
    }
}

/// Handler registration method for an event, e.g. `onTextChanged`.
pub fn handler_method(event: &str) -> String {
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
pub fn relative_specifier(from: &str, to: &str) -> String {
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
