
export declare class Inputs {
  constructor(kind: "text" | "password" | "number" | "email" | "phone", key: "done" | "go" | "next" | "search" | "send", limit: number);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: "text" | "password" | "number" | "email" | "phone";
  $on(name: string, handler: (model: "text" | "password" | "number" | "email" | "phone", event: object) => unknown): () => void;
  /** Renders into a view sink, or a generated mini program view component, e.g. `page.selectComponent("#view")`. */
  mount(view: object): void;
  unmount(): void;
}
