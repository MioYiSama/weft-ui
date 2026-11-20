//! WeChat mini program backend.
//!
//! The logic layer comes from
//! [`crate::bridge::remote_module`],
//! with bare `@weft-ui/runtime/remote` imports for Vite to resolve and bundle.
//!
//! The view layer ([`transform`]) generates files relative to the app's
//! `weft/` directory: the `view.*` files define one component containing
//! every app template, the sink's layout table, and the shared stylesheet.
//! The `index.*` files define the page hosting it.
//! The component and page use `getApp().weft`, registered by the bundled
//! `@weft-ui/platform` entry, without importing logic or runtime files.

mod elements;
mod table;
mod wxml;

use crate::elements::CSS;
use crate::style;
use crate::transform::{Context, OutputFile};

/// Generates the WeChat mini program's view component and host page.
///
/// # Panics
///
/// Panics without a layout in `cx`.
#[must_use]
pub fn transform(cx: &Context<'_>) -> Vec<OutputFile> {
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
                "{templates}<block wx:if=\"{{{{t}}}}\"><template is=\"{{{{t}}}}\" data=\"{{{{...r, wParent: ''}}}}\"/></block>\n"
            ),
        },
        OutputFile {
            path: "view.wxss".to_owned(),
            contents: format!("{CSS}{}", style::theme_styles(layout.theme())),
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
            contents: "<weft-view id=\"weft\" class=\"w-root\"/>\n".to_owned(),
        },
        OutputFile {
            path: "index.wxss".to_owned(),
            contents: "@import \"./view.wxss\";\npage{height:100%;overflow:hidden}\n".to_owned(),
        },
    ]
}
