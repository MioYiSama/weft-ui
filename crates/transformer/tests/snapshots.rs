//! Generated sources of each fixture, per platform.
//!
//! The snapshots exist to review changes to generated code; behavior is
//! tested end to end in the JS packages. Run the corresponding fixture test
//! with `WEFT_UPDATE_SNAPSHOTS=1` to rewrite its snapshots.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use weft_ui_parser::{MemoryLoader, load};
use weft_ui_transformer::transform::Platform;
use weft_ui_transformer::{AppOutput, compile_app};

struct Fixture {
    name: &'static str,
    source: &'static str,
}

impl Fixture {
    fn compile(&self, platform: Platform) -> AppOutput {
        let file = format!("{}.weft", self.name);
        let mut loader = MemoryLoader::new();
        loader.insert(&file, self.source);
        let output = compile_app(load(&[&file], &mut loader), platform);
        assert!(
            output.compilation.diagnostics.is_empty(),
            "{}",
            output.compilation.render_diagnostics()
        );
        output
    }

    fn snapshot_dir(&self) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/snapshots")
            .join(self.name)
    }
}

const TODO_LIST: Fixture = Fixture {
    name: "TodoList",
    source: include_str!("fixtures/TodoList.weft"),
};
const GALLERY: Fixture = Fixture {
    name: "Gallery",
    source: include_str!("fixtures/Gallery.weft"),
};

/// Every generated file of the app, as the platform's output directory would
/// hold it: logic modules at `X.js` / `X.d.ts`, then the view files.
fn generated_files(output: AppOutput) -> BTreeMap<String, String> {
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

fn check(fixture: &Fixture, platform: Platform, generated: &BTreeMap<String, String>) {
    let dir = fixture.snapshot_dir().join(platform.as_str());
    if std::env::var_os("WEFT_UPDATE_SNAPSHOTS").is_some() {
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("snapshot directory is writable");
        for (path, contents) in generated {
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
        "generated files of {}/{platform}",
        fixture.name
    );
    for (path, contents) in generated {
        assert_eq!(
            contents, &stored[path],
            "{}/{platform}/{path}",
            fixture.name
        );
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
fn todo_list() {
    check_fixture(&TODO_LIST);
}

#[test]
fn gallery() {
    check_fixture(&GALLERY);
}

fn check_fixture(fixture: &Fixture) {
    let web = generated_files(fixture.compile(Platform::Web));
    check(fixture, Platform::Web, &web);

    let wechat = fixture.compile(Platform::Wechat);
    let layout = wechat
        .layout
        .as_ref()
        .expect("wechat has a layout")
        .to_json();
    check_layout(fixture, &layout);
    let wechat = generated_files(wechat);
    check(fixture, Platform::Wechat, &wechat);

    check_native(fixture, &wechat, &layout);
}

#[test]
fn color_transform_and_unified_wechat_view() {
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
        let output = compile_app(load(&["ui/C.weft"], &mut loader), platform);
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
    let output = compile_app(load(&["ui/C.weft"], &mut loader), Platform::Wechat);
    assert!(
        output.compilation.diagnostics.is_empty(),
        "{}",
        output.compilation.render_diagnostics()
    );
    assert!(output.logic[0].code.contains("from \"./D.weft\""));
}

/// The view protocol layout is the same for every non-Web platform.
fn check_layout(fixture: &Fixture, json: &str) {
    let path = fixture.snapshot_dir().join("layout.json");
    if std::env::var_os("WEFT_UPDATE_SNAPSHOTS").is_some() {
        fs::write(&path, json).expect("snapshot is writable");
        return;
    }
    let stored =
        fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    assert_eq!(json, stored, "{}/layout.json", fixture.name);
}

/// Native platforms run the mini program's logic modules with another sink,
/// next to the layout and the views of their backend.
fn check_native(fixture: &Fixture, wechat: &BTreeMap<String, String>, layout: &str) {
    let name = fixture.name;
    let js = format!("{name}.js");
    let dts = format!("{name}.d.ts");
    for (platform, view) in [
        (Platform::Macos, "Generated.swift"),
        (Platform::Android, "src/main/kotlin/Generated.kt"),
        (Platform::Windows, "Generated.cs"),
        (Platform::Linux, "Generated.cpp"),
    ] {
        let native = generated_files(fixture.compile(platform));
        let mut expected = [js.as_str(), dts.as_str(), "layout.json", view];
        expected.sort_unstable();
        assert_eq!(
            native.keys().collect::<Vec<_>>(),
            expected,
            "{name}/{platform}"
        );
        assert_eq!(native[&js], wechat[&js], "{name}/{platform}/{js}");
        assert_eq!(native[&dts], wechat[&dts], "{name}/{platform}/{dts}");
        assert_eq!(
            native["layout.json"], layout,
            "{name}/{platform}/layout.json"
        );
        // iOS and macOS share one SwiftUI backend; differences live in WeftKit.
        if platform == Platform::Macos {
            let ios = generated_files(fixture.compile(Platform::Ios));
            assert_eq!(ios, native, "{name}/ios differs from macos");
        }
        check(fixture, platform, &native);
    }
}
