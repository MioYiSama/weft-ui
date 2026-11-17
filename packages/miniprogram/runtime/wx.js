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
var __classPrivateFieldSet = (this && this.__classPrivateFieldSet) || function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _App_create;
import { AppBase, currentEpoch, enqueue, flushSync, onCommit, schedule, watch, } from "./core.js";
export * from "./core.js";
/* ------------------------------------------------------------------------ */
/* Change tracking and handlers                                              */
/* ------------------------------------------------------------------------ */
/** Changed data since the last commit: a block and one of its keys, or `null` for the whole block. */
let pending = [];
/** Mounted apps whose whole view has not been sent yet. */
const fresh = new Set();
function pend(block, key) {
    pending.push([block, key]);
    schedule();
}
const handlers = new Map();
let lastHandler = 0;
/**
 * The `setData` path of `key` in `block` (the block itself for `null`) and
 * the app it belongs to, or `null` when the block is not part of a mounted
 * view.
 */
function locate(block, key) {
    let path = key === null ? block.seg : `${block.seg}.${key}`;
    let top = block;
    while (top.up) {
        top = top.up;
        path = `${top.seg}.${path}`;
    }
    return top instanceof Instance && top.app.$root === top ? [top.app, path] : null;
}
/** Sends the changes of the flush to every view, one `setData` per view. */
function commit() {
    if (pending.length === 0 && fresh.size === 0)
        return;
    const changes = new Map();
    const changesOf = (app) => {
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
        if (block.dead)
            continue;
        const found = locate(block, key);
        if (found)
            changesOf(found[0]).set(found[1], key === null ? block.o : block.o[key]);
    }
    for (const app of fresh) {
        const data = changesOf(app);
        data.set("t", app.$template);
        data.set("r", app.$root.o);
    }
    fresh.clear();
    for (const [app, data] of changes)
        app.$view.setData(uncovered(data));
}
/** Drops paths that lie inside another changed path. */
function uncovered(data) {
    const out = {};
    outer: for (const [path, value] of data) {
        for (let i = 0; i < path.length; i++) {
            const c = path[i];
            if ((c === "." || c === "[") && data.has(path.slice(0, i)))
                continue outer;
        }
        out[path] = value;
    }
    return out;
}
onCommit(commit);
/* ------------------------------------------------------------------------ */
/* Blocks                                                                    */
/* ------------------------------------------------------------------------ */
const noop = () => { };
/**
 * Data of one block. `up` holds this block's data under the key `seg`
 * (`b3`, `c5`, `y6`, or `l7[2]` for rows), which forms the `setData` path.
 */
export class Block {
    constructor() {
        this.o = {};
        this.up = null;
        this.seg = "";
        this.born = currentEpoch();
        this.dead = false;
        /** Set when the block was created through a component slot. */
        this.slot = null;
        /** Ids of the handlers registered by this block. */
        this.hs = null;
        /** Re-runs dirty bindings and walks nested blocks. */
        this.u = noop;
        /** Releases nested blocks, components, watches, and handlers. */
        this.x = noop;
    }
    /** Places the data in `up`, which is still being built, under `seg`. */
    mount(up, seg) {
        this.up = up;
        this.seg = seg;
        up.o[seg] = this.o;
    }
    /** Stores a recomputed value; only changes are sent. */
    set(key, value) {
        if (this.o[key] === value)
            return;
        this.o[key] = value;
        pend(this, key);
    }
    /** Registers an element's handler and stores its id under `key`. */
    on(key, f, controlled) {
        const id = ++lastHandler;
        handlers.set(id, { f, b: this, k: controlled });
        (this.hs ?? (this.hs = [])).push(id);
        this.o[key] = id;
    }
    update() {
        this.u();
    }
    destroy() {
        this.dead = true;
        this.x();
        if (this.hs)
            for (const id of this.hs)
                handlers.delete(id);
        this.slot?.live.delete(this);
    }
}
/** A block with its own dirty bits: a component instance or a `for` row. */
class ScopeBlock extends Block {
    constructor(words, depth) {
        super();
        this.queued = false;
        this.d = new Array(words).fill(0);
        this.depth = depth;
    }
    mark(bit) {
        this.d[bit >> 5] |= 1 << (bit & 31);
        enqueue(this);
    }
    flush() {
        for (const word of this.d) {
            if (word !== 0) {
                this.update();
                return;
            }
        }
    }
    update() {
        this.u();
        this.d.fill(0);
    }
}
/** One item of a keyed `for` list; its key is stored as `k` for `wx:key`. */
export class Row extends ScopeBlock {
    constructor(words, depth, shape, item) {
        super(words, depth);
        this.item = item;
        this.w = watch(shape, this, item);
    }
    setItem(item) {
        this.item = item;
        this.w.set(item);
    }
    destroy() {
        this.w.dispose();
        super.destroy();
    }
}
/** A component instance: parameter and state values plus their watches. */
export class Instance extends ScopeBlock {
    constructor(app, words, depth, args, children) {
        super(words, depth);
        this.s = [];
        this.pw = [];
        this.sw = [];
        this.app = app;
        this.p = args;
        this.children = children;
    }
    arg(index, value) {
        this.p[index] = value;
        this.pw[index]?.set(value);
    }
    state(index, value) {
        this.s[index] = value;
        this.sw[index].set(value);
    }
    destroy() {
        for (const w of this.pw)
            w?.dispose();
        for (const w of this.sw)
            w.dispose();
        super.destroy();
    }
}
/**
 * Runtime of an `if / else if / else` chain: the active index is stored
 * under one key of `up` and the branch's data under another; `-1` renders
 * nothing.
 */
