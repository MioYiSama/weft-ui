
export declare class Assets {
  constructor(source: string, fit: "cover" | "contain" | "fill", family: string, tint: string);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: string;
  $on(name: string, handler: (model: string, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}
