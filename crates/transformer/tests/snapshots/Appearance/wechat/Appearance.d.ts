
export interface AppearanceModel {
  opacity: number;
  hidden: boolean;
  background: string;
  radius: number;
  border: number;
  borderColor: string;
  shadow: "none" | "small" | "medium" | "large";
  clip: boolean;
}

export declare class Appearance {
  constructor(model: AppearanceModel);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: AppearanceModel;
  $on(name: string, handler: (model: AppearanceModel, event: object) => unknown): () => void;
  /** Renders into a view sink, or a generated mini program view component, e.g. `page.selectComponent("#view")`. */
  mount(view: object): void;
  unmount(): void;
}
