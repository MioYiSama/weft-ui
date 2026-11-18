//! Parses `.weft` source into a [`weft_ui_ast::File`] and loads its imports.
//!
//! [`load`] parses an import closure into a [`weft_ui_ast::ParsedProgram`],
//! the input of `weft_ui_transformer`.

pub mod lexer;
pub mod loader;
mod modules;
mod parser;
pub mod token;

pub use loader::{MemoryLoader, SourceLoader};
pub use modules::load;
pub use parser::parse;
