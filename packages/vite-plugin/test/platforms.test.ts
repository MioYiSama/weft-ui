// @vitest-environment node
import { existsSync, realpathSync } from "node:fs";
import { mkdir, mkdtemp, readFile, readdir, rm, stat, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { launch } from "@weft-ui/miniprogram/testing";
import { Window } from "happy-dom";
import { createBuilder } from "vite";
import { afterEach, beforeEach, expect, test } from "vitest";
import type { Platform } from "@weft-ui/compiler";
import { WEFT_ANDROID } from "../src/android.ts";
import weft from "../src/index.ts";
import { WEFT_WINUI } from "../src/windows.ts";

const APP = `model Model {
    clicked: Bool,
    tint: String
}

event Click
event Mount

component App(model: Model) {
    Column(onMount = Mount) {
        if (model.clicked) {
            Text("You clicked!")
        } else {
            Text("Hello, world!")
        }
        Text("tinted", color = model.tint)
        Button(onClick = Click) {
            Text("Click me")
        }
    }
}
`;

const INDEX = `import { App } from "./App.weft";
import { platform, run, storage } from "@weft-ui/platform";

const app = new App({ clicked: false, tint: "red" });
app.onMount(async (model) => {
  model.clicked = (await storage.get("clicked")) === "true";
});
app.onClick((model) => {
  model.clicked = true;
  void storage.set("clicked", "true");
  void storage.set("os", platform.os);
});
run(app);
`;

const HTML = `<!doctype html>
<html><body><div id="app"></div><script type="module" src="/src/index.ts"></script></body></html>
`;

let root: string;

beforeEach(async () => {
  root = await mkdtemp(join(tmpdir(), "weft-vite-"));
  await mkdir(join(root, "src"));
  await writeFile(join(root, "src/App.weft"), APP);
  await writeFile(join(root, "src/index.ts"), INDEX);
  await writeFile(join(root, "index.html"), HTML);
  // The project resolves @weft-ui/platform and @weft-ui/runtime through this package's dependencies.
  const modules = realpathSync(join(dirname(fileURLToPath(import.meta.url)), "../node_modules"));
  await symlink(modules, join(root, "node_modules"), "junction");
});

afterEach(async () => {
  await rm(root, { recursive: true, force: true });
});

async function build(platform: string | undefined, platforms: Platform[] = ["web", "wechat"]) {
  const previous = process.env.WEFT_PLATFORM;
  if (platform === undefined) delete process.env.WEFT_PLATFORM;
  else process.env.WEFT_PLATFORM = platform;
  try {
    const builder = await createBuilder({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [weft({ platforms, app: { id: "com.example.test", name: "Test \"App\"" } })],
    });
    await builder.buildApp();
  } finally {
    if (previous === undefined) delete process.env.WEFT_PLATFORM;
    else process.env.WEFT_PLATFORM = previous;
  }
}

/**
 * Lets the first flush and the storage round trips of a launched app run.
 * The bundle carries its own runtime copy, so there is no promise to await;
 * zero-delay macrotasks drain its microtask chains.
 */
async function settle() {
  for (let i = 0; i < 3; i++) {
    const { promise, resolve } = Promise.withResolvers<void>();
    setTimeout(resolve);
    await promise;
  }
}

/** The `wx` storage and request APIs, backed by a map. */
function fakeWx() {
  const stored = new Map<string, string>();
  type Callbacks = { key: string; data?: string; success?(result: object): void; fail?(error: object): void };
  return {
    stored,
    getStorage({ key, success, fail }: Callbacks) {
      if (stored.has(key)) success?.({ data: stored.get(key) });
      else fail?.({ errMsg: "getStorage:fail data not found" });
    },
    setStorage({ key, data, success }: Callbacks) {
      stored.set(key, data!);
      success?.({});
    },
    removeStorage({ key, success }: Callbacks) {
      stored.delete(key);
      success?.({});
    },
  };
}

test("an unset WEFT_PLATFORM builds the Web app into apps/web", async () => {
  await build(undefined);
  expect(existsSync(join(root, "apps/wechat"))).toBe(false);
  const html = await readFile(join(root, "apps/web/index.html"), "utf8");
  const script = /src="\/(assets\/[^"]+\.js)"/.exec(html)![1]!;

  const window = new Window();
  const { document } = window;
  document.body.innerHTML = '<div id="app"></div>';
  window.localStorage.setItem("clicked", "true");
  // The entry chunk has no imports; run it as a classic script in the window's context.
  window.eval(await readFile(join(root, "apps/web", script), "utf8"));
  await settle();
  const spans = [...document.querySelectorAll("#app span")];
  expect(spans.map((span) => span.textContent)).toEqual(["You clicked!", "tinted", "Click me"]);
  expect(spans[1]!.getAttribute("style")).toMatch(/color:\s*transparent/);
  await window.happyDOM.close();
});

