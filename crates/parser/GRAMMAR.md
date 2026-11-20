# Weft grammar

[中文](GRAMMAR.zh-CN.md)

This document defines the normative grammar of the `.weft` language. The parser (`weft_ui_parser::parse`) implements this grammar. For semantic rules (scope, types, and dependencies), see [Semantic constraints](#semantic-constraints).

## Lexical rules

- Source files are encoded in UTF-8. Newlines, like other whitespace, are insignificant except as token separators.
- Comments: `// line comment` and `/* block comment */`. Block comments do not nest.
- Identifiers: `[A-Za-z_][A-Za-z0-9_]*`.
- Reserved keywords: `if` `else` `for` `in` `true` `false` `null`.
- Contextual keywords (ordinary identifiers elsewhere, such as the `model` parameter in Demo):
  - `import` (followed by a string) and `model` / `event` / `component` (followed by an identifier) begin declarations only at the top level;
  - `state` (followed by an identifier) begins a state declaration only in a component body;
  - `key` appears only in a `for` header.
- Integers: `[0-9]+`, within the range of `i64`. Floating-point numbers: `[0-9]+ "." [0-9]+ ([eE] [+-]? [0-9]+)?` or `[0-9]+ [eE] [+-]? [0-9]+`.
- Strings: `"..."`, with escapes `\n` `\r` `\t` `\0` `\\` `\"` `\$` `\u{XXXX}`.
  `${ expression }` interpolates an expression, which may itself contain strings. Strings cannot span multiple lines.
- Symbols: `( ) { } [ ] , : ; . ?. ? ?? = += -= == != < <= > >= + - * / % ! && || ->`

## Grammar

EBNF notation: `X?` means optional, `X*` means zero or more occurrences, `X+` means one or more occurrences, and `"tok"` denotes a token.

```text
File        = Import* Item* EOF ;
Import      = "import" STRING ;               (* no interpolation *)
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
Arg         = ( IDENT "=" )? Expr ;              (* named arguments must follow positional arguments *)

Expr        = IfExpr | Binary ;
IfExpr      = "if" "(" Expr ")" Expr "else" Expr ;
Binary      = Unary ( BinOp Unary )* ;            (* grouped according to the precedence below *)
Unary       = ( "!" | "-" ) Unary | Postfix ;
Postfix     = Primary ( "." IDENT | "?." IDENT | "(" ArgList? ")" )* ;
Primary     = INT | FLOAT | STRING | "true" | "false" | "null"
            | IDENT | "(" Expr ")" | Lambda ;
Lambda      = "{" ( IDENT "->" )? ( Assign ";"? )* "}" ;
Assign      = IDENT ( "=" | "+=" | "-=" ) Expr ;
```

Binary operator precedence (lowest to highest; all operators are left-associative):

| Precedence | Operators           |
| ---------- | ------------------- |
| 1          | `??`                |
| 2          | `\|\|`              |
| 3          | `&&`                |
| 4          | `==` `!=`           |
| 5          | `<` `<=` `>` `>=`   |
| 6          | `+` `-`             |
| 7          | `*` `/` `%`         |

Notes:

- `state` declarations are allowed only at the top level of a component body, before all nodes.
- Nodes require no separators. An opening `(` or `{` immediately after `Name` belongs to that node, even if the `{` is on the next line.
- In node position, `if` renders conditionally and its branches are blocks of nodes. In expression position, `if` requires an `else` and its branches are expressions.
- Calls are allowed only after identifiers or member accesses: `Remove(todo.id)`, `name.trim()`.
- A lambda may only be used directly as the value of an event argument.
- A `for` header can omit `key` syntactically. Semantic analysis reports its absence to provide a more precise diagnostic.

## Error recovery

The parser continues after an error to report as many errors as possible:

- Top level: Skip to the next `import` / `model` / `event` / `component`.
- Node blocks: Skip unrecognized tokens, skipping over matched delimiters as a unit, until the next token that can start a node or `}`.
- Argument and field lists: Skip to the next `,` or closing delimiter.
- Report only the first error at a given location.

Every AST node carries a source span (file and byte offsets). Diagnostics are located as `file:line:column` and include a source excerpt.

## Semantic constraints

### Scope and imports

- Each file can see its own declarations and those of files it directly imports with `import`. Imports are not transitive.
- Within the import closure, `model`, `event`, and `component` declarations share one namespace. Duplicate names and names that collide with built-in elements are errors.
- Host-side event registration methods are derived from event names (`onFooBar` / `on_foo_bar`). Two event names that differ only in case and underscores (such as `FooBar` and `foo_bar`) are therefore also an error.
- Import cycles and missing files are errors.
- Name lookup within a component proceeds in this order: lambda parameters → `for` variables → `state` → component parameters. Names may not shadow existing names within a component.

### Types

- Primitive types are `Int` `Float` `Bool` `String`. Other types are compound types `T[]` and `T?` and `model` names.
- A component parameter can also be `Children` (a slot, at most one per component). The caller fills it with a trailing block, and the component body uses the parameter name as a node.
- Assignment compatibility: identical types; `Int` → `Float`; `null` → `T?`; `T` → `T?`.
- Arithmetic: `Int op Int → Int` (integer division); the result is `Float` if either operand is `Float`. `String + String → String`.
- Comparisons `< <= > >=` apply only to numbers or strings. `== !=` apply only to scalar types and their nullable forms.
- `a ?? b` requires `a: T?`. `a?.f` requires a nullable receiver and produces a nullable result. Using `?.` on a non-nullable value emits a warning.
- String interpolation accepts only `Int` `Float` `Bool` `String`.
- A `model` cannot refer directly to itself except through an array or a nullable type.

### Built-in members

| Receiver         | Members                                                      |
| ---------------- | ------------------------------------------------------------ |
| `String`         | `length: Int` `isEmpty: Bool` `trim()` `uppercase()` `lowercase()` `contains(String): Bool` |
| `T[]`            | `length: Int` `isEmpty: Bool` `contains(T): Bool` (`T` is scalar) |
| `Int` `Bool`     | `toString(): String`                                         |
| `Float`          | `toString(): String` `toFixed(Int): String`                  |

`length` / `isEmpty` are properties; the rest are methods.

### Event arguments

Element event arguments have payload types (`onClick` has no payload; `Input.onChange` carries a `String`). Accepted values:

- `Event`: Use the event name as a function. Its parameter count must match the payload count (0 or 1). The payload, if any, supplies the sole parameter.
- `Event(args)`: Construct the event instance directly. Arguments are bound at render time, and the payload is ignored.
- `{ stmts }` / `{ v -> stmts }`: An inline lambda. It can assign only to `state`; `v` receives the payload.
- `onMount` / `onUnmount` accept only events, not lambdas.

### Views

- An `if` condition must be `Bool`.
- A `for` must have a `key` of type `Int` or `String`. The value being iterated must be an array.
- Custom components cannot be instantiated recursively except through an `if` / `for`.

### Built-in elements and layout

The compiler rejects elements, properties, and events marked unsupported for the target platform. Those marked ignored are accepted but have no effect. Layout semantics are shared where supported; appearance follows each platform's native style.

<!-- elements:begin -->
<!-- Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit. -->

| Element | Positional property | Events | Children |
| --- | --- | --- | --- |
| `Column` | — | — | Any |
| `Row` | — | — | Any |
| `Text` | `value` | — | None |
| `Button` | — | `onClick` | Any |
| `Input` | `value` | `onChange(String)` | None |
| `Checkbox` | `checked` | `onChange(Bool)` | None |
| `Image` | `src` | — | None |

See the [element reference](https://weftui.mioyi.net/reference/elements/) for full properties, events, and platform support.
<!-- elements:end -->

All elements also have `onMount` / `onUnmount`.

- **Lengths**: `Int`, measured in logical pixels: pt on iOS / macOS, dp on Android, DIP on Windows, and CSS px on the web. Mini programs use `px`, not `rpx`.
- **`Column` / `Row`**: Children are laid out in order from the start of the main axis (vertical for `Column`, horizontal for `Row`). They align to the start of the cross axis, without stretching or automatically filling the available space. As in CSS, `gap` is the space between adjacent children, and `padding` is the same inset on all four sides. When platform defaults differ, the backend sets the alignment explicitly, for example `align-items:flex-start` on the web and in mini programs, and `VStack(alignment: .leading)` in SwiftUI.
- **`width` / `height`**: Fixed dimensions. If omitted, the content determines the size.
- **Colors**: A `Color` property accepts `#RGB`, `#RRGGBB`, or `#RRGGBBAA` (case-insensitive hexadecimal) and is treated as `String` during type checking. Literals are validated at compile time: invalid literals are errors, and valid ones are parsed into RGBA and stored in the view layout. The platform parses dynamic values at runtime, treating invalid values as transparent and emitting a warning in development builds.
- **`Image`**: `src` is an image URL. On platforms without a built-in control for remote images, the core downloads the image.

### Dependency analysis

- A dependency consists of a root and a field path. The root is a component parameter, `state`, or a `for` variable, as in `model.user.name`, `expanded`, or `todo.name`.
  The path traverses only non-nullable `model` fields and stops at an array, a nullable type, a primitive type, or a built-in member such as `length`.
- Dirty bits: Component parameters and `state` are expanded to leaves by the rules above, with one bit per leaf. Each `for` also has a separate set of per-item dirty bits expanded according to the element type.
  A dependency covers every leaf for which it is a prefix. For example, passing `model.user` to a child component depends on every bit under `model.user`.
- Binding sites: Element properties, component arguments, `if` conditions (the union of the conditions across the `if / else if` chain), the `for` list and key, and event constructions with arguments (arguments are bound at render time).
  Values with no dependencies are static and do not generate bindings. Lambdas are evaluated when triggered and do not generate bindings either.
