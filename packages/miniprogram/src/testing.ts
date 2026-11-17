/**
 * Headless stand-in for a generated view component, for tests that run
 * without WeChat DevTools.
 *
 * `loadView` imports the view component written by `build` (its `Component`
 * call is captured), records every `setData` payload, applies the paths to
 * the view data like the mini program framework does, and renders the
 * generated WXML against that data. The renderer covers the WXML subset the
 * compiler emits: `<template name/is/data>`, `wx:if` / `wx:elif` / `wx:else`,
 * `wx:for` with `wx:for-item`, and `{{path}}`, `{{path === n}}`, `{{true}}`
 * and `{{...path}}` expressions.
 */

import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

type Data = Record<string, unknown>;

/** A rendered element; text children are strings. */
export interface Element {
  tag: string;
  attrs: Data;
  children: (Element | string)[];
}

interface ViewOptions {
  data: Data;
  lifetimes: { created(this: unknown): void; detached(this: unknown): void };
  methods: { $e(this: unknown, event: object): unknown };
}

/** A mounted-looking view component instance. */
export interface TestView {
  data: Data;
  /** Every `setData` payload in order, deep-copied when it was sent. */
  calls: Data[];
  setData(data: Data): void;
  /** The view as the WXML would render it now. */
  render(): Element[];
  /** Text content of the rendered view, in document order. */
  texts(): string[];
  /** Rendered elements with tag `tag`, in document order. */
  all(tag: string): Element[];
  /** Fires the element's `bindtap`. */
  tap(element: Element): void;
  /** Fires `bindinput`; returns what the handler returned (the replacement text, if any). */
  input(element: Element, value: string): unknown;
  /** Fires `bindchange` with `value`. */
  change(element: Element, value: unknown): unknown;
  /** Runs the component's `detached` lifetime. */
  detach(): void;
}

/**
 * Loads the view component at `path` (without extension, e.g.
 * `out/TodoList-view`). Modules it imports are shared with the test, so
 * import root components from the same output directory.
 */
export async function loadView(path: string): Promise<TestView> {
  const scope = globalThis as { Component?: (options: ViewOptions) => void };
  const previous = scope.Component;
  let options: ViewOptions | undefined;
  scope.Component = (captured) => {
    options = captured;
  };
  try {
    // The path is only known at run time; a fresh query per load makes every
    // view module run and register its own options.
    await import(`${pathToFileURL(`${path}.js`).href}?view=${++loads}`);
  } finally {
    scope.Component = previous;
  }
  if (!options) throw new Error(`${path}.js did not call Component()`);
  const templates = parseTemplates(await readFile(`${path}.wxml`, "utf8"));
  return createView(options, templates);
}

let loads = 0;

function createView(options: ViewOptions, templates: Templates): TestView {
  const view: TestView & Data = {
    data: structuredClone(options.data),
    calls: [],
    setData(data) {
      const copy = structuredClone(data);
      view.calls.push(copy);
      for (const [path, value] of Object.entries(copy)) assign(view.data, path, value);
    },
    render: () => renderNodes(templates.root, view.data, templates),
    texts: () => texts(view.render()),
    all: (tag) => all(view.render(), tag),
    tap: (element) => {
      fire(view, element, "bindtap", undefined);
    },
    input: (element, value) => fire(view, element, "bindinput", value),
    change: (element, value) => fire(view, element, "bindchange", value),
    detach: () => options.lifetimes.detached.call(view),
  };
  for (const [name, method] of Object.entries(options.methods)) view[name] = method;
  options.lifetimes.created.call(view);
  return view;
}

function fire(view: TestView & Data, element: Element, attr: string, value: unknown): unknown {
  const method = element.attrs[attr];
  if (typeof method !== "string") throw new Error(`<${element.tag}> has no ${attr}`);
  const handler = view[method];
  if (typeof handler !== "function") throw new Error(`the view has no method ${method}`);
  return handler.call(view, {
    currentTarget: { dataset: { h: element.attrs["data-h"] } },
    detail: { value },
  });
}

/** Applies a `setData` path such as `r.b8.l7[2].n3_0`. */
function assign(data: Data, path: string, value: unknown): void {
  const keys = path.match(/[^.[\]]+/g) ?? [];
  let target: Data = data;
  for (let i = 0; i < keys.length - 1; i++) {
    const next = target[keys[i]];
    if (next === null || typeof next !== "object") throw new Error(`setData path ${path} crosses a non-object`);
    target = next as Data;
  }
  target[keys[keys.length - 1]] = value;
}

/* ---- WXML ---------------------------------------------------------------- */

interface Source {
  tag: string;
  attrs: Record<string, string>;
  children: (Source | string)[];
}

interface Templates {
  named: Map<string, Source[]>;
  root: Source[];
}

function parseTemplates(wxml: string): Templates {
  const named = new Map<string, Source[]>();
  const root: Source[] = [];
  for (const node of parse(wxml)) {
    if (typeof node === "string") continue;
    if (node.tag === "template" && node.attrs.name !== undefined) named.set(node.attrs.name, node.children as Source[]);
    else root.push(node);
  }
  return { named, root };
}

