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
 * carries handler ids (`data-h`) that the sink maps back to
 * `(scope, event slot)`.
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
var _WechatSink_instances, _WechatSink_view, _WechatSink_layout, _WechatSink_session, _WechatSink_scopes, _WechatSink_root, _WechatSink_rootFresh, _WechatSink_created, _WechatSink_pending, _WechatSink_handlers, _WechatSink_lastHandler, _WechatSink_scope, _WechatSink_pend, _WechatSink_locate, _WechatSink_destroy, _WechatSink_reset;
const PX = 1;
const BOLD = 2;
/** Renders one app into a view component through `setData`. */
export class WechatSink {
    constructor(view, layout) {
        _WechatSink_instances.add(this);
        _WechatSink_view.set(this, void 0);
        _WechatSink_layout.set(this, void 0);
        _WechatSink_session.set(this, null);
        _WechatSink_scopes.set(this, new Map());
        _WechatSink_root.set(this, null);
        _WechatSink_rootFresh.set(this, false);
        _WechatSink_created.set(this, []);
        /** Changed data since the last commit: a scope and one of its keys, or `null` for the whole scope. */
        _WechatSink_pending.set(this, []);
        /** Handler id → scope and event slot. */
        _WechatSink_handlers.set(this, new Map());
        _WechatSink_lastHandler.set(this, 0);
        __classPrivateFieldSet(this, _WechatSink_view, view, "f");
        __classPrivateFieldSet(this, _WechatSink_layout, layout, "f");
    }
    open(session) {
        if (session.hash !== __classPrivateFieldGet(this, _WechatSink_layout, "f").hash) {
            throw new Error(`weft: this view component was generated for another build of the app (layout ${__classPrivateFieldGet(this, _WechatSink_layout, "f").hash}, app ${session.hash})`);
        }
        __classPrivateFieldGet(this, _WechatSink_session, "f")?.detach();
        __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_reset).call(this);
        __classPrivateFieldSet(this, _WechatSink_session, session, "f");
    }
    create(id, template) {
        var _a;
        const keys = __classPrivateFieldGet(this, _WechatSink_layout, "f").templates[template];
        const scope = { id, keys, o: {}, up: null, seg: "", fresh: true, dead: false, kids: [], hs: [] };
        const o = scope.o;
        if (keys.t !== undefined)
            o.t = keys.t;
        if (keys.d)
            Object.assign(o, keys.d);
        if (keys.e) {
            for (let slot = 0; slot < keys.e.length; slot++) {
                const handler = __classPrivateFieldSet(this, _WechatSink_lastHandler, (_a = __classPrivateFieldGet(this, _WechatSink_lastHandler, "f"), ++_a), "f");
                __classPrivateFieldGet(this, _WechatSink_handlers, "f").set(handler, [scope, slot]);
                scope.hs.push(handler);
                o[keys.e[slot][0]] = handler;
            }
        }
        if (keys.p) {
            for (const point of keys.p) {
                if (typeof point === "string") {
                    if (point[0] === "l")
                        o[point] = [];
                }
                else {
                    o[point[0]] = -1;
                    o[point[1]] = null;
                }
            }
        }
        __classPrivateFieldGet(this, _WechatSink_scopes, "f").set(id, scope);
        __classPrivateFieldGet(this, _WechatSink_created, "f").push(scope);
    }
    set(id, slot, value) {
        const scope = __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_scope).call(this, id);
        const [key, unit] = scope.keys.v[slot];
        scope.o[key] = unit === PX ? `${value}px` : unit === BOLD ? (value ? "bold" : "normal") : value;
        if (!scope.fresh)
            __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_pend).call(this, scope, key);
    }
    branch(id, point, child) {
        const scope = __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_scope).call(this, id);
        const [indexKey, dataKey] = scope.keys.p[point];
        const old = scope.kids[point];
        if (old)
            __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_destroy).call(this, old);
        const next = child === 0 ? null : __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_scope).call(this, child);
        scope.kids[point] = next;
        if (next) {
            next.up = scope;
            next.seg = dataKey;
        }
        scope.o[indexKey] = next ? next.keys.i : -1;
        scope.o[dataKey] = next ? next.o : null;
        if (!scope.fresh) {
            __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_pend).call(this, scope, indexKey);
            __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_pend).call(this, scope, dataKey);
        }
    }
    /**
     * Rows appended behind unchanged ones are sent by index; any other change
     * of the row order sends the whole array. A row's `k` (`wx:key`) is its
     * scope id.
     */
    rows(id, point, children) {
        const scope = __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_scope).call(this, id);
        const key = scope.keys.p[point];
        const old = scope.kids[point] ?? [];
        const rows = children.map((child) => __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_scope).call(this, child));
        const kept = new Set(rows);
        for (const row of old)
            if (!kept.has(row))
                __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_destroy).call(this, row);
        let appended = rows.length >= old.length;
        for (let i = 0; appended && i < old.length; i++)
            appended = rows[i] === old[i];
        for (let i = 0; i < rows.length; i++) {
            const row = rows[i];
            row.up = scope;
            row.seg = `${key}[${i}]`;
            row.o.k = row.id;
        }
        scope.kids[point] = rows;
        if (appended) {
            const data = scope.o[key];
            for (let i = old.length; i < rows.length; i++) {
                data.push(rows[i].o);
                if (!scope.fresh)
                    __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_pend).call(this, rows[i], null);
            }
        }
        else {
            scope.o[key] = rows.map((row) => row.o);
            if (!scope.fresh)
                __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_pend).call(this, scope, key);
        }
    }
    attach(id, point, child) {
        const next = __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_scope).call(this, child);
        if (id === 0) {
            next.seg = "r";
            __classPrivateFieldSet(this, _WechatSink_root, next, "f");
            __classPrivateFieldSet(this, _WechatSink_rootFresh, true, "f");
            return;
        }
        const scope = __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_scope).call(this, id);
        const key = scope.keys.p[point];
        scope.kids[point] = next;
        next.up = scope;
        next.seg = key;
        scope.o[key] = next.o;
        if (!scope.fresh)
            __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_pend).call(this, scope, key);
    }
    /** Sends the batch with one `setData`. */
    commit() {
        const data = new Map();
        for (const [scope, key] of __classPrivateFieldGet(this, _WechatSink_pending, "f")) {
            if (scope.dead)
                continue;
            const path = __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_locate).call(this, scope, key);
            if (path !== null)
                data.set(path, key === null ? scope.o : scope.o[key]);
        }
        if (__classPrivateFieldGet(this, _WechatSink_rootFresh, "f") && __classPrivateFieldGet(this, _WechatSink_root, "f")) {
            data.set("t", __classPrivateFieldGet(this, _WechatSink_root, "f").keys.n);
            data.set("r", __classPrivateFieldGet(this, _WechatSink_root, "f").o);
        }
        __classPrivateFieldSet(this, _WechatSink_pending, [], "f");
        __classPrivateFieldSet(this, _WechatSink_rootFresh, false, "f");
        for (const scope of __classPrivateFieldGet(this, _WechatSink_created, "f"))
            scope.fresh = false;
        __classPrivateFieldSet(this, _WechatSink_created, [], "f");
        if (data.size > 0)
            __classPrivateFieldGet(this, _WechatSink_view, "f").setData(uncovered(data));
    }
    close() {
        __classPrivateFieldSet(this, _WechatSink_session, null, "f");
        __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_reset).call(this);
        __classPrivateFieldGet(this, _WechatSink_view, "f").setData({ t: "", r: null });
    }
    /** The view went away: releases the app without touching the view. */
    detach() {
        const session = __classPrivateFieldGet(this, _WechatSink_session, "f");
        __classPrivateFieldSet(this, _WechatSink_session, null, "f");
        session?.detach();
        __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_reset).call(this);
    }
    /**
     * Dispatches a view event. A rejected text edit is replaced by returning
     * the bound value (`bindinput` replaces the input's content with the
     * return value); a rejected switch toggle is reverted by two `setData`
     * calls, because the view's data still holds the bound value.
     */
    event(event) {
        const found = __classPrivateFieldGet(this, _WechatSink_handlers, "f").get(Number(event.currentTarget.dataset.h));
        const session = __classPrivateFieldGet(this, _WechatSink_session, "f");
        if (!found || !session)
            return undefined;
        const [scope, slot] = found;
        const value = event.detail?.value;
        if (session.dispatch(scope.id, slot, value) || scope.dead)
            return undefined;
        const controls = scope.keys.e[slot][1];
        const key = scope.keys.v[controls][0];
        const bound = scope.o[key];
        if (typeof bound === "string")
            return bound;
        const path = __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_locate).call(this, scope, key);
        if (path !== null) {
            __classPrivateFieldGet(this, _WechatSink_view, "f").setData({ [path]: value });
            __classPrivateFieldGet(this, _WechatSink_view, "f").setData({ [path]: bound });
        }
        return undefined;
    }
}
_WechatSink_view = new WeakMap(), _WechatSink_layout = new WeakMap(), _WechatSink_session = new WeakMap(), _WechatSink_scopes = new WeakMap(), _WechatSink_root = new WeakMap(), _WechatSink_rootFresh = new WeakMap(), _WechatSink_created = new WeakMap(), _WechatSink_pending = new WeakMap(), _WechatSink_handlers = new WeakMap(), _WechatSink_lastHandler = new WeakMap(), _WechatSink_instances = new WeakSet(), _WechatSink_scope = function _WechatSink_scope(id) {
    const scope = __classPrivateFieldGet(this, _WechatSink_scopes, "f").get(id);
    if (!scope)
        throw new Error(`weft: unknown scope ${id}`);
    return scope;
}, _WechatSink_pend = function _WechatSink_pend(scope, key) {
    __classPrivateFieldGet(this, _WechatSink_pending, "f").push([scope, key]);
}, _WechatSink_locate = function _WechatSink_locate(scope, key) {
    let path = key === null ? scope.seg : `${scope.seg}.${key}`;
    let top = scope;
    while (top.up) {
        top = top.up;
        path = `${top.seg}.${path}`;
    }
    return top === __classPrivateFieldGet(this, _WechatSink_root, "f") ? path : null;
}, _WechatSink_destroy = function _WechatSink_destroy(scope) {
    scope.dead = true;
    __classPrivateFieldGet(this, _WechatSink_scopes, "f").delete(scope.id);
    for (const handler of scope.hs)
        __classPrivateFieldGet(this, _WechatSink_handlers, "f").delete(handler);
    for (const kid of scope.kids) {
        if (Array.isArray(kid))
            for (const row of kid)
                __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_destroy).call(this, row);
        else if (kid)
            __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_destroy).call(this, kid);
    }
}, _WechatSink_reset = function _WechatSink_reset() {
    if (__classPrivateFieldGet(this, _WechatSink_root, "f"))
        __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_destroy).call(this, __classPrivateFieldGet(this, _WechatSink_root, "f"));
    for (const scope of __classPrivateFieldGet(this, _WechatSink_scopes, "f").values())
        __classPrivateFieldGet(this, _WechatSink_instances, "m", _WechatSink_destroy).call(this, scope);
    __classPrivateFieldSet(this, _WechatSink_root, null, "f");
    __classPrivateFieldSet(this, _WechatSink_rootFresh, false, "f");
    __classPrivateFieldSet(this, _WechatSink_created, [], "f");
    __classPrivateFieldSet(this, _WechatSink_pending, [], "f");
};
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
/**
 * Options of the generated view component. The view releases its app when
 * it is detached from the page.
 */
export function view(layout) {
    return {
        data: { t: "", r: null },
        lifetimes: {
            created() {
                this.$sink = new WechatSink(this, layout);
            },
            detached() {
                this.$sink?.detach();
            },
        },
        methods: {
            $e(event) {
                return this.$sink?.event(event);
            },
        },
    };
}
