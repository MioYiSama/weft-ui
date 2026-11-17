//! WebAssembly bindings for compiling Weft apps with a JavaScript source loader.

use std::io;

use js_sys::Function;
use wasm_bindgen::prelude::*;
use weft_ui_compiler::SourceLoader;
use weft_ui_compiler::codegen::Platform;

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
    /// Diagnostics with source locations and snippets; empty when there are none.
    pub diagnostics: String,
    /// Whether compilation reported any errors.
    #[wasm_bindgen(js_name = hasErrors)]
    pub has_errors: bool,
    /// Normalized paths of every loaded file.
    pub files: Vec<String>,
}

/// Compiles `entries` and their imports as one app for `platform`.
///
/// `load(path)` must return source text. Thrown exceptions become compiler diagnostics.
///
/// # Errors
///
/// Throws when `platform` is unknown or has no backend yet.
#[wasm_bindgen(js_name = compileApp)]
pub fn compile_app(
    entries: Vec<String>,
    platform: &str,
    load: &Function,
) -> Result<AppOutput, JsError> {
    let platform: Platform = platform.parse()?;
    let entries: Vec<&str> = entries.iter().map(String::as_str).collect();
    let output = weft_ui_driver::compile_app(&entries, platform, &mut JsLoader(load))?;
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
    Ok(AppOutput {
        logic_sources,
        logic_code,
        logic_dts,
        view_paths,
        view_contents,
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
