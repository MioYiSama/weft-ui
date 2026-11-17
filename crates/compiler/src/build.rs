//! Build-script integration for Rust hosts.
//!
//! [`compile`] is called from `build.rs`; the generated module is included
//! with `weft_ui::include_weft!`. This is the only part of the compiler that
//! touches the file system.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::SourceLoader;
use crate::codegen::rust;

/// Loads modules relative to the package root.
struct FsLoader {
    root: PathBuf,
}

impl SourceLoader for FsLoader {
    fn load(&mut self, path: &str) -> io::Result<String> {
        fs::read_to_string(self.root.join(path))
    }
}

/// Why [`compile`] failed.
#[non_exhaustive]
pub enum CompileError {
    /// The sources have errors; holds the rendered diagnostics.
    Diagnostics(String),
    /// A Cargo build-script environment variable is missing.
    MissingEnv(&'static str),
    /// Writing the generated module failed.
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Diagnostics(rendered) => {
                write!(f, "weft compilation failed:\n{rendered}")
            }
            CompileError::MissingEnv(name) => write!(
                f,
                "environment variable `{name}` is not set; call `compile` from a build script"
            ),
            CompileError::Io { path, source } => {
                write!(f, "cannot write `{}`: {source}", path.display())
            }
        }
    }
}

/// Same as `Display`, so `compile(..).unwrap()` in a build script prints
/// readable diagnostics.
impl fmt::Debug for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for CompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CompileError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Compiles the `.weft` module at `entry` (relative to the package root) and
/// its imports into `$OUT_DIR/weft/<file stem>.rs`.
///
/// Every loaded file is reported with `cargo:rerun-if-changed`, and warnings
/// are forwarded as `cargo:warning` lines.
///
/// # Errors
///
/// Fails when the sources have errors, when not run from a build script, or
/// when the output cannot be written.
pub fn compile(entry: impl AsRef<Path>) -> Result<(), CompileError> {
    let entry = entry.as_ref();
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or(CompileError::MissingEnv("CARGO_MANIFEST_DIR"))?;
    let out_dir = std::env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(CompileError::MissingEnv("OUT_DIR"))?;

    let entry_path = entry.to_string_lossy().replace('\\', "/");
    let mut loader = FsLoader { root };
    let compilation = crate::check(&entry_path, &mut loader);
    for file in compilation.sources.files() {
        println!(
            "cargo:rerun-if-changed={}",
            loader.root.join(file.path()).display()
        );
    }
    if compilation.has_errors() {
        return Err(CompileError::Diagnostics(compilation.render_diagnostics()));
    }
    if !compilation.diagnostics.is_empty() {
        for line in compilation.render_diagnostics().lines() {
            println!("cargo:warning={line}");
        }
    }
    let program = compilation
        .program
        .as_ref()
        .expect("a compilation without errors has a program");

    let stem = entry
        .file_stem()
        .map_or_else(|| "weft".into(), |s| s.to_string_lossy());
    let dir = out_dir.join("weft");
    let path = dir.join(format!("{stem}.rs"));
    let code = rust::generate(program);
    fs::create_dir_all(&dir)
        .and_then(|()| fs::write(&path, code))
        .map_err(|source| CompileError::Io { path, source })
}
