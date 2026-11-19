//! Checks the compiler parser against the repository's tracked Weft fixtures.

use std::fs;
use std::path::Path;
use std::process::Command;

use weft_ui_ast::FileId;
use weft_ui_parser::parse;

#[test]
fn tracked_weft_fixtures_parse_without_diagnostics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("git")
        .current_dir(&root)
        .args(["ls-files", "-z", "--", "*.weft"])
        .output()
        .expect("run git ls-files to list Weft fixtures");
    assert!(
        output.status.success(),
        "git ls-files failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let fixtures = std::str::from_utf8(&output.stdout).expect("fixture paths are UTF-8");
    let mut failures = Vec::new();
    for fixture in fixtures.split_terminator('\0') {
        let name = fixture.rsplit_once('/').map_or(fixture, |(_, name)| name);
        if name.starts_with("Broken") {
            continue;
        }
        let source = fs::read_to_string(root.join(fixture))
            .unwrap_or_else(|error| panic!("read {fixture}: {error}"));
        let (_, diagnostics) = parse(FileId(0), &source);
        if !diagnostics.is_empty() {
            failures.push(format!("{fixture}:\n{diagnostics:#?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
