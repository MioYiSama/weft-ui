/**
 * Runtime for weft-generated Web DOM modules.
 *
 * Generated code owns the view structure and computes which bindings to
 * re-run from dirty bits; this module only provides reactive model storage,
 * dirty-bit scopes, a microtask flush scheduler, and the dynamic DOM blocks
 * (`if` branches, keyed `for` lists, component slots).
 */
/** Base class of generated model classes; fields are stored by index. */
export class Model {
    /** Field values in declaration order. */
    $v;
    /** Watches notified with the index of every changed field. */
    $subs = new Set();
    /** Indices of fields holding a model instance (`M` or `M?`). */
    static $m = [];
    constructor(values) {
        this.$v = values;
    }
    /** Stores a field and notifies watchers when the value changed. */
    $set(field, value) {
        if (this.$v[field] === value)
            return;
        this.$v[field] = value;
        for (const listener of this.$subs)
            listener(field);
    }
}
export const id = (value) => value;
/**
 * Converter for `T[]`: converts every element and freezes the result, so
 * in-place mutation (`push`, index assignment) fails loudly instead of being
 * silently missed by change tracking. Already converted frozen arrays are
 * returned unchanged.
 */
export function arrayOf(conv) {
    return (input) => {
        // Generated code only passes arrays here (checked by the host's types).
        const value = input;
        let out = null;
        for (let i = 0; i < value.length; i++) {
            const item = conv(value[i]);
            if (out) {
                out.push(item);
            }
            else if (item !== value[i]) {
                out = value.slice(0, i);
                out.push(item);
            }
        }
        if (!out) {
            if (Object.isFrozen(value))
                return value;
            out = value.slice();
        }
        return Object.freeze(out);
    };
}
/** Converter for `T?`: `undefined` becomes `null`. */
export function optional(conv) {
    return (value) => (value == null ? null : conv(value));
}
/** Creates a watch for `value`; the initial value marks nothing. */
export function watch(shape, scope, value) {
    if (typeof shape === "number")
        return new LeafWatch(shape, scope, value);
    if (Array.isArray(shape))
        return new FieldsWatch(shape, scope, value);
    return new DeepWatch(shape.d, scope, value);
}
class LeafWatch {
    bit;
    scope;
    cur;
    constructor(bit, scope, value) {
        this.bit = bit;
        this.scope = scope;
        this.cur = value;
    }
    set(value) {
        if (value === this.cur)
            return;
        this.cur = value;
        this.scope.mark(this.bit);
    }
    dispose() { }
}
/** A non-optional model flattened into one watch per field. */
class FieldsWatch {
    cur;
    fields;
    listener;
    constructor(shapes, scope, value) {
        this.cur = value;
        this.fields = shapes.map((shape, i) => watch(shape, scope, value.$v[i]));
        this.listener = (field) => this.fields[field].set(this.cur.$v[field]);
        value.$subs.add(this.listener);
    }
    set(value) {
        if (value === this.cur)
            return;
        const next = value;
        this.cur.$subs.delete(this.listener);
        this.cur = next;
        next.$subs.add(this.listener);
        for (let i = 0; i < this.fields.length; i++)
            this.fields[i].set(next.$v[i]);
    }
    dispose() {
        this.cur.$subs.delete(this.listener);
        for (const field of this.fields)
            field.dispose();
    }
}
/**
 * One bit covering an optional model and every model reachable from it
 * without passing through an array (array elements are only readable through
 * `for` items, which watch themselves).
 */
