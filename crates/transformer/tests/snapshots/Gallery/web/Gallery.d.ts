
export interface GalleryStyle {
  padding: number;
  gap: number;
  width: number;
  height: number;
  background: string;
  color: string;
  size: number;
  bold: boolean;
}

export interface GalleryModel {
  style: GalleryStyle;
  checked: boolean;
  imageUrl: string;
  labels: readonly string[];
}

export type Checked = { value: boolean };

export declare class Panel {
  constructor();
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: undefined;
  $on(name: string, handler: (model: undefined, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}

export declare class Gallery {
  constructor(model: GalleryModel);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: GalleryModel;
  onChecked(handler: (model: GalleryModel, event: Checked) => void): () => void;
  $on(name: string, handler: (model: GalleryModel, event: object) => unknown): () => void;
  mount(target: Node): void;
  unmount(): void;
}
