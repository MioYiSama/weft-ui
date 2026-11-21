// @vitest-environment node
import { cp, mkdir, mkdtemp, readFile, readdir, rename, rm, symlink, writeFile } from "node:fs/promises";
import { realpathSync } from "node:fs";
import type { AddressInfo } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import weft from "@weft-ui/vite";
import { chromium } from "playwright";
import { createBuilder, createServer, preview } from "vite";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { launch } from "../src/miniprogram.ts";

const fixture = fileURLToPath(new URL("./fixtures/assets/", import.meta.url));
const modules = realpathSync(fileURLToPath(new URL("../node_modules", import.meta.url)));
let root: string;

beforeEach(async () => {
  root = await mkdtemp(join(tmpdir(), "weft-assets-"));
  await mkdir(join(root, "src"));
  await cp(join(fixture, "assets"), join(root, "res"), { recursive: true });
  await cp(join(fixture, "App.weft"), join(root, "src/App.weft"));
  await writeFile(join(root, "src/index.ts"), `
import { run } from "@weft-ui/platform";
import { App } from "./App.weft";
run(new App());
`);
  await writeFile(join(root, "index.html"), '<div id="app"></div><script type="module" src="/src/index.ts"></script>');
  await symlink(modules, join(root, "node_modules"), "junction");
});

afterEach(async () => {
  await rm(root, { recursive: true, force: true });
  vi.unstubAllEnvs();
});

function config() {
  return { root, configFile: false as const, logLevel: "silent" as const };
}

async function build(platform: "web" | "wechat", base = "/") {
  vi.stubEnv("WEFT_PLATFORM", platform);
  await (await createBuilder({
    ...config(), base,
    plugins: [weft({ assets: "res", platforms: [platform], app: { id: "com.example.assets" } })],
  })).buildApp();
}

function origin(address: AddressInfo | string | null): string {
  if (!address || typeof address === "string") throw new Error("Vite server has no TCP address");
  return `http://127.0.0.1:${address.port}`;
}

test.each(["/preview/", "./"])("Web emits assets and resolves images and fonts with base %s", async (base) => {
  await build("web", base);
  const output = join(root, "apps/web/weft-assets");
  expect((await readdir(output)).sort()).toEqual(["fonts", "logo.png", "logo@2x.png", "only_high@2x.png", "star.svg"]);
  expect((await readdir(join(output, "fonts"))).sort()).toEqual(["Inter-Bold.ttf", "Inter-Regular.ttf"]);
  const server = await preview({ ...config(), base, build: { outDir: "apps/web" }, preview: { host: "127.0.0.1", port: 0 } });
  try {
    const prefix = base === "./" ? "/" : base;
    const url = `${origin(server.httpServer.address())}${prefix}`;
    const browser = await chromium.launch();
    try {
      const page = await browser.newPage({ deviceScaleFactor: base === "./" ? 1 : 2 });
      await page.goto(url);
      const image = page.locator("img").first();
      const paths = ["logo.png", "logo@2x.png"].map((file) => base === "./" ? `${url}weft-assets/${file}` : `${prefix}weft-assets/${file}`);
      expect(await image.getAttribute("src")).toBe(paths[0]);
      expect(await image.getAttribute("srcset")).toBe(`${paths[0]} 1x, ${paths[1]} 2x`);
      const selected = base === "./" ? "logo.png" : "logo@2x.png";
      expect(await image.evaluate((node) => (node as HTMLImageElement).currentSrc)).toBe(`${url}weft-assets/${selected}`);
      expect(await image.evaluate((node) => getComputedStyle(node).objectFit)).toBe("contain");
      expect(await image.evaluate((node) => getComputedStyle(node).backgroundColor)).toBe("rgb(51, 68, 85)");
      const sizes = await page.locator("img").evaluateAll((nodes) => nodes.slice(1, 4).map((node) => [node.clientWidth, node.clientHeight]));
      expect(sizes).toEqual([[64, 40], [40, 25], [64, 40]]);
      const fonts = await page.evaluate(async () => {
        await document.fonts.ready;
        return Array.from(document.fonts, ({ family, weight, status }) => ({ family, weight, status })).sort((a, b) => a.weight.localeCompare(b.weight));
      });
      expect(fonts).toEqual([
        { family: "Inter", weight: "400", status: "loaded" },
        { family: "Inter", weight: "700", status: "loaded" },
      ]);
      expect(await page.locator("style[data-weft]").textContent()).toContain('font-family: "Inter"');
      expect(await page.locator("style[data-weft]").textContent()).toContain("font-display: block");
      await page.getByText("Toggle first image").click();
      expect(await image.getAttribute("src")).toBeNull();
      expect(await image.getAttribute("srcset")).toBeNull();
      expect(await image.evaluate((node) => getComputedStyle(node).objectFit)).toBe("fill");
      expect(await image.evaluate((node) => getComputedStyle(node).backgroundColor)).toBe("rgb(255, 187, 102)");
    } finally {
      await browser.close();
    }
  } finally {
    await server.close();
  }
}, 15_000);

