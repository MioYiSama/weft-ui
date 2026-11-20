
export interface Model {
  flag: boolean;
}

export declare class Badge {
  constructor(tint: string);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: string;
  $on(name: string, handler: (model: string, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}

export declare class Theme {
  constructor(model: Model);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: Model;
  $on(name: string, handler: (model: Model, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}
