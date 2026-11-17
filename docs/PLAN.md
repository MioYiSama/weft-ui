# PLAN

核心理念：提供自定义DSL，编译期生成视图代码，宿主语言负责状态和逻辑。

借鉴：

- Kotlin的DSL语法
- Elm的模型解耦
- Slint和QtQuick的DSL理念

## Demo

DSL (`TodoList.weft`):

```weft
model Todo {
  id: Int,
  name: String,
}

model TodoListModel {
  todos: Todo[],
  draft: String,
}

event TextChanged(value: String)
event Remove(id: Int)

component TodoList(model: TodoListModel) {
  state expanded = false

  Column {
    Input(model.draft, onChange = TextChanged)

    Button(onClick = { expanded = !expanded }) { Text("expand") }

    if (!expanded) {
      return
    }

    for (todo in model.todos) {
        Row {
          Text(todo.name)
          Button(onClick = Remove(todo.id)) {
            Text("x")
          }
        }
      }
  }
}
```

宿主（Rust）：

```rust
// build.rs
fn main() {
  weft_ui_compiler::compile("TodoList.weft").unwrap();
}
```

```rust
mod TodoList {
weft_ui::include!("...");
}

fn main() {
  let todo_list = TodoList::new(TodoList::Model{draft:"", todos: vec![]});

    todo_list.on_remove(|model, e|{
        todo!();
    });

  weft_ui::run(todo_list);
}
```

宿主（JavaScript）：

```ts
import TodoList from "TodoList.weft";

const todoList = new TodoList({ draft: "", todos: [] });

todoList.onRemove((model, e) => {
  // TODO
});
```

## 视图更新

编译期依赖分析 编译器从 DSL 里静态算出"Text 节点 #3 只依赖 model.count”

## 渲染后端

Web 端输出 DOM，小程序端输出 WXML/WXSS，桌面端用自绘或原生。

## 工具链

- Rust 用 `build.rs`；JS/TS 用 Vite 插件
- LSP 和格式化工具
- DSL 解释器：实时预览和热重载
