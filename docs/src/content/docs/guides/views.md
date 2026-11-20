---
title: Write Weft UI views
description: Build .weft views with typed parameters, layout, keyed lists, local state, and reusable components.
sidebar:
  order: 2
---

A `.weft` file describes what to render and which events to send. Use typed parameters to receive data from the logic layer and local `state` for details that belong to a single view. The [example app](https://github.com/weft-ui/weft-ui/blob/master/example/src/App.weft) uses both layout and events to build a todo list.

## Declare models, events, and components

Put any `import "./File.weft"` lines first. Then declare your data shapes with `model`, the messages your logic can handle with `event`, and renderable views with `component`. A component body contains nodes, with no separators between them.

Save this reusable row as `Todo.weft`:

```weft
model Todo {
  id: Int,
  name: String,
  done: Bool,
}

event Toggle(id: Int)
event Remove(id: Int)

component TodoRow(todo: Todo, showRemove: Bool = true) {
  Row(gap = 8) {
    Checkbox(todo.done, onChange = Toggle(todo.id))
    Text(todo.name)
    if (showRemove) {
      Button(onClick = Remove(todo.id)) {
        Text("Remove")
      }
    }
  }
}
```

`Int`, `Float`, `Bool`, and `String` are the primitive types. A model name such as `Todo` is a type too, with fields you read as `todo.name`. `Todo[]` is an array of todos; `Todo?` is a todo or `null`. An `Int` can be used where a `Float` is expected, and a `T` or `null` can be used where `T?` is expected.

Call a component by name. Positional arguments follow parameter declaration order, and named arguments come after any positional ones. `TodoRow(todo)` uses the default `true` for `showRemove`; `TodoRow(todo, showRemove = false)` overrides it. Supply each value parameter that has no default, even if its type is nullable.

## Build a layout

Use `Column` to stack children vertically and `Row` to place them side by side. Both start at the beginning of the main axis and align children to the start of the cross axis without stretching them. `gap` adds space between neighboring children; `padding` adds the same inset on all four sides. Set `width` or `height` for a fixed dimension; otherwise the content determines the size.

For example, these nodes can go inside a component body:

```weft
Row(gap = 8, padding = 12, background = "#EEF2FF") {
  Image("https://avatars.githubusercontent.com/u/69631?s=64", width = 32, height = 32)
  Text("Tasks", size = 20, bold = true, color = "#1E3A8A")
}
```

See the [built-in element reference](/reference/elements/) for each element’s properties, events, and platform support.

The required value for `Text`, `Input`, `Checkbox`, or `Image` can be the first positional argument, as shown above. Lengths, including text `size`, are `Int` values in logical pixels, with no unit suffix. These map to pt on iOS and macOS, dp on Android, DIP on Windows, CSS px on the web, and `px` (not `rpx`) in mini programs. Layout semantics are shared, but controls can look different across platforms.

Colors such as `Text.color` or `Row.background` are strings in `#RGB`, `#RRGGBB`, or `#RRGGBBAA` form. Invalid literals are compile errors. Invalid dynamic colors render as transparent and warn in development builds.

## Compute property values

Use expressions directly in arguments. Numeric arithmetic supports `+`, `-`, `*`, `/`, and `%`; division between two `Int` values is integer division. `+` also joins two strings. Use `==` and `!=` for scalar equality, `<`, `<=`, `>`, and `>=` to order numbers or strings, and `!`, `&&`, and `||` for Boolean logic.

```weft
component Estimate(title: String, note: String? = null, hours: Float = 0.0) {
  Column(gap = 4) {
    Text(title.trim())
    Text(note?.trim() ?? "No note")
    Text(if (hours > 0.0) "Estimate: ${hours.toFixed(1)} h" else "Not estimated")
  }
}
```

Use `${...}` to interpolate an `Int`, `Float`, `Bool`, or `String` inside a string. An `if` expression returns one value: the test must be `Bool`, `else` is required, and the two values must have compatible types. Use `?.` to access a member on a nullable value, and `??` to supply a fallback when the left side is `null`. `length` and `isEmpty` are properties of strings and arrays, so they need no parentheses; `trim()` and `toFixed(1)` are method calls.

## Render conditions and lists

An `if` node chooses which block of nodes to render, rather than returning a value. Its test must still be `Bool`, but `else` is optional, and you can chain tests with `else if`. A `for` node iterates an array and requires `; key = ...` in its header. Use a stable, unique `Int` or `String` for each item, such as a todo ID, so reordering the array preserves the identity of each row.

Save this as `TodoList.weft` next to `Todo.weft`:

```weft
import "./Todo.weft"

component TodoList(todos: Todo[]) {
  Column(gap = 8) {
    if (todos.isEmpty) {
      Text("No tasks yet")
    } else {
      for (todo in todos; key = todo.id) {
        TodoRow(todo)
      }
      Text("${todos.length} tasks")
    }
  }
}
```

## State

Declare component-private `state` at the top of the component body, before any nodes. Each component instance has its own state. The initializer can supply the type, or you can write it explicitly, as in `state draft: String = ""`. If the initializer is `null`, give it a nullable type explicitly.

```weft
component DraftPreview() {
  state draft: String = ""
  state preview = false

  Column(gap = 8) {
    Input(draft, placeholder = "New todo", onChange = { value -> draft = value })
    Button(onClick = { preview = !preview }) {
      Text(if (preview) "Hide preview" else "Show preview")
    }
    if (preview && !draft.trim().isEmpty) {
      Text(draft.trim())
    }
  }
}
```

An inline handler can assign to `state`, but not to component parameters, model fields, or loop variables. When `draft` or `preview` changes, the parts of the view that read it update. To change app data rather than local presentation, send an event to the [logic layer](/guides/logic/).

## Connect events

Choose the handler form based on which data you want to use:

- **Event name:** `onChange = DraftChanged` passes the `Input` payload to an event declared as `event DraftChanged(value: String)`. The event must take a single compatible argument if the element provides a payload, or no arguments if it does not. A button's `onClick` has no payload; a checkbox's `onChange` provides a `Bool`.
- **Event with arguments:** `onChange = Toggle(todo.id)` binds the row ID at render time and ignores the checkbox payload. Use `EventName()` if you want to ignore a payload and send an event with no arguments.
- **Inline lambda:** `{ value -> draft = value }` receives the payload and writes local state when triggered. Use `{ preview = !preview }` for a handler that takes no payload parameter. Lambdas can only appear directly as event argument values.

`onMount` and `onUnmount` are available on built-in elements and component calls. They fire when that element or component mounts or unmounts, provide no payload, and accept only declared events, with or without explicit arguments, not lambdas.

```weft
event DraftChanged(value: String)
event DraftMounted
event DraftUnmounted

component DraftField(value: String) {
  Input(value, onChange = DraftChanged, onMount = DraftMounted, onUnmount = DraftUnmounted)
}
```

## Compose components across files

A component can declare one `Children` parameter as a slot. Place the parameter name where the supplied nodes should render, and pass those nodes in a trailing block at the call site. The slot has no default value; omitting the block supplies no children.

Save this `App.weft` beside the two todo files:

```weft
import "./Todo.weft"
import "./TodoList.weft"

component Panel(title: String, content: Children) {
  Column(padding = 16, gap = 12) {
    Text(title, size = 20, bold = true)
    content
  }
}

component App(todos: Todo[]) {
  Panel("My tasks") {
    TodoList(todos)
  }
}
```

Import paths are relative to the file that imports them. Imports are not transitive: `App.weft` imports `TodoList.weft` to use `TodoList`, and it imports `Todo.weft` directly to name the `Todo` type. Use distinct names for models, events, and components across the connected files, and avoid names of built-in elements.

For the complete syntax and type rules, see the [Weft UI grammar specification](https://github.com/weft-ui/weft-ui/blob/master/crates/parser/GRAMMAR.md).
