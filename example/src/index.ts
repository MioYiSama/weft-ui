import { App } from "./App.weft";
// Platform APIs share one signature everywhere; unsupported ones reject with PlatformError.
import { run, storage } from "@weft-ui/platform";

/** Longest todo name; the input refuses longer drafts. */
const MAX_LENGTH = 12;

const app = new App({
  clicked: false,
  draft: "",
  todos: [],
});
let nextId = 1;

app.onMount(async (model) => {
  model.clicked = (await storage.get("clicked")) === "true";
});

app.onClick((model) => {
  model.clicked = true;
  void storage.set("clicked", "true");
});

app.onDraft((model, { value }) => {
  if (value.length <= MAX_LENGTH) model.draft = value;
});

app.onAdd((model) => {
  model.todos = [...model.todos, { id: nextId++, name: model.draft.trim(), done: false }];
  model.draft = "";
});

app.onToggle((model, { id }) => {
  const todo = model.todos.find((t) => t.id === id);
  if (todo) todo.done = !todo.done;
});

app.onRemove((model, { id }) => {
  model.todos = model.todos.filter((t) => t.id !== id);
});

run(app);
