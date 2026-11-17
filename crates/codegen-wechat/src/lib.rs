//! WeChat mini program backend.
//!
//! The logic layer is the remote mode of `weft-ui-codegen-js`
//! (`remote_module`), with bare `@weft-ui/runtime/remote` imports for Vite
//! to resolve and bundle.
//!
//! The view layer ([`Wechat`]) generates files relative to the app's
//! `weft/` directory: `view.js`, `view.json`, and `view.wxml` define one
//! component containing every app template and the sink's layout table.
//! `index.js`, `index.json`, and `index.wxml` define the page hosting it.
//! The component and page use `getApp().weft`, registered by the bundled
//! `@weft-ui/platform` entry, without importing logic or runtime files.

mod elements;
mod table;
mod wxml;

use weft_ui_compiler::codegen::{Codegen, Context, OutputFile};

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
        vec![
            OutputFile {
                path: "view.js".to_owned(),
                contents: format!(
                    "Component(getApp().weft.view({}));\n",
                    table::object(program, layout).trim_end(),
                ),
            },
            OutputFile {
                path: "view.json".to_owned(),
                contents: "{\n  \"component\": true\n}\n".to_owned(),
            },
            OutputFile {
                path: "view.wxml".to_owned(),
                contents: format!(
                    "{templates}<block wx:if=\"{{{{t}}}}\"><template is=\"{{{{t}}}}\" data=\"{{{{...r}}}}\"/></block>\n"
                ),
            },
            OutputFile {
                path: "index.js".to_owned(),
                contents: concat!(
                    "Page({\n",
                    "  onReady() {\n",
                    "    getApp().weft.mount(this.selectComponent(\"#weft\"));\n",
                    "  },\n",
                    "  onUnload() {\n",
                    "    getApp().weft.unmount();\n",
                    "  }\n",
                    "});\n",
                )
                .to_owned(),
            },
            OutputFile {
                path: "index.json".to_owned(),
                contents: "{\n  \"usingComponents\": {\n    \"weft-view\": \"./view\"\n  }\n}\n"
                    .to_owned(),
            },
            OutputFile {
                path: "index.wxml".to_owned(),
                contents: "<weft-view id=\"weft\"/>\n".to_owned(),
            },
        ]
    }
}
