//! Generated sources of the TodoList fixture, per platform.
//!
//! The snapshots exist to review changes to generated code; behavior is
//! tested end to end in the JS packages. Run with `WEFT_UPDATE_SNAPSHOTS=1`
//! to rewrite them.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use weft_ui_compiler::MemoryLoader;
use weft_ui_compiler::codegen::Platform;
use weft_ui_driver::compile_app;

const TODO_LIST: &str = include_str!("../../compiler/tests/fixtures/TodoList.weft");

/// Every generated file of the app, as the platform's output directory would
/// hold it: logic modules at `X.js` / `X.d.ts`, then the view files.
fn generate(platform: Platform) -> BTreeMap<String, String> {
    let mut loader = MemoryLoader::new();
    loader.insert("TodoList.weft", TODO_LIST);
    let output = compile_app(&["TodoList.weft"], platform, &mut loader)
        .unwrap_or_else(|err| panic!("{err}"));
    assert!(
        output.compilation.diagnostics.is_empty(),
        "{}",
        output.compilation.render_diagnostics()
    );
    let mut files = BTreeMap::new();
    for module in output.logic {
        let stem = module
            .source
            .strip_suffix(".weft")
            .unwrap_or(&module.source);
        files.insert(format!("{stem}.js"), module.code);
        files.insert(format!("{stem}.d.ts"), module.dts);
    }
    files.extend(output.views.into_iter().map(|f| (f.path, f.contents)));
    files
}

fn check(platform: Platform) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots/TodoList")
        .join(platform.as_str());
    let generated = generate(platform);
    if std::env::var_os("WEFT_UPDATE_SNAPSHOTS").is_some() {
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("snapshot directory is writable");
        for (path, contents) in &generated {
            fs::write(dir.join(path), contents).expect("snapshot is writable");
        }
        return;
    }
    let mut stored = BTreeMap::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display())) {
        let path = entry.expect("snapshot directory is readable").path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("snapshot names are UTF-8")
            .to_owned();
        let contents = fs::read_to_string(&path).expect("snapshot is readable");
        stored.insert(name, contents);
    }
    assert_eq!(
        generated.keys().collect::<Vec<_>>(),
        stored.keys().collect::<Vec<_>>(),
        "generated files of {platform}"
    );
    for (path, contents) in &generated {
        assert_eq!(contents, &stored[path], "{platform}/{path}");
    }
}

#[test]
fn web() {
    check(Platform::Web);
}

#[test]
fn wechat() {
    check(Platform::Wechat);
}

/// The view protocol layout is the same for every non-Web platform.
#[test]
fn layout() {
    let mut loader = MemoryLoader::new();
    loader.insert("TodoList.weft", TODO_LIST);
    let output = compile_app(&["TodoList.weft"], Platform::Wechat, &mut loader)
        .unwrap_or_else(|err| panic!("{err}"));
    let json = output.layout.expect("wechat has a layout").to_json();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/TodoList/layout.json");
    if std::env::var_os("WEFT_UPDATE_SNAPSHOTS").is_some() {
        fs::write(&path, &json).expect("snapshot is writable");
        return;
    }
    let stored =
        fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    assert_eq!(json, stored, "TodoList/layout.json");
}

#[test]
fn native_platforms_have_no_backend_yet() {
    let mut loader = MemoryLoader::new();
    let err = compile_app(&["TodoList.weft"], Platform::Ios, &mut loader)
        .expect_err("ios has no backend in P0");
    assert_eq!(err.to_string(), "platform `ios` has no backend yet");
}
