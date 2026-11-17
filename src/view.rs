//! Static description of a component's view and the assets served for it.

use std::borrow::Cow;

/// JS runtime shared by all view modules (`@weft-ui/runtime`).
const RUNTIME_JS: &str = include_str!("js/index.js");
/// Page side of the host protocol (`@weft-ui/runtime/remote`).
const REMOTE_JS: &str = include_str!("js/remote.js");

/// One generated Web module of the import closure.
#[derive(Debug)]
pub struct Module {
    /// URL path, e.g. `/ui/TodoList.weft`.
    pub path: &'static str,
    pub code: &'static str,
}

/// Everything the WebView needs to show one root component.
#[derive(Debug)]
pub struct View {
    /// Window title.
    pub title: &'static str,
    /// URL path of the module exporting the component class.
    pub entry: &'static str,
    /// Export name of the component class.
    pub export: &'static str,
    /// Events forwarded to the host.
    pub events: &'static [&'static str],
    pub modules: &'static [Module],
}

/// A response body with its content type.
pub(crate) struct Asset {
    pub(crate) mime: &'static str,
    pub(crate) body: Cow<'static, [u8]>,
}

impl View {
    /// Page that loads the component and connects it to the host.
    pub(crate) fn index_html(&self) -> String {
        let entry = serde_json::to_string(self.entry).expect("strings serialize");
        let events = serde_json::to_string(self.events).expect("strings serialize");
        let mut title = String::with_capacity(self.title.len());
        for c in self.title.chars() {
            match c {
                '&' => title.push_str("&amp;"),
                '<' => title.push_str("&lt;"),
                c => title.push(c),
            }
        }
        format!(
            r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title}</title>
  </head>
  <body>
    <script type="module">
      import {{ connect }} from "/__weft/remote.js";
      import {{ {export} as App }} from {entry};
      connect(App, {events}, document.body, (message) => window.ipc.postMessage(message));
    </script>
  </body>
</html>
"#,
            export = self.export,
        )
    }

    /// Asset served at the URL `path`; `index_html` is the page for `/`.
    pub(crate) fn asset(&self, path: &str, index_html: &str) -> Option<Asset> {
        const JS: &str = "text/javascript; charset=utf-8";
        let (mime, body): (&str, Cow<'static, [u8]>) = match path {
            "/" | "/index.html" => (
                "text/html; charset=utf-8",
                Cow::Owned(index_html.as_bytes().to_vec()),
            ),
            "/__weft/index.js" => (JS, Cow::Borrowed(RUNTIME_JS.as_bytes())),
            "/__weft/remote.js" => (JS, Cow::Borrowed(REMOTE_JS.as_bytes())),
            _ => {
                let module = self.modules.iter().find(|m| m.path == path)?;
                (JS, Cow::Borrowed(module.code.as_bytes()))
            }
        };
        Some(Asset { mime, body })
    }
}
