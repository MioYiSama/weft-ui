//! TypeScript content mapper transform for `.weft` files.
//!
//! TypeScript 7.1 hands files with an extension registered under
//! `contentMappers` in a tsconfig to a mapper process, which answers each
//! `transform` request with TypeScript text, span mappings back to the
//! original file, and diagnostics
//! (<https://github.com/microsoft/typescript-go/pull/4712>). [`transform`]
//! answers one request: the text is the declarations of the file's Web logic
//! module, the same text `@weft-ui/vite` writes to `X.d.weft.ts`, and the
//! names of models, fields, events, components and parameters map back to
//! their declarations. `@weft-ui/content-mapper` serves it over the
//! protocol's JSON-RPC.
//!
//! Imported `.weft` files are read through a [`SourceLoader`]. The text
//! re-exports them with `export *`, but still names which import declares
//! each type the file uses, so moving a declaration between imported files
//! changes the text of an importer that TypeScript may have cached.

use std::fmt;
use std::io;
use std::str::FromStr;

use weft_ui_parser::SourceLoader;
use weft_ui_parser::loader::normalize;
use weft_ui_transformer::transform::MappingKind;

/// Unit of the positions and lengths in a [`Transform`]; the protocol's
/// `PositionEncoding`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionEncoding {
    /// Bytes of UTF-8 text.
    Utf8,
    /// UTF-16 code units, as JavaScript strings count them.
    Utf16,
}

impl PositionEncoding {
    /// The protocol name: `utf-8` or `utf-16`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            PositionEncoding::Utf8 => "utf-8",
            PositionEncoding::Utf16 => "utf-16",
        }
    }
}

impl fmt::Display for PositionEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error for a string that names no [`PositionEncoding`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePositionEncodingError(String);

impl fmt::Display for ParsePositionEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown position encoding `{}`, expected `utf-8` or `utf-16`",
            self.0
        )
    }
}

impl std::error::Error for ParsePositionEncodingError {}

impl FromStr for PositionEncoding {
    type Err = ParsePositionEncodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        [PositionEncoding::Utf8, PositionEncoding::Utf16]
            .into_iter()
            .find(|e| e.as_str() == s)
            .ok_or_else(|| ParsePositionEncodingError(s.to_owned()))
    }
}

/// The protocol's `SpanMapKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpanMapKind {
    /// Same length and content in both texts.
    Verbatim,
    /// Corresponding spans with different content.
    Atom,
    /// Different content naming the same entity; diagnostics show the
    /// original text.
    Alias,
}

impl From<SpanMapKind> for u32 {
    fn from(kind: SpanMapKind) -> u32 {
        match kind {
            SpanMapKind::Verbatim => 0,
            SpanMapKind::Atom => 1,
            SpanMapKind::Alias => 2,
        }
    }
}

/// `SpanMapFeature` bits of the language service features mappings enable.
mod feature {
    pub(crate) const DEFINITION: u32 = 1 << 3;
    pub(crate) const TYPE_DEFINITION: u32 = 1 << 4;
    pub(crate) const IMPLEMENTATION: u32 = 1 << 5;
    pub(crate) const REFERENCES: u32 = 1 << 6;
}

/// Features of every mapping: navigation between TypeScript and `.weft`
/// files. Hover, completion and the other features of `.weft` files belong
/// to the Weft language server, and an edit through a mapping (rename, code
/// actions) would change a declaration without its uses in the `.weft` file.
pub const FEATURES: u32 =
    feature::DEFINITION | feature::TYPE_DEFINITION | feature::IMPLEMENTATION | feature::REFERENCES;

/// The protocol's `SpanMapping` tuple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanMapping {
    pub virtual_start: u32,
    pub virtual_length: u32,
    pub original_start: u32,
    pub original_length: u32,
    pub kind: SpanMapKind,
    /// `SpanMapFeature` bits; always [`FEATURES`].
    pub features: u32,
}

/// An error in the transformed file; the protocol's `MapperDiagnostic`
/// without its `code`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapperDiagnostic {
    /// The message, followed by its notes on separate lines.
    pub message: String,
    pub start: u32,
    pub length: u32,
}

