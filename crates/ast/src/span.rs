//! Source files, byte spans, and line/column lookup.

use std::fmt;

/// Index of a source file inside a [`SourceMap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FileId(pub u32);

impl FileId {
    /// Position of this file in [`SourceMap::files`].
    #[must_use]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Half-open byte range `start..end` inside one source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[must_use]
    pub fn new(file: FileId, start: u32, end: u32) -> Self {
        debug_assert!(start <= end, "inverted span {start}..{end}");
        Self { file, start, end }
    }

    /// Smallest span covering both `self` and `other` (same file).
    #[must_use]
    pub fn to(self, other: Span) -> Span {
        debug_assert_eq!(self.file, other.file, "joining spans of different files");
        Span::new(
            self.file,
            self.start.min(other.start),
            self.end.max(other.end),
        )
    }

    /// Zero-width span at the end of `self`.
    #[must_use]
    pub fn end_point(self) -> Span {
        Span::new(self.file, self.end, self.end)
    }
}

/// One loaded source file with a precomputed line index.
#[derive(Debug, Clone)]
pub struct SourceFile {
    path: String,
    text: String,
    line_starts: Vec<u32>,
}

/// 1-based line and column; the column counts characters, not bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
    pub line: u32,
    pub col: u32,
}

impl fmt::Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl SourceFile {
    #[must_use]
    pub fn new(path: String, text: String) -> Self {
        let line_starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| offset(i + 1)))
            .collect();
        Self {
            path,
            text,
            line_starts,
        }
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Zero-based line index containing byte `pos`.
    #[must_use]
    pub fn line_index(&self, pos: u32) -> usize {
        self.line_starts.partition_point(|&s| s <= pos) - 1
    }

    #[must_use]
    pub fn line_col(&self, pos: u32) -> LineCol {
        let line = self.line_index(pos);
        let start = self.line_starts[line] as usize;
        let col = self.text[start..pos as usize].chars().count();
        LineCol {
            line: offset(line + 1),
            col: offset(col + 1),
        }
    }

    /// Text of the zero-based line `line`, without the trailing newline.
    #[must_use]
    pub fn line_text(&self, line: usize) -> &str {
        let start = self.line_starts[line] as usize;
        let end = self
            .line_starts
            .get(line + 1)
            .map_or(self.text.len(), |&e| e as usize);
        self.text[start..end].trim_end_matches(['\n', '\r'])
    }

    /// Byte offset where the zero-based line `line` starts.
    #[must_use]
    pub fn line_start(&self, line: usize) -> u32 {
        self.line_starts[line]
    }
}

/// All source files of one compilation.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, path: String, text: String) -> FileId {
        let id = FileId(offset(self.files.len()));
        self.files.push(SourceFile::new(path, text));
        id
    }

    /// # Panics
    ///
    /// Panics if `id` was not created by this map.
    #[must_use]
    pub fn get(&self, id: FileId) -> &SourceFile {
        &self.files[id.index()]
    }

    #[must_use]
    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    /// Source text covered by `span`.
    #[must_use]
    pub fn snippet(&self, span: Span) -> &str {
        &self.get(span.file).text()[span.start as usize..span.end as usize]
    }
}

/// Converts a byte offset or count to the `u32` used by spans.
///
/// # Panics
///
/// Panics for sources larger than 4 GiB, which the compiler does not support.
#[must_use]
pub fn offset(n: usize) -> u32 {
    u32::try_from(n).expect("source files larger than 4 GiB are not supported")
}
