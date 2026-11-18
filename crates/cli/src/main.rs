//! `weft`: command-line tools for Weft UI.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use weft_ui_compiler::SourceMap;
use weft_ui_compiler::diagnostic::render_all;
use weft_ui_fmt::{FormatError, Options};

const USAGE: &str = "\
Usage: weft <command> [options]

Commands:
  fmt [--check] [paths...]   Format .weft files in place (default path: `.`);
                             directories are searched recursively, honoring
                             .gitignore and skipping hidden files.
                             --check  list unformatted files and exit 1
                                      instead of writing
  fmt --stdin [--path <p>]   Format stdin to stdout; <p> names it in errors
  lsp                        Run the language server on stdin/stdout

Options:
  -h, --help                 Print this help
  -V, --version              Print the version
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("fmt") => fmt(&args[1..]),
        Some("lsp") => match weft_ui_lsp::run_stdio() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("weft lsp: {error}");
                ExitCode::FAILURE
            }
        },
        Some("-V" | "--version") => {
            println!("weft {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("-h" | "--help") => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => usage_error("expected a command"),
    }
}

fn usage_error(message: &str) -> ExitCode {
    eprintln!("weft: {message}\n\n{USAGE}");
    ExitCode::from(2)
}

fn fmt(args: &[String]) -> ExitCode {
    let mut check = false;
    let mut stdin = false;
    let mut stdin_path = None;
    let mut paths = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" => check = true,
            "--stdin" => stdin = true,
            "--path" => match args.next() {
                Some(path) => stdin_path = Some(path.clone()),
                None => return usage_error("`--path` needs a value"),
            },
            flag if flag.starts_with('-') => {
                return usage_error(&format!("unknown option `{flag}`"));
            }
            path => paths.push(PathBuf::from(path)),
        }
    }
    let options = Options::default();
    if stdin {
        if check || !paths.is_empty() {
            return usage_error("`--stdin` cannot be combined with `--check` or paths");
        }
        return fmt_stdin(stdin_path.as_deref().unwrap_or("<stdin>"), &options);
    }
    if stdin_path.is_some() {
        return usage_error("`--path` only applies to `--stdin`");
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    let mut files = Vec::new();
    for path in &paths {
        if let Err(error) = collect(path, &mut files) {
            eprintln!("error: {}: {error}", path.display());
            return ExitCode::FAILURE;
        }
    }
    let mut failed = false;
    let mut unformatted = false;
    for file in files {
        let display = file.display().to_string();
        let source = match std::fs::read_to_string(&file) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("error: {display}: {error}");
                failed = true;
                continue;
            }
        };
        match weft_ui_fmt::format(&source, &options) {
            Ok(formatted) if formatted == source => {}
            Ok(_) if check => {
                println!("{display}");
                unformatted = true;
            }
            Ok(formatted) => {
                if let Err(error) = std::fs::write(&file, formatted) {
                    eprintln!("error: {display}: {error}");
                    failed = true;
                }
            }
            Err(error) => {
                report(&display, source, &error);
                failed = true;
            }
        }
    }
    if failed || unformatted {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn fmt_stdin(path: &str, options: &Options) -> ExitCode {
    let mut source = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut source) {
        eprintln!("error: reading stdin: {error}");
        return ExitCode::FAILURE;
    }
    match weft_ui_fmt::format(&source, options) {
        Ok(formatted) => {
            if let Err(error) = io::stdout().write_all(formatted.as_bytes()) {
                eprintln!("error: writing stdout: {error}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            report(path, source, &error);
            ExitCode::FAILURE
        }
    }
}

fn report(path: &str, source: String, error: &FormatError) {
    match error {
        FormatError::Syntax(diagnostics) => {
            let mut sources = SourceMap::new();
            sources.add(path.to_owned(), source);
            eprintln!("{}", render_all(diagnostics, &sources));
        }
        FormatError::Internal(_) => eprintln!("error: {path}: {error}"),
    }
}

/// Adds `path` if it is a file, or every `.weft` file below it.
fn collect(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if !path.is_dir() {
        // Named explicitly: format it whatever its extension.
        std::fs::metadata(path)?;
        files.push(path.to_owned());
        return Ok(());
    }
    let mut found = Vec::new();
    for entry in ignore::WalkBuilder::new(path).build() {
        let entry = entry.map_err(io::Error::other)?;
        let path = entry.path();
        if entry.file_type().is_some_and(|t| t.is_file())
            && path.extension().is_some_and(|e| e == "weft")
        {
            found.push(path.to_owned());
        }
    }
    found.sort();
    files.append(&mut found);
    Ok(())
}
