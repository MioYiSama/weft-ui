//! Parsed import closure: the parser's output and the input of semantic
//! analysis.

use crate::{Diagnostic, File, FileId, SourceMap, Span};

/// One parsed module and the direct imports that resolved successfully.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    /// Syntax tree, including error nodes if parsing failed.
    pub ast: File,
    /// Successfully loaded direct imports, with the span of the `import`.
    pub imports: Vec<(FileId, Span)>,
}

/// Entry modules and every module they import, parsed.
#[derive(Debug, Clone)]
pub struct ParsedProgram {
    /// Source text of every module.
    pub sources: SourceMap,
    /// Modules in source-map order, indexed by [`FileId::index`].
    pub files: Vec<ParsedFile>,
    /// Entry modules in the given order, without duplicates.
    pub entries: Vec<FileId>,
    /// Loading and parsing errors and warnings.
    pub diagnostics: Vec<Diagnostic>,
}
