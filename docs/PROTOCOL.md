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
- **值槽** `values[i]`：节点的一个动态属性，`type` 是元素属性声明的类型：`Int` / `Bool` / `String` / `Color`。`Color` 槽的值是字符串，由平台解析（规则见 [GRAMMAR.md 的内置元素与布局](GRAMMAR.md#内置元素与布局)）；静态颜色已在编译期校验，`props` 里写成规范化的 `#rrggbb` 或 `#rrggbbaa`。受控属性（`Input.value`、`Checkbox.checked`）即使是常量也总是值槽。
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

小程序 sink（`./wechat`）不编码，直接把 ops 翻译成 `setData` 路径，数据形状见 [PLAN.md 的小程序后端](PLAN.md#m4小程序后端)；无头渲染器经过二进制编码；原生 sink（`./native`）把缓冲交给 `__weft.commit`，见下一节。

## 原生内核

原生平台的 JS bundle 跑在 `native/core` 的内核里：内核嵌入 Hermes，执行 `bundle.hbc`，通过 C ABI（[`native/core/include/weft/runtime.h`](../native/core/include/weft/runtime.h)）和平台壳通信。内核不开线程，所有函数都在 UI 线程上调用，且不能在 `weft_host` 回调里重入。

### `__weft` 全局

内核在 bundle 运行前安装，`@weft-ui/runtime/native` 和 `@weft-ui/platform` 的原生实现通过它工作：

| 成员                              | 含义                                                         |
| --------------------------------- | ------------------------------------------------------------ |
| `open(hash, dispatch)`            | sink 的 `open`。`hash` 与平台壳传给 `weft_runtime_new` 的布局哈希不同就抛错，已有 app 打开时也抛错 |
| `commit(buffer, offset, length)`  | 一批 ops。内核把这段字节直接交给 `weft_host.commit`，不复制  |
| `close()`                         | sink 的 `close`；之后的事件被忽略                            |
| `supports(module)`                | 平台壳是否实现平台模块 `module`                              |
| `call(module, method, args?)`     | 平台调用，返回 `Promise<ArrayBuffer>`。`args` 是字符串（按 UTF-8 发送）、`ArrayBuffer` 或其视图。模块不受支持时立即以 `Error` 拒绝 |
| `version`                         | 平台壳报告的系统版本（`weft_host.version`）                  |

内核还注入 `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval`、`queueMicrotask` 和 `console`（`debug` / `log` / `info` / `warn` / `error` 对应 `weft_host.log` 的 0–3 级；非字符串参数按 JSON 输出，错误对象输出 `stack`）。`TextEncoder` / `TextDecoder` 由 Hermes 自带。

### 事件循环

- `weft_runtime_new` 执行 bundle（bundle 在其中挂载 app），之后每个会执行 JS 的入口（`new`、`tick`、`dispatch`、`resolve`）返回前都清空微任务队列。所以 `weft_runtime_new` 返回时第一批 ops 已经提交。
- 定时器按到期时间、同一时间按设置顺序执行，每个定时器之后清空一次微任务；在一次 `tick` 里新设的定时器留到下一次。内核用 `weft_host.schedule(delay)` 请求下一次 `tick`，平台壳可以合并请求，提前或多余的 `tick` 无害。
- 未捕获的异常（bundle、定时器、微任务、事件处理函数里）以 `WEFT_LOG_ERROR` 报告，不终止运行时。bundle 本身抛错（包括布局哈希不一致）时 `weft_runtime_new` 返回 `NULL`。

### 事件载荷

`weft_runtime_dispatch(rt, scope, slot, payload, len)` 的 `payload` 是一个按[二进制编码](#二进制编码版本-1)的值（标签字节加数据），无载荷的事件传空。返回值即 `Session.dispatch` 的返回值。

### 平台调用

`__weft.call` 分配调用 id 并调用 `weft_host.call(id, module, method, args)`；平台壳异步完成后在 UI 线程调用 `weft_runtime_resolve(id, ok, result)`，每个 id 恰好一次。`ok` 时 Promise 以 `result` 的 `ArrayBuffer` 副本兑现，否则以消息为 `result`（UTF-8）的 `Error` 拒绝。字节的含义由各模块约定：

| 模块      | 方法                         | 参数                                                         | 结果                                       |
| --------- | ---------------------------- | ------------------------------------------------------------ | ------------------------------------------ |
| `fetch`   | `request`                    | 帧：`u32 头长度（小端） \| 头（UTF-8 JSON）\| 请求体`，头是 `{"url", "method", "headers"}` | 同样的帧，头是 `{"status"}`，之后是响应体 |
| `storage` | `get` / `set` / `remove`     | JSON 数组：`[key]` 或 `[key, value]`                         | `get`：JSON 字符串或 `null`；其余为空      |

`@weft-ui/platform` 在 `supports(api)` 为假时直接抛 `PlatformError`，不发起调用。

### 列表移动

自己移动原生行的平台壳（WinUI、GTK）可以用 `weft_stable_rows(old, new, stable)`：它标出新顺序里保持原位的行（旧位置的最长递增子序列），其余行需要移动或插入。C++ 平台壳可以用 `weft/ops.hpp` 解码 ops，解码不分配内存，字符串和行列表直接指向这批字节。
