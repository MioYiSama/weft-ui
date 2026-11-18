//! Application-level compilation for Weft UI.
//!
//! [`compile_app`] checks every `.weft` file of an app as one program and
//! dispatches the target [`Platform`] to its backends: the JS logic modules
//! come from `weft-ui-codegen-js` (through the Web backend for `web`), the
//! view files from the platform's [`Codegen`] backend.

use weft_ui_codegen_compose::Compose;
use weft_ui_codegen_js::JsModule;
use weft_ui_codegen_swiftui::SwiftUi;
use weft_ui_codegen_wechat::Wechat;
use weft_ui_codegen_winui::WinUi;
use weft_ui_compiler::codegen::{Codegen, Context, OutputFile, Platform};
use weft_ui_compiler::ir::Program;
use weft_ui_compiler::view::ViewLayout;
use weft_ui_compiler::{Compilation, SourceLoader};

/// Result of [`compile_app`].
#[derive(Debug, Clone)]
pub struct AppOutput {
    /// Sources, diagnostics and, without errors, the program.
    pub compilation: Compilation,
    /// One logic module per `.weft` file, entries first; empty on errors.
    pub logic: Vec<JsModule>,
    /// View files relative to `apps/<platform>/weft/`; empty on errors.
    pub views: Vec<OutputFile>,
    /// View protocol layout shared by the logic modules and the view files;
    /// `None` on errors and for `web`.
    pub layout: Option<ViewLayout>,
}

/// Compiles `entries` and their imports as one program for `platform`.
///
/// Declaration names must be unique across the whole app. On compilation
/// errors the output holds only diagnostics.
///
/// Native platforms get the remote mode logic modules, `layout.json` (the
/// layout the native views implement, including its hash) and the views of
/// their backend: SwiftUI for iOS and macOS, Jetpack Compose for Android,
/// and WinUI 3 for Windows. Linux does not have a view backend yet.
pub fn compile_app(
    entries: &[&str],
    platform: Platform,
    loader: &mut dyn SourceLoader,
) -> AppOutput {
    let compilation = weft_ui_compiler::check_many(entries, loader);
    let mut output = AppOutput {
        compilation,
        logic: Vec::new(),
        views: Vec::new(),
        layout: None,
    };
    if let Some(program) = &output.compilation.program {
        (output.logic, output.views, output.layout) = generate(program, platform);
    }
    output
}

fn generate(
    program: &Program,
    platform: Platform,
) -> (Vec<JsModule>, Vec<OutputFile>, Option<ViewLayout>) {
    let files = program.modules.iter().map(|m| m.file);
    let remote = |layout: &ViewLayout| {
        files
            .clone()
            .map(|file| weft_ui_codegen_js::remote_module(program, layout, file))
            .collect()
    };
    match platform {
        // DOM code lives in the logic modules; there are no view files.
        Platform::Web => (
            files
                .clone()
                .map(|file| weft_ui_codegen_web::module(program, file))
                .collect(),
            Vec::new(),
            None,
        ),
        Platform::Wechat => {
            let layout = ViewLayout::new(program);
            let views = Wechat.generate(&Context::new(program, Some(&layout), platform));
            (remote(&layout), views, Some(layout))
        }
        Platform::Ios
        | Platform::Macos
        | Platform::Android
        | Platform::Windows
        | Platform::Linux => {
            let layout = ViewLayout::new(program);
            let mut views = vec![OutputFile {
                path: "layout.json".to_owned(),
                contents: layout.to_json(),
            }];
            let cx = Context::new(program, Some(&layout), platform);
            if matches!(platform, Platform::Ios | Platform::Macos) {
                views.extend(SwiftUi.generate(&cx));
            } else if platform == Platform::Android {
                views.extend(Compose.generate(&cx));
            } else if platform == Platform::Windows {
                views.extend(WinUi.generate(&cx));
            }
            (remote(&layout), views, Some(layout))
        }
    }
}
