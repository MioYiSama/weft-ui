/**
 * Backend-independent half of the runtime: reactive model storage, watches
 * that map model changes to dirty bits, the flush scheduler, expression
 * helpers, and event handler registration. `index.ts` (Web DOM) and `wx.ts`
 * (WeChat mini program) build their blocks on top of it.
 */
var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _AppBase_handlers;
/** Base class of generated model classes; fields are stored by index. */
export class Model {
    constructor(values) {
        /** Watches notified with the index of every changed field. */
        this.$subs = new Set();
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
/** Indices of fields holding a model instance (`M` or `M?`). */
Model.$m = [];
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
    constructor(bit, scope, value) {
        this.subs = new Map();
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
const commits = [];
const defer = microtask();
/** Mini program logic layers may lack `queueMicrotask`; promises always work. */
function microtask() {
    const scope = globalThis;
    if ("queueMicrotask" in scope && typeof scope.queueMicrotask === "function") {
        const queue = scope.queueMicrotask;
        return (task) => queue.call(scope, task);
    }
    return (task) => {
        void Promise.resolve().then(task);
    };
}
/** The current flush counter; blocks remember the flush they were born in. */
export function currentEpoch() {
    return epoch;
}
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
/** Runs `restore` after the next flush, or once all holds are released. */
export function restoreAfterFlush(restore) {
    afterFlush.push(() => {
        if (holds > 0)
            deferredRestores.push(restore);
        else
            restore();
    });
    schedule();
}
/** Registers a hook that runs in every flush after the dirty scopes are updated. */
export function onCommit(hook) {
    commits.push(hook);
}
/** Schedules a flush in a microtask unless one is pending. */
export function schedule() {
    if (!scheduled) {
        scheduled = true;
        defer(() => {
            if (scheduled)
                flush();
        });
    }
}
/** Runs the pending flush now instead of in its microtask. */
export function flushSync() {
    if (scheduled)
        flush();
}
export function enqueue(job) {
    if (job.queued)
        return;
    job.queued = true;
    queue.push(job);
    schedule();
}
/**
 * Updates every dirty scope, parents before children, runs the commit hooks,
 * then dispatches the lifecycle events collected during the flush. Handlers
 * that modify models schedule the next flush instead of re-entering this one.
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
        for (const commit of commits)
            commit();
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
    // Not `Promise.withResolvers`: mini program JS engines predate ES2024.
    return new Promise((resolve) => {
        afterFlush.push(resolve);
        schedule();
    });
}
/** Queues a lifecycle event for dispatch after the current flush. */
export function lifecycle(app, name, payload) {
    pendingEvents.push([app, name, payload]);
    schedule();
}
/** Event handler registration shared by the backends' root component classes. */
export class AppBase {
    constructor(args) {
        _AppBase_handlers.set(this, new Map());
        this.$args = args;
    }
    /** Value of the first parameter: the object event handlers receive. */
    get model() {
        return this.$args[0];
    }
    $on(name, handler) {
        let handlers = __classPrivateFieldGet(this, _AppBase_handlers, "f").get(name);
        if (!handlers) {
            handlers = new Set();
            __classPrivateFieldGet(this, _AppBase_handlers, "f").set(name, handlers);
        }
        handlers.add(handler);
        return () => {
            handlers.delete(handler);
        };
    }
    $emit(name, payload) {
        const handlers = __classPrivateFieldGet(this, _AppBase_handlers, "f").get(name);
        if (!handlers)
            return;
        for (const handler of handlers)
            handler(this.$args[0], payload);
    }
}
_AppBase_handlers = new WeakMap();
/* ------------------------------------------------------------------------ */
/* Expression helpers used by generated code                                 */
/* ------------------------------------------------------------------------ */
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