test("WeChat copies assets, chooses density, and embeds Inter in the page stylesheet", async () => {
  await build("wechat");
  const project = join(root, "apps/wechat");
  const weftDir = join(project, "weft");
  expect((await readdir(join(weftDir, "assets"))).sort()).toEqual(["logo.png", "logo@2x.png", "only_high@2x.png", "star.svg"]);
  const wxss = await readFile(join(weftDir, "index.wxss"), "utf8");
  expect(wxss).toContain('@import "./view.wxss";');
  for (const [weight, file] of [[400, "Inter-Regular.ttf"], [700, "Inter-Bold.ttf"]] as const) {
    const data = (await readFile(join(fixture, "assets/fonts", file))).toString("base64");
    expect(wxss.includes(`font-family: "Inter"; font-weight: ${weight}; font-style: normal; src: url("data:font/ttf;base64,${data}")`)).toBe(true);
  }
  const app = launch(project, { wx: { getWindowInfo: () => ({ pixelRatio: 1.5 }) } });
  try {
    await Promise.resolve();
    const images = app.view.all("image");
    expect(images.map(({ attrs }) => attrs.src)).toEqual([
      "/weft/assets/logo@2x.png", "/weft/assets/logo@2x.png", "/weft/assets/logo@2x.png",
      "/weft/assets/only_high@2x.png", "/weft/assets/star.svg",
    ]);
    expect(images.map(({ attrs }) => attrs.mode)).toEqual(["aspectFit", "aspectFill", "aspectFill", "aspectFill", "aspectFill"]);
    expect(images[0]!.attrs.style).toContain("background-color:var(--w-image-placeholder)");
    for (const image of images.slice(0, 4)) app.view.fire(image, "load", { width: 128, height: 80 });
    const loaded = app.view.all("image");
    expect(loaded[0]!.attrs.style).not.toContain("background-color:var(--w-image-placeholder)");
    expect(loaded[1]!.attrs.style).toContain("width:64px;height:40px");
    expect(loaded[2]!.attrs.style).toContain("height:auto;aspect-ratio:128/80;width:40px");
    expect(loaded[3]!.attrs.style).toContain("width:64px;height:40px");
    app.view.tap(app.view.all("button")[0]!);
    await Promise.resolve();
    expect(app.view.all("image")[0]!.attrs).toMatchObject({ src: "", mode: "scaleToFill" });
    expect(app.view.all("image")[0]!.attrs.style).toContain("background-color:var(--w-image-placeholder)");
  } finally {
    app.unload();
  }
});

