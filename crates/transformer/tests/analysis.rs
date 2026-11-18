//! M1: semantic analysis, dependency analysis and the view IR.

use weft_ui_parser::MemoryLoader;
use weft_ui_transformer::ir::{BindingKind, Handler, NodeKind, ParamId, PropValue, SlotRef};
use weft_ui_transformer::{Compilation, check, check_many, dependency_report};

const DEMO: &str = include_str!("fixtures/TodoList.weft");

fn compile(files: &[(&str, &str)]) -> Compilation {
    let mut loader = MemoryLoader::new();
    for (path, text) in files {
        loader.insert(path, *text);
    }
    check(files[0].0, &mut loader)
}

/// `path:line:col: message` for every diagnostic.
fn diagnostics(files: &[(&str, &str)]) -> Vec<String> {
    let compilation = compile(files);
    compilation
        .diagnostics
        .iter()
        .map(|d| {
            let file = compilation.sources.get(d.span.file);
            format!(
                "{}:{}: {}",
                file.path(),
                file.line_col(d.span.start),
                d.message
            )
        })
        .collect()
}

#[test]
fn color_literals_accept_hex_formats() {
    let src = r##"
component C() {
  Column(background = "#aBc") {
    Row(background = "#AABBCCdd") {
      Text("x", color = "#012345")
    }
  }
}
"##;
    assert_eq!(diagnostics(&[("c.weft", src)]), Vec::<String>::new());
}

#[test]
fn invalid_color_literals_report_the_value_span() {
    let src = r#"component C() {
  Column(background = "red") {
    Text("x", color = "gray")
  }
}"#;
    assert_eq!(
        diagnostics(&[("c.weft", src)]),
        [
            "c.weft:2:23: invalid color `red`: expected `#RGB`, `#RRGGBB` or `#RRGGBBAA`",
            "c.weft:3:23: invalid color `gray`: expected `#RGB`, `#RRGGBB` or `#RRGGBBAA`",
        ]
    );
}

#[test]
fn dynamic_colors_accept_strings_without_literal_validation() {
    let src = r#"
component C(color: String) {
  Column(background = color) { Text("x", color = color) }
}
"#;
    assert_eq!(diagnostics(&[("c.weft", src)]), Vec::<String>::new());
}

#[test]
fn layout_preserves_color_property_types_and_normalizes_constants() {
    use weft_ui_transformer::color::Color;
    use weft_ui_transformer::view::{Constant, ValueType, ViewLayout, ViewNodeKind};

    let src = r##"
component C(color: String) {
  Column(background = "#AbC") {
    Text("x", color = color)
    Text("y", color = "#AABBCC80")
  }
}
"##;
    let compilation = compile(&[("c.weft", src)]);
    let program = compilation
        .program
        .as_ref()
        .expect("valid color props compile");
    let layout = ViewLayout::new(program);
    let template = &layout.templates()[0];
    assert_eq!(template.values.len(), 1);
    assert_eq!(template.values[0].prop, "color");
    assert_eq!(template.values[0].ty, ValueType::Color);
    let constants: Vec<&Constant> = template
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            ViewNodeKind::Element { props, .. } => Some(props),
            ViewNodeKind::Point(_) => None,
        })
        .flatten()
        .filter(|prop| matches!(prop.name, "background" | "color"))
        .map(|prop| &prop.value)
        .collect();
    assert_eq!(
        constants,
        [
            &Constant::Color(Color {
                r: 170,
                g: 187,
                b: 204,
                a: 255
            }),
            &Constant::Color(Color {
                r: 170,
                g: 187,
                b: 204,
                a: 128
            }),
        ]
    );
    assert_eq!(constants[0].to_string(), "\"#aabbcc\"");
    assert_eq!(constants[1].to_string(), "\"#aabbcc80\"");
    assert!(layout.to_json().contains("\"type\": \"Color\""));
}

#[test]
fn colors_still_require_string_values() {
    let src = r#"component C() {
  Column(background = 1) {
    Text("x", color = false)
  }
}"#;
    assert_eq!(
        diagnostics(&[("c.weft", src)]),
        [
            "c.weft:2:23: mismatched types: expected `String`, found `Int`",
            "c.weft:3:23: mismatched types: expected `String`, found `Bool`",
        ]
    );
}

