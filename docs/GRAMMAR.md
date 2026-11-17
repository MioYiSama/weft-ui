# Weft 语法（M0 定稿）

本文是 `.weft` 语言的规范语法，解析器（`weft_ui_compiler::syntax`）按此实现。语义规则（作用域、类型、依赖）见文末[语义约束](#语义约束)。

## 词法

- 源码为 UTF-8。换行与其他空白一样不敏感，只用来分隔 token。
- 注释：`// 行注释`、`/* 块注释 */`（块注释不嵌套）。
- 标识符：`[A-Za-z_][A-Za-z0-9_]*`。
- 保留关键字：`if` `else` `for` `in` `true` `false` `null`。
- 上下文关键字（其他位置是普通标识符，例如 Demo 中的参数 `model`）：
  - `import`（后跟字符串）、`model` / `event` / `component`（后跟标识符）只在顶层开启声明；
  - `state`（后跟标识符）只在组件体内开启状态声明；
  - `key` 只在 `for` 头部出现。
- 整数：`[0-9]+`，范围为 `i64`。浮点：`[0-9]+ "." [0-9]+ ([eE] [+-]? [0-9]+)?` 或 `[0-9]+ [eE] [+-]? [0-9]+`。
- 字符串：`"..."`，转义 `\n` `\r` `\t` `\0` `\\` `\"` `\$` `\u{XXXX}`。
  `${ 表达式 }` 为插值，插值内部可以再写字符串。字符串不能跨行。
- 符号：`( ) { } [ ] , : ; . ?. ? ?? = += -= == != < <= > >= + - * / % ! && || ->`

## 文法

EBNF 记法：`X?` 可选，`X*` 零或多次，`X+` 一或多次，`"tok"` 为 token。

```ebnf
File        = Import* Item* EOF ;
Import      = "import" STRING ;               (* 不允许插值 *)
Item        = Model | Event | Component ;

Model       = "model" IDENT "{" FieldList? "}" ;
Event       = "event" IDENT ( "(" FieldList? ")" )? ;
FieldList   = Field ( "," Field )* ","? ;
Field       = IDENT ":" Type ;

Component   = "component" IDENT "(" ParamList? ")" "{" StateDecl* Node* "}" ;
ParamList   = Param ( "," Param )* ","? ;
Param       = IDENT ":" Type ( "=" Expr )? ;
StateDecl   = "state" IDENT ( ":" Type )? "=" Expr ;

Type        = IDENT TypeSuffix* ;
TypeSuffix  = "[" "]" | "?" ;

Block       = "{" Node* "}" ;
Node        = ElementNode | IfNode | ForNode ;
ElementNode = Path ( "(" ArgList? ")" )? Block? ;
Path        = IDENT ( "." IDENT )* ;
IfNode      = "if" "(" Expr ")" Block ( "else" ( IfNode | Block ) )? ;
ForNode     = "for" "(" IDENT "in" Expr ( ";" "key" "=" Expr )? ")" Block ;

ArgList     = Arg ( "," Arg )* ","? ;
Arg         = ( IDENT "=" )? Expr ;              (* 命名参数必须在位置参数之后 *)

Expr        = IfExpr | Binary ;
IfExpr      = "if" "(" Expr ")" Expr "else" Expr ;
Binary      = Unary ( BinOp Unary )* ;            (* 按下方优先级结合 *)
Unary       = ( "!" | "-" ) Unary | Postfix ;
Postfix     = Primary ( "." IDENT | "?." IDENT | "(" ArgList? ")" )* ;
Primary     = INT | FLOAT | STRING | "true" | "false" | "null"
            | IDENT | "(" Expr ")" | Lambda ;
Lambda      = "{" ( IDENT "->" )? ( Assign ";"? )* "}" ;
Assign      = IDENT ( "=" | "+=" | "-=" ) Expr ;
```

二元运算符优先级（从低到高，全部左结合）：

| 优先级 | 运算符              |
| ------ | ------------------- |
| 1      | `??`                |
| 2      | `\|\|`              |
| 3      | `&&`                |
| 4      | `==` `!=`           |
| 5      | `<` `<=` `>` `>=`   |
| 6      | `+` `-`             |
| 7      | `*` `/` `%`         |

说明：

- `state` 只能写在组件体顶层，并且在所有节点之前。
- 节点之间不需要分隔符。`Name` 后面紧跟的 `(` 和 `{` 都属于这个节点，包括换行之后的 `{`。
- `if` 出现在节点位置时是条件渲染，分支是节点块；出现在表达式位置时是条件表达式，必须带 `else`，分支是表达式。
- 调用只允许出现在标识符或成员访问之后：`Remove(todo.id)`、`name.trim()`。
- Lambda 只能直接作为事件参数的值使用。
- `for` 头部的 `key` 在语法上可以省略，缺失时由语义分析报错，以便给出更准确的提示。

## 错误恢复

解析器在出错后继续解析，尽量报告所有错误：

- 顶层：跳到下一个 `import` / `model` / `event` / `component`。
- 节点块：跳过无法识别的 token（括号成对跳过），直到下一个节点起始 token 或 `}`。
- 参数/字段列表：跳到下一个 `,` 或右括号。
- 同一位置只报告第一个错误。

所有 AST 节点都带源码区间（文件 + 字节偏移），诊断以 `文件:行:列` 定位并附带源码片段。

## 语义约束

### 作用域与导入

- 每个文件可见：本文件声明 + 直接 `import` 的文件声明。导入不传递。
- 导入闭包内 `model`、`event`、`component` 共用一个命名空间，任何重名都报错；也不能与内置元素重名。
- 宿主侧的事件注册方法由事件名派生（`onFooBar` / `on_foo_bar`），因此忽略大小写和下划线后相同的两个事件名（如 `FooBar` 与 `foo_bar`）也报错。
- 循环导入、找不到的文件都报错。
- 组件内名称查找顺序：lambda 参数 → `for` 变量 → `state` → 组件参数。组件内不允许遮蔽已有名称。

### 类型

- 基础类型 `Int` `Float` `Bool` `String`，复合类型 `T[]`、`T?`，以及 `model` 名。
- 组件参数还可以是 `Children`（插槽，每个组件至多一个）。调用方用尾随块填充，组件体内以参数名作为节点使用。
- 赋值兼容：相同类型；`Int` → `Float`；`null` → `T?`；`T` → `T?`。
- 算术：`Int op Int → Int`（整除），含 `Float` 时为 `Float`；`String + String → String`。
- 比较 `< <= > >=` 只用于数值或字符串；`== !=` 只用于标量及其可空类型。
- `a ?? b` 要求 `a: T?`；`a?.f` 要求 `a` 可空，结果可空。对非空值用 `?.` 给出警告。
- 字符串插值只接受 `Int` `Float` `Bool` `String`。
- `model` 不允许不经过数组或可空类型的直接递归。

### 内置成员

| 接收者           | 成员                                                         |
| ---------------- | ------------------------------------------------------------ |
| `String`         | `length: Int` `isEmpty: Bool` `trim()` `uppercase()` `lowercase()` `contains(String): Bool` |
| `T[]`            | `length: Int` `isEmpty: Bool` `contains(T): Bool`（`T` 为标量） |
| `Int` `Bool`     | `toString(): String`                                         |
| `Float`          | `toString(): String` `toFixed(Int): String`                  |

`length` / `isEmpty` 是属性，其余是方法。

### 事件参数

元素的事件参数有载荷类型（`onClick` 无载荷，`Input.onChange` 载荷为 `String`）。可接受的值：

- `Event`：事件名当函数用，事件参数个数必须等于载荷个数（0 或 1），载荷填充唯一参数。
- `Event(args)`：直接构造事件实例，参数在渲染时绑定，忽略载荷。
- `{ stmts }` / `{ v -> stmts }`：内联 lambda，只能给 `state` 赋值；`v` 接收载荷。
- `onMount` / `onUnmount` 只接受事件，不接受 lambda。

### 视图

- `if` 条件为 `Bool`。
- `for` 必须有 `key`，类型为 `Int` 或 `String`；被迭代对象必须是数组。
- 自定义组件不允许不经过 `if` / `for` 的递归实例化。

### 内置元素与布局

所有平台支持同一组内置元素和属性（P2 定稿）。只保证布局语义一致，外观贴近各平台原生风格。

| 元素       | 属性（位置参数加粗）                                         | 事件                         | 子节点 |
| ---------- | ------------------------------------------------------------ | ---------------------------- | ------ |
| `Column`   | `padding` `gap` `width` `height`: `Int`，`background`: `Color` | —                            | 有     |
| `Row`      | 同 `Column`                                                  | —                            | 有     |
| `Text`     | **`value`**: `String`，`color`: `Color`，`size`: `Int`，`bold`: `Bool` | —                  | 无     |
| `Button`   | `disabled`: `Bool`                                           | `onClick`                    | 有     |
| `Input`    | **`value`**: `String`，`placeholder`: `String`，`disabled`: `Bool` | `onChange(String)`     | 无     |
| `Checkbox` | **`checked`**: `Bool`，`disabled`: `Bool`                    | `onChange(Bool)`             | 无     |
| `Image`    | **`src`**: `String`，`width` `height`: `Int`                 | —                            | 无     |

所有元素另有 `onMount` / `onUnmount`。

- **长度**：`Int`，单位是逻辑像素：iOS / macOS 的 pt、Android 的 dp、Windows 的 DIP、Web 的 CSS px；小程序用 `px`，不用 `rpx`。
- **`Column` / `Row`**：子元素沿主轴（`Column` 纵向、`Row` 横向）从起点依次排列，交叉轴对齐起点，不拉伸、不自动撑满；`gap` 是相邻子元素的间距，`padding` 是四边相同的内边距，语义同 CSS。平台默认值不同时由后端显式设置，例如 Web 和小程序写 `align-items:flex-start`，SwiftUI 写 `VStack(alignment: .leading)`。
- **`width` / `height`**：固定尺寸；不写时由内容决定。
- **颜色**：`Color` 属性接受 `#RGB`、`#RRGGBB` 或 `#RRGGBBAA`（十六进制，大小写不限），在类型检查中按 `String` 处理。字面量在编译期校验，不合法时报错，并预解析为 RGBA 写进视图布局；动态值由平台在运行时解析，不合法时按透明处理，在开发构建中打警告。
- **`Image`**：`src` 是图片 URL，没有内置网络图片控件的平台由内核下载。

### 依赖分析

- 依赖是“根 + 字段路径”：根为组件参数、`state` 或 `for` 变量，例如 `model.user.name`、`expanded`、`todo.name`。
  路径只穿过非空 `model` 字段，遇到数组、可空类型、基础类型或内置成员（如 `length`）即截止。
- 脏位：组件参数和 `state` 按上述规则展开到叶子，每个叶子一位；每个 `for` 另有一套按元素类型展开的逐项脏位。
  依赖覆盖所有以它为前缀的叶子，例如把 `model.user` 传给子组件会依赖 `model.user` 下的每一位。
- 绑定点：元素属性、组件实参、`if` 条件（整条 `if / else if` 链的条件并集）、`for` 列表与 key、带参数的事件构造（参数在渲染时绑定）。
  没有依赖的值是静态的，不生成绑定；lambda 在触发时求值，也不生成绑定。
