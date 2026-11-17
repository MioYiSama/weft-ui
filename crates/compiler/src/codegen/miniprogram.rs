//! WeChat mini program backend with the JavaScript host binding.
//!
//! [`generate`] turns an import closure into the files of a mini program
//! directory, at the module paths with `.weft` replaced:
//!
//! - every `.weft` file becomes an ES module `X.js` plus declarations
//!   `X.d.ts`, with the same models, events and root component classes as
//!   the Web backend (`mount` takes a view component instead of a DOM node);
//! - the entry also gets a view component `X-view.js` / `.json` / `.wxml`
//!   whose WXML holds the templates of every component in the closure, so it
//!   can render any of them as the root.
//!
//! Module paths should be relative and stay below the output directory; the
//! JavaScript runtime is expected at [`MiniprogramOptions::runtime`].

use crate::ir::Program;

use super::web::{self, Backend};

/// Options for [`generate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiniprogramOptions {
    /// Path of the runtime module (`@weft-ui/runtime`'s mini program build),
    /// relative to the output directory.
    pub runtime: String,
}

impl Default for MiniprogramOptions {
    fn default() -> Self {
        Self {
            runtime: "weft-runtime/wx.js".to_owned(),
        }
    }
}

/// A generated file, at a `/`-separated path relative to the output directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub path: String,
    pub contents: String,
}

/// Generates the modules of every file in the program's import closure and
/// the view component of its entry.
#[must_use]
pub fn generate(program: &Program, options: &MiniprogramOptions) -> Vec<File> {
    let mut files = Vec::with_capacity(program.modules.len() * 2 + 3);
    for module in &program.modules {
        let path = web::wx_module_path(&module.path);
        let runtime = web::relative_specifier(&path, &options.runtime);
        let generated = web::backend_module(program, module.file, &runtime, Backend::Wx);
        files.push(File {
            path: format!("{}.d.ts", path.strip_suffix(".js").unwrap_or(&path)),
            contents: generated.dts,
        });
        files.push(File {
            path,
            contents: generated.code,
        });
    }

    let entry = program
        .modules
        .iter()
        .find(|m| m.file == program.entry)
        .expect("the entry is part of the program");
    let view = view_path(&entry.path);
    let names: Vec<String> = program
        .components
        .iter()
        .map(|c| web::js_string(&c.name))
        .collect();
    files.push(File {
        path: format!("{view}.js"),
        contents: format!(
            "import {{ view }} from {};\n\nComponent(view([{}]));\n",
            web::js_string(&web::relative_specifier(&view, &options.runtime)),
            names.join(", ")
        ),
    });
    files.push(File {
        path: format!("{view}.json"),
        contents: "{\n  \"component\": true\n}\n".to_owned(),
    });
    files.push(File {
        path: format!("{view}.wxml"),
        contents: format!(
            "{}<block wx:if=\"{{{{t}}}}\"><template is=\"{{{{t}}}}\" data=\"{{{{...r}}}}\"/></block>\n",
            web::wxml_templates(program)
        ),
    });
    files
}

/// Path of the entry's view component, without extension.
fn view_path(entry: &str) -> String {
    format!("{}-view", entry.strip_suffix(".weft").unwrap_or(entry))
}