export class Branches {
    constructor(up, indexKey, blockKey, make) {
        this.index = -1;
        this.block = null;
        this.up = up;
        this.indexKey = indexKey;
        this.blockKey = blockKey;
        this.make = make;
        up.o[indexKey] = -1;
        up.o[blockKey] = null;
    }
    set(index) {
        if (index === this.index)
            return;
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
    update() {
        const block = this.block;
        if (block && block.born !== currentEpoch())
            block.update();
    }
    destroy() {
        this.block?.destroy();
        this.block = null;
    }
}
/** Runtime of a keyed `for` loop; rows are stored as an array under `key` of `up`. */
export class List {
    constructor(up, key, make, keyOf) {
        this.rows = [];
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
    set(items) {
        const old = this.rows;
        const byKey = new Map();
        for (let i = 0; i < old.length; i++)
            byKey.set(this.keyOf(old[i].item), i);
        const rows = new Array(items.length);
        const reused = new Uint8Array(old.length);
        const seen = new Set();
        for (let i = 0; i < items.length; i++) {
            const item = items[i];
            const key = this.keyOf(item);
            if (seen.has(key))
                throw new Error(`weft: duplicate key \`${String(key)}\` in for loop`);
            seen.add(key);
            const j = byKey.get(key);
            if (j !== undefined && !reused[j]) {
                reused[j] = 1;
                const row = old[j];
                if (row.item !== item)
                    row.setItem(item);
                rows[i] = row;
            }
            else {
                const row = this.make(item);
                row.up = this.up;
                row.o.k = key;
                rows[i] = row;
            }
        }
        for (let j = 0; j < old.length; j++) {
            if (!reused[j])
                old[j].destroy();
        }
        let appended = rows.length >= old.length;
        for (let i = 0; appended && i < old.length; i++)
            appended = rows[i] === old[i];
        for (let i = 0; i < rows.length; i++)
            rows[i].seg = `${this.key}[${i}]`;
        this.rows = rows;
        if (appended) {
            const data = this.up.o[this.key];
            for (let i = old.length; i < rows.length; i++) {
                data.push(rows[i].o);
                pend(rows[i], null);
            }
        }
        else {
            this.up.set(this.key, rows.map((row) => row.o));
        }
    }
    update() {
        const epoch = currentEpoch();
        for (const row of this.rows) {
            if (row.born !== epoch)
                row.update();
        }
    }
    destroy() {
        for (const row of this.rows)
            row.destroy();
        this.rows = [];
    }
}
/**
 * The caller's block for a component's `Children` parameter. The callee
 * creates instances where it renders the slot; the caller updates them. The
 * view renders them with the caller's template named in `t`.
 */
export class Slot {
    constructor(template, make) {
        this.live = new Set();
        this.template = template;
        this.make = make;
    }
    create() {
        const block = this.make();
        block.o.t = this.template;
        block.slot = this;
        this.live.add(block);
        return block;
    }
    update() {
        const epoch = currentEpoch();
        for (const block of this.live) {
            if (block.born !== epoch)
                block.update();
        }
    }
}
/** Base class of the generated root component classes. */
export class App extends AppBase {
    constructor(create, args, template) {
        super(args);
        _App_create.set(this, void 0);
        this.$root = null;
        this.$view = null;
        __classPrivateFieldSet(this, _App_create, create, "f");
        this.$template = template;
    }
    /** Renders into a view component; the first `setData` follows in a microtask. */
    mount(view) {
        if (this.$root)
            throw new Error("weft: component is already mounted");
        if (typeof view?.setData !== "function") {
            throw new Error("weft: mount expects a view component instance, e.g. `this.selectComponent(\"#view\")`");
        }
        if (!view.$names?.includes(this.$template)) {
            throw new Error(`weft: this view component cannot render \`${this.$template}\`; use the view generated for an entry that imports it`);
        }
        view.$app?.unmount();
        view.$app = this;
        this.$view = view;
        const root = __classPrivateFieldGet(this, _App_create, "f").call(this, this, this.$args, null, 0);
        root.seg = "r";
        this.$root = root;
        fresh.add(this);
        schedule();
    }
    unmount() {
        const view = this.$view;
        if (!this.$detach() || !view)
            return;
        view.setData({ t: "", r: null });
    }
    /** Releases the root without touching the view; `false` if not mounted. */
    $detach() {
        const root = this.$root;
        if (!root)
            return false;
        this.$root = null;
        fresh.delete(this);
        if (this.$view?.$app === this)
            this.$view.$app = null;
        root.destroy();
        return true;
    }
}
_App_create = new WeakMap();
/**
 * Runs the handler of an element and flushes synchronously, so controlled
 * elements can be restored right away: a rejected text edit is replaced by
 * returning the bound value (`bindinput` replaces the input's content with
 * the return value), a rejected switch toggle is reverted by two `setData`
 * calls, because the view's data still holds the bound value.
 */
function dispatch(view, event) {
    const handler = handlers.get(Number(event.currentTarget.dataset.h));
    if (!handler || handler.b.dead)
        return undefined;
    const value = event.detail?.value;
    handler.f?.(value);
    flushSync();
    const { b: block, k: key } = handler;
    if (key === null || block.dead)
        return undefined;
    const bound = block.o[key];
    if (bound === value)
        return undefined;
    if (typeof bound === "string")
        return bound;
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
export function view(names) {
    return {
        data: { t: "", r: null },
        lifetimes: {
            created() {
                this.$names = names;
            },
            detached() {
                this.$app?.$detach();
            },
        },
        methods: {
            $e(event) {
                return dispatch(this, event);
            },
        },
    };
}
