//! WeChat mini program backend.
//!
//! The logic layer is the remote mode of `weft-ui-codegen-js`: [`module`]
//! generates the module of a `.weft` file, meant to be written to
//! [`remote_module_path`] next to its `.d.ts`, with the runtime at
//! [`RUNTIME`] below the output directory.
//!
//! The view layer ([`Wechat`]) is a view component `X-view.js` / `.json` /
//! `.wxml` per entry `X.weft`. Its WXML holds the templates of every
//! component of the app, so it can render any of them as the root.
//!
//! Module paths should be relative and stay below the output directory.

mod wxml;

use weft_ui_codegen_js::js;
use weft_ui_codegen_js::{JsModule, Mode, remote_module_path};
use weft_ui_compiler::codegen::{Codegen, Context, OutputFile};
use weft_ui_compiler::ir::Program;
use weft_ui_compiler::span::FileId;

/// Path of the runtime module (`@weft-ui/runtime`'s mini program build),
/// relative to the output directory.
pub const RUNTIME: &str = "weft-runtime/wx.js";

/// Generates the logic module of `file`, importing the runtime relative to
/// the module's [`remote_module_path`].
///
/// # Panics
///
/// Panics if `file` is not part of the program.
#[must_use]
pub fn module(program: &Program, file: FileId) -> JsModule {
    let source = &program.module(file).path;
    let runtime = js::relative_specifier(&remote_module_path(source), RUNTIME);
    weft_ui_codegen_js::generate(program, file, &runtime, Mode::Remote)
}

/// View components of the mini program backend.
#[derive(Debug, Clone, Copy, Default)]
pub struct Wechat;

impl Codegen for Wechat {
    fn generate(&self, cx: &Context<'_>) -> Vec<OutputFile> {
        let program = cx.program;
        let templates = wxml::templates(program);
        let names: Vec<String> = program
            .components
            .iter()
            .map(|c| js::string(&c.name))
            .collect();
        let names = names.join(", ");
        let mut files = Vec::with_capacity(program.entries.len() * 3);
        for &entry in &program.entries {
            let view = view_path(&program.module(entry).path);
            files.push(OutputFile {
                path: format!("{view}.js"),
                contents: format!(
                    "import {{ view }} from {};\n\nComponent(view([{names}]));\n",
                    js::string(&js::relative_specifier(&view, RUNTIME)),
                ),
            });
            files.push(OutputFile {
                path: format!("{view}.json"),
                contents: "{\n  \"component\": true\n}\n".to_owned(),
            });
            files.push(OutputFile {
                path: format!("{view}.wxml"),
                contents: format!(
                    "{templates}<block wx:if=\"{{{{t}}}}\"><template is=\"{{{{t}}}}\" data=\"{{{{...r}}}}\"/></block>\n"
                ),
            });
        }
        files
    }
}

/// Path of an entry's view component, without extension.
fn view_path(entry: &str) -> String {
    format!("{}-view", entry.strip_suffix(".weft").unwrap_or(entry))
}
