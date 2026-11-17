import { cpSync, existsSync, globSync, mkdirSync, readFileSync, readdirSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { type EnvironmentOptions, type Plugin, defaultClientConditions, defaultServerConditions } from "vite";
import { type AppResult, type Platform, compileApp } from "@weft-ui/compiler";

/** Every target platform, in the order of docs/PLAN-v2.md. */
export const PLATFORMS: readonly Platform[] = ["web", "wechat", "ios", "macos", "android", "windows", "linux"];

/** Options of the Weft Vite plugin. */
export interface WeftOptions {
  /** Platforms `vite build --app` builds when `WEFT_PLATFORM` is unset; `WEFT_PLATFORM` must name one of them. Defaults to every platform. */
  platforms?: Platform[];
  /** Values filled into the project templates copied to `apps/<platform>/` on the first build. */
  app?: {
    /** Bundle ID / package name. */
    id?: string;
    /** Display name; defaults to the name of the root directory. */
    name?: string;
  };
  /** Glob, relative to the root, of the `.weft` files forming the app-level program of non-Web platforms. */
  include?: string;
  /** Logic entry of non-Web platforms, relative to the root; Web builds start from `index.html`. */
  entry?: string;
  /** Writes `X.d.weft.ts` next to every `X.weft`; defaults to `true`. */
  dts?: boolean;
}

function writeDeclarations(path: string, declarations: string) {
  const output = join(dirname(path), `${basename(path, ".weft")}.d.weft.ts`);
  if (!existsSync(output) || readFileSync(output, "utf8") !== declarations) {
    writeFileSync(output, declarations);
  }
}

function* weftFiles(directory: string): Generator<string> {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.name.startsWith(".") || ["node_modules", "dist", "apps"].includes(entry.name)) continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) yield* weftFiles(path);
    else if (entry.isFile() && entry.name.endsWith(".weft")) yield path.replaceAll("\\", "/");
  }
}

function isPlatform(name: string): name is Platform {
  return (PLATFORMS as readonly string[]).includes(name);
}

/**
 * Platforms to build: `WEFT_PLATFORM` (one name or a comma-separated list),
 * else every configured platform for `vite build --app`, else `web`.
 */
function selectPlatforms(configured: readonly Platform[], app: boolean): Platform[] {
  const requested = process.env.WEFT_PLATFORM?.trim();
  if (!requested) return app ? [...configured] : ["web"];
  const names = requested.split(",").map((name) => name.trim()).filter(Boolean);
  for (const name of names) {
    if (!isPlatform(name)) {
      throw new Error(`WEFT_PLATFORM: unknown platform "${name}", expected one of: ${PLATFORMS.join(", ")}`);
    }
    if (!configured.includes(name)) {
      throw new Error(`WEFT_PLATFORM: "${name}" is not in the platforms option of weft(): ${configured.join(", ")}`);
    }
  }
  return [...new Set(names as Platform[])];
}

/** Output directory of the Vite build: the whole `apps/web/`, or the JS of the generated area. */
function outDir(root: string, platform: Platform): string {
  return platform === "web" ? join(root, "apps/web") : join(root, "apps", platform, "weft/js");
}

/**
 * Copies the platform's project template into `apps/<platform>/` unless the
 * directory already holds anything besides the generated area: after the
 * first build the template belongs to the user.
 */
function copyTemplate(root: string, platform: Platform, app: { id: string; name: string }) {
  const project = join(root, "apps", platform);
  if (existsSync(project) && readdirSync(project).some((name) => name !== "weft")) return;
  const template = fileURLToPath(new URL(`../templates/${platform}/`, import.meta.url));
  if (!existsSync(template)) throw new Error(`weft: there is no project template for ${platform} yet`);
  for (const file of globSync("**/*", { cwd: template, withFileTypes: true })) {
    if (!file.isFile()) continue;
    const source = join(file.parentPath, file.name);
    // npm drops `.gitignore` files from packages, so templates store them as `_gitignore`.
    const path = relative(template, source).replace(/(^|[\\/])_gitignore$/, "$1.gitignore");
    const target = join(project, path);
    mkdirSync(dirname(target), { recursive: true });
    if (!file.name.endsWith(".json")) {
      cpSync(source, target);
      continue;
    }
    // JSON templates hold placeholders inside string literals.
    const escape = (value: string) => JSON.stringify(value).slice(1, -1);
    writeFileSync(target, readFileSync(source, "utf8")
      .replaceAll("__WEFT_APP_NAME__", escape(app.name))
      .replaceAll("__WEFT_APP_ID__", escape(app.id)));
  }
}

/** Rewrites `apps/<platform>/weft/` apart from the bundle that Vite wrote into `js/`. */
function writeGenerated(root: string, platform: Platform, result: AppResult) {
  const generated = join(root, "apps", platform, "weft");
  for (const name of readdirSync(generated)) {
    if (name !== "js") rmSync(join(generated, name), { recursive: true, force: true });
  }
  for (const file of result.views) {
    const path = join(generated, file.path);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, file.contents);
  }
}

