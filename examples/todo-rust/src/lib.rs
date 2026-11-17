//! The todo list demo with a Rust host; `main.rs` runs it in a window.

pub mod todo_list {
    weft_ui::include_weft!("TodoList");
}

use todo_list::{Todo, TodoList, TodoListModel, events};

/// The demo component with its initial model and event handlers.
#[must_use]
pub fn app() -> TodoList {
    let todo = |id, name: &str| Todo {
        id,
        name: name.to_owned(),
    };
    let app = TodoList::new(TodoListModel {
        draft: String::new(),
        todos: vec![
            todo(1, "Describe a view in Weft"),
            todo(2, "Expand the list"),
            todo(3, "Remove a finished task"),
        ],
    });

    app.on_text_changed(|model, e: events::TextChanged| {
        model.set_draft(e.value);
    });

    app.on_remove(|model, e: events::Remove| {
        model.todos_mut().retain(|t| t.id != e.id);
    });

    app
}
