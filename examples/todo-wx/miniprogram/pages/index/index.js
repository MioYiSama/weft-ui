import { TodoList } from "../../weft/TodoList.js";

Page({
  onLoad() {
    const app = new TodoList({
      draft: "",
      todos: [
        { id: 1, name: "Describe a view in Weft" },
        { id: 2, name: "Expand the list" },
        { id: 3, name: "Remove a finished task" },
      ],
    });

    app.onTextChanged((model, e) => {
      model.draft = e.value;
    });

    app.onRemove((model, e) => {
      model.todos = model.todos.filter((t) => t.id !== e.id);
    });

    app.mount(this.selectComponent("#todo"));
  },
});