/**
 * Compiles `.weft` imports and builds one Vite environment per platform.
 *
 * `vite build` builds the platforms named by `WEFT_PLATFORM` (default `web`;
 * `vite build --app` without it builds every configured platform). Web is a
 * plain Vite app written to `apps/web/`. Every other platform compiles all
 * `.weft` files matched by `include` as one program, bundles `entry` into a
 * single file in `apps/<platform>/weft/js/`, writes the generated views
 * next to it, and copies the project template into `apps/<platform>/` on
 * the first build. Environments not named after a platform (dev server,
 * Vitest) compile for the Web.
 */
export default function weft(options: WeftOptions = {}): Plugin {
  const configured = options.platforms ?? [...PLATFORMS];
  const include = options.include ?? "src/**/*.weft";
  const entry = options.entry ?? "src/index.ts";
  let root = process.cwd();
  let selected: Platform[] = [];
  /** App-level compilation of each non-Web environment; inline plugins are shared by every environment. */
  const apps = new Map<string, AppResult>();

  const platformOf = (name: string): Platform => (isPlatform(name) ? name : "web");

  return {
    name: "weft",
    enforce: "pre",
    config(config, env) {
      // Module ids are real paths; `.weft` sources are matched against them.
      root = realpathSync(resolve(config.root ?? process.cwd()));
      if (env.command !== "build") return;
      // The CLI passes `builder: {}` for `--app`; the plugin's own builder below makes plain `vite build` an app build.
      selected = selectPlatforms(configured, config.builder !== undefined);
      return {
        builder: {},
        environments: Object.fromEntries(selected.map((platform) => [platform, {}])),
      };
    },
    configEnvironment(name, config) {
      const platform = platformOf(name);
      if (!isPlatform(name)) {
        // Vite makes every environment except `client` a server consumer by default.
        const server = (config.consumer ?? (name === "client" ? "client" : "server")) === "server";
        return {
          resolve: { conditions: [`weft-${platform}`, ...(server ? defaultServerConditions : defaultClientConditions)] },
          define: { __WEFT_PLATFORM__: JSON.stringify(platform) },
        };
      }
      // Platform apps run in a browser, a mini program logic layer or Hermes: never a server.
      const environment: EnvironmentOptions = {
        consumer: "client",
        resolve: { conditions: [`weft-${platform}`, ...defaultClientConditions] },
        define: { __WEFT_PLATFORM__: JSON.stringify(platform) },
      };
      if (platform === "web") return { ...environment, build: { outDir: outDir(root, platform) } };
      return {
        ...environment,
        build: {
          outDir: outDir(root, platform),
          emptyOutDir: true,
          copyPublicDir: false,
          // Mini program logic layers run ES2020; the Hermes syntax target is settled in P3.
          target: "es2020",
          rolldownOptions: {
            input: resolve(root, entry),
            output: {
              format: platform === "wechat" ? "cjs" : "iife",
              entryFileNames: "index.js",
              codeSplitting: false,
            },
          },
        },
      };
    },
    async buildApp(builder) {
      for (const platform of selected) await builder.build(builder.environments[platform]!);
    },
    async buildStart() {
      if (options.dts !== false) {
        for (const file of weftFiles(root)) {
          const output = await compileApp([file], "web", (path) => readFileSync(path, "utf8"));
          const logic = output.logic.find((module) => module.source === file);
          if (!output.hasErrors && logic) writeDeclarations(file, logic.dts);
        }
      }
      const platform = platformOf(this.environment.name);
      if (platform === "web") return;
      const entries = globSync(include, { cwd: root }).map((path) => resolve(root, path).replaceAll("\\", "/"));
      if (entries.length === 0) this.error(`weft: no .weft files match ${include} in ${root}`);
      const app = await compileApp(entries, platform, (path) => readFileSync(path, "utf8"));
      apps.set(this.environment.name, app);
      for (const file of app.files) this.addWatchFile(file);
      if (app.hasErrors) this.error(app.diagnostics);
      if (app.diagnostics) this.warn(app.diagnostics);
    },
    async transform(code, id) {
      const file = id.split("?", 1)[0]!.replaceAll("\\", "/");
      if (!file.endsWith(".weft")) return;
      const platform = platformOf(this.environment.name);
      if (platform !== "web") {
        const logic = apps.get(this.environment.name)?.logic.find((module) => module.source === file);
        if (!logic) this.error(`${file} is not part of the app; include it with the include option (${include})`);
        return { code: logic.code, map: null };
      }
      const output = await compileApp([file], "web", (path) => (path === file ? code : readFileSync(path, "utf8")));
      for (const loaded of output.files) this.addWatchFile(loaded);
      if (output.hasErrors) this.error(output.diagnostics);
      if (output.diagnostics) this.warn(output.diagnostics);
      const logic = output.logic.find((module) => module.source === file)!;
      if (options.dts !== false) writeDeclarations(file, logic.dts);
      return { code: logic.code, map: null };
    },
    writeBundle() {
      const platform = platformOf(this.environment.name);
      const app = apps.get(this.environment.name);
      if (platform === "web" || !app) return;
      copyTemplate(root, platform, {
        id: options.app?.id ?? "",
        name: options.app?.name ?? basename(root),
      });
      writeGenerated(root, platform, app);
    },
  };
}