function parse(wxml: string): (Source | string)[] {
  const top: Source = { tag: "", attrs: {}, children: [] };
  const stack = [top];
  const token = /<\/([\w-]+)>|<([\w-]+)((?:\s+[\w:.-]+(?:="[^"]*")?)*)\s*(\/?)>|([^<]+)/g;
  for (const match of wxml.matchAll(token)) {
    const [, close, open, attrText, selfClosing, text] = match;
    const parent = stack[stack.length - 1];
    if (close) {
      if (parent.tag !== close) throw new Error(`unbalanced </${close}> in WXML`);
      stack.pop();
    } else if (open) {
      const attrs: Record<string, string> = {};
      for (const [, name, value] of attrText.matchAll(/([\w:.-]+)(?:="([^"]*)")?/g)) attrs[name] = value ?? "";
      const node: Source = { tag: open, attrs, children: [] };
      parent.children.push(node);
      if (!selfClosing) stack.push(node);
    } else if (text.trim() !== "") {
      parent.children.push(text);
    }
  }
  if (stack.length !== 1) throw new Error(`unclosed <${stack[stack.length - 1].tag}> in WXML`);
  return top.children;
}

/** Variables visible in a template: its data plus enclosing `wx:for` items. */
type Scope = Data;

function renderNodes(nodes: (Source | string)[], scope: Scope, templates: Templates): Element[] {
  const out: (Element | string)[] = [];
  renderInto(nodes, scope, templates, out);
  return out.filter((node): node is Element => typeof node !== "string");
}

function renderInto(nodes: (Source | string)[], scope: Scope, templates: Templates, out: (Element | string)[]): void {
  let chainTaken = false;
  for (const node of nodes) {
    if (typeof node === "string") {
      out.push(interpolate(node, scope));
      continue;
    }
    const { attrs } = node;
    if ("wx:if" in attrs) {
      chainTaken = Boolean(evaluate(attrs["wx:if"], scope));
      if (!chainTaken) continue;
    } else if ("wx:elif" in attrs) {
      if (chainTaken) continue;
      chainTaken = Boolean(evaluate(attrs["wx:elif"], scope));
      if (!chainTaken) continue;
    } else if ("wx:else" in attrs) {
      if (chainTaken) continue;
    }
    if ("wx:for" in attrs) {
      const items = evaluate(attrs["wx:for"], scope);
      const name = attrs["wx:for-item"] ?? "item";
      for (const item of Array.isArray(items) ? items : []) {
        renderElement(node, { ...scope, [name]: item }, templates, out);
      }
      continue;
    }
    renderElement(node, scope, templates, out);
  }
}

function renderElement(node: Source, scope: Scope, templates: Templates, out: (Element | string)[]): void {
  if (node.tag === "block") {
    renderInto(node.children, scope, templates, out);
    return;
  }
  if (node.tag === "template") {
    const name = evaluate(node.attrs.is, scope);
    const body = typeof name === "string" ? templates.named.get(name) : undefined;
    if (!body) throw new Error(`WXML template \`${String(name)}\` is not defined`);
    const data = node.attrs.data === undefined ? {} : evaluate(node.attrs.data, scope);
    renderInto(body, (data ?? {}) as Scope, templates, out);
    return;
  }
  const attrs: Data = {};
  for (const [name, value] of Object.entries(node.attrs)) {
    if (!name.startsWith("wx:")) attrs[name] = evaluate(value, scope);
  }
  const children: (Element | string)[] = [];
  renderInto(node.children, scope, templates, children);
  out.push({ tag: node.tag, attrs, children });
}

/** An attribute value: a lone `{{expr}}` keeps its type, anything else is a string. */
function evaluate(text: string, scope: Scope): unknown {
  const lone = /^\{\{(.*)\}\}$/.exec(text);
  if (lone && !lone[1].includes("}}")) return expression(lone[1].trim(), scope);
  return interpolate(text, scope);
}

function interpolate(text: string, scope: Scope): string {
  return text.replace(/\{\{(.*?)\}\}/g, (_, expr: string) => {
    const value = expression(expr.trim(), scope);
    return value === undefined || value === null ? "" : String(value);
  });
}

function expression(expr: string, scope: Scope): unknown {
  if (expr.startsWith("...")) return path(expr.slice(3), scope);
  const comparison = /^(.+?)\s*===\s*(-?\d+)$/.exec(expr);
  if (comparison) return path(comparison[1], scope) === Number(comparison[2]);
  if (expr === "true" || expr === "false") return expr === "true";
  return path(expr, scope);
}

function path(expr: string, scope: Scope): unknown {
  let value: unknown = scope;
  for (const key of expr.match(/[^.[\]]+/g) ?? []) {
    if (value === null || typeof value !== "object") return undefined;
    value = (value as Data)[key];
  }
  return value;
}

function texts(nodes: (Element | string)[]): string[] {
  return nodes.flatMap((node) => (typeof node === "string" ? [node] : texts(node.children)));
}

function all(nodes: (Element | string)[], tag: string): Element[] {
  return nodes.flatMap((node) => {
    if (typeof node === "string") return [];
    const nested = all(node.children, tag);
    return node.tag === tag ? [node, ...nested] : nested;
  });
}
