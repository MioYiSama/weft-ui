---
title: 编写 Weft UI 视图
description: 用类型化参数、布局、带键列表、局部状态和可复用组件编写 .weft 视图。
sidebar:
  order: 2
---

`.weft` 文件描述要渲染的内容和要发出的事件。视图通过类型明确的参数接收逻辑层的数据，用局部 `state` 保存只属于当前视图的状态。[示例应用](https://github.com/weft-ui/weft-ui/blob/master/example/src/App.weft)用布局和事件实现了一个待办清单。

## 声明模型、事件和组件

如果要导入其他文件，把 `import "./File.weft"` 写在文件开头。接下来，用 `model` 描述数据结构，用 `event` 声明逻辑层可以处理的消息，用 `component` 定义可渲染的视图。组件体里的节点之间不需要分隔符。

把下面的待办行组件保存为 `Todo.weft`：

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

`Int`、`Float`、`Bool` 和 `String` 是基础类型。`Todo` 这样的模型名也可以用作类型，用 `todo.name` 这样的写法读取字段。`Todo[]` 是待办事项数组；`Todo?` 可以是一个待办事项，也可以是 `null`。需要 `Float` 的地方可以传 `Int`；需要 `T?` 的地方可以传 `T` 或 `null`。

用组件名实例化组件。位置参数按声明顺序传入，命名参数写在位置参数之后。`TodoRow(todo)` 会让 `showRemove` 采用默认值 `true`，而 `TodoRow(todo, showRemove = false)` 会覆盖默认值。没有默认值的普通参数都必须传入，即使它的类型可空。

## 排列元素

`Column` 将子节点纵向排列，`Row` 将子节点横向排列。两者都从主轴起点依次排列，子节点在交叉轴上对齐起点，不会拉伸。`gap` 是相邻子节点之间的距离，`padding` 是四边相同的内边距。用 `width` 或 `height` 固定相应尺寸；不写时由内容决定。

例如，可以把这些节点写在组件体内：

```weft
Row(gap = 8, padding = 12, background = "#EEF2FF") {
  Image("https://avatars.githubusercontent.com/u/69631?s=64", width = 32, height = 32)
  Text("Tasks", size = 20, bold = true, color = "#1E3A8A")
}
```

各元素的属性、事件和平台支持情况见[内置元素参考](/zh/reference/elements/)。

`Text`、`Input`、`Checkbox` 和 `Image` 的必填值都可以作为第一个位置参数，如上例所示。长度（包括字号 `size`）用逻辑像素计量，直接写 `Int`，不加单位后缀。它对应 iOS 和 macOS 的 pt、Android 的 dp、Windows 的 DIP、Web 的 CSS px，以及小程序的 `px`（不是 `rpx`）。各平台的布局语义一致，控件外观可能不同。

`Text.color`、`Row.background` 等颜色属性接收 `#RGB`、`#RRGGBB` 或 `#RRGGBBAA` 格式的字符串。非法颜色字面量会在编译时报错；非法的动态颜色值按透明处理，并在开发构建中发出警告。

## 用表达式计算属性值

实参可以直接写表达式。数值运算支持 `+`、`-`、`*`、`/` 和 `%`，两个 `Int` 相除是整除。`+` 也能拼接两个字符串。用 `==`、`!=` 判断标量是否相等，用 `<`、`<=`、`>`、`>=` 比较数值或字符串的大小，用 `!`、`&&`、`||` 组合布尔条件。

```weft
component Estimate(title: String, note: String? = null, hours: Float = 0.0) {
  Column(gap = 4) {
    Text(title.trim())
    Text(note?.trim() ?? "No note")
    Text(if (hours > 0.0) "Estimate: ${hours.toFixed(1)} h" else "Not estimated")
  }
}
```

在字符串里用 `${...}` 插入 `Int`、`Float`、`Bool` 或 `String` 类型的值。`if` 表达式返回一个值，条件必须是 `Bool`，必须有 `else`，两个分支的值类型要兼容。用 `?.` 访问可空值的成员，用 `??` 为左侧是 `null` 的情况提供备用值。`length` 和 `isEmpty` 是字符串和数组的属性，不加括号；`trim()` 和 `toFixed(1)` 则是方法调用。

## 按条件渲染和遍历列表

`if` 节点选择要渲染的节点块，不返回值。条件仍然必须是 `Bool`，但 `else` 可以省略，也可以用 `else if` 串接多个条件。`for` 节点遍历数组，循环头必须写 `; key = ...`。给每一项一个稳定且唯一的 `Int` 或 `String` 键，例如待办事项的 ID。这样数组重排时，Weft UI 仍能识别原来的各行。

把下面的代码保存为 `TodoList.weft`，与 `Todo.weft` 放在同一目录：

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

## 局部状态

组件私有的 `state` 写在组件体开头，放在所有节点之前。每个组件实例都有独立的状态。类型可以从初始值推断，也可以像 `state draft: String = ""` 一样显式标注。初始值为 `null` 时，必须显式标注可空类型。

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

内联处理器只能给 `state` 赋值，不能修改组件参数、模型字段或循环变量。`draft` 或 `preview` 变化后，读取它们的视图会随之更新。要修改应用数据，而非局部显示状态，请发出事件交给[逻辑层](/zh/guides/logic/)处理。

## 处理事件

根据需要使用的数据，选择事件参数的写法：

- **只写事件名：** `onChange = DraftChanged` 会将 `Input` 的载荷传给声明为 `event DraftChanged(value: String)` 的事件。有载荷时，事件必须有一个类型兼容的参数；没有载荷时，事件不能带参数。按钮的 `onClick` 没有载荷，复选框的 `onChange` 提供一个 `Bool`。
- **显式传参：** `onChange = Toggle(todo.id)` 在渲染时绑定当前行的 ID，忽略复选框的载荷。要忽略载荷并发出一个不带参数的事件，可以写 `EventName()`。
- **内联 lambda：** `{ value -> draft = value }` 在触发时接收载荷并修改局部状态。无需接收载荷时，可以写成 `{ preview = !preview }`。lambda 只能直接作为事件参数的值使用。

内置元素和组件调用都可以使用 `onMount` 和 `onUnmount`。它们分别在元素或组件挂载和卸载时触发，不提供载荷，只接受已声明的事件，可以显式传参，但不能使用 lambda。

```weft
event DraftChanged(value: String)
event DraftMounted
event DraftUnmounted

component DraftField(value: String) {
  Input(value, onChange = DraftChanged, onMount = DraftMounted, onUnmount = DraftUnmounted)
}
```

## 跨文件组合组件

组件可以声明多个 `Children` 参数作为插槽。在组件体内，把参数名作为节点写在内容要出现的位置。调用时，在尾随块中用 `name: { ... }` 填充同名插槽；不在具名块里的节点填入第一个 `Children` 参数。插槽可以留空，但同一个插槽不能填两次。

把下面的 `App.weft` 和前面两个待办文件放在同一目录：

```weft
import "./Todo.weft"
import "./TodoList.weft"

component Panel(title: String, content: Children, footer: Children) {
  Column(padding = 16, gap = 12) {
    Text(title, size = 20, bold = true)
    content
    footer
  }
}

component App(todos: Todo[]) {
  Panel("我的待办") {
    TodoList(todos)
    footer: { Text("列表结束") }
  }
}
```

导入路径相对于发起导入的文件。导入不传递：`App.weft` 要使用 `TodoList`，就导入 `TodoList.weft`；要引用 `Todo` 类型，还要直接导入 `Todo.weft`。通过导入关联起来的文件中，模型、事件和组件不能重名，也不能与内置元素重名。

完整的语法和类型规则见 [Weft UI 语法规范](https://github.com/weft-ui/weft-ui/blob/master/crates/parser/GRAMMAR.zh-CN.md)。
