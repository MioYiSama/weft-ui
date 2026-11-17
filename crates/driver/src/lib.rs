//! Application-level compilation for Weft UI.
//!
//! [`compile_app`] checks every `.weft` file of an app as one program and
//! dispatches the target [`Platform`] to its backends: the JS logic modules
//! come from `weft-ui-codegen-js` (through the Web backend for `web`), the
//! view files from the platform's [`Codegen`] backend.

use std::fmt;

use weft_ui_codegen_js::JsModule;
use weft_ui_codegen_wechat::Wechat;
use weft_ui_compiler::codegen::{Codegen, Context, OutputFile, Platform};
use weft_ui_compiler::ir::Program;
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
}

/// The platform has no backend yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedPlatformError(Platform);

impl UnsupportedPlatformError {
    #[must_use]
    pub fn platform(self) -> Platform {
        self.0
    }
}

impl fmt::Display for UnsupportedPlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "platform `{}` has no backend yet", self.0)
    }
}

impl std::error::Error for UnsupportedPlatformError {}

/// Compiles `entries` and their imports as one program for `platform`.
///
/// Declaration names must be unique across the whole app. On compilation
/// errors the output holds only diagnostics.
///
/// # Errors
///
/// Fails before loading any source if `platform` has no backend yet.
pub fn compile_app(
    entries: &[&str],
    platform: Platform,
    loader: &mut dyn SourceLoader,
) -> Result<AppOutput, UnsupportedPlatformError> {
    let backend = Backend::of(platform)?;
    let compilation = weft_ui_compiler::check_many(entries, loader);
    let (logic, views) = match &compilation.program {
        Some(program) => backend.generate(program, platform),
        None => (Vec::new(), Vec::new()),
    };
    Ok(AppOutput {
        compilation,
        logic,
        views,
    })
}

/// Platforms with an implemented backend.
#[derive(Debug, Clone, Copy)]
enum Backend {
    Web,
    Wechat,
}

impl Backend {
    fn of(platform: Platform) -> Result<Self, UnsupportedPlatformError> {
        match platform {
            Platform::Web => Ok(Backend::Web),
            Platform::Wechat => Ok(Backend::Wechat),
            Platform::Ios
            | Platform::Macos
            | Platform::Android
            | Platform::Windows
            | Platform::Linux => Err(UnsupportedPlatformError(platform)),
        }
    }

    fn generate(self, program: &Program, platform: Platform) -> (Vec<JsModule>, Vec<OutputFile>) {
        let files = program.modules.iter().map(|m| m.file);
        match self {
            // DOM code lives in the logic modules; there are no view files.
            Backend::Web => (
                files
                    .map(|file| weft_ui_codegen_web::module(program, file))
                    .collect(),
                Vec::new(),
            ),
            Backend::Wechat => (
                files
                    .map(|file| weft_ui_codegen_wechat::module(program, file))
                    .collect(),
                Wechat.generate(&Context::new(program, platform)),
            ),
        }
    }
}
