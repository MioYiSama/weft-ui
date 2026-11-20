//! Token kinds produced by the lexer.

use std::fmt;

use weft_ui_ast::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    Int,
    Float,
    /// Whole string literal including quotes and interpolations.
    Str,

    // Reserved keywords. `import`, `model`, `event`, `component`, `type` and `state`
    // are contextual and lexed as identifiers.
    If,
    Else,
    For,
    In,
    True,
    False,
    Null,

    // Punctuation.
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semi,
    Dot,
    QuestionDot,
    Question,
    QuestionQuestion,
    Eq,
    PlusEq,
    MinusEq,
    EqEq,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    AndAnd,
    Pipe,
    OrOr,
    Arrow,

    /// Unrecognized input; already reported by the lexer.
    Unknown,
    Eof,
}

impl TokenKind {
    #[must_use]
    pub fn keyword(ident: &str) -> Option<TokenKind> {
        Some(match ident {
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            _ => return None,
        })
    }

    /// Human-readable description used in "expected X, found Y" messages.
    #[must_use]
    pub fn describe(self) -> &'static str {
        use TokenKind::*;
        match self {
            Ident => "identifier",
            Int => "integer",
            Float => "float",
            Str => "string",
            If => "`if`",
            Else => "`else`",
            For => "`for`",
            In => "`in`",
            True => "`true`",
            False => "`false`",
            Null => "`null`",
            LParen => "`(`",
            RParen => "`)`",
            LBrace => "`{`",
            RBrace => "`}`",
            LBracket => "`[`",
            RBracket => "`]`",
            Comma => "`,`",
            Colon => "`:`",
            Semi => "`;`",
            Dot => "`.`",
            QuestionDot => "`?.`",
            Question => "`?`",
            QuestionQuestion => "`??`",
            Eq => "`=`",
            PlusEq => "`+=`",
            MinusEq => "`-=`",
            EqEq => "`==`",
            BangEq => "`!=`",
            Lt => "`<`",
            LtEq => "`<=`",
            Gt => "`>`",
            GtEq => "`>=`",
            Plus => "`+`",
            Minus => "`-`",
            Star => "`*`",
            Slash => "`/`",
            Percent => "`%`",
            Bang => "`!`",
            AndAnd => "`&&`",
            Pipe => "`|`",
            OrOr => "`||`",
            Arrow => "`->`",
            Unknown => "unknown token",
            Eof => "end of file",
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.describe())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
