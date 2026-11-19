//! Shared input and output types for the platform backend modules.
//!
//! The [`crate::bridge`] module generates the JS logic layer, while each
//! view backend module produces the view files for its platform.

use std::fmt;
use std::str::FromStr;

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
