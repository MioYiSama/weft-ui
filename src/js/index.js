/**
 * Runtime for weft-generated Web DOM modules.
 *
 * Generated code owns the view structure and computes which bindings to
 * re-run from dirty bits; this module only provides the dynamic DOM blocks
 * (`if` branches, keyed `for` lists, component slots) on top of the reactive
 * model storage and flush scheduler in `core.ts`.
 */
import { AppBase, currentEpoch, enqueue, restoreAfterFlush, watch, } from "./core.js";
export * from "./core.js";
/* ------------------------------------------------------------------------ */
/* Blocks                                                                    */
/* ------------------------------------------------------------------------ */
const noop = () => { };
/**
 * A range of sibling DOM nodes created from one template. `first` is always a
 * static element or a marker comment, and `last` a static element or the
 * anchor of a dynamic child, so the range stays valid while nested content
 * changes.
 */
export class Block {
    first;
    last;
    /** Holds the nodes until the first mount. */
    frag = null;
    born = currentEpoch();
    /** Set when the block was created through a component slot. */
    slot = null;
    /** Re-runs dirty bindings and walks nested blocks. */
    u = noop;
    /** Releases nested blocks, components, and watches; leaves the DOM alone. */
    x = noop;
    init(frag) {
        this.frag = frag;
        this.first = frag.firstChild;
        this.last = frag.lastChild;
    }
    mount(parent, anchor) {
        if (this.frag) {
            parent.insertBefore(this.frag, anchor);
            this.frag = null;
            return;
        }
        let node = this.first;
        for (;;) {
            const next = node.nextSibling;
            parent.insertBefore(node, anchor);
            if (node === this.last)
                break;
            node = next;
        }
    }
    update() {
        this.u();
    }
    /** `detach` removes the nodes; nested blocks go away with their parent. */
    destroy(detach) {
        this.x();
        this.slot?.live.delete(this);
        if (!detach || this.frag)
            return;
        let node = this.first;
        for (;;) {
            const next = node.nextSibling;
            node.remove();
            if (node === this.last)
                break;
            node = next;
        }
    }
}
/** A block with its own dirty bits: a component instance or a `for` row. */
class ScopeBlock extends Block {
    d;
    depth;
    queued = false;
    dead = false;
    constructor(words, depth) {
        super();
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
    destroy(detach) {
        this.dead = true;
        super.destroy(detach);
    }
}
/** One item of a keyed `for` list. */
export class Row extends ScopeBlock {
    item;
    w;
    constructor(words, depth, shape, item) {
        super(words, depth);
        this.item = item;
        this.w = watch(shape, this, item);
    }
    setItem(item) {
        this.item = item;
        this.w.set(item);
    }
    destroy(detach) {
        this.w.dispose();
        super.destroy(detach);
    }
}
/** A component instance: parameter and state values plus their watches. */
export class Instance extends ScopeBlock {
    app;
    p;
    s = [];
    pw = [];
    sw = [];
    /** Caller-supplied block for the `Children` parameter. */
    children;
    constructor(app, words, depth, args, children) {
        super(words, depth);
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
    destroy(detach) {
        for (const w of this.pw)
            w?.dispose();
        for (const w of this.sw)
            w.dispose();
        super.destroy(detach);
    }
}
/** Runtime of an `if / else if / else` chain; branch `-1` renders nothing. */
export class Branches {
    anchor;
    make;
    index = -1;
    block = null;
    constructor(anchor, make) {
        this.anchor = anchor;
        this.make = make;
    }
    set(index) {
        if (index === this.index)
            return;
        this.index = index;
        this.block?.destroy(true);
        this.block = this.make(index);
        this.block?.mount(this.anchor.parentNode, this.anchor);
    }
    update() {
        const block = this.block;
        if (block && block.born !== currentEpoch())
            block.update();
    }
    destroy() {
        this.block?.destroy(false);
        this.block = null;
    }
}
/** Runtime of a keyed `for` loop. */
export class List {
    anchor;
    make;
    key;
    rows = [];
    constructor(anchor, make, key) {
        this.anchor = anchor;
        this.make = make;
        this.key = key;
    }
    /**
     * Reconciles rows with `items` by key: reused rows receive the new item
     * (marking only the fields that changed), unmatched rows are destroyed, new
     * rows are created, and only rows outside the longest increasing run of old
     * positions are moved.
     */
    set(items) {
        const old = this.rows;
        const byKey = new Map();
        for (let i = 0; i < old.length; i++)
            byKey.set(this.key(old[i].item), i);
        const rows = new Array(items.length);
        const sources = new Array(items.length);
        const reused = new Uint8Array(old.length);
        const seen = new Set();
        for (let i = 0; i < items.length; i++) {
            const item = items[i];
            const key = this.key(item);
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
                sources[i] = j;
            }
            else {
                rows[i] = this.make(item);
                sources[i] = -1;
            }
        }
        for (let j = 0; j < old.length; j++) {
            if (!reused[j])
                old[j].destroy(true);
        }
        const stay = increasingRun(sources);
        const parent = this.anchor.parentNode;
        let next = this.anchor;
        for (let i = rows.length - 1; i >= 0; i--) {
            if (!stay[i])
                rows[i].mount(parent, next);
            next = rows[i].first;
        }
        this.rows = rows;
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
            row.destroy(false);
        this.rows = [];
    }
}
/**
 * Marks the positions of a longest strictly increasing subsequence of
 * `sources`, ignoring `-1` entries (new rows).
 */
function increasingRun(sources) {
    const n = sources.length;
    const prev = new Array(n).fill(-1);
    /** `tails[k]`: index of the smallest tail of an increasing run of length k + 1. */
    const tails = [];
    for (let i = 0; i < n; i++) {
        const value = sources[i];
        if (value < 0)
            continue;
        let lo = 0;
        let hi = tails.length;
        while (lo < hi) {
            const mid = (lo + hi) >> 1;
            if (sources[tails[mid]] < value)
                lo = mid + 1;
            else
                hi = mid;
        }
        if (lo > 0)
            prev[i] = tails[lo - 1];
        tails[lo] = i;
    }
    const stay = new Array(n).fill(false);
    let i = tails.length > 0 ? tails[tails.length - 1] : -1;
    while (i >= 0) {
        stay[i] = true;
        i = prev[i];
    }
    return stay;
}
/**
 * The caller's block for a component's `Children` parameter. The callee
 * creates instances where it renders the slot; the caller updates them.
 */
export class Slot {
    make;
    live = new Set();
    constructor(make) {
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
/** Base class of the generated root component classes. */
export class App extends AppBase {
    #create;
    #root = null;
    constructor(create, args) {
        super(args);
        this.#create = create;
    }
    mount(target) {
        if (this.#root)
            throw new Error("weft: component is already mounted");
        const root = this.#create(this, this.$args, null, 0);
        root.mount(target, null);
        this.#root = root;
    }
    unmount() {
        const root = this.#root;
        if (!root)
            return;
        this.#root = null;
        root.destroy(true);
    }
}
/* ------------------------------------------------------------------------ */
/* DOM helpers used by generated code                                        */
/* ------------------------------------------------------------------------ */
/** Lazily parsed `<template>`; returns a fresh clone per call. */
export function template(html) {
    let tpl = null;
    return () => {
        if (!tpl) {
            tpl = document.createElement("template");
            tpl.innerHTML = html;
        }
        return tpl.content.cloneNode(true);
    };
}
export function value(node, value) {
    node.$v = value;
    if (node.value !== value)
        node.value = value;
}
export function checked(node, value) {
    node.$v = value;
    if (node.checked !== value)
        node.checked = value;
}
/**
 * Text input events. After the flush triggered by the handler, the input is
 * reset to the bound value, so only the host decides what it shows.
 */
export function input(node, handler) {
    node.addEventListener("input", () => {
        handler?.(node.value);
        restoreAfterFlush(() => {
            if (node.value !== node.$v)
                node.value = node.$v;
        });
    });
}
export function checkbox(node, handler) {
    node.addEventListener("change", () => {
        handler?.(node.checked);
        restoreAfterFlush(() => {
            if (node.checked !== node.$v)
                node.checked = node.$v;
        });
    });
}
