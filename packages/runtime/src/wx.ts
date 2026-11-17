/**
 * Runtime for weft-generated WeChat mini program modules.
 *
 * The logic layer keeps one plain data object per block (component instance,
 * `if` branch, `for` row, slot content), nested like the view: a component's
 * WXML template renders exactly that tree. Update functions store values
 * that changed with `Block.set`; at the end of every flush the changed keys
 * become `setData` paths such as `r.b8.l7[2].n3_0`, and paths below another
 * changed path are dropped. Event handlers stay in the logic layer: the view
 * only carries their ids (`data-h`), and `$e` of the view component looks
 * them up.
 */

import {
  AppBase,
  type Job,
  type Marker,
  type Shape,
  type Watch,
  currentEpoch,
  enqueue,
  flushSync,
  onCommit,
  schedule,
  watch,
} from "./core.ts";

export * from "./core.ts";

type Data = Record<string, unknown>;

/* ------------------------------------------------------------------------ */
/* Change tracking and handlers                                              */
/* ------------------------------------------------------------------------ */

/** Changed data since the last commit: a block and one of its keys, or `null` for the whole block. */
let pending: [Block, string | null][] = [];
/** Mounted apps whose whole view has not been sent yet. */
const fresh = new Set<App>();

function pend(block: Block, key: string | null): void {
  pending.push([block, key]);
  schedule();
}

interface Handler {
  /** Receives the element's value (`detail.value`); `null` for controlled elements without handler. */
  f: ((value: unknown) => void) | null;
  b: Block;
  /** Data key of the controlled value restored when the host rejects an edit. */
  k: string | null;
}

const handlers = new Map<number, Handler>();
let lastHandler = 0;

/**
 * The `setData` path of `key` in `block` (the block itself for `null`) and
 * the app it belongs to, or `null` when the block is not part of a mounted
 * view.
 */
function locate(block: Block, key: string | null): [App, string] | null {
  let path = key === null ? block.seg : `${block.seg}.${key}`;
  let top = block;
  while (top.up) {
    top = top.up;
    path = `${top.seg}.${path}`;
  }
  return top instanceof Instance && top.app.$root === top ? [top.app, path] : null;
}

