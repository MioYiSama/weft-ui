//! Checks Weft UI programs and emits code for platform backends.
//!
//! [`check`] takes a [`ParsedProgram`] (entry modules and their imports, as
//! loaded by `weft_ui_parser::load`), resolves and type checks it, and lowers
//! it to a typed [`ir::Program`] with dependency information for every
//! binding. [`compile_app`] dispatches this
//! IR to the backend modules for the selected [`transform::Platform`].
//! Platforms with a separate view layer share a [`view::ViewLayout`] between
//! their view files and the JS logic modules emitted by [`bridge`].

pub mod bridge;
pub mod color;
pub mod compose;
mod deps;
pub mod gtk;
pub mod ide;
pub mod ir;
mod report;
mod sema;
pub mod swiftui;
pub mod transform;
pub mod view;
pub mod web;
pub mod wechat;
pub mod winui;

use weft_ui_ast::{Diagnostic, ParsedProgram, SourceMap, diagnostic};

use crate::bridge::JsModule;
use crate::ir::Program;
use crate::transform::{Context, OutputFile, Platform};
use crate::view::ViewLayout;

pub use report::dependency_report;

/// Outcome of checking an import closure.
#[derive(Debug, Clone)]
pub struct Compilation {
    /// Source text and locations for checked modules.
    pub sources: SourceMap,
    /// Errors and warnings, ordered by file and position.
    pub diagnostics: Vec<Diagnostic>,
    /// Present only when no errors were reported.
    pub program: Option<ir::Program>,
}

impl Compilation {
    /// Whether any diagnostic is an error rather than a warning.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_error)
    }

    /// Renders all diagnostics with source snippets.
    #[must_use]
    pub fn render_diagnostics(&self) -> String {
        diagnostic::render_all(&self.diagnostics, &self.sources)
    }
}

/// Resolves and type checks a parsed import closure as one program, so
/// declaration names must be unique across all of its modules.
///
/// The returned diagnostics include those of `parsed`.
#[must_use]
pub fn check(parsed: ParsedProgram) -> Compilation {
    let analysis = sema::analyze(parsed);
    Compilation {
        sources: analysis.sources,
        diagnostics: analysis.diagnostics,
        program: analysis.program,
    }
}

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

/// Checks a parsed app (see [`check`]) and transforms it for `platform`.
///
/// On compilation errors, no logic modules or view files are generated.
///
/// Native platforms get the remote mode logic modules, `layout.json` (the
/// layout the native views implement, including its hash) and the views of
/// their backend: SwiftUI for iOS and macOS, Jetpack Compose for Android,
/// WinUI 3 for Windows, and GTK 4 for Linux.
#[must_use]
pub fn compile_app(parsed: ParsedProgram, platform: Platform) -> AppOutput {
    let compilation = check(parsed);
    let mut output = AppOutput {
        compilation,
        logic: Vec::new(),
        views: Vec::new(),
        layout: None,
    };
    if let Some(program) = &output.compilation.program {
        (output.logic, output.views, output.layout) = transform_program(program, platform);
    }
    output
}

fn transform_program(
    program: &Program,
    platform: Platform,
) -> (Vec<JsModule>, Vec<OutputFile>, Option<ViewLayout>) {
    let files = || program.modules.iter().map(|module| module.file);
    let remote = |layout: &ViewLayout| {
        files()
            .map(|file| crate::bridge::remote_module(program, layout, file))
            .collect()
    };
    match platform {
        // DOM code lives in the logic modules; there are no view files.
        Platform::Web => (
            files()
                .map(|file| crate::web::module(program, file))
                .collect(),
            Vec::new(),
            None,
        ),
        Platform::Wechat => {
            let layout = ViewLayout::new(program);
            let cx = Context::new(program, Some(&layout), platform);
            let views = crate::wechat::transform(&cx);
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
            views.extend(match platform {
                Platform::Ios | Platform::Macos => crate::swiftui::transform(&cx),
                Platform::Android => crate::compose::transform(&cx),
                Platform::Windows => crate::winui::transform(&cx),
                Platform::Linux => crate::gtk::transform(&cx),
                Platform::Web | Platform::Wechat => unreachable!("{platform} is not native"),
            });
            (remote(&layout), views, Some(layout))
        }
    }
}
