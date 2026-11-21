
export declare class Assets {
  constructor(source: string, fit: "cover" | "contain" | "fill", family: string, tint: string);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: string;
  $on(name: string, handler: (model: string, event: object) => unknown): () => void;
  /** Renders into a view sink, or a generated mini program view component, e.g. `page.selectComponent("#view")`. */
  mount(view: object): void;
  unmount(): void;
}