#[test]
fn demo_dependency_report() {
    let compilation = compile(&[("ui/TodoList.weft", DEMO)]);
    let program = compilation
        .program
        .as_ref()
        .unwrap_or_else(|| panic!("{}", compilation.render_diagnostics()));
    let expected = "\
component TodoList(model: TodoListModel)
  slots:
    #0     model.todos
    #1     model.draft
    #2     expanded
  loop todo: Todo
    todo#0 todo.id
    todo#1 todo.name
  bindings:
    b0   Column > Input                         value      model.draft  #1
    b1   Column > Button > Text                 value      expanded     #2
    b2   Column > if                            condition  expanded     #2
    b3   Column > if > for todo                 list       model.todos  #0
    b4   Column > if > for todo                 key        todo.id      todo#0
    b5   Column > if > for todo > Row > Text    value      todo.name    todo#1
    b6   Column > if > for todo > Row > Button  onClick    todo.id      todo#0
  emits: TextChanged, Remove
";
    assert_eq!(dependency_report(program), expected);
}

#[test]
fn demo_handlers_lower_to_emits_and_state_writes() {
    let compilation = compile(&[("TodoList.weft", DEMO)]);
    let program = compilation.program.expect("demo compiles");
    let list = program
        .component_named("TodoList")
        .expect("component exists");
    let handlers: Vec<&Handler> = list
        .nodes
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Element { props, .. } => Some(props),
            _ => None,
        })
        .flatten()
        .filter_map(|p| match &p.value {
            PropValue::Handler(h) => Some(h),
            PropValue::Expr(_) => None,
        })
        .collect();
    let summary: Vec<String> = handlers
        .iter()
        .map(|h| match h {
            Handler::Emit { event, args } => {
                format!("emit {} ({} args)", program.event(*event).name, args.len())
            }
            Handler::Lambda { body, .. } => {
                let states: Vec<&str> = body
                    .iter()
                    .map(|a| list.states[a.state.index()].name.as_str())
                    .collect();
                format!("set {}", states.join(","))
            }
        })
        .collect();
    assert_eq!(
        summary,
        [
            "emit TextChanged (1 args)",
            "set expanded",
            "emit Remove (1 args)"
        ]
    );
}

