/**
 * WeChat mini program sink: translates view protocol ops into `setData`
 * paths, and the generated view component that hosts it.
 *
 * The sink keeps one plain data object per scope, nested like the view: a
 * component's WXML template renders exactly that tree. The generated table
 * (`weft-layout.js`) maps every slot, event and mount point of a layout
 * template to a data key. At the end of every batch the changed keys become
 * `setData` paths such as `r.b8.l7[2].n3_0`, and paths below another changed
 * path are dropped. Event handlers stay in the logic layer: the view only
 * carries handler ids (`data-e<type>`) that the sink maps back to
 * `(scope, event slot)`.
 */

import { declarations } from "./css.ts";
import { CSS_PROPS } from "./elements.ts";
import { sameValue, type Session, type Sink, type Value } from "./remote.ts";

type Data = Record<string, unknown>;

/** Data keys of one layout template (see `crates/transformer/src/wechat/table.rs`). */
export interface TemplateKeys {
  /** WXML template rendering a component body as the root. */
  n?: string;
  /** WXML template of slot content, stored in its data as `t`. */
  t?: string;
  /** Branch index of an `if` branch template. */
  i?: number;
  /** Value key, optional CSS mapping index, and optional picker binding index. */
  v?: [key: string, cssIndex?: number | null, picker?: number][];
  /** Handler key, controlled value slot or -1, and optional event behavior. */
  e?: [key: string, controls: number, kind?: "number" | "textarea"][];
  /** Label, selected index, options and optional placeholder data keys per picker. */
  q?: [label: string, selected: string, options: string, placeholder?: string][];
  /** Key per mount point; `[indexKey, dataKey]` for branches. */
  p?: (string | [string, string])[];
  /** Constants that cannot be inlined into WXML. */
  d?: Data;
}

/** The generated table of an app. */
export interface Layout {
  hash: string;
  /** Names of tokens defined in the view stylesheet. */
  theme?: readonly string[];
  templates: TemplateKeys[];
}

/** The generated view component instance (`Component(view(layout))`). */
export interface View {
  setData(data: Data): void;
  $sink?: WechatSink;
}

/** The event object of `bindtap` / `bindinput` / `bindchange`. */
interface ViewEvent {
  type: string;
  currentTarget: { dataset: Data };
  detail?: { value?: unknown };
}

interface Scope {
  id: number;
  keys: TemplateKeys;
  o: Data;
  /** The scope holding this one's data under `seg` (`b3`, `c5`, `y6`, or `l7[2]` for rows). */
  up: Scope | null;
  seg: string;
  /** Created in the current batch, so its data has not been sent yet. */
  fresh: boolean;
  dead: boolean;
  /** Mounted children by point: one scope, or the rows of a list. */
  kids: (Scope | Scope[] | null)[];
  /** Handler ids of the scope's event slots. */
  hs: number[];
}

/** Renders one app into a view component through `setData`. */
export class WechatSink implements Sink {
  #view: View;
  #layout: Layout;
  #theme: ReadonlySet<string> | null;
  #session: Session | null = null;
  #scopes = new Map<number, Scope>();
  #root: Scope | null = null;
  #rootFresh = false;
  #created: Scope[] = [];
  /** Changed data since the last commit: a scope and one of its keys, or `null` for the whole scope. */
  #pending: [Scope, string | null][] = [];
  /** Handler id → scope and event slot. */
  #handlers = new Map<number, [Scope, number]>();
  #lastHandler = 0;

  constructor(view: View, layout: Layout) {
    this.#view = view;
    this.#layout = layout;
    this.#theme = layout.theme ? new Set(layout.theme) : null;
  }

  open(session: Session): void {
    if (session.hash !== this.#layout.hash) {
      throw new Error(
        `weft: this view component was generated for another build of the app (layout ${this.#layout.hash}, app ${session.hash})`,
      );
    }
    this.#session?.detach();
    this.#reset();
    this.#session = session;
  }