test("Linux passes the asset directory to weft_views and copies media and fonts", async () => {
test("Android packages density variants, VectorDrawables and Inter font faces", async () => {
  await cp(join(root, "res/images/logo@2x.png"), join(root, "res/images/logo@3x.png"));
  vi.stubEnv("WEFT_PLATFORM", "linux");
  vi.stubEnv("WEFT_PLATFORM", "android");
  vi.stubEnv("WEFT_NATIVE", "0");
  await (await createBuilder({
    ...config(),
    plugins: [weft({ assets: "res", platforms: ["linux"], app: { id: "com.example.assets" } })],
    plugins: [weft({ assets: "res", platforms: ["android"], app: { id: "com.example.assets" } })],
  })).buildApp();
  const weftDir = join(root, "apps/linux/weft");
  const cmake = await readFile(join(weftDir, "CMakeLists.txt"), "utf8");
  expect(cmake).toContain('weft_views("${CMAKE_CURRENT_LIST_DIR}/Generated.cpp" "${CMAKE_CURRENT_LIST_DIR}/bundle.hbc" "${CMAKE_CURRENT_LIST_DIR}/assets")');
  const directory = join(weftDir, "assets");
  const images = ["logo.png", "logo@2x.png", "logo@3x.png", "only_high@2x.png"];
  const fonts = ["Inter-Bold.ttf", "Inter-Regular.ttf"];
  expect((await readdir(directory)).sort()).toEqual(["fonts", ...images, "star.svg"]);
  expect((await readdir(join(directory, "fonts"))).sort()).toEqual(fonts);
  for (const [input, output] of [
    ...images.map((name) => [`images/${name}`, name]),
    ["icons/star.svg", "star.svg"],
    ...fonts.map((name) => [`fonts/${name}`, `fonts/${name}`]),
  ] as [string, string][]) {
    const copied = await readFile(join(directory, output));
    expect(copied.equals(await readFile(join(root, "res", input))), output).toBe(true);
  const res = join(root, "apps/android/weft/src/main/res");
  expect(await readdir(join(res, "drawable-mdpi"))).toEqual(["weft_logo.png"]);
  expect((await readdir(join(res, "drawable-xhdpi"))).sort()).toEqual(["weft_logo.png", "weft_only_high.png"]);
  expect(await readdir(join(res, "drawable-xxhdpi"))).toEqual(["weft_logo.png"]);
  const icon = await readFile(join(res, "drawable/weft_star.xml"), "utf8");
  expect(icon).toContain('<vector xmlns:android="http://schemas.android.com/apk/res/android"');
  expect(icon).toMatch(/android:pathData="[Mm][^"]+"/);
  const fonts = join(res, "font");
  expect((await readdir(fonts)).sort()).toEqual(["weft_inter.xml", "weft_inter_400.ttf", "weft_inter_700.ttf"]);
  const family = await readFile(join(fonts, "weft_inter.xml"), "utf8");
  for (const weight of [400, 700]) {
    expect(family).toContain(`<font android:font="@font/weft_inter_${weight}" android:fontWeight="${weight}" android:fontStyle="normal" />`);
  }
});

test("Windows copies asset content through Weft.targets", async () => {
  await cp(join(root, "res/images/logo@2x.png"), join(root, "res/images/logo@3x.png"));
  vi.stubEnv("WEFT_PLATFORM", "windows");
  vi.stubEnv("WEFT_NATIVE", "0");
  await (await createBuilder({
    ...config(),
    plugins: [weft({ assets: "res", platforms: ["windows"], app: { id: "com.example.assets" } })],
  })).buildApp();
  const weftDir = join(root, "apps/windows/weft");
  const targets = await readFile(join(weftDir, "Weft.targets"), "utf8");
  expect(targets).toContain(String.raw`<Content Include="$(MSBuildThisFileDirectory)assets\**\*" Link="weft\assets\%(RecursiveDir)%(Filename)%(Extension)" CopyToOutputDirectory="PreserveNewest" />`);
  const directory = join(weftDir, "assets");
  expect((await readdir(directory)).sort()).toEqual([
    "fonts", "index.json", "logo.scale-100.png", "logo.scale-200.png", "logo.scale-300.png", "only_high.scale-200.png", "star.svg",
  ]);
  expect((await readdir(join(directory, "fonts"))).sort()).toEqual(["Inter-Bold.ttf", "Inter-Regular.ttf"]);
  expect(JSON.parse(await readFile(join(directory, "index.json"), "utf8"))).toEqual({
    Images: {
      logo: [100, 200, 300].map((scale) => ({ File: `logo.scale-${scale}.png`, Scale: scale / 100 })),
      only_high: [{ File: "only_high.scale-200.png", Scale: 2 }],
      star: [{ File: "star.svg", Scale: 1 }],
    },
    Fonts: [
      { File: "Inter-Bold.ttf", Family: "Inter", Weight: 700, Italic: false },
      { File: "Inter-Regular.ttf", Family: "Inter", Weight: 400, Italic: false },
    ],
  });
});

test("dev server reloads after a variant is added to the configured asset directory", async () => {
  const server = await createServer({
    ...config(), base: "/dev/", plugins: [weft({ assets: "res" })], server: { host: "127.0.0.1", port: 0 },
  });
  try {
    await server.listen();
    const browser = await chromium.launch();
    try {
      const page = await browser.newPage();
      const connected = page.waitForEvent("console", { predicate: (message) => message.text().includes("[vite] connected") });
      await page.goto(`${origin(server.httpServer?.address() ?? null)}/dev/`);
      await connected;
      const image = page.locator("img").first();
      expect(await image.getAttribute("srcset")).not.toContain("3x");
      await Promise.all([
        page.waitForEvent("load"),
        cp(join(root, "res/images/logo@2x.png"), join(root, "res/images/logo@3x.png")),
      ]);
      expect(await image.getAttribute("srcset")).toContain("/dev/weft-assets/logo@3x.png 3x");
    } finally {
      await browser.close();
    }
  } finally {
    await server.close();
  }
}, 15_000);

test("rejects a name outside a-z, 0-9 and _", async () => {
  await rename(join(root, "res/images/logo.png"), join(root, "res/images/Bad-name.png"));
  await expect(build("web")).rejects.toThrow("weft: asset name `Bad-name` may only contain a-z, 0-9 and _");
});

test("rejects an image and an icon sharing a name", async () => {
  await rename(join(root, "res/icons/star.svg"), join(root, "res/icons/logo.svg"));
  await expect(build("wechat")).rejects.toThrow("weft: asset `logo` is both an image and an icon");
});

test("rejects fonts without a recognized filename weight", async () => {
  await rename(join(root, "res/fonts/Inter-Bold.ttf"), join(root, "res/fonts/Inter-Heavy.ttf"));
  await expect(build("web")).rejects.toThrow("weft: font file `Inter-Heavy.ttf` must end in -<Weight>[Italic]");
});
