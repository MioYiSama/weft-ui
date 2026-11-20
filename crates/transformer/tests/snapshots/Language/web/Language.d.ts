
export interface LanguageItem {
  id: number;
  kind: "text" | "image";
  name: string;
}

export interface LanguageModel {
  name: string;
  items: readonly LanguageItem[];
  mode: "compact" | "full";
  rating: number;
}

export type Rated = { value: number };

export declare class ModeLabel {
  constructor(mode?: "compact" | "full");
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: "compact" | "full";
  $on(name: string, handler: (model: "compact" | "full", event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}

export declare class Rating {
  constructor(value: number);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: number;
  $on(name: string, handler: (model: number, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}

export declare class ForwardRating {
  constructor();
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: undefined;
  $on(name: string, handler: (model: undefined, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}

export declare class Language {
  constructor(model: LanguageModel);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: LanguageModel;
  onRated(handler: (model: LanguageModel, event: Rated) => void): () => void;
  $on(name: string, handler: (model: LanguageModel, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}
