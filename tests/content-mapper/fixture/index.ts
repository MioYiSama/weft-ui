import { App, type Todo } from "./App.weft";

const app = new App({ todos: [] });
app.onClick((list) => {
  list.todos = [];
});
export const todo: Todo = { id: "1", name: "first" };
