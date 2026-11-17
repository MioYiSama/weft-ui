//! Compiler for the Weft UI description language.
//!
//! [`check`] loads an entry module and its imports through a [`SourceLoader`],
//! reports diagnostics, and on success yields the typed view IR
//! ([`ir::Program`]) with dependency information for every binding.

mod deps;
pub mod diagnostic;
pub mod ir;
pub mod loader;
mod report;
mod sema;
pub mod span;
pub mod syntax;

pub use diagnostic::{Diagnostic, Severity};
pub use loader::{MemoryLoader, SourceLoader};
pub use report::dependency_report;
pub use span::{FileId, SourceMap, Span};

/// Outcome of checking an import closure.
#[derive(Debug, Clone)]
pub struct Compilation {
    pub sources: SourceMap,
    /// Errors and warnings, ordered by file and position.
    pub diagnostics: Vec<Diagnostic>,
    /// Present only when no errors were reported.
    pub program: Option<ir::Program>,
}

impl Compilation {
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_error)
    }

    /// Renders all diagnostics with source snippets.
    #[must_use]
    pub fn render_diagnostics(&self) -> String {
        diagnostic::render_all(&self.diagnostics, &self.sources)
    }
}

/// Parses, resolves, and type checks `entry` and every module it imports.
///
/// `entry` is a `/`-separated path handed to `loader`; imports are resolved
/// relative to the importing module.
pub fn check(entry: &str, loader: &mut dyn SourceLoader) -> Compilation {
    let analysis = sema::analyze(entry, loader);
    Compilation {
        sources: analysis.sources,
        diagnostics: analysis.diagnostics,
        program: analysis.program,
    }
}
