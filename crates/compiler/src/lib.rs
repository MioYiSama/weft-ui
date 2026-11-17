//! Compiler for the Weft UI description language.
//!
//! [`check_many`] loads entry modules and their imports through a
//! [`SourceLoader`], reports diagnostics, and on success yields the typed view
//! IR ([`ir::Program`]) with dependency information for every binding.
//! [`view::ViewLayout`] derives the view protocol layout from it for the
//! platforms whose view layer is separate from the JS logic layer.
//!
//! Platform backends implement [`codegen::Codegen`] in their own crates; the
//! `weft-ui-driver` crate dispatches a [`codegen::Platform`] to them.

pub mod codegen;
pub mod color;
mod deps;
pub mod diagnostic;
pub mod ir;
pub mod loader;
mod report;
mod sema;
pub mod span;
pub mod syntax;
pub mod view;

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
/// Same as [`check_many`] with a single entry.
pub fn check(entry: &str, loader: &mut dyn SourceLoader) -> Compilation {
    check_many(&[entry], loader)
}

/// Parses, resolves, and type checks `entries` and every module they import
/// as one program, so declaration names must be unique across all of them.
///
/// Entries are `/`-separated paths handed to `loader`; imports are resolved
/// relative to the importing module. Entries come first in the program's
/// file order.
pub fn check_many(entries: &[&str], loader: &mut dyn SourceLoader) -> Compilation {
    let analysis = sema::analyze(entries, loader);
    Compilation {
        sources: analysis.sources,
        diagnostics: analysis.diagnostics,
        program: analysis.program,
    }
}
