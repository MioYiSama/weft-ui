//! Canonical formatting for syntactically valid Weft source files.

mod layout;
mod print;

use std::error::Error;
use std::fmt;

use weft_ui_ast::{Diagnostic, FileId, Span};
use weft_ui_parser::lexer::lex_with_comments;
use weft_ui_parser::token::{Token, TokenKind};

/// Indentation added for each nested level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Indent {
    /// Indent by the given number of spaces.
    Spaces(u8),
    /// Indent by one tab, measured as four columns.
    Tab,
}

/// Controls indentation and when groups break across lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Options {
    /// Indentation per level.
    pub indent: Indent,
    /// Target line width in columns. Unbreakable text may exceed this width.
    pub max_width: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            indent: Indent::Spaces(4),
            max_width: 100,
        }
    }
}

/// Why the formatter could not produce output for a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// The compiler reported a syntax error. All diagnostics refer to file zero.
    Syntax(Vec<Diagnostic>),
    /// The formatter could not preserve the source tokens or comments.
    Internal(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(diagnostics) => {
                f.write_str("cannot format invalid Weft source")?;
                if let Some(first) = diagnostics.iter().find(|d| d.is_error()) {
                    write!(f, ": {}", first.message)?;
                }
                Ok(())
            }
            Self::Internal(message) => {
                write!(f, "formatter could not preserve Weft source: {message}")
            }
        }
    }
}

impl Error for FormatError {}

/// Formats one Weft source file without resolving imports or checking semantics.
///
/// # Errors
///
/// Returns [`FormatError::Syntax`] for lexer or parser errors, or
/// [`FormatError::Internal`] if the formatter cannot preserve all tokens and comments.
pub fn format(source: &str, options: &Options) -> Result<String, FormatError> {
    let file_id = FileId(0);
    let (file, mut diagnostics) = weft_ui_parser::parse(file_id, source);
    if diagnostics.iter().any(Diagnostic::is_error) {
        return Err(FormatError::Syntax(diagnostics));
    }
    let (tokens, comments) = lex_with_comments(file_id, source, 0, &mut diagnostics);
    if diagnostics.iter().any(Diagnostic::is_error) {
        return Err(FormatError::Syntax(diagnostics));
    }
    let (output, elided) = print::print(source, &tokens, &comments, &file, options)?;
    diagnostics.clear();
    let (output_tokens, output_comments) = lex_with_comments(file_id, &output, 0, &mut diagnostics);
    if let Some(first) = diagnostics.iter().find(|d| d.is_error()) {
        return Err(FormatError::Internal(format!(
            "output does not lex at byte {}: {}",
            first.span.start, first.message,
        )));
    }
    check_tokens(source, &tokens, &elided, &output, &output_tokens)?;
    if !comments
        .iter()
        .zip(&output_comments)
        .all(|(&before, &after)| {
            // Compare per line so CRLF and line-final whitespace in block comments
            // can be normalized without changing any authored comment content.
            text_at(source, before)
                .lines()
                .map(str::trim_end)
                .eq(text_at(&output, after).lines().map(str::trim_end))
        })
        || comments.len() != output_comments.len()
    {
        return Err(FormatError::Internal(
            "comment text or order changed in output".into(),
        ));
    }
    Ok(output)
}

fn check_tokens(
    source: &str,
    tokens: &[Token],
    elided: &[usize],
    output: &str,
    output_tokens: &[Token],
) -> Result<(), FormatError> {
    let mut before = normalized_tokens(source, tokens, elided);
    let mut after = normalized_tokens(output, output_tokens, &[]);
    for index in 0.. {
        match (before.next(), after.next()) {
            (None, None) => break,
            (before, after) if before == after => {}
            (before, after) => {
                return Err(FormatError::Internal(format!(
                    "token {} changed from {before:?} to {after:?}",
                    index + 1,
                )));
            }
        }
    }
    Ok(())
}

fn normalized_tokens<'src>(
    source: &'src str,
    tokens: &[Token],
    mut elided: &[usize],
) -> impl Iterator<Item = (TokenKind, &'src str)> {
    let mut tokens = tokens.iter().enumerate().peekable();
    let mut delimiters = Vec::new();
    std::iter::from_fn(move || {
        use TokenKind as T;
        while let Some((index, token)) = tokens.next() {
            if elided.first() == Some(&index) {
                elided = &elided[1..];
                continue;
            }
            match token.kind {
                T::LBrace | T::LParen | T::LBracket => delimiters.push(token.kind),
                T::RBrace | T::RParen | T::RBracket => {
                    delimiters.pop();
                }
                // A lambda separator is directly inside its brace pair, even
                // if the lambda itself is inside a for header. A for key's
                // separator is directly inside the header's parentheses.
                T::Semi if delimiters.last() == Some(&T::LBrace) => continue,
                T::Comma if matches!(tokens.peek(), Some((_, next)) if matches!(next.kind, T::RBrace | T::RParen)) =>
                {
                    continue;
                }
                T::Eof => break,
                _ => {}
            }
            return Some((token.kind, text_at(source, token.span)));
        }
        None
    })
}

fn text_at(source: &str, span: Span) -> &str {
    &source[span.start as usize..span.end as usize]
}
