//! Lexing and parsing of `.weft` source into an [`ast::File`].

pub mod ast;
pub mod lexer;
mod parser;
pub mod token;

pub use parser::parse;