class DeepWatch {
    bit;
    scope;
    cur;
    subs = new Map();
    constructor(bit, scope, value) {
        this.bit = bit;
        this.scope = scope;
        this.cur = value;
        this.track();
    }
    set(value) {
        if (value === this.cur)
            return;
        this.cur = value;
        this.track();
        this.scope.mark(this.bit);
    }
    /** Re-subscribes to the reachable models, keeping existing listeners. */
    track() {
        const next = new Map();
        const visit = (value) => {
            if (!(value instanceof Model) || next.has(value))
                return;
            const nested = value.constructor.$m;
            let listener = this.subs.get(value);
            if (!listener) {
                listener = (field) => {
                    if (nested.includes(field))
                        this.track();
                    this.scope.mark(this.bit);
                };
                value.$subs.add(listener);
            }
            next.set(value, listener);
            for (const field of nested)
                visit(value.$v[field]);
        };
        visit(this.cur);
        for (const [model, listener] of this.subs) {
            if (!next.has(model))
                model.$subs.delete(listener);
        }
        this.subs = next;
    }
    dispose() {
        for (const [model, listener] of this.subs)
            model.$subs.delete(listener);
        this.subs.clear();
    }
}
let queue = [];
let scheduled = false;
/** Incremented by every flush; blocks created during a flush skip its updates. */
let epoch = 0;
let pendingEvents = [];
let afterFlush = [];
let deferredRestores = [];
let holds = 0;
/** Defers controlled-input restores until every hold is released. */
export function hold() {
    holds++;
    let released = false;
    return () => {
        if (released)
            return;
        released = true;
        holds--;
        schedule();
    };
}
function restoreAfterFlush(restore) {
    afterFlush.push(() => {
        if (holds > 0)
            deferredRestores.push(restore);
        else
            restore();
    });
    schedule();
}
function schedule() {
    if (!scheduled) {
        scheduled = true;
        queueMicrotask(flush);
    }
}
function enqueue(job) {
    if (job.queued)
        return;
    job.queued = true;
    queue.push(job);
    schedule();
}
/**
 * Updates every dirty scope, parents before children, then dispatches the
 * lifecycle events collected during the flush. Handlers that modify models
 * schedule the next flush instead of re-entering this one.
 */
function flush() {
    epoch++;
    let batch = [];
    try {
        while (queue.length > 0) {
            batch = queue;
            queue = [];
            batch.sort((a, b) => a.depth - b.depth);
            for (const job of batch) {
                job.queued = false;
                if (!job.dead)
                    job.flush();
            }
        }
    }
    finally {
        // After an exception, unprocessed jobs must be able to queue again.
        for (const job of batch)
            job.queued = false;
        for (const job of queue)
            job.queued = false;
        queue = [];
        scheduled = false;
    }
    const events = pendingEvents;
    const callbacks = afterFlush;
    pendingEvents = [];
    afterFlush = [];
    for (const [app, name, payload] of events)
        app.$emit(name, payload);
    if (holds === 0) {
        const restores = deferredRestores;
        deferredRestores = [];
        for (const restore of restores)
            restore();
    }
    for (const callback of callbacks)
        callback();
}
/** Resolves after the next flush (one is scheduled if none is pending). */
export function tick() {
    const { promise, resolve } = Promise.withResolvers();
    afterFlush.push(resolve);
    schedule();
    return promise;
}
/** Queues a lifecycle event for dispatch after the current flush. */
export function lifecycle(app, name, payload) {
    pendingEvents.push([app, name, payload]);
    schedule();
}
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
    born = epoch;
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
        if (block && block.born !== epoch)
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
        for (const block of this.live) {
            if (block.born !== epoch)
                block.update();
        }
    }
}
/** Base class of the generated root component classes. */
export class App {
    #create;
    #args;
    #handlers = new Map();
    #root = null;
    constructor(create, args) {
        this.#create = create;
        this.#args = args;
    }
    /** Value of the first parameter: the object event handlers receive. */
    get model() {
        return this.#args[0];
    }
    mount(target) {
        if (this.#root)
            throw new Error("weft: component is already mounted");
        const root = this.#create(this, this.#args, null, 0);
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
    $on(name, handler) {
        let handlers = this.#handlers.get(name);
        if (!handlers) {
            handlers = new Set();
            this.#handlers.set(name, handlers);
        }
        handlers.add(handler);
        return () => {
            handlers.delete(handler);
        };
    }
    $emit(name, payload) {
        const handlers = this.#handlers.get(name);
        if (!handlers)
            return;
        for (const handler of handlers)
            handler(this.#args[0], payload);
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
export function px(value) {
    return `${value}px`;
}
export function str(value) {
    return String(value);
}
/** `Int / Int`: truncating division. */
export function idiv(a, b) {
    return Math.trunc(a / b);
}
/** Applies `f` to a non-null receiver (`?.` on built-in members). */
export function safe(value, f) {
    return value == null ? null : f(value);
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
