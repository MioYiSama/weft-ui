
export interface LanguageItem {
  id: number;
  kind: "text" | "image";
  name: string;
}

export interface LanguageModel {
  name: string;
  items: readonly LanguageItem[];
  mode: "compact" | "full";
}

export declare class ModeLabel {
  constructor(mode?: "compact" | "full");
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: "compact" | "full";
  $on(name: string, handler: (model: "compact" | "full", event: object) => unknown): () => void;
  /** Renders into a view sink, or a generated mini program view component, e.g. `page.selectComponent("#view")`. */
  mount(view: object): void;
  unmount(): void;
}

export declare class Language {
  constructor(model: LanguageModel);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: LanguageModel;
  $on(name: string, handler: (model: LanguageModel, event: object) => unknown): () => void;
  /** Renders into a view sink, or a generated mini program view component, e.g. `page.selectComponent("#view")`. */
  mount(view: object): void;
  unmount(): void;
}
