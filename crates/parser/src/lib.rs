//! Parses `.weft` source into a [`weft_ui_ast::File`] and loads its imports.

pub mod lexer;
pub mod loader;
pub mod modules;
mod parser;
pub mod token;

pub use loader::{MemoryLoader, SourceLoader};
pub use parser::parse;
