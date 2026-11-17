//! Prints the binding dependencies of a `.weft` module.
//!
//! Usage: `cargo run -p weft-ui-compiler --example deps -- path/to/File.weft`

use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs, io};

use weft_ui_compiler::{SourceLoader, check, dependency_report};

/// Loads modules from the file system, relative to the current directory.
struct FsLoader;

impl SourceLoader for FsLoader {
    fn load(&mut self, path: &str) -> io::Result<String> {
        fs::read_to_string(PathBuf::from(path))
    }
}

fn main() -> ExitCode {
    let Some(entry) = env::args().nth(1) else {
        eprintln!("usage: deps <entry.weft>");
        return ExitCode::FAILURE;
    };
    let compilation = check(&entry, &mut FsLoader);
    if !compilation.diagnostics.is_empty() {
        eprintln!("{}", compilation.render_diagnostics());
    }
    match &compilation.program {
        Some(program) => {
            print!("{}", dependency_report(program));
            ExitCode::SUCCESS
        }
        None => ExitCode::FAILURE,
    }
}
