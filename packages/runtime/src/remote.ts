/** Connects a generated Web DOM view to a host that owns its model. */
import { type App, hold } from "./index.ts";

type View = Pick<App, "model" | "$on" | "mount" | "unmount">;

/** Messages the host sends through `globalThis.__weft`. */
export interface Remote<Args extends unknown[] = unknown[]> {
  /** Replaces the mounted view with a complete model snapshot. */
  start(message: { args: Args }): void;
  /** Applies model patches before acknowledging events in FIFO order. */
  apply(message: { patches: [string[], unknown][]; acks: number }): void;
}

declare global {
  var __weft: Remote;
}

/** Installs the page bridge and announces that it can receive a snapshot. */
export function connect<Args extends unknown[]>(
  Component: new (...args: Args) => View,
  events: readonly string[],
  target: Node,
  post: (message: string) => void,
): Remote<Args> {
  let app: View | undefined;
  const pending: (() => void)[] = [];
  const remote: Remote<Args> = {
    start({ args }) {
      app?.unmount();
      for (const release of pending) release();
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
        let parent = app!.model as Record<string, unknown>;
        for (let i = 0; i < path.length - 1; i++) {
          parent = parent[path[i]] as Record<string, unknown>;
        }
        parent[path[path.length - 1]] = value;
      }
      for (const release of pending.splice(0, acks)) release();
    },
  };
  globalThis.__weft = remote;
  post(JSON.stringify({ type: "ready" }));
  return remote;
}