/** Sends the changes of the flush to every view, one `setData` per view. */
function commit(): void {
  if (pending.length === 0 && fresh.size === 0) return;
  const changes = new Map<App, Map<string, unknown>>();
  const changesOf = (app: App): Map<string, unknown> => {
    let data = changes.get(app);
    if (!data) {
      data = new Map();
      changes.set(app, data);
    }
    return data;
  };
  const entries = pending;
  pending = [];
  for (const [block, key] of entries) {
    if (block.dead) continue;
    const found = locate(block, key);
    if (found) changesOf(found[0]).set(found[1], key === null ? block.o : block.o[key]);
  }
  for (const app of fresh) {
    const data = changesOf(app);
    data.set("t", app.$template);
    data.set("r", app.$root!.o);
  }
  fresh.clear();
  for (const [app, data] of changes) app.$view!.setData(uncovered(data));
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

onCommit(commit);

/* ------------------------------------------------------------------------ */
/* Blocks                                                                    */
/* ------------------------------------------------------------------------ */

const noop = (): void => {};

/**
 * Data of one block. `up` holds this block's data under the key `seg`
 * (`b3`, `c5`, `y6`, or `l7[2]` for rows), which forms the `setData` path.
 */
export class Block {
  o: Data = {};
  up: Block | null = null;
  seg = "";
  born = currentEpoch();
  dead = false;
  /** Set when the block was created through a component slot. */
  slot: Slot | null = null;
  /** Ids of the handlers registered by this block. */
  hs: number[] | null = null;
  /** Re-runs dirty bindings and walks nested blocks. */
  u: () => void = noop;
  /** Releases nested blocks, components, watches, and handlers. */
  x: () => void = noop;

  /** Places the data in `up`, which is still being built, under `seg`. */
  mount(up: Block, seg: string): void {
    this.up = up;
    this.seg = seg;
    up.o[seg] = this.o;
  }

  /** Stores a recomputed value; only changes are sent. */
  set(key: string, value: unknown): void {
    if (this.o[key] === value) return;
    this.o[key] = value;
    pend(this, key);
  }

  /** Registers an element's handler and stores its id under `key`. */
  on(key: string, f: ((value: unknown) => void) | null, controlled: string | null): void {
    const id = ++lastHandler;
    handlers.set(id, { f, b: this, k: controlled });
    (this.hs ??= []).push(id);
    this.o[key] = id;
  }

  update(): void {
    this.u();
  }

  destroy(): void {
    this.dead = true;
    this.x();
    if (this.hs) for (const id of this.hs) handlers.delete(id);
    this.slot?.live.delete(this);
  }
}

/** A block with its own dirty bits: a component instance or a `for` row. */
class ScopeBlock extends Block implements Job, Marker {
  d: number[];
  depth: number;
  queued = false;

  constructor(words: number, depth: number) {
    super();
    this.d = new Array<number>(words).fill(0);
    this.depth = depth;
  }

  mark(bit: number): void {
    this.d[bit >> 5] |= 1 << (bit & 31);
    enqueue(this);
  }

  flush(): void {
    for (const word of this.d) {
      if (word !== 0) {
        this.update();
        return;
      }
    }
  }

  override update(): void {
    this.u();
    this.d.fill(0);
  }
}

/** One item of a keyed `for` list; its key is stored as `k` for `wx:key`. */
export class Row extends ScopeBlock {
  item: unknown;
  w: Watch;

  constructor(words: number, depth: number, shape: Shape, item: unknown) {
    super(words, depth);
    this.item = item;
    this.w = watch(shape, this, item);
  }

  setItem(item: unknown): void {
    this.item = item;
    this.w.set(item);
  }

  override destroy(): void {
    this.w.dispose();
    super.destroy();
  }
}

/** A component instance: parameter and state values plus their watches. */
export class Instance extends ScopeBlock {
  app: App;
  p: unknown[];
  s: unknown[] = [];
  pw: (Watch | undefined)[] = [];
  sw: Watch[] = [];
  /** Caller-supplied block for the `Children` parameter. */
  children: Slot | null;

  constructor(app: App, words: number, depth: number, args: unknown[], children: Slot | null) {
    super(words, depth);
    this.app = app;
    this.p = args;
    this.children = children;
  }

  arg(index: number, value: unknown): void {
    this.p[index] = value;
    this.pw[index]?.set(value);
  }

  state(index: number, value: unknown): void {
    this.s[index] = value;
    this.sw[index].set(value);
  }

  override destroy(): void {
    for (const w of this.pw) w?.dispose();
    for (const w of this.sw) w.dispose();
    super.destroy();
  }
}

/**
 * Runtime of an `if / else if / else` chain: the active index is stored
 * under one key of `up` and the branch's data under another; `-1` renders
 * nothing.
 */
export class Branches {
  up: Block;
  indexKey: string;
  blockKey: string;
  make: (index: number) => Block | null;
  index = -1;
  block: Block | null = null;

  constructor(up: Block, indexKey: string, blockKey: string, make: (index: number) => Block | null) {
    this.up = up;
    this.indexKey = indexKey;
    this.blockKey = blockKey;
    this.make = make;
    up.o[indexKey] = -1;
    up.o[blockKey] = null;
  }

  set(index: number): void {
    if (index === this.index) return;
    this.index = index;
    this.block?.destroy();
    const block = this.make(index);
    this.block = block;
    if (block) {
      block.up = this.up;
      block.seg = this.blockKey;
    }
    this.up.set(this.indexKey, index);
    this.up.set(this.blockKey, block ? block.o : null);
  }

  update(): void {
    const block = this.block;
    if (block && block.born !== currentEpoch()) block.update();
  }

  destroy(): void {
    this.block?.destroy();
    this.block = null;
  }
}

/** Runtime of a keyed `for` loop; rows are stored as an array under `key` of `up`. */
export class List {
  up: Block;
  key: string;
  make: (item: unknown) => Row;
  keyOf: (item: unknown) => unknown;
  rows: Row[] = [];

  constructor(up: Block, key: string, make: (item: unknown) => Row, keyOf: (item: unknown) => unknown) {
    this.up = up;
    this.key = key;
    this.make = make;
    this.keyOf = keyOf;
    up.o[key] = [];
  }

  /**
   * Reconciles rows with `items` by key: reused rows receive the new item
   * (marking only the fields that changed), unmatched rows are destroyed and
   * new rows are created. Rows appended behind unchanged ones are sent by
   * index; any other change of the row order sends the whole array.
   */
  set(items: readonly unknown[]): void {
    const old = this.rows;
    const byKey = new Map<unknown, number>();
    for (let i = 0; i < old.length; i++) byKey.set(this.keyOf(old[i].item), i);

    const rows = new Array<Row>(items.length);
    const reused = new Uint8Array(old.length);
    const seen = new Set<unknown>();
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      const key = this.keyOf(item);
      if (seen.has(key)) throw new Error(`weft: duplicate key \`${String(key)}\` in for loop`);
      seen.add(key);
      const j = byKey.get(key);
      if (j !== undefined && !reused[j]) {
        reused[j] = 1;
        const row = old[j];
        if (row.item !== item) row.setItem(item);
        rows[i] = row;
      } else {
        const row = this.make(item);
        row.up = this.up;
        row.o.k = key;
        rows[i] = row;
      }
    }
    for (let j = 0; j < old.length; j++) {
      if (!reused[j]) old[j].destroy();
    }

    let appended = rows.length >= old.length;
    for (let i = 0; appended && i < old.length; i++) appended = rows[i] === old[i];
    for (let i = 0; i < rows.length; i++) rows[i].seg = `${this.key}[${i}]`;
    this.rows = rows;
    if (appended) {
      const data = this.up.o[this.key] as Data[];
      for (let i = old.length; i < rows.length; i++) {
        data.push(rows[i].o);
        pend(rows[i], null);
      }
    } else {
      this.up.set(
        this.key,
        rows.map((row) => row.o),
      );
    }
  }

  update(): void {
    const epoch = currentEpoch();
    for (const row of this.rows) {
      if (row.born !== epoch) row.update();
    }
  }

  destroy(): void {
    for (const row of this.rows) row.destroy();
    this.rows = [];
  }
}

