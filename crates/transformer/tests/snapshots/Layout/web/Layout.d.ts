
export interface LayoutModel {
  width: number;
  inset: number;
  gap: number;
  grow: number;
  ratio: number;
  fill: boolean;
  align: "start" | "center" | "end" | "stretch";
  justify: "start" | "spaceAround" | "spaceEvenly";
  alignment: "topStart" | "center" | "bottomEnd";
  indicators: boolean;
  spacer: number;
  labels: readonly string[];
}

export declare class Overlay {
  constructor();
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: undefined;
  $on(name: string, handler: (model: undefined, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}

export declare class Layout {
  constructor(model: LayoutModel);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: LayoutModel;
  $on(name: string, handler: (model: LayoutModel, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}
