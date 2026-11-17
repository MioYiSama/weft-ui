
export interface Todo {
  id: number;
  name: string;
}

export interface TodoListModel {
  todos: readonly Todo[];
  draft: string;
}

export type TextChanged = { value: string };
export type Remove = { id: number };

export declare class TodoList {
  constructor(model: TodoListModel);
  /** Value of the first parameter; event handlers receive the same object. */
  readonly model: TodoListModel;
  onTextChanged(handler: (model: TodoListModel, event: TextChanged) => void): () => void;
  onRemove(handler: (model: TodoListModel, event: Remove) => void): () => void;
  $on(name: string, handler: (model: TodoListModel, event: object) => unknown): () => void;
  /** Renders into a view sink, or a generated mini program view component, e.g. `page.selectComponent("#view")`. */
  mount(view: object): void;
  unmount(): void;
}