/// Result of [`transform`]: TypeScript text with the `.ts` extension.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Transform {
    pub text: String,
    /// Ordered by virtual start.
    pub mappings: Vec<SpanMapping>,
    /// Errors located in the transformed file. Errors elsewhere are reported
    /// when TypeScript transforms the files the text imports.
    pub diagnostics: Vec<MapperDiagnostic>,
}

/// Transforms the `.weft` file at `path`, whose text is `content`.
///
/// `path` is `/`-separated; `loader` supplies the files it imports. Without
/// errors in the import closure, the text holds the module's declarations;
/// otherwise it re-exports the file's direct imports, so TypeScript still
/// transforms them and reports their errors.
#[must_use]
pub fn transform(
    path: &str,
    content: &str,
    loader: &mut dyn SourceLoader,
    encoding: PositionEncoding,
) -> Transform {
    let path = normalize(path);
    let parsed = weft_ui_parser::load(
        &[path.as_str()],
        &mut Overlay {
            path: &path,
            content,
            loader,
        },
    );
    let entry = parsed.entries[0];
    let imports: Vec<String> = parsed.files[entry.index()]
        .imports
        .iter()
        .map(|&(file, _)| parsed.sources.get(file).path().to_owned())
        .collect();
    let compilation = weft_ui_transformer::check(parsed);

    let mut original = Positions::new(content, encoding);
    let diagnostics = compilation
        .diagnostics
        .iter()
        .filter(|d| d.is_error() && d.span.file == entry)
        .map(|d| {
            let start = original.at(d.span.start);
            let mut message = d.message.clone();
            for note in &d.notes {
                message.push_str("\nnote: ");
                message.push_str(note);
            }
            MapperDiagnostic {
                message,
                start,
                length: original.at(d.span.end) - start,
            }
        })
        .collect();

    let Some(program) = &compilation.program else {
        return Transform {
            text: weft_ui_transformer::import_declarations(
                &path,
                imports.iter().map(String::as_str),
            ),
            mappings: Vec::new(),
            diagnostics,
        };
    };
    let declarations = weft_ui_transformer::declarations(program, entry);
    let mut generated = Positions::new(&declarations.text, encoding);
    let mappings = declarations
        .mappings
        .iter()
        .map(|m| {
            let virtual_start = generated.at(m.start);
            let original_start = original.at(m.original.start);
            SpanMapping {
                virtual_start,
                virtual_length: generated.at(m.start + m.len) - virtual_start,
                original_start,
                original_length: original.at(m.original.end) - original_start,
                kind: match m.kind {
                    MappingKind::Verbatim => SpanMapKind::Verbatim,
                    MappingKind::Alias => SpanMapKind::Alias,
                    MappingKind::Atom => SpanMapKind::Atom,
                },
                features: FEATURES,
            }
        })
        .collect();
    Transform {
        text: declarations.text,
        mappings,
        diagnostics,
    }
}

/// Serves the file being transformed from its unsaved content.
struct Overlay<'a> {
    path: &'a str,
    content: &'a str,
    loader: &'a mut dyn SourceLoader,
}

impl SourceLoader for Overlay<'_> {
    fn load(&mut self, path: &str) -> io::Result<String> {
        if path == self.path {
            Ok(self.content.to_owned())
        } else {
            self.loader.load(path)
        }
    }
}

/// Converts byte offsets of one text to a [`PositionEncoding`].
///
/// UTF-16 positions are counted from the previous query, so mostly
/// ascending queries take linear time overall.
struct Positions<'a> {
    text: &'a str,
    encoding: PositionEncoding,
    /// Last query: byte offset and its position.
    last: (u32, u32),
}

impl<'a> Positions<'a> {
    fn new(text: &'a str, encoding: PositionEncoding) -> Self {
        Self {
            text,
            encoding,
            last: (0, 0),
        }
    }

    fn at(&mut self, byte: u32) -> u32 {
        match self.encoding {
            PositionEncoding::Utf8 => byte,
            PositionEncoding::Utf16 => {
                let (from, position) = if byte >= self.last.0 {
                    self.last
                } else {
                    (0, 0)
                };
                let units = self.text[from as usize..byte as usize]
                    .encode_utf16()
                    .count();
                let units = u32::try_from(units).expect("spans are within 4 GiB sources");
                self.last = (byte, position + units);
                self.last.1
            }
        }
    }
}
