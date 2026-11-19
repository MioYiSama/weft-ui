//! Shared input and output types for the platform backend modules.
//!
//! The compiler generates the JavaScript logic layer and the view files for
//! the selected platform.

use std::fmt;
use std::str::FromStr;

use weft_ui_ast::Span;

use crate::ir::Program;
use crate::view::ViewLayout;

pub use crate::protocol::Platform;

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error for a string that names no [`Platform`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePlatformError(String);

impl fmt::Display for ParsePlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = Platform::ALL.iter().map(|p| p.as_str()).collect();
        write!(
            f,
            "unknown platform `{}`, expected one of: {}",
            self.0,
            names.join(", ")
        )
    }
}

impl std::error::Error for ParsePlatformError {}

impl FromStr for Platform {
    type Err = ParsePlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Platform::ALL
            .into_iter()
            .find(|p| p.as_str() == s)
            .ok_or_else(|| ParsePlatformError(s.to_owned()))
    }
}

/// Generated logic module of one `.weft` file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsModule {
    /// Normalized path of the `.weft` file.
    pub source: String,
    /// ES module source.
    pub code: String,
    /// TypeScript declarations for `code`.
    pub dts: String,
}

/// TypeScript declarations of one logic module ([`JsModule::dts`]), with
/// the fragments that name declarations of its `.weft` file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Declarations {
    pub text: String,
    /// Ordered by [`Mapping::start`]; fragments do not overlap.
    pub mappings: Vec<Mapping>,
}

/// A fragment of generated text that stands for a span of the `.weft` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapping {
    /// Byte offset in the generated text.
    pub start: u32,
    /// Byte length in the generated text.
    pub len: u32,
    /// Always in the module's own file.
    pub original: Span,
    pub kind: MappingKind,
}

/// How a generated fragment relates to its original span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MappingKind {
    /// Same text: a declaration's name.
    Verbatim,
    /// Same declaration under another name, e.g. `delete$` for `delete`.
    Alias,
    /// A different entity derived from the original, e.g. the `onClick`
    /// handler method of the event `Click`.
    Atom,
}

/// Input to a platform backend module.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Context<'p> {
    /// Application-level program: every `.weft` file of the app.
    pub program: &'p Program,
    /// View protocol layout of the program; `None` for `web`, whose logic
    /// modules write the DOM directly.
    pub layout: Option<&'p ViewLayout>,
    pub platform: Platform,
}

impl<'p> Context<'p> {
    #[must_use]
    pub fn new(program: &'p Program, layout: Option<&'p ViewLayout>, platform: Platform) -> Self {
        Self {
            program,
            layout,
            platform,
        }
    }
}

/// A generated file at a `/`-separated path relative to the platform's
/// generated directory (`apps/<platform>/weft/`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFile {
    pub path: String,
    pub contents: String,
}
