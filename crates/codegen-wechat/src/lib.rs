//! WeChat mini program backend.
//!
//! The logic layer is the remote mode of `weft-ui-codegen-js`: [`module`]
//! generates the module of a `.weft` file, meant to be written to
//! [`remote_module_path`] next to its `.d.ts`, with the runtime at
//! [`RUNTIME`] below the output directory.
//!
//! The view layer ([`Wechat`]) is a view component `X-view.js` / `.json` /
//! `.wxml` per entry `X.weft`. Its WXML holds the templates of every
//! component of the app, so it can render any of them as the root. The
//! view protocol ops are translated into `setData` paths by the runtime's
//! WeChat sink, using the table in [`LAYOUT`] that maps the slots, events
//! and mount points of every layout template to WXML data keys.
//!
//! Module paths should be relative and stay below the output directory.

mod elements;
mod table;
mod wxml;

use weft_ui_codegen_js::js;
use weft_ui_codegen_js::{JsModule, Mode, remote_module_path};
use weft_ui_compiler::codegen::{Codegen, Context, OutputFile};
use weft_ui_compiler::ir::Program;
use weft_ui_compiler::span::FileId;
use weft_ui_compiler::view::ViewLayout;

/// Path of the generic block runtime (`@weft-ui/runtime/remote`'s mini
/// program build) imported by logic modules, relative to the output
/// directory.
pub const RUNTIME: &str = "weft-runtime/remote.js";

/// Path of the WeChat sink and view component runtime, relative to the
/// output directory.
pub const SINK: &str = "weft-runtime/wechat.js";

/// Path of the generated data key table, relative to the output directory.
pub const LAYOUT: &str = "weft-layout.js";

/// Generates the logic module of `file`, importing the runtime relative to
/// the module's [`remote_module_path`].
///
/// # Panics
///
/// Panics if `file` is not part of the program or `layout` was derived from
/// another program.
#[must_use]
pub fn module(program: &Program, layout: &ViewLayout, file: FileId) -> JsModule {
    let source = &program.module(file).path;
    let runtime = js::relative_specifier(&remote_module_path(source), RUNTIME);
    weft_ui_codegen_js::generate(program, file, &runtime, Mode::Remote(layout))
}

/// View components of the mini program backend.
#[derive(Debug, Clone, Copy, Default)]
pub struct Wechat;

impl Codegen for Wechat {
    /// # Panics
    ///
    /// Panics without a layout in `cx`.
    fn generate(&self, cx: &Context<'_>) -> Vec<OutputFile> {
        let program = cx.program;
        let layout = cx.layout.expect("the mini program view needs the layout");
        let templates = wxml::templates(program, layout);
        let mut files = Vec::with_capacity(program.entries.len() * 3 + 1);
        files.push(OutputFile {
            path: LAYOUT.to_owned(),
            contents: table::module(program, layout),
        });
        for &entry in &program.entries {
            let view = view_path(&program.module(entry).path);
            files.push(OutputFile {
                path: format!("{view}.js"),
                contents: format!(
                    "import {{ view }} from {};\nimport {{ layout }} from {};\n\nComponent(view(layout));\n",
                    js::string(&js::relative_specifier(&view, SINK)),
                    js::string(&js::relative_specifier(&view, LAYOUT)),
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
