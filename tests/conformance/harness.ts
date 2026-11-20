/**
 * Runs one scripted fixture on every platform that can run in Node: the
 * headless renderer (the view protocol's reference implementation), the
 * mini program (WXML interpreter) and the Web (happy-dom). Every platform's
 * view is reduced to the same neutral tree so they can be compared.
 */

import { mkdtemp } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { tick as webTick } from "@weft-ui/runtime";
import { ELEMENTS } from "@weft-ui/runtime/elements";
import { type Value, tick as remoteTick } from "@weft-ui/runtime/remote";
import { type Renderer, type ViewElement, render } from "@weft-ui/runtime/testing";
import { Window } from "happy-dom";
import { type Element as WxElement, compile, loadView } from "../src/miniprogram.ts";

const window = new Window();
// The DOM runtime parses its templates with the global `document`.
globalThis.document = window.document as unknown as Document;

/**
 * A generated root component. Scripts are typed with the Web class; the
 * mini program class has the same shape apart from `mount`'s target.
 */
export interface AppLike {
  readonly model: unknown;
  mount(target: never): void;
  unmount(): void;
}

export type AppClass<A extends AppLike> = new (model: A["model"]) => A;

/** User actions, addressing elements by type and document order. */
export interface Actions {
  click(type: string, index: number): void;
  /** Edits a controlled `Input` or `Checkbox`. */
  edit(type: string, index: number, value: string | boolean): void;
}

export interface Step<M> {
  name: string;
  run(actions: Actions, model: M): void;
}

export interface Script<A extends AppLike> {
  /** Directory holding the fixture's `.weft` files. */
  dir: string;
  /** Entry file, relative to `dir`. */
  entry: string;
  /** Root component name. */
  component: string;
  /** The root component as the importing build compiles it: for the Web in Vitest, in remote mode in native bundles. */
  App: AppClass<A>;
  model(): A["model"];
  /** Registers host handlers; `log` records what they see, compared across platforms. */
  setup(app: A, log: (entry: string) => void): void;
  steps: Step<A["model"]>[];
}

/** A platform-neutral element: its type, main values, and children. */
export interface Neutral {
  type: string;
  props: Record<string, Value>;
  children: Neutral[];
}

export interface Driver<A extends AppLike> extends Actions {
  name: string;
  app: A;
  /** What the host handlers logged. */
  log: string[];
  tree(): Neutral[];
  /** Waits for the flush scheduled by the last action. */
  settle(): Promise<void>;
}

/** Props compared across platforms; styling differs by design. */
const MAIN: Record<string, readonly string[]> = Object.fromEntries(
  Object.entries(ELEMENTS).flatMap(([name, { main }]) => main === null ? [] : [[name, [main]] as const]),
);

/** A raw platform element's neutral type and main props, with its raw children. */
interface Classified<R> {
  type: string;
  props: Record<string, Value>;
  children: readonly R[];
}

/** Reduces a platform tree; `classify` returns `null` for raw nodes to skip. */
function reduce<R>(
  roots: readonly R[],
  classify: (raw: R) => Classified<R> | null,
  found: [R, string][] = [],
): Neutral[] {
  return roots.flatMap((raw) => {
    const node = classify(raw);
    if (!node) return [];
    found.push([raw, node.type]);
    return [{ type: node.type, props: node.props, children: reduce(node.children, classify, found) }];
  });
}

function nth<R>(found: [R, string][], type: string, index: number): R {
  const raw = found.filter(([, t]) => t === type)[index]?.[0];
  if (raw === undefined) throw new Error(`no ${type} #${index}`);
  return raw;
}

/** Output of the mini program build shared by the headless and WXML drivers. */
export interface Built<A extends AppLike> {
  out: string;
  layout: string;
  App: AppClass<A>;
  tick: () => Promise<void>;
}

export async function buildFixture<A extends AppLike>(script: Script<A>): Promise<Built<A>> {
  const out = await mkdtemp(join(tmpdir(), "weft-conformance-"));
  const { layout } = await compile({ entries: [join(script.dir, script.entry)], outDir: out });
  const stem = script.entry.replace(/\.weft$/, "");
  // Generated modules exist only after the build.
  const module = await import(pathToFileURL(join(out, `${stem}.js`)).href);
  return { out, layout, App: module[script.component] as AppClass<A>, tick: remoteTick };
}

function start<A extends AppLike>(script: Script<A>, App: AppClass<A>, target: unknown): { app: A; log: string[] } {
  const log: string[] = [];
  const app = new App(script.model());
  script.setup(app, (entry) => log.push(entry));
  // Each platform's mount takes its own target type.
  app.mount(target as never);
  return { app, log };
}

