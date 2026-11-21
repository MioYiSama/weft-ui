
export declare class Controls {
  constructor(options: readonly string[], color: string, progress: number, limit: number);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: readonly string[];
  $on(name: string, handler: (model: readonly string[], event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}
