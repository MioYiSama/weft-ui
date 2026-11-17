/** Connects a generated Web DOM view to a host that owns its model. */
import { hold } from "./index.js";
/** Installs the page bridge and announces that it can receive a snapshot. */
export function connect(Component, events, target, post) {
    let app;
    const pending = [];
    const remote = {
        start({ args }) {
            app?.unmount();
            for (const release of pending)
                release();
            pending.length = 0;
            app = new Component(...args);
            for (const name of events) {
                app.$on(name, (_model, payload) => {
                    pending.push(hold());
                    post(JSON.stringify({ type: "event", name, payload }));
                });
            }
            app.mount(target);
        },
        apply({ patches, acks }) {
            for (const [path, value] of patches) {
                let parent = app.model;
                for (let i = 0; i < path.length - 1; i++) {
                    parent = parent[path[i]];
                }
                parent[path[path.length - 1]] = value;
            }
            for (const release of pending.splice(0, acks))
                release();
        },
    };
    globalThis.__weft = remote;
    post(JSON.stringify({ type: "ready" }));
    return remote;
}
