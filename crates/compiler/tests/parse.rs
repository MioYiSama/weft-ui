//! M0: the Demo parses, and syntax errors are reported with positions.

use weft_ui_compiler::syntax::ast::{Item, NodeKind};
use weft_ui_compiler::syntax::parse;
use weft_ui_compiler::{FileId, SourceMap, diagnostic};

const DEMO: &str = include_str!("fixtures/TodoList.weft");

#[test]
fn demo_parses_without_diagnostics() {
    let (file, diags) = parse(FileId(0), DEMO);
    assert!(diags.is_empty(), "{diags:#?}");
    let names: Vec<&str> = file.items.iter().map(|i| i.name().name.as_str()).collect();
    assert_eq!(
        names,
        ["Todo", "TodoListModel", "TextChanged", "Remove", "TodoList"]
    );

    let Some(Item::Component(list)) = file.items.last() else {
        panic!("last item is the TodoList component");
    };
    assert_eq!(
        list.params[0].name.name, "model",
        "`model` is usable as a parameter name"
    );
    assert_eq!(list.states[0].name.name, "expanded");
    let NodeKind::Element(column) = &list.body[0].kind else {
        panic!("root is an element");
    };
    let children = &column.children.as_ref().expect("Column has a block").nodes;
    assert!(matches!(children[2].kind, NodeKind::If(_)));
}

fn render(src: &str) -> String {
    let mut sources = SourceMap::new();
    let id = sources.add("broken.weft".into(), src.into());
    let (_, diags) = parse(id, src);
    diagnostic::render_all(&diags, &sources)
}

#[test]
fn syntax_errors_point_at_source_locations() {
    let src = "\
model Todo {
  id: Int
  name: String,
}

component List(items: Todo[]) {
  Column {
    Text(\"count: ${items.length\")
    Button(onClick = ) { Text(\"x\") }
  }
}
";
    let expected = "\
error: expected `,` or `}`, found identifier
 --> broken.weft:3:3
  |
3 |   name: String,
  |   ^^^^

error: unterminated string literal
 --> broken.weft:8:10
  |
8 |     Text(\"count: ${items.length\")
  |          ^ string starts here

error: expected `,` or `)`, found identifier
 --> broken.weft:9:5
  |
9 |     Button(onClick = ) { Text(\"x\") }
  |     ^^^^^^

error: expected `)`, found `}`
  --> broken.weft:10:3
   |
10 |   }
   |   ^
";
    assert_eq!(render(src), expected);
}

#[test]
fn recovery_reports_errors_in_later_items() {
    let src = "\
component A() {
  Button(onClick = ) { Text(\"x\") }
  state late = 1
}
event E(id: Int id2)
";
    let rendered = render(src);
    let locations: Vec<&str> = rendered
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("--> "))
        .collect();
    assert_eq!(
        locations,
        ["broken.weft:2:20", "broken.weft:3:3", "broken.weft:5:17"],
        "{rendered}"
    );
}
