
export interface TypographyModel {
  value: string;
  weight: "regular" | "light" | "medium" | "semibold" | "bold";
  decoration: "none" | "underline" | "lineThrough";
  lines: number;
  hidden: boolean;
  clip: boolean;
  italic: boolean;
}

export declare class Typography {
  constructor(model: TypographyModel);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: TypographyModel;
  $on(name: string, handler: (model: TypographyModel, event: object) => unknown): () => void;
  /** Renders into a view sink, or a generated mini program view component, e.g. `page.selectComponent("#view")`. */
  mount(view: object): void;
  unmount(): void;
}
