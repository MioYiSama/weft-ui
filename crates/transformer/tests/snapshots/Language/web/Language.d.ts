
export interface LanguageItem {
  id: number;
  kind: "text" | "image";
}

export interface LanguageModel {
  items: readonly LanguageItem[];
  mode: "compact" | "full";
}

export declare class ModeLabel {
  constructor(mode?: "compact" | "full");
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: "compact" | "full";
  $on(name: string, handler: (model: "compact" | "full", event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}

export declare class Language {
  constructor(model: LanguageModel);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: LanguageModel;
  $on(name: string, handler: (model: LanguageModel, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}
