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
    let output = compile_app(&["TodoList.weft"], platform, &mut loader);
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
            let path = dir.join(path);
            let parent = path.parent().expect("snapshot has a parent directory");
            fs::create_dir_all(parent).expect("snapshot directory is writable");
            fs::write(path, contents).expect("snapshot is writable");
        }
        return;
    }
    let mut stored = BTreeMap::new();
    read_snapshots(&dir, "", &mut stored);
    assert_eq!(
        generated.keys().collect::<Vec<_>>(),
        stored.keys().collect::<Vec<_>>(),
        "generated files of {platform}"
    );
    for (path, contents) in &generated {
        assert_eq!(contents, &stored[path], "{platform}/{path}");
    }
}

fn read_snapshots(dir: &Path, prefix: &str, stored: &mut BTreeMap<String, String>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display())) {
        let entry = entry.expect("snapshot directory is readable");
        let name = entry.file_name();
        let name = name.to_str().expect("snapshot names are UTF-8");
        let path = entry.path();
        let file_type = entry.file_type().expect("snapshot type is readable");
        if file_type.is_dir() {
            read_snapshots(&path, &format!("{prefix}{name}/"), stored);
        } else {
            let contents = fs::read_to_string(&path).expect("snapshot is readable");
            stored.insert(format!("{prefix}{name}"), contents);
        }
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

#[test]
fn color_codegen_and_unified_wechat_view() {
    let src = r##"
component C(color: String) {
  Column(background = color) {
    Row(background = "#AbC") {
      Text("x", color = color)
      Text("y", color = "#12345680")
    }
  }
}
"##;
    let mut loader = MemoryLoader::new();
    loader.insert("ui/C.weft", src);
    for platform in [Platform::Web, Platform::Wechat] {
        let output = compile_app(&["ui/C.weft"], platform, &mut loader);
        assert!(
            output.compilation.diagnostics.is_empty(),
            "{}",
            output.compilation.render_diagnostics()
        );
        let code = &output.logic[0].code;
        if platform == Platform::Web {
            assert_eq!(code.matches("$.color(").count(), 4);
            assert!(code.contains("background:#aabbcc"));
            assert!(code.contains("color:#12345680"));
            assert_eq!(code.matches("align-items:flex-start").count(), 2);
            continue;
        }
        assert!(code.starts_with("import * as $ from \"@weft-ui/runtime/remote\";"));
        let files: BTreeMap<&str, &str> = output
            .views
            .iter()
            .map(|file| (file.path.as_str(), file.contents.as_str()))
            .collect();
        assert_eq!(
            files.keys().copied().collect::<Vec<_>>(),
            [
                "index.js",
                "index.json",
                "index.wxml",
                "view.js",
                "view.json",
                "view.wxml"
            ]
        );
        assert!(files["view.js"].starts_with("Component(getApp().weft.view({\n"));
        assert!(files["view.js"].ends_with("}));\n"));
        assert_eq!(files["view.js"].matches(", 3]").count(), 2);
        assert!(!files["view.js"].contains("import "));
        assert!(!files["view.js"].contains("require("));
        assert!(files["view.wxml"].contains("background:#aabbcc"));
        assert!(files["view.wxml"].contains("color:#12345680"));
        assert_eq!(
            files["view.wxml"].matches("align-items:flex-start").count(),
            2
        );
        assert!(
            files["index.js"].contains("getApp().weft.mount(this.selectComponent(\"#weft\"));")
        );
        assert!(files["index.js"].contains("getApp().weft.unmount();"));
    }
}

#[test]
fn remote_imports_preserve_weft_extensions() {
    let mut loader = MemoryLoader::new();
    loader.insert("ui/C.weft", "import \"./D.weft\"\ncomponent C() { D() }");
    loader.insert("ui/D.weft", "component D() { Text(\"x\") }");
    let output = compile_app(&["ui/C.weft"], Platform::Wechat, &mut loader);
    assert!(
        output.compilation.diagnostics.is_empty(),
        "{}",
        output.compilation.render_diagnostics()
    );
    assert!(output.logic[0].code.contains("from \"./D.weft\""));
}

/// The view protocol layout is the same for every non-Web platform.
#[test]
fn layout() {
    let mut loader = MemoryLoader::new();
    loader.insert("TodoList.weft", TODO_LIST);
    let output = compile_app(&["TodoList.weft"], Platform::Wechat, &mut loader);
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
fn macos() {
    check(Platform::Macos);
}

#[test]
fn android() {
    check(Platform::Android);
}

#[test]
fn windows() {
    check(Platform::Windows);
}

/// iOS and macOS share one SwiftUI backend; platform differences live in WeftKit.
#[test]
fn ios_views_equal_macos_views() {
    assert_eq!(generate(Platform::Ios), generate(Platform::Macos));
}

/// Native platforms run the mini program's logic modules with another sink,
/// next to the layout and the views of their backend.
#[test]
fn native_platforms_share_the_remote_logic() {
    let wechat = generate(Platform::Wechat);
    for (platform, views) in [
        (Platform::Ios, &["Generated.swift"][..]),
        (Platform::Macos, &["Generated.swift"]),
        (Platform::Android, &["src/main/kotlin/Generated.kt"]),
        (Platform::Windows, &["Generated.cs"]),
        (Platform::Linux, &[]),
    ] {
        let native = generate(platform);
        let mut expected = vec!["TodoList.d.ts", "TodoList.js", "layout.json"];
        expected.extend(views);
        expected.sort_unstable();
        assert_eq!(native.keys().collect::<Vec<_>>(), expected, "{platform}");
        assert_eq!(native["TodoList.js"], wechat["TodoList.js"], "{platform}");
        assert_eq!(
            native["TodoList.d.ts"], wechat["TodoList.d.ts"],
            "{platform}"
        );
        let stored = include_str!("snapshots/TodoList/layout.json");
        assert_eq!(native["layout.json"], stored, "{platform}");
    }
}