  create(id: number, template: number): void {
    const keys = this.#layout.templates[template];
    const scope: Scope = { id, keys, o: {}, up: null, seg: "", fresh: true, dead: false, kids: [], hs: [] };
    const o = scope.o;
    if (keys.t !== undefined) o.t = keys.t;
    if (keys.d) Object.assign(o, keys.d);
    if (keys.e) {
      for (let slot = 0; slot < keys.e.length; slot++) {
        const handler = ++this.#lastHandler;
        this.#handlers.set(handler, [scope, slot]);
        scope.hs.push(handler);
        o[keys.e[slot][0]] = handler;
      }
    }
    if (keys.p) {
      for (const point of keys.p) {
        if (typeof point === "string") {
          if (point[0] === "l") o[point] = [];
        } else {
          o[point[0]] = -1;
          o[point[1]] = null;
        }
      }
    }
    this.#scopes.set(id, scope);
    this.#created.push(scope);
  }

  set(id: number, slot: number, value: Value): void {
    const scope = this.#scope(id);
    const [key, cssIndex, picker] = scope.keys.v![slot];
    const next = cssIndex == null ? value : declarations(CSS_PROPS[cssIndex], value, this.#theme);
    if (sameValue(scope.o[key], next)) return;
    scope.o[key] = next;
    if (!scope.fresh) this.#pend(scope, key);
    if (picker !== undefined) {
      const [label, selected, options, placeholder] = scope.keys.q![picker]!;
      const index = scope.o[selected] as number;
      const labels = scope.o[options] as readonly string[] | undefined;
      const text = labels?.[index] ?? (placeholder === undefined ? "" : scope.o[placeholder] ?? "");
      if (scope.o[label] !== text) {
        scope.o[label] = text;
        if (!scope.fresh) this.#pend(scope, label);
      }
    }
  }

  branch(id: number, point: number, child: number): void {
    const scope = this.#scope(id);
    const [indexKey, dataKey] = scope.keys.p![point] as [string, string];
    const old = scope.kids[point] as Scope | null | undefined;
    if (old) this.#destroy(old);
    const next = child === 0 ? null : this.#scope(child);
    scope.kids[point] = next;
    if (next) {
      next.up = scope;
      next.seg = dataKey;
    }
    scope.o[indexKey] = next ? next.keys.i! : -1;
    scope.o[dataKey] = next ? next.o : null;
    if (!scope.fresh) {
      this.#pend(scope, indexKey);
      this.#pend(scope, dataKey);
    }
  }

  /**
   * Rows appended behind unchanged ones are sent by index; any other change
   * of the row order sends the whole array. A row's `k` (`wx:key`) is its
   * scope id.
   */
  rows(id: number, point: number, children: readonly number[]): void {
    const scope = this.#scope(id);
    const key = scope.keys.p![point] as string;
    const old = (scope.kids[point] as Scope[] | undefined) ?? [];
    const rows = children.map((child) => this.#scope(child));
    const kept = new Set(rows);
    for (const row of old) if (!kept.has(row)) this.#destroy(row);
    let appended = rows.length >= old.length;
    for (let i = 0; appended && i < old.length; i++) appended = rows[i] === old[i];
    for (let i = 0; i < rows.length; i++) {
      const row = rows[i];
      row.up = scope;
      row.seg = `${key}[${i}]`;
      row.o.k = row.id;
    }
    scope.kids[point] = rows;
    if (appended) {
      const data = scope.o[key] as Data[];
      for (let i = old.length; i < rows.length; i++) {
        data.push(rows[i].o);
        if (!scope.fresh) this.#pend(rows[i], null);
      }
    } else {
      scope.o[key] = rows.map((row) => row.o);
      if (!scope.fresh) this.#pend(scope, key);
    }
  }

  attach(id: number, point: number, child: number): void {
    const next = this.#scope(child);
    if (id === 0) {
      next.seg = "r";
      this.#root = next;
      this.#rootFresh = true;
      return;
    }
    const scope = this.#scope(id);
    const key = scope.keys.p![point] as string;
    scope.kids[point] = next;
    next.up = scope;
    next.seg = key;
    scope.o[key] = next.o;
    if (!scope.fresh) this.#pend(scope, key);
  }

  /** Sends the batch with one `setData`. */
  commit(): void {
    const data = new Map<string, unknown>();
    for (const [scope, key] of this.#pending) {
      if (scope.dead) continue;
      const path = this.#locate(scope, key);
      if (path !== null) data.set(path, key === null ? scope.o : scope.o[key]);
    }
    if (this.#rootFresh && this.#root) {
      data.set("t", this.#root.keys.n);
      data.set("r", this.#root.o);
    }
    this.#pending = [];
    this.#rootFresh = false;
    for (const scope of this.#created) scope.fresh = false;
    this.#created = [];
    if (data.size > 0) this.#view.setData(uncovered(data));
  }

  close(): void {
    this.#session = null;
    this.#reset();
    this.#view.setData({ t: "", r: null });
  }

  /** The view went away: releases the app without touching the view. */
  detach(): void {
    const session = this.#session;
    this.#session = null;
    session?.detach();
    this.#reset();
  }

  /**
   * Dispatches a view event. Input can return replacement text from bindinput;
   * textarea and the other controls instead need two setData calls because
   * the view's data still holds the bound value.
   */
  event(event: ViewEvent): string | undefined {
    const found = this.#handlers.get(Number(event.currentTarget.dataset[`e${event.type}`]));
    const session = this.#session;
    if (!found || !session) return undefined;
    const [scope, slot] = found;
    const [, controls, kind] = scope.keys.e![slot]!;
    let detail: unknown;
    switch (event.type) {
      case "focus": detail = true; break;
      case "blur": detail = false; break;
      case "input":
      case "change":
      case "changing": detail = event.detail?.value; break;
    }
    // The selector's change event sends a string, even though its index is an Int.
    const value = kind === "number" ? Number(detail) : detail;
    if (session.dispatch(scope.id, slot, value) || scope.dead || controls < 0) return undefined;
    const key = scope.keys.v![controls][0];
    const bound = scope.o[key];
    if (typeof bound === "string" && kind !== "textarea") return bound;
    const path = this.#locate(scope, key);
    if (path !== null) {
      this.#view.setData({ [path]: value });
      this.#view.setData({ [path]: bound });
    }
    return undefined;
  }

  #scope(id: number): Scope {
    const scope = this.#scopes.get(id);
    if (!scope) throw new Error(`weft: unknown scope ${id}`);
    return scope;
  }

  #pend(scope: Scope, key: string | null): void {
    this.#pending.push([scope, key]);
  }

  /** The `setData` path of `key` in `scope` (the scope itself for `null`), or `null` when not mounted at the root. */
  #locate(scope: Scope, key: string | null): string | null {
    let path = key === null ? scope.seg : `${scope.seg}.${key}`;
    let top = scope;
    while (top.up) {
      top = top.up;
      path = `${top.seg}.${path}`;
    }
    return top === this.#root ? path : null;
  }

  /** Destroys a scope and its descendants. */
  #destroy(scope: Scope): void {
    scope.dead = true;
    this.#scopes.delete(scope.id);
    for (const handler of scope.hs) this.#handlers.delete(handler);
    for (const kid of scope.kids) {
      if (Array.isArray(kid)) for (const row of kid) this.#destroy(row);
      else if (kid) this.#destroy(kid);
    }
  }

  #reset(): void {
    if (this.#root) this.#destroy(this.#root);
    for (const scope of this.#scopes.values()) this.#destroy(scope);
    this.#root = null;
    this.#rootFresh = false;
    this.#created = [];
    this.#pending = [];
  }
}

/** Drops paths that lie inside another changed path. */
function uncovered(data: Map<string, unknown>): Data {
  const out: Data = {};
  outer: for (const [path, value] of data) {
    for (let i = 0; i < path.length; i++) {
      const c = path[i];
      if ((c === "." || c === "[") && data.has(path.slice(0, i))) continue outer;
    }
    out[path] = value;
  }
  return out;
}

/**
 * Options of the generated view component. The view releases its app when
 * it is detached from the page.
 */
export function view(layout: Layout) {
  return {
    data: { t: "", r: null },
    lifetimes: {
      created(this: View) {
        this.$sink = new WechatSink(this, layout);
      },
      detached(this: View) {
        this.$sink?.detach();
      },
    },
    methods: {
      $e(this: View, event: ViewEvent): string | undefined {
        return this.$sink?.event(event);
      },
    },
  };
}
