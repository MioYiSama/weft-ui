# 视图协议

JS 逻辑层（`@weft-ui/runtime/remote` 和生成的 remote 模式逻辑模块）与视图层（小程序 WXML、原生平台、无头渲染器）之间的约定。设计背景见 [PLAN-v2.md 的视图协议](PLAN-v2.md#视图协议)，本文是实现以此为准的规范。

参考实现是 `@weft-ui/runtime/testing` 的无头渲染器：它把每批 ops 编码、解码、按布局校验后应用到内存树上。`conformance/` 的黄金文件就是它记录的 ops 文本。

## 布局

`ViewLayout`（`crates/compiler/src/view.rs`）从整个应用的 Program 推导，JSON 形式由 WASM 的 `compileApp(...).layout` 给出，例子见 `crates/driver/tests/snapshots/TodoList/layout.json`。

```json
{
  "hash": "a6bb96edf9a9ad2d",
  "templates": [
    {
      "label": "TodoList",
      "kind": "component",
      "roots": [0],
      "nodes": [{"element": "Column", "props": {}, "children": [1, 2]}, {"point": 0}],
      "values": [{"node": 1, "prop": "value", "type": "String"}],
      "events": [{"node": 1, "event": "onChange", "payload": "String", "controls": 0}],
      "points": [{"node": 2, "kind": "branch", "templates": [1]}]
    }
  ]
}
```

- **模板编号**：先是每个组件体，按 Program 中组件的顺序，所以模板 `i` 就是组件 `i` 的组件体；之后按出现顺序是各组件内的 `if` 分支（每个分支一个）、`for` 行体和插槽内容。`label` 只用于调试：`TodoList`、`TodoList:if8.0`（节点 8 的第 0 个分支）、`TodoList:for7`、`Board:slot12`。
- **节点**：`nodes` 是模板内的局部节点表，`roots` 是顶层节点。元素节点带静态属性 `props`（没有绑定的字面量，平台直接内联进模板，不占槽位）和子节点；`{"point": n}` 是挂载点 `n` 在树中的位置。
- **值槽** `values[i]`：节点的一个动态属性，`type` 是 `Int` / `Bool` / `String`。受控属性（`Input.value`、`Checkbox.checked`）即使是常量也总是值槽。
- **事件槽** `events[i]`：节点的一个事件属性，`payload` 是载荷类型（`null` 表示无载荷）。`Input` / `Checkbox` 总有 `onChange` 事件槽，`controls` 是它控制的值槽下标，其余事件为 `null`。生命周期属性（`onMount` / `onUnmount`）由 JS 处理，不进入布局。
- **挂载点** `points[i]`：`branch`（`templates` 依次是各分支的模板）、`list`（`template` 是行体）、`instance`（`template` 是被实例化组件的组件体）或 `slot`（插槽内容由使用方提供，可以是任意 `slot` 种类的模板）。
- **哈希**：去掉 `hash` 字段后的 JSON 的 FNV-1a 64 位哈希，16 位十六进制。它同时写进 JS 逻辑（`App` 构造参数）和平台视图，sink 在 `open` 时比对，不一致直接报错。

## Scope

运行时每个模板实例是一个 scope。id 由 JS 按 app 分配，从 1 开始递增，不复用。scope 0 是虚拟的根，它只有挂载点 0：根组件实例用 `attach 0 0 <id>` 挂上去。

scope 在 `create` 时创建，此时还没挂上树。它被 `branch` 换掉、不再出现在 `rows` 里、或者祖先被销毁时隐式销毁，后代一起销毁。`unmount` 时 sink 收到 `close`，所有 scope 一起作废。

## Ops

| op       | 参数                     | 含义                                                         |
| -------- | ------------------------ | ------------------------------------------------------------ |
| `create` | scope, template          | 创建实例                                                     |
| `set`    | scope, slot, value       | 写值槽，值的类型与槽的类型一致                               |
| `branch` | scope, point, child \| 0 | `branch` 挂载点切换到 child（0 表示没有分支），旧的子 scope 销毁 |
| `rows`   | scope, point, [child…]   | `list` 挂载点的完整新顺序，不在新列表里的行销毁              |
| `attach` | scope, point, child      | `instance` / `slot` 挂载点挂上 child，每个挂载点只发一次     |

保证：

- 一批 ops 对应一次 flush，sink 在 `commit` 时整批应用。
- 新 scope 的 `create` 和它所有值槽的 `set` 都在同一批里，且先于挂载它的 op。平台总是先建好子树再插入。
- 新 scope 的 `branch` 挂载点为空、`list` 挂载点没有行；状态不变时不发 op（空列表不发 `rows`，顺序不变的列表不重发，值没变不发 `set`）。
- 同一个挂载点上的 child 只能是该挂载点允许的模板的实例，一个 scope 最多挂载一次。

## 事件

反方向只有一种消息：`Session.dispatch(scope, slot, payload) → boolean`，`slot` 是事件槽下标，`payload` 的类型由事件槽决定。JS 同步执行处理函数并 flush，所以返回前 sink 已经收到这次事件引起的 ops。返回值表示受控编辑是否被接受：事件槽没有 `controls`、scope 已销毁、或者被控值槽的当前值等于载荷时为 `true`。返回 `false` 时视图把控件恢复成被控值槽的当前值（宿主改写过的值已经通过 `set` 到达）。已销毁或未知 scope 的事件被忽略并返回 `true`。

## 二进制编码（版本 1）

一批 ops 是一段字节：第一个字节是版本号 `1`，之后是连续的 op。整数都是 u32 小端。

| opcode | op       | 字段                                       |
| ------ | -------- | ------------------------------------------ |
| 1      | `create` | scope, template                            |
| 2      | `set`    | scope, slot, value                         |
| 3      | `branch` | scope, point, child                        |
| 4      | `rows`   | scope, point, count, child × count         |
| 5      | `attach` | scope, point, child                        |

值以一个 u8 标签开头：`0` false、`1` true、`2` 数字（f64 小端，`Int` 槽的值也按 f64 传）、`3` 字符串（u32 字节长度 + UTF-8）。JS 侧的 `OpWriter`（`@weft-ui/runtime` 的 `src/ops.ts`）复用同一块缓冲，每批的字节只在下一次写入前有效。

## 文本形式

调试 dump 和黄金文件使用的一行一个 op 的形式，字符串和布尔值按 JSON 书写：

```
create 3 2
set 3 0 "milk"
rows 2 0 [3 4 5]
branch 1 0 0
attach 0 0 1
```

无头渲染器在 `#` 后附上模板 label 或槽位说明，例如 `create 1 0  # TodoList`、`set 1 0 ""  # Input.value`。

## Sink

`Sink` 是平台一侧的接口（`src/remote.ts`）：

- `open(session)`：mount 时调用一次，`session` 带 `hash`、`dispatch` 和 `detach`（平台视图先于 app 消失时调用）。
- `create` / `set` / `branch` / `rows` / `attach`：记录本批 ops。
- `commit()`：一次 flush 结束，应用整批。
- `close()`：unmount，丢弃所有 scope。

小程序 sink（`./wechat`）不编码，直接把 ops 翻译成 `setData` 路径，数据形状见 [PLAN.md 的小程序后端](PLAN.md#m4小程序后端)；无头渲染器经过二进制编码；原生 sink（P3）把缓冲交给 `__weft.commit`。
