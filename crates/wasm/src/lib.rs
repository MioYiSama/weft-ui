//! WebAssembly bindings for compiling Weft modules with a JavaScript source loader.

use std::io;

use js_sys::Function;
use wasm_bindgen::prelude::*;
use weft_ui_compiler::SourceLoader;
use weft_ui_compiler::codegen::miniprogram::{self, MiniprogramOptions};
use weft_ui_compiler::codegen::web::{self, WebOptions};

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

/// Generated modules, rendered diagnostics, and all loaded source paths.
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct CompileOutput {
    /// JavaScript module, present only when compilation succeeds.
    pub code: Option<String>,
    /// TypeScript declarations, present only when compilation succeeds.
    pub dts: Option<String>,
    /// Diagnostics with source locations and snippets; empty when there are none.
    pub diagnostics: String,
    /// Whether compilation reported any errors.
    #[wasm_bindgen(js_name = hasErrors)]
    pub has_errors: bool,
    /// Normalized paths of every loaded file in the import closure.
    pub files: Vec<String>,
}

/// Checks an entry and generates Web bindings using the synchronous JavaScript loader.
///
/// `load(path)` must return source text. Thrown exceptions become compiler diagnostics.
/// `runtime` names the runtime module imported by the generated JavaScript.
#[wasm_bindgen]
#[must_use]
pub fn compile(entry: &str, load: &Function, runtime: &str) -> CompileOutput {
    let compilation = weft_ui_compiler::check(entry, &mut JsLoader(load));
    let has_errors = compilation.has_errors();
    let diagnostics = compilation.render_diagnostics();
    let files = compilation
        .sources
        .files()
        .iter()
        .map(|file| file.path().to_owned())
        .collect();
    let generated = compilation
        .program
        .as_ref()
        .filter(|_| !has_errors)
        .map(|program| {
            web::generate(
                program,
                &WebOptions {
                    runtime: runtime.to_owned(),
                },
            )
        });
    let (code, dts) = match generated {
        Some(module) => (Some(module.code), Some(module.dts)),
        None => (None, None),
    };
    CompileOutput {
        code,
        dts,
        diagnostics,
        has_errors,
        files,
    }
}

/// Generated mini program files, rendered diagnostics, and all loaded source paths.
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct MiniprogramOutput {
    /// Output paths relative to the output directory, present only on success.
    pub paths: Option<Vec<String>>,
    /// Output contents corresponding to `paths`, present only on success.
    pub contents: Option<Vec<String>>,
    /// Diagnostics with source locations and snippets; empty when there are none.
    pub diagnostics: String,
    /// Whether compilation reported any errors.
    #[wasm_bindgen(js_name = hasErrors)]
    pub has_errors: bool,
    /// Normalized paths of every loaded file in the import closure.
    pub files: Vec<String>,
}

/// Checks an entry and generates mini program files using a synchronous JavaScript loader.
///
/// `load(path)` must return source text. Thrown exceptions become compiler diagnostics.
/// Source paths must be relative to the output root. `runtime` is relative to that root.
#[wasm_bindgen(js_name = compileMiniprogram)]
#[must_use]
pub fn compile_miniprogram(entry: &str, load: &Function, runtime: &str) -> MiniprogramOutput {
    let compilation = weft_ui_compiler::check(entry, &mut JsLoader(load));
    let has_errors = compilation.has_errors();
    let diagnostics = compilation.render_diagnostics();
    let files = compilation
        .sources
        .files()
        .iter()
        .map(|file| file.path().to_owned())
        .collect();
    let generated = compilation
        .program
        .as_ref()
        .filter(|_| !has_errors)
        .map(|program| {
            miniprogram::generate(
                program,
                &MiniprogramOptions {
                    runtime: runtime.to_owned(),
                },
            )
        });
    let (paths, contents) = match generated {
        Some(files) => {
            let (paths, contents) = files
                .into_iter()
                .map(|file| (file.path, file.contents))
                .unzip();
            (Some(paths), Some(contents))
        }
        None => (None, None),
    };
    MiniprogramOutput {
        paths,
        contents,
        diagnostics,
        has_errors,
        files,
    }
}
