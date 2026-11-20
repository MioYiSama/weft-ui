//! Theme declarations and references shared by expression and style lowering.

use std::collections::HashMap;

use weft_ui_ast::{Diagnostic, FileId, Ident, Span, ThemeDecl, ThemeValue};

use crate::color::Color;
use crate::ir::{Expr, ExprKind, Theme, Type};

use super::Checker;
use super::record::{ProbeContext, Target};

pub(super) const NAME: &str = "theme";

pub(super) struct Tokens<'a> {
    pub output: Theme,
    by_name: HashMap<&'a str, Token<'a>>,
}

struct Token<'a> {
    entry: &'a weft_ui_ast::ThemeEntry,
    valid: bool,
}

impl Token<'_> {
    fn ty(&self) -> Type {
        match self.entry.value {
            ThemeValue::Color { .. } => Type::Color,
            ThemeValue::Int(_) => Type::Int,
            ThemeValue::Float(_) => Type::Float,
        }
    }

    fn target(&self) -> Target {
        Target::ThemeToken {
            ty: self.ty(),
            def: self.entry.name.span,
            declaration: self.entry.span,
        }
    }
}

impl<'a> Tokens<'a> {
    pub fn check(cx: &Checker<'_>, decl: &'a ThemeDecl, diags: &mut Vec<Diagnostic>) -> Self {
        let mut output = Theme {
            span: decl.span,
            colors: Vec::new(),
        };
        let mut by_name: HashMap<&str, Token<'_>> = HashMap::with_capacity(decl.entries.len());
        for entry in &decl.entries {
            let name = entry.name.name.as_str();
            if let Some(previous) = by_name.get(name) {
                diags.push(
                    Diagnostic::error(entry.name.span, format!("duplicate theme token `{name}`"))
                        .with_secondary(previous.entry.name.span, "first declared here"),
                );
                continue;
            }
            let valid = if let ThemeValue::Color { light, dark } = &entry.value {
                let light = literal_color(&light.0, light.1, diags);
                let dark = match dark {
                    Some((value, span)) => literal_color(value, *span, diags),
                    None => light,
                };
                if let (Some(light), Some(dark)) = (light, dark) {
                    output.colors.push((name.to_owned(), light, dark));
                    true
                } else {
                    false
                }
            } else {
                true
            };
            let token = Token { entry, valid };
            cx.record(entry.name.span, || token.target());
            by_name.insert(name, token);
        }
        Self { output, by_name }
    }
}

impl Checker<'_> {
    pub(super) fn theme_visible(&self, file: FileId) -> bool {
        self.theme
            .as_ref()
            .is_some_and(|theme| self.visible[file.index()].contains(&theme.output.span.file))
    }

    pub(super) fn theme_expr(&self, name: &Ident, span: Span, diags: &mut Vec<Diagnostic>) -> Expr {
        let visible = self.theme_visible(span.file);
        if self.is_probe(&name.name) {
            let mut candidates: Vec<_> = self
                .theme
                .iter()
                .filter(|_| visible)
                .flat_map(|theme| theme.by_name.values())
                .collect();
            candidates.sort_by_key(|token| token.entry.span.start);
            self.set_probe(
                ProbeContext::Member,
                candidates.into_iter().map(Token::target).collect(),
            );
        }
        let error = || Expr {
            kind: ExprKind::Error,
            ty: Type::Error,
            span,
        };
        let Some(theme) = &self.theme else {
            diags.push(unknown(name));
            return error();
        };
        let Some(token) = theme.by_name.get(name.name.as_str()) else {
            diags.push(unknown(name));
            return error();
        };
        self.record(name.span, || token.target());
        if !visible {
            let path = self.sources.get(theme.output.span.file).path();
            diags.push(
                Diagnostic::error(
                    name.span,
                    format!(
                        "`theme.{}` is declared in `{path}`, which is not imported here",
                        name.name
                    ),
                )
                .with_secondary(token.entry.name.span, "theme token declared here")
                .with_note("imports are not transitive; import the declaring file directly"),
            );
            return error();
        }
        if !token.valid {
            return error();
        }
        let kind = match token.entry.value {
            ThemeValue::Color { .. } => ExprKind::Theme(name.name.clone()),
            ThemeValue::Int(value) => ExprKind::Int(value),
            ThemeValue::Float(value) => ExprKind::Float(value),
        };
        Expr {
            kind,
            ty: token.ty(),
            span,
        }
    }
}

fn unknown(name: &Ident) -> Diagnostic {
    Diagnostic::error(name.span, format!("unknown theme token `{}`", name.name))
}

pub(super) fn literal_color(value: &str, span: Span, diags: &mut Vec<Diagnostic>) -> Option<Color> {
    let color = Color::parse(value);
    if color.is_none() {
        diags.push(Diagnostic::error(
            span,
            format!("invalid color `{value}`: expected `#RGB`, `#RRGGBB` or `#RRGGBBAA`"),
        ));
    }
    color
}

/// Rejects local bindings that would shadow the theme namespace.
pub(super) fn reserved(name: &str, span: Span, diags: &mut Vec<Diagnostic>) -> bool {
    if name == NAME {
        diags.push(Diagnostic::error(span, "`theme` is reserved"));
        true
    } else {
        false
    }
}
