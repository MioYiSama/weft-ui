//! WebAssembly bindings for compiling Weft apps and mapping `.weft` files to
//! TypeScript, with a JavaScript source loader.

use std::io;

use js_sys::Function;
use wasm_bindgen::prelude::*;
use weft_ui_content_mapper::PositionEncoding;
use weft_ui_parser::SourceLoader;
use weft_ui_transformer::transform::Platform;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = String)]
    fn exception_message(value: &JsValue) -> String;
}

struct JsLoader<'load>(&'load Function);

impl SourceLoader for JsLoader<'_> {
    fn load(&mut self, path: &str) -> io::Result<String> {
        self.0
            .call1(&JsValue::UNDEFINED, &JsValue::from_str(path))
            .map_err(|error| io::Error::new(io::ErrorKind::NotFound, exception_message(&error)))?
            .as_string()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "source loader must return a string",
                )
            })
    }
}

/// Generated logic modules and view files, rendered diagnostics, and all
/// loaded source paths. Parallel arrays keep the boundary to plain strings.
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct AppOutput {
    /// Normalized `.weft` path of each logic module; empty on errors.
    #[wasm_bindgen(js_name = logicSources)]
    pub logic_sources: Vec<String>,
    /// ES module source of each logic module.
    #[wasm_bindgen(js_name = logicCode)]
    pub logic_code: Vec<String>,
    /// TypeScript declarations of each logic module.
    #[wasm_bindgen(js_name = logicDts)]
    pub logic_dts: Vec<String>,
    /// View file paths relative to the platform's generated directory.
    #[wasm_bindgen(js_name = viewPaths)]
    pub view_paths: Vec<String>,
    /// View file contents corresponding to `viewPaths`.
    #[wasm_bindgen(js_name = viewContents)]
    pub view_contents: Vec<String>,
    /// View protocol layout as JSON; empty on errors and for `web`.
    pub layout: String,
    /// Hexadecimal layout hash; empty when there is no layout.
    #[wasm_bindgen(js_name = layoutHash)]
    pub layout_hash: String,
    /// Diagnostics with source locations and snippets; empty when there are none.
    pub diagnostics: String,
    /// Whether compilation reported any errors.
    #[wasm_bindgen(js_name = hasErrors)]
    pub has_errors: bool,
    /// Normalized paths of every loaded file.
    pub files: Vec<String>,
}

/// TypeScript text of one `.weft` file for TypeScript's content mappers.
/// Flat arrays keep the boundary to plain strings and numbers.
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct ContentMapping {
    /// TypeScript text, parsed as a `.ts` file.
    pub text: String,
    /// Six numbers per `SpanMapping` tuple: virtual start and length,
    /// original start and length, `SpanMapKind`, `SpanMapFeature` bits.
    pub mappings: Vec<u32>,
    /// Message of each error in the file.
    #[wasm_bindgen(js_name = diagnosticMessages)]
    pub diagnostic_messages: Vec<String>,
    /// Start and length of each error in `diagnosticMessages`.
    #[wasm_bindgen(js_name = diagnosticSpans)]
    pub diagnostic_spans: Vec<u32>,
}

/// Maps the `.weft` file at `path`, whose unsaved text is `content`, to
/// TypeScript (see the `weft-ui-content-mapper` crate). Positions are in
/// `encoding`, `utf-8` or `utf-16`.
///
/// `load(path)` must return the source text of imported files. Thrown
/// exceptions become diagnostics of the importing file.
///
/// # Errors
///
/// Throws when `encoding` is unknown.
#[wasm_bindgen(js_name = mapContent)]
pub fn map_content(
    path: &str,
    content: &str,
    encoding: &str,
    load: &Function,
) -> Result<ContentMapping, JsError> {
    let encoding: PositionEncoding = encoding.parse()?;
    let result = weft_ui_content_mapper::transform(path, content, &mut JsLoader(load), encoding);
    let mappings = result
        .mappings
        .iter()
        .flat_map(|m| {
            [
                m.virtual_start,
                m.virtual_length,
                m.original_start,
                m.original_length,
                m.kind.into(),
                m.features,
            ]
        })
        .collect();
    let diagnostic_spans = result
        .diagnostics
        .iter()
        .flat_map(|d| [d.start, d.length])
        .collect();
    Ok(ContentMapping {
        text: result.text,
        mappings,
        diagnostic_messages: result.diagnostics.into_iter().map(|d| d.message).collect(),
        diagnostic_spans,
    })
}

/// Compiles `entries` and their imports as one app for `platform`.
///
/// `load(path)` must return source text. Thrown exceptions become compiler diagnostics.
///
/// # Errors
///
/// Throws when `platform` is unknown.
#[wasm_bindgen(js_name = compileApp)]
pub fn compile_app(
    entries: Vec<String>,
    platform: &str,
    load: &Function,
) -> Result<AppOutput, JsError> {
    let platform: Platform = platform.parse()?;
    let entries: Vec<&str> = entries.iter().map(String::as_str).collect();
    let parsed = weft_ui_parser::load(&entries, &mut JsLoader(load));
    let output = weft_ui_transformer::compile_app(parsed, platform);
    let compilation = &output.compilation;
    let (logic_sources, (logic_code, logic_dts)) = output
        .logic
        .into_iter()
        .map(|module| (module.source, (module.code, module.dts)))
        .unzip();
    let (view_paths, view_contents) = output
        .views
        .into_iter()
        .map(|file| (file.path, file.contents))
        .unzip();
    let (layout, layout_hash) = output.layout.as_ref().map_or_else(Default::default, |l| {
        (l.to_json(), format!("{:016x}", l.hash()))
    });
    Ok(AppOutput {
        logic_sources,
        logic_code,
        logic_dts,
        view_paths,
        view_contents,
        layout,
        layout_hash,
        diagnostics: compilation.render_diagnostics(),
        has_errors: compilation.has_errors(),
        files: compilation
            .sources
            .files()
            .iter()
            .map(|file| file.path().to_owned())
            .collect(),
    })
}