/** The headless renderer; its batches are the golden ops. */
export function headless<A extends AppLike>(script: Script<A>, built: Built<A>): Driver<A> & { renderer: Renderer } {
  const renderer = render(built.layout);
  const { app, log } = start(script, built.App, renderer);
  const find = (type: string, index: number) => {
    const found: [ViewElement, string][] = [];
    reduce(renderer.tree(), classify, found);
    return nth(found, type, index);
  };
  const classify = (e: ViewElement) => ({
    type: e.type,
    props: Object.fromEntries((MAIN[e.type] ?? []).map((p) => [p, e.props[p]])),
    children: e.children,
  });
  return {
    name: "headless",
    renderer,
    app,
    log,
    tree: () => reduce(renderer.tree(), classify),
    click: (type, index) => renderer.click(find(type, index)),
    edit: (type, index, value) => {
      renderer.edit(find(type, index), value);
    },
    settle: built.tick,
  };
}

function wxClassify(e: WxElement | string): Classified<WxElement | string> | null {
  if (typeof e === "string") return null;
  const text = (node: WxElement | string): string =>
    typeof node === "string" ? node : node.children.map(text).join("");
  switch (e.tag) {
    case "view": {
      const column = String(e.attrs.style).includes("flex-direction:column");
      return { type: column ? "Column" : "Row", props: {}, children: e.children };
    }
    case "text":
      return { type: "Text", props: { value: text(e) }, children: [] };
    case "button":
      return { type: "Button", props: {}, children: e.children };
    case "input":
      return { type: "Input", props: { value: String(e.attrs.value ?? "") }, children: [] };
    case "switch":
      return { type: "Checkbox", props: { checked: e.attrs.checked === true }, children: [] };
    case "image":
      return { type: "Image", props: { src: String(e.attrs.src ?? "") }, children: [] };
    default:
      throw new Error(`unexpected WXML <${e.tag}>`);
  }
}

/** The mini program view component, rendered by the WXML interpreter. */
export async function wechat<A extends AppLike>(script: Script<A>, built: Built<A>): Promise<Driver<A>> {
  const view = loadView(built.out);
  const { app, log } = start(script, built.App, view);
  const find = (type: string, index: number) => {
    const found: [WxElement | string, string][] = [];
    reduce(view.render(), wxClassify, found);
    return nth(found, type, index) as WxElement;
  };
  return {
    name: "wechat",
    app,
    log,
    tree: () => reduce(view.render(), wxClassify),
    click: (type, index) => view.tap(find(type, index)),
    edit: (type, index, value) => {
      const element = find(type, index);
      if (type === "Input") view.input(element, value as string);
      else view.change(element, value);
    },
    settle: built.tick,
  };
}

function domClassify(e: Element): Classified<Element> {
  const children = [...e.children];
  switch (e.tagName) {
    case "DIV": {
      const column = (e as HTMLElement).style.flexDirection === "column";
      return { type: column ? "Column" : "Row", props: {}, children };
    }
    case "SPAN":
      return { type: "Text", props: { value: e.textContent ?? "" }, children: [] };
    case "BUTTON":
      return { type: "Button", props: {}, children };
    case "INPUT": {
      const input = e as HTMLInputElement;
      return input.type === "checkbox"
        ? { type: "Checkbox", props: { checked: input.checked }, children: [] }
        : { type: "Input", props: { value: input.value }, children: [] };
    }
    case "IMG":
      return { type: "Image", props: { src: e.getAttribute("src") ?? "" }, children: [] };
    default:
      throw new Error(`unexpected <${e.tagName}>`);
  }
}

/** The Web DOM backend in happy-dom. */
export function web<A extends AppLike>(script: Script<A>): Driver<A> {
  const target = document.createElement("div");
  document.body.append(target);
  const { app, log } = start(script, script.App, target);
  const find = (type: string, index: number) => {
    const found: [Element, string][] = [];
    reduce([...target.children], domClassify, found);
    return nth(found, type, index);
  };
  return {
    name: "web",
    app,
    log,
    tree: () => reduce([...target.children], domClassify),
    click: (type, index) => (find(type, index) as HTMLElement).click(),
    edit: (type, index, value) => {
      const input = find(type, index) as HTMLInputElement;
      if (type === "Input") input.value = value as string;
      else input.checked = value as boolean;
      // Use the event class of the happy-dom window that owns the element.
      const event = new window.Event(type === "Input" ? "input" : "change") as unknown as Event;
      input.dispatchEvent(event);
    },
    settle: webTick,
  };
}