test("WEFT_PLATFORM=wechat generates a mini program project that runs", async () => {
  await build("wechat");
  const project = join(root, "apps/wechat");
  expect((await readdir(project)).sort()).toEqual([".gitignore", "app.js", "app.json", "project.config.json", "weft"]);
  expect((await readdir(join(project, "weft"))).sort())
    .toEqual(["index.js", "index.json", "index.wxml", "js", "view.js", "view.json", "view.wxml"]);
  expect(await readdir(join(project, "weft/js"))).toEqual(["index.js"]);
  expect(JSON.parse(await readFile(join(project, "app.json"), "utf8")).window.navigationBarTitleText).toBe("Test \"App\"");
  expect(existsSync(join(root, "apps/web"))).toBe(false);

  const wx = fakeWx();
  const program = launch(project, { wx });
  await settle();
  const { view } = program;
  expect(view.texts()).toEqual(["Hello, world!", "tinted", "Click me"]);
  expect(String(view.all("text")[1]!.attrs.style)).toContain("color:transparent");

  view.tap(view.all("button")[0]!);
  await settle();
  expect(view.texts()).toEqual(["You clicked!", "tinted", "Click me"]);
  expect(Object.fromEntries(wx.stored)).toEqual({ clicked: "true", os: "wechat" });

  // A relaunch reads the stored state back on mount.
  program.unload();
  const relaunched = launch(project, { wx });
  await settle();
  expect(relaunched.view.texts()[0]).toBe("You clicked!");
});

test("the template is copied once, the generated area is rewritten on every build", async () => {
  await build("wechat");
  const project = join(root, "apps/wechat");
  await writeFile(join(project, "app.json"), '{"pages": ["weft/index"]}\n');
  await writeFile(join(project, "weft/stale.js"), "");
  await writeFile(join(project, "weft/js/stale.js"), "");
  await build("wechat");
  expect(await readFile(join(project, "app.json"), "utf8")).toBe('{"pages": ["weft/index"]}\n');
  expect(existsSync(join(project, "weft/stale.js"))).toBe(false);
  expect(existsSync(join(project, "weft/js/stale.js"))).toBe(false);
  expect(existsSync(join(project, "weft/view.wxml"))).toBe(true);
});

test("a comma-separated WEFT_PLATFORM builds each platform", async () => {
  await build("web, wechat");
  expect(existsSync(join(root, "apps/web/index.html"))).toBe(true);
  expect(existsSync(join(root, "apps/wechat/weft/js/index.js"))).toBe(true);
});

test("a native platform bundles the logic for the kernel and compiles it to Hermes bytecode", async () => {
  await build("linux", ["linux"]);
  const generated = join(root, "apps/linux/weft");
  expect((await readdir(generated)).sort()).toEqual(["bundle.hbc", "js", "layout.json"]);
  // The Hermes bytecode file magic, little-endian.
  const hbc = await readFile(join(generated, "bundle.hbc"));
  expect(hbc.readBigUInt64LE(0)).toBe(0x1f1903c103bc1fc6n);
  const layout = JSON.parse(await readFile(join(generated, "layout.json"), "utf8"));
  expect(layout.templates[0].label).toBe("App");
  // `@weft-ui/platform` resolves to the native implementation, which mounts through the kernel.
  const bundle = await readFile(join(generated, "js/index.js"), "utf8");
  expect(bundle).toContain("__weft");
  expect(bundle).toContain(layout.hash);
});

test("Apple platforms generate an Xcode project around the WeftApp package", async () => {
  const previous = process.env.WEFT_NATIVE;
  process.env.WEFT_NATIVE = "0";
  try {
    await build("ios", ["ios"]);
  } finally {
    if (previous === undefined) delete process.env.WEFT_NATIVE;
    else process.env.WEFT_NATIVE = previous;
  }
  const project = join(root, "apps/ios");
  expect((await readdir(project)).sort()).toEqual([".gitignore", "App", "App.xcodeproj", "UITests", "weft"]);
  expect((await readdir(join(project, "weft"))).sort())
    .toEqual(["Generated.swift", "Package.swift", "bundle.hbc", "js", "layout.json"]);
  const pbxproj = await readFile(join(project, "App.xcodeproj/project.pbxproj"), "utf8");
  expect(pbxproj).toContain('PRODUCT_NAME = "Test \\"App\\"";');
  expect(pbxproj).toContain('PRODUCT_BUNDLE_IDENTIFIER = "com.example.test";');
  const scheme = await readFile(join(project, "App.xcodeproj/xcshareddata/xcschemes/App.xcscheme"), "utf8");
  expect(scheme).toContain('BuildableName = "Test &quot;App&quot;.app"');
  // The views implement the layout the bundle was compiled with.
  const { hash } = JSON.parse(await readFile(join(project, "weft/layout.json"), "utf8"));
  expect(await readFile(join(project, "weft/Generated.swift"), "utf8")).toContain(`layoutHash: 0x${hash},`);
  expect(await readFile(join(project, "weft/Package.swift"), "utf8")).toMatch(/\.package\(path: ".*native\/apple"\)/);
});