#[test]
fn nested_model_paths_get_their_own_bits() {
    let src = "
model User { name: String, age: Int }
model Page { user: User, title: String }

component Badge(user: User) { Text(user.name) }

component Profile(page: Page) {
  Column {
    Text(page.user.name)
    Text(\"${page.title} (${page.user.age})\")
    Badge(page.user)
  }
}
";
    let compilation = compile(&[("profile.weft", src)]);
    let program = compilation
        .program
        .as_ref()
        .unwrap_or_else(|| panic!("{}", compilation.render_diagnostics()));
    let profile = program
        .component_named("Profile")
        .expect("component exists");
    let bits: Vec<(BindingKind, Vec<SlotRef>)> = profile
        .bindings
        .iter()
        .map(|b| (b.kind, b.bits.clone()))
        .collect();
    // Slots: #0 page.user.name, #1 page.user.age, #2 page.title.
    assert_eq!(
        bits,
        [
            (BindingKind::Prop("value"), vec![SlotRef::Component(0)]),
            (
                BindingKind::Prop("value"),
                vec![SlotRef::Component(1), SlotRef::Component(2)]
            ),
            (
                BindingKind::Arg(ParamId(0)),
                vec![SlotRef::Component(0), SlotRef::Component(1)]
            ),
        ]
    );
}

#[test]
fn view_cannot_assign_model_or_skip_keys() {
    let src = "
model M { items: String[], count: Int }
component C(model: M) {
  state open = false
  Column {
    Button(onClick = { model = model; open = 1 }) { Text(\"x\") }
    for (item in model.items) { Text(item) }
    Text(model.count)
  }
}
";
    assert_eq!(
        diagnostics(&[("c.weft", src)]),
        [
            "c.weft:6:24: cannot assign to parameter `model`",
            "c.weft:6:46: mismatched types: expected `Bool`, found `Int`",
            "c.weft:7:5: `for` requires a key",
            "c.weft:8:10: mismatched types: expected `String`, found `Int`",
        ]
    );
}

#[test]
fn event_handler_shapes_are_checked() {
    let src = "
event Remove(id: Int)
event Loaded
component C() {
  Column(onMount = Loaded) {
    Button(onClick = Remove) { Text(\"a\") }
    Input(\"\", onChange = Remove)
    Input(\"\", onChange = { v -> v = \"\" })
    Button(onClick = Remove(\"x\")) { Text(\"b\") }
    Text(\"c\", onUnmount = { })
  }
}
";
    assert_eq!(
        diagnostics(&[("c.weft", src)]),
        [
            "c.weft:6:22: event `Remove` takes 1 argument, but `onClick` provides no value",
            "c.weft:7:26: `onChange` provides a `String`, but event `Remove` expects `Int`",
            "c.weft:8:33: cannot assign to lambda parameter `v`",
            "c.weft:9:29: mismatched types: expected `Int`, found `String`",
            "c.weft:10:27: `onUnmount` only accepts an event, not a lambda",
        ]
    );
}

#[test]
fn imports_are_scoped_and_share_one_namespace() {
    let app = "
import \"./widgets/button.weft\"
component App(user: User) { Fancy() }
";
    let button = "
import \"../models.weft\"
component Fancy() { Text(\"hi\") }
model Todo { id: Int }
";
    let models = "
model User { name: String }
model Todo { id: Int }
";
    assert_eq!(
        diagnostics(&[
            ("ui/app.weft", app),
            ("ui/widgets/button.weft", button),
            ("ui/models.weft", models),
        ]),
        [
            "ui/app.weft:3:21: `User` is declared in `ui/models.weft`, which is not imported here",
            "ui/models.weft:3:7: `Todo` is declared more than once",
        ]
    );
}

#[test]
fn events_with_clashing_handler_names_are_rejected() {
    let src = "
event FooBar
event foo_bar(id: Int)
event Other
";
    assert_eq!(
        diagnostics(&[("app.weft", src)]),
        ["app.weft:3:7: event `foo_bar` clashes with event `FooBar` in host bindings"]
    );
}

#[test]
fn import_cycles_and_missing_modules_are_reported() {
    let a = "import \"b.weft\"\nimport \"missing.weft\"\n";
    let b = "import \"a.weft\"\n";
    assert_eq!(
        diagnostics(&[("a.weft", a), ("b.weft", b)]),
        [
            "a.weft:2:8: cannot load module `missing.weft`: no such module `missing.weft`",
            "b.weft:1:1: import cycle",
        ]
    );
}

#[test]
fn unguarded_recursion_and_recursive_models_are_rejected() {
    let src = "
model Node { label: String, next: Node, children: Node[] }
model Tree { root: Node?, children: Tree[] }
component Loop(n: Int) { Loop(n) }
component Ok(tree: Tree) {
  for (child in tree.children; key = child.children.length) { Ok(child) }
}
";
    assert_eq!(
        diagnostics(&[("r.weft", src)]),
        [
            "r.weft:2:29: recursive model has infinite size: Node -> Node",
            "r.weft:4:26: component renders itself unconditionally: Loop -> Loop",
        ]
    );
}

#[test]
fn expressions_are_type_checked() {
    let src = "
model U { name: String?, score: Float, tags: String[] }
component C(u: U) {
  state n: Int? = null
  Column {
    Text(u.name ?? \"anon\")
    Text(u.name.length.toString())
    Text(\"${u.score + 1} ${u.tags.contains(\"a\")} ${n ?? 0}\")
    Text(u.tags)
    Text(if (u.tags.isEmpty) \"none\" else u.tags.length)
    Text(u.name?.trim() ?? u.tags.length)
  }
}
";
    assert_eq!(
        diagnostics(&[("c.weft", src)]),
        [
            "c.weft:7:17: value of type `String?` may be null; use `?.length`",
            "c.weft:9:10: mismatched types: expected `String`, found `String[]`",
            "c.weft:10:10: `if` branches have different types: `String` and `Int`",
            "c.weft:11:28: mismatched types: expected `String`, found `Int`",
        ]
    );
}

#[test]
fn app_programs_share_imports_and_names() {
    let mut loader = MemoryLoader::new();
    loader.insert("m.weft", "model M { name: String }");
    loader.insert(
        "a.weft",
        "import \"./m.weft\"\ncomponent A(m: M) { Text(m.name) }",
    );
    loader.insert(
        "b.weft",
        "import \"./m.weft\"\ncomponent B(m: M) { Text(m.name) }",
    );
    loader.insert("c.weft", "component A() { Text(\"c\") }");

    let app = check_many(&["a.weft", "b.weft", "./a.weft"], &mut loader);
    let program = app
        .program
        .as_ref()
        .unwrap_or_else(|| panic!("{}", app.render_diagnostics()));
    let paths: Vec<&str> = program.modules.iter().map(|m| m.path.as_str()).collect();
    assert_eq!(paths, ["a.weft", "b.weft", "m.weft"]);
    let entries: Vec<&str> = program
        .entries
        .iter()
        .map(|&file| program.module(file).path.as_str())
        .collect();
    assert_eq!(entries, ["a.weft", "b.weft"]);

    let clash = check_many(&["a.weft", "c.weft"], &mut loader);
    assert!(clash.program.is_none());
    let messages: Vec<&str> = clash
        .diagnostics
        .iter()
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(messages, ["`A` is declared more than once"]);
}