/**
 * The caller's block for a component's `Children` parameter. The callee
 * creates instances where it renders the slot; the caller updates them. The
 * view renders them with the caller's template named in `t`.
 */
export class Slot {
  template: string;
  make: () => Block;
  live: Set<Block> = new Set();

  constructor(template: string, make: () => Block) {
    this.template = template;
    this.make = make;
  }

  create(): Block {
    const block = this.make();
    block.o.t = this.template;
    block.slot = this;
    this.live.add(block);
    return block;
  }

  update(): void {
    const epoch = currentEpoch();
    for (const block of this.live) {
      if (block.born !== epoch) block.update();
    }
  }
}

/* ------------------------------------------------------------------------ */
/* Host-facing component and view                                            */
/* ------------------------------------------------------------------------ */

/** The generated view component instance (`Component(view([...]))`). */
export interface View {
  setData(data: Data): void;
  /** Templates the view's WXML defines. */
  $names?: readonly string[];
  $app?: App | null;
}

type Create = (app: App, args: unknown[], children: Slot | null, depth: number) => Instance;

/** Base class of the generated root component classes. */
export class App extends AppBase {
  #create: Create;
  /** Template rendering the root component. */
  readonly $template: string;
  $root: Instance | null = null;
  $view: View | null = null;

  constructor(create: Create, args: unknown[], template: string) {
    super(args);
    this.#create = create;
    this.$template = template;
  }

  /** Renders into a view component; the first `setData` follows in a microtask. */
  mount(view: View): void {
    if (this.$root) throw new Error("weft: component is already mounted");
    if (typeof view?.setData !== "function") {
      throw new Error("weft: mount expects a view component instance, e.g. `this.selectComponent(\"#view\")`");
    }
    if (!view.$names?.includes(this.$template)) {
      throw new Error(
        `weft: this view component cannot render \`${this.$template}\`; use the view generated for an entry that imports it`,
      );
    }
    view.$app?.unmount();
    view.$app = this;
    this.$view = view;
    const root = this.#create(this, this.$args, null, 0);
    root.seg = "r";
    this.$root = root;
    fresh.add(this);
    schedule();
  }

  unmount(): void {
    const view = this.$view;
    if (!this.$detach() || !view) return;
    view.setData({ t: "", r: null });
  }

  /** Releases the root without touching the view; `false` if not mounted. */
  $detach(): boolean {
    const root = this.$root;
    if (!root) return false;
    this.$root = null;
    fresh.delete(this);
    if (this.$view?.$app === this) this.$view.$app = null;
    root.destroy();
    return true;
  }
}

/** The event object of `bindtap` / `bindinput` / `bindchange`. */
interface ViewEvent {
  currentTarget: { dataset: { h?: unknown } };
  detail?: { value?: unknown };
}

/**
 * Runs the handler of an element and flushes synchronously, so controlled
 * elements can be restored right away: a rejected text edit is replaced by
 * returning the bound value (`bindinput` replaces the input's content with
 * the return value), a rejected switch toggle is reverted by two `setData`
 * calls, because the view's data still holds the bound value.
 */
function dispatch(view: View, event: ViewEvent): string | undefined {
  const handler = handlers.get(Number(event.currentTarget.dataset.h));
  if (!handler || handler.b.dead) return undefined;
  const value = event.detail?.value;
  handler.f?.(value);
  flushSync();
  const { b: block, k: key } = handler;
  if (key === null || block.dead) return undefined;
  const bound = block.o[key];
  if (bound === value) return undefined;
  if (typeof bound === "string") return bound;
  const found = locate(block, key);
  if (found) {
    view.setData({ [found[1]]: value });
    view.setData({ [found[1]]: bound });
  }
  return undefined;
}

/**
 * Options of the generated view component: `names` lists the templates of
 * its WXML. The view releases its app when it is detached from the page.
 */
export function view(names: readonly string[]) {
  return {
    data: { t: "", r: null },
    lifetimes: {
      created(this: View) {
        this.$names = names;
      },
      detached(this: View) {
        this.$app?.$detach();
      },
    },
    methods: {
      $e(this: View, event: ViewEvent): string | undefined {
        return dispatch(this, event);
      },
    },
  };
}
