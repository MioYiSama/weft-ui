
export interface TapModel {
  image: string;
  opacity: number;
}

export type Activated = { target: string };

export declare class Tap {
  constructor(model: TapModel);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: TapModel;
  onActivated(handler: (model: TapModel, event: Activated) => void): () => void;
  $on(name: string, handler: (model: TapModel, event: object) => unknown): () => void;
  /** Renders into a view sink, or a generated mini program view component, e.g. `page.selectComponent("#view")`. */
  mount(view: object): void;
  unmount(): void;
}