test("Android generates a Gradle project around the generated module", async () => {
  const previous = process.env.WEFT_NATIVE;
  process.env.WEFT_NATIVE = "0";
  try {
    await build("android", ["android"]);
  } finally {
    if (previous === undefined) delete process.env.WEFT_NATIVE;
    else process.env.WEFT_NATIVE = previous;
  }
  const project = join(root, "apps/android");
  expect((await readdir(project)).sort())
    .toEqual([".gitignore", "app", "build.gradle.kts", "gradle", "gradle.properties", "gradlew", "gradlew.bat", "settings.gradle.kts", "weft"]);
  expect((await stat(join(project, "gradlew"))).mode & 0o111).not.toBe(0);
  expect((await readdir(join(project, "weft"))).sort())
    .toEqual(["build.gradle.kts", "js", "layout.json", "settings.gradle.kts", "src"]);
  expect(await readFile(join(project, "app/build.gradle.kts"), "utf8")).toContain('applicationId = "com.example.test"');
  expect(await readFile(join(project, "app/src/main/res/values/strings.xml"), "utf8"))
    .toContain('<string name="app_name">Test \\&quot;App\\&quot;</string>');
  // The bytecode is an asset of the generated module, the views implement the layout it was compiled with.
  const hbc = await readFile(join(project, "weft/src/main/assets/weft/bundle.hbc"));
  expect(hbc.readBigUInt64LE(0)).toBe(0x1f1903c103bc1fc6n);
  const { hash } = JSON.parse(await readFile(join(project, "weft/layout.json"), "utf8"));
  expect(await readFile(join(project, "weft/src/main/kotlin/Generated.kt"), "utf8")).toContain(`layoutHash = 0x${hash}uL`);
  // The shell is referenced relative to the project, so it builds wherever the workspace is.
  const settings = await readFile(join(project, "weft/settings.gradle.kts"), "utf8");
  const shell = relative(realpathSync(project), WEFT_ANDROID).replaceAll("\\", "/");
  expect(settings).toContain(`project(":weft-android").projectDir = settingsDir.resolve("${shell}")`);
  expect(existsSync(join(WEFT_ANDROID, "build.gradle.kts"))).toBe(true);
});

test("Windows generates a dotnet project around the generated views", async () => {
  const previous = process.env.WEFT_NATIVE;
  process.env.WEFT_NATIVE = "0";
  try {
    await build("windows", ["windows"]);
  } finally {
    if (previous === undefined) delete process.env.WEFT_NATIVE;
    else process.env.WEFT_NATIVE = previous;
  }
  const project = join(root, "apps/windows");
  expect((await readdir(project)).sort())
    .toEqual([".gitignore", "App.csproj", "App.xaml", "App.xaml.cs", "UITests", "app.manifest", "weft"]);
  expect((await readdir(join(project, "weft"))).sort())
    .toEqual(["Generated.cs", "Weft.targets", "bundle.hbc", "js", "layout.json"]);
  const csproj = await readFile(join(project, "App.csproj"), "utf8");
  expect(csproj).toContain("<Product>Test &quot;App&quot;</Product>");
  expect(csproj).toContain('<AssemblyMetadata Include="WeftAppId" Value="com.example.test" />');
  expect(await readFile(join(project, "App.xaml.cs"), "utf8")).toContain('Title = "Test \\"App\\""');
  // The views implement the layout the bundle was compiled with.
  const { hash } = JSON.parse(await readFile(join(project, "weft/layout.json"), "utf8"));
  expect(await readFile(join(project, "weft/Generated.cs"), "utf8")).toContain(`LayoutHash = 0x${hash}UL;`);
  // The shell is referenced relative to the generated area, so the project builds wherever the workspace is.
  const targets = await readFile(join(project, "weft/Weft.targets"), "utf8");
  const shell = relative(realpathSync(join(project, "weft")), WEFT_WINUI).replaceAll("/", "\\");
  expect(targets).toContain(`<Import Project="$(MSBuildThisFileDirectory)${shell}\\Weft.WinUI.targets" />`);
  expect(existsSync(join(WEFT_WINUI, "Weft.WinUI.targets"))).toBe(true);
});

test("compile errors fail the build with rendered diagnostics", async () => {
  await writeFile(join(root, "src/Broken.weft"), "component Broken() {\n    Text(1)\n}\n");
  await expect(build("wechat")).rejects.toThrow(/mismatched types: expected `String`, found `Int`[\s\S]*Broken\.weft:2:/);
});

test("WEFT_PLATFORM must name a configured platform", async () => {
  await expect(build("mac")).rejects.toThrow(/unknown platform "mac"/);
  await expect(build("ios")).rejects.toThrow(/"ios" is not in the platforms option/);
});
