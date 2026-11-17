/**
 * Block runtime of the view protocol (docs/PROTOCOL.md), shared by every
 * platform whose view layer is separate from the logic layer: the WeChat
 * mini program and the native platforms.
 *
 * Every block (component instance, `if` branch, `for` row, slot content) is
 * a scope of a layout template. Generated create code writes the scope's
 * value slots and mounts child scopes; update functions write the values
 * that changed. Each write becomes an op for the app's sink: the WeChat sink
 * turns ops into `setData` paths, the native sink encodes them for the
 * native kernel, and the headless renderer (`./testing`) applies them to a
 * tree in memory. Event handlers stay in the logic layer: the view reports
 * `(scope, event slot)` and `dispatch` runs the handler.
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
var _App_instances, _App_create, _App_dispatch;
import { AppBase, currentEpoch, enqueue, flushSync, onCommit, schedule, watch, } from "./core.js";
export * from "./core.js";
/** Mounted apps; their sinks end a batch after every flush. */
const mounted = new Set();
onCommit(() => {
    for (const app of mounted)
        app.$sink.commit();
});
const noop = () => { };
/** A scope: one instance of a layout template. */
export class Block {
    constructor(app, template) {
        /** Last value written to each slot. */
        this.v = [];
        /** Handlers by event slot. */
        this.h = null;
        this.born = currentEpoch();
        this.dead = false;
        /** Set when the block was created through a component slot. */
        this.slot = null;
        /** Re-runs dirty bindings and walks nested blocks. */
        this.u = noop;
        /** Releases nested blocks, components and watches. */
        this.x = noop;
        this.app = app;
        this.id = ++app.$lastScope;
        app.$scopes.set(this.id, this);
        app.$sink.create(this.id, template);
    }
    /** Writes a value slot; only changes are sent. */
    set(slot, value) {
        if (this.v[slot] === value)
            return;
        this.v[slot] = value;
        this.app.$sink.set(this.id, slot, value);
    }
    /** Registers the handler of an event slot and the value slot it controls (`-1` for none). */
    on(slot, f, controls) {
        (this.h ?? (this.h = []))[slot] = { f, k: controls };
    }
    /** Mounts a component instance or slot content at `point`. */
    attach(point, child) {
        this.app.$sink.attach(this.id, point, child.id);
    }
    update() {
        this.u();
    }
    destroy() {
        this.dead = true;
        this.x();
        this.app.$scopes.delete(this.id);
        this.slot?.live.delete(this);
    }
}
/** A block with its own dirty bits: a component instance or a `for` row. */
class ScopeBlock extends Block {
    constructor(app, template, words, depth) {
        super(app, template);
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
/** One item of a keyed `for` list. */
export class Row extends ScopeBlock {
    constructor(app, template, words, depth, shape, item) {
        super(app, template, words, depth);
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
    constructor(app, template, words, depth, args, children) {
        super(app, template, words, depth);
        this.s = [];
        this.pw = [];
        this.sw = [];
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
/** Runtime of an `if / else if / else` chain at a branch point; `-1` renders nothing. */
export class Branches {
    constructor(up, point, make) {
        this.index = -1;
        this.block = null;
        this.up = up;
        this.point = point;
        this.make = make;
    }
    set(index) {
        if (index === this.index)
            return;
        this.index = index;
        this.block?.destroy();
        const block = this.make(index);
        this.block = block;
        const up = this.up;
        up.app.$sink.branch(up.id, this.point, block ? block.id : 0);
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
/** Runtime of a keyed `for` loop at a list point. */
export class List {
    constructor(up, point, make, keyOf) {
        this.rows = [];
        this.up = up;
        this.point = point;
        this.make = make;
        this.keyOf = keyOf;
    }
    /**
     * Reconciles rows with `items` by key: reused rows receive the new item
     * (marking only the fields that changed), unmatched rows are destroyed and
     * new rows are created. The new order is sent only when it changed.
     */
    set(items) {
        const old = this.rows;
        const byKey = new Map();
        for (let i = 0; i < old.length; i++)
            byKey.set(this.keyOf(old[i].item), i);
        const rows = new Array(items.length);
        const reused = new Uint8Array(old.length);
        const seen = new Set();
        let changed = items.length !== old.length;
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
                if (j !== i)
                    changed = true;
            }
            else {
                rows[i] = this.make(item);
                changed = true;
            }
        }
        for (let j = 0; j < old.length; j++) {
            if (!reused[j])
                old[j].destroy();
        }
        this.rows = rows;
        if (changed) {
            const up = this.up;
            up.app.$sink.rows(up.id, this.point, rows.map((row) => row.id));
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
 * creates instances where it renders the slot; the caller updates them.
 */
export class Slot {
    constructor(make) {
        this.live = new Set();
        this.make = make;
    }
    create() {
        const block = this.make();
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
function sinkOf(target) {
    const sink = target && "$sink" in target ? target.$sink : target;
    if (typeof sink?.create !== "function") {
        throw new Error("weft: mount expects a view sink or a generated view component instance, e.g. `this.selectComponent(\"#view\")`");
    }
    return sink;
}
/** Base class of the generated root component classes. */
export class App extends AppBase {
    constructor(create, args, hash) {
        super(args);
        _App_instances.add(this);
        _App_create.set(this, void 0);
        this.$root = null;
        this.$sink = null;
        /** Live scopes by id; ids are allocated per app, starting at 1. */
        this.$scopes = new Map();
        this.$lastScope = 0;
        __classPrivateFieldSet(this, _App_create, create, "f");
        this.$hash = hash;
    }
    /** Renders into `target`; the first batch follows in a microtask. */
    mount(target) {
        if (this.$root)
            throw new Error("weft: component is already mounted");
        const sink = sinkOf(target);
        sink.open({
            hash: this.$hash,
            dispatch: (scope, slot, payload) => __classPrivateFieldGet(this, _App_instances, "m", _App_dispatch).call(this, scope, slot, payload),
            detach: () => this.$detach(),
        });
        this.$sink = sink;
        mounted.add(this);
        const root = __classPrivateFieldGet(this, _App_create, "f").call(this, this, this.$args, null, 0);
        this.$root = root;
        sink.attach(0, 0, root.id);
        schedule();
    }
    unmount() {
        const sink = this.$sink;
        if (this.$detach())
            sink.close();
    }
    /** Releases the root without calling the sink; `false` if not mounted. */
    $detach() {
        const root = this.$root;
        if (!root)
            return false;
        this.$root = null;
        mounted.delete(this);
        root.destroy();
        this.$sink = null;
        return true;
    }
}
_App_create = new WeakMap(), _App_instances = new WeakSet(), _App_dispatch = function _App_dispatch(scope, slot, payload) {
    const block = this.$scopes.get(scope);
    const handler = block?.h?.[slot];
    if (!block || !handler)
        return true;
    handler.f?.(payload);
    flushSync();
    return handler.k < 0 || block.dead || block.v[handler.k] === payload;
};
