//! Semantic queries for editor tooling.
//!
//! [`analyze`] runs the same analysis as [`crate::check_many`] and records an
//! [`Index`] on the way: every resolved name occurrence (for hover and go to
//! definition), the argument lists of calls (for argument name completion)
//! and, when asked to, what is valid at a *probe*: an identifier the editor
//! puts at the cursor to learn which names can be completed there.
//!
//! Recording works on broken code: analysis continues after errors, and the
//! index holds whatever was resolved.

use std::cell::RefCell;

use crate::Diagnostic;
use crate::ir::{ComponentId, EventId, ModelId, ParamKind, Program, Type};
use crate::loader::SourceLoader;
use crate::sema;
use crate::sema::builtins::{self, ElementSpec, PropKind, PropSpec, PropType};
use crate::span::{FileId, SourceMap, Span};

/// Outcome of [`analyze`].
#[derive(Debug, Clone)]
pub struct IdeAnalysis {
    pub sources: SourceMap,
    /// Errors and warnings, ordered by file and position.
    pub diagnostics: Vec<Diagnostic>,
    pub index: Index,
}

/// Analyzes `entries` and their imports like [`crate::check_many`] and
/// records an [`Index`].
///
/// With `probe`, the index also describes the first place where an
/// identifier with exactly that name is resolved (see [`Probe`]). Pick a name
/// that cannot clash with user code; diagnostics about it are not meaningful.
pub fn analyze(
    entries: &[&str],
    loader: &mut dyn SourceLoader,
    probe: Option<&str>,
) -> IdeAnalysis {
    let recorder = Recorder {
        probe_name: probe.map(str::to_owned),
        ..Recorder::default()
    };
    let analysis = sema::analyze_recording(entries, loader, Some(RefCell::new(recorder)));
    let (recorder, program) = analysis
        .recording
        .expect("analysis returns the recorder it was given");
    let resolve = Resolver {
        program: &program,
        sources: &analysis.sources,
    };
    let index = Index {
        occurrences: recorder
            .refs
            .into_iter()
            .map(|(span, target)| Occurrence {
                span,
                symbol: resolve.symbol(target),
            })
            .collect(),
        calls: recorder
            .calls
            .into_iter()
            .map(|call| {
                let params: Vec<Symbol> =
                    call.params.into_iter().map(|t| resolve.symbol(t)).collect();
                let given = params
                    .iter()
                    .zip(call.given)
                    .filter(|(_, given)| *given)
                    .map(|(p, _)| p.name.clone())
                    .collect();
                CallSite {
                    args: call.args,
                    params,
                    given,
                }
            })
            .collect(),
        probe: recorder.probe.map(|probe| Probe {
            context: probe.context,
            candidates: probe
                .candidates
                .into_iter()
                .map(|t| resolve.symbol(t))
                .collect(),
        }),
    };
    IdeAnalysis {
        sources: analysis.sources,
        diagnostics: analysis.diagnostics,
        index,
    }
}

/// What the analysis learned about names in the analyzed files.
#[derive(Debug, Clone, Default)]
pub struct Index {
    /// Every resolved name, at its declaration and at each use, unordered.
    pub occurrences: Vec<Occurrence>,
    /// Argument lists of element, component and event constructor calls.
    pub calls: Vec<CallSite>,
    /// Present when the probe name was resolved somewhere.
    pub probe: Option<Probe>,
}

impl Index {
    /// The innermost occurrence whose span contains `offset` in `file`; a
    /// cursor right after a name still counts as on it.
    #[must_use]
    pub fn occurrence_at(&self, file: FileId, offset: u32) -> Option<&Occurrence> {
        self.occurrences
            .iter()
            .filter(|o| o.span.file == file && o.span.start <= offset && offset <= o.span.end)
            .min_by_key(|o| o.span.end - o.span.start)
    }

    /// The innermost call whose argument list contains `offset` in `file`.
    #[must_use]
    pub fn call_at(&self, file: FileId, offset: u32) -> Option<&CallSite> {
        self.calls
            .iter()
            .filter(|c| c.args.file == file && c.args.start < offset && offset < c.args.end)
            .min_by_key(|c| c.args.end - c.args.start)
    }

    /// All occurrences of the symbol defined at `def`.
    pub fn references(&self, def: Span) -> impl Iterator<Item = &Occurrence> {
        self.occurrences
            .iter()
            .filter(move |o| o.symbol.def == Some(def))
    }
}

/// A name in the source and the symbol it resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    pub span: Span,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub name: String,
    /// Signature in Weft-like syntax, e.g. `state count: Int` or
    /// `Todo.name: String`.
    pub detail: String,
    /// Span of the declaring name; `None` for built-ins.
    pub def: Option<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Model,
    Event,
    Component,
    /// Model field or event parameter.
    Field,
    /// Component parameter.
    Param,
    State,
    LoopVar,
    LambdaParam,
    /// Built-in element.
    Element,
    /// Built-in element property, including `onMount` / `onUnmount`.
    Prop,
    /// Built-in member such as `length` or `trim()`.
    Member,
    /// Built-in type such as `Int` or `Children`.
    Type,
}

/// An argument list; `args` spans from the callee name to the end of the
/// call, excluding a trailing children block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSite {
    pub args: Span,
    /// Parameters that can be passed by name, in declaration order.
    pub params: Vec<Symbol>,
    /// Names of the parameters already given an argument.
    pub given: Vec<String>,
}

/// Names valid where the probe identifier was resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    pub context: ProbeContext,
    /// Names that can replace the probe, in a stable order.
    pub candidates: Vec<Symbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeContext {
    /// A node in a component body: elements, components and slots.
    Node,
    /// An expression: parameters, states, loop variables, lambda parameters.
    Value,
    /// An event handler value: events.
    Handler,
    /// After `.` / `?.`: fields and built-in members of the receiver.
    Member,
    /// A type annotation: built-in types and models.
    Type,
    /// The target of an assignment in a lambda: states.
    AssignTarget,
}

// ---- recording -----------------------------------------------------------

/// What an occurrence refers to; program-level declarations are resolved to
/// [`Symbol`]s once the whole program is known.
#[derive(Debug, Clone)]
pub(crate) enum Target {
    Model(ModelId),
    Event(EventId),
    Component(ComponentId),
    Field(ModelId, usize),
    EventParam(EventId, usize),
    ComponentParam(ComponentId, usize),
    /// Locals and built-ins, complete when recorded.
    Symbol(Symbol),
}

#[derive(Debug)]
pub(crate) struct RawCall {
    pub args: Span,
    pub params: Vec<Target>,
    /// Per parameter: whether an argument was matched to it.
    pub given: Vec<bool>,
}

#[derive(Debug)]
pub(crate) struct RawProbe {
    pub context: ProbeContext,
    pub candidates: Vec<Target>,
}

/// Collects the index during analysis; owned by the checker in a `RefCell`
/// because the checker is shared immutably while bodies are lowered.
#[derive(Debug, Default)]
pub(crate) struct Recorder {
    pub probe_name: Option<String>,
    pub refs: Vec<(Span, Target)>,
    pub calls: Vec<RawCall>,
    pub probe: Option<RawProbe>,
}

impl Recorder {
    pub(crate) fn is_probe(&self, name: &str) -> bool {
        self.probe.is_none() && self.probe_name.as_deref() == Some(name)
    }
}

// ---- built-in symbols ----------------------------------------------------

/// Built-in types valid in annotations; `Children` only for parameters.
pub(crate) const BUILTIN_TYPES: [&str; 5] = ["Int", "Float", "Bool", "String", "Children"];

/// Every built-in member name, for completion after `.`.
pub(crate) const MEMBER_NAMES: [&str; 8] = [
    "length",
    "isEmpty",
    "trim",
    "uppercase",
    "lowercase",
    "contains",
    "toString",
    "toFixed",
];

pub(crate) fn type_symbol(name: &str) -> Symbol {
    Symbol {
        kind: SymbolKind::Type,
        name: name.to_owned(),
        detail: name.to_owned(),
        def: None,
    }
}

fn prop_type_name(ty: PropType) -> &'static str {
    match ty {
        PropType::Int => "Int",
        PropType::Bool => "Bool",
        PropType::String => "String",
        PropType::Color => "Color",
    }
}

fn prop_signature(prop: &PropSpec) -> String {
    match prop.kind {
        PropKind::Value(ty) => format!("{}: {}", prop.name, prop_type_name(ty)),
        PropKind::Handler { payload: None } | PropKind::Lifecycle => {
            format!("{}: event", prop.name)
        }
        PropKind::Handler { payload: Some(ty) } => {
            format!("{}: event({})", prop.name, prop_type_name(ty))
        }
    }
}

pub(crate) fn prop_symbol(element: Option<&ElementSpec>, prop: &PropSpec) -> Symbol {
    let signature = prop_signature(prop);
    Symbol {
        kind: SymbolKind::Prop,
        name: prop.name.to_owned(),
        detail: match element {
            Some(element) => format!("{}.{signature}", element.name),
            None => signature,
        },
        def: None,
    }
}

pub(crate) fn element_symbol(spec: &ElementSpec) -> Symbol {
    let props: Vec<String> = spec.props.iter().map(prop_signature).collect();
    let children = if spec.children { " { … }" } else { "" };
    Symbol {
        kind: SymbolKind::Element,
        name: spec.name.to_owned(),
        detail: format!("{}({}){children}", spec.name, props.join(", ")),
        def: None,
    }
}

pub(crate) fn element_symbols() -> impl Iterator<Item = Symbol> {
    builtins::ELEMENTS.iter().map(element_symbol)
}

/// Built-in member `name` of a non-optional `recv`, displayed with `display`.
pub(crate) fn member_symbol(
    recv: &Type,
    name: &str,
    display: impl Fn(&Type) -> String,
) -> Option<Symbol> {
    let sig = builtins::member(recv, name)?;
    let detail = match sig.kind {
        builtins::MemberKind::Property => format!("{name}: {}", display(&sig.ret)),
        builtins::MemberKind::Method => {
            let params: Vec<String> = sig.params.iter().map(&display).collect();
            format!("{name}({}): {}", params.join(", "), display(&sig.ret))
        }
    };
    Some(Symbol {
        kind: SymbolKind::Member,
        name: name.to_owned(),
        detail,
        def: None,
    })
}

/// Displays a type, with `unknown` for types that failed to resolve.
pub(crate) fn display_type(ty: &Type, model_name: &dyn Fn(ModelId) -> String) -> String {
    match ty {
        Type::Int => "Int".to_owned(),
        Type::Float => "Float".to_owned(),
        Type::Bool => "Bool".to_owned(),
        Type::String => "String".to_owned(),
        Type::Model(id) => model_name(*id),
        Type::Array(elem) => format!("{}[]", display_type(elem, model_name)),
        Type::Optional(elem) => format!("{}?", display_type(elem, model_name)),
        Type::Null => "null".to_owned(),
        Type::Error => "unknown".to_owned(),
    }
}

// ---- resolution ----------------------------------------------------------

struct Resolver<'a> {
    program: &'a Program,
    sources: &'a SourceMap,
}

impl Resolver<'_> {
    fn ty(&self, ty: &Type) -> String {
        display_type(ty, &|id| self.program.model(id).name.clone())
    }

    fn symbol(&self, target: Target) -> Symbol {
        let program = self.program;
        match target {
            Target::Symbol(symbol) => symbol,
            Target::Model(id) => {
                let model = program.model(id);
                let fields: String = model
                    .fields
                    .iter()
                    .map(|f| format!("\n    {}: {},", f.name, self.ty(&f.ty)))
                    .collect();
                let body = if fields.is_empty() {
                    "{}".to_owned()
                } else {
                    format!("{{{fields}\n}}")
                };
                Symbol {
                    kind: SymbolKind::Model,
                    name: model.name.clone(),
                    detail: format!("model {} {body}", model.name),
                    def: Some(model.span),
                }
            }
            Target::Event(id) => {
                let event = program.event(id);
                let params = if event.params.is_empty() {
                    String::new()
                } else {
                    let list: Vec<String> = event
                        .params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, self.ty(&p.ty)))
                        .collect();
                    format!("({})", list.join(", "))
                };
                Symbol {
                    kind: SymbolKind::Event,
                    name: event.name.clone(),
                    detail: format!("event {}{params}", event.name),
                    def: Some(event.span),
                }
            }
            Target::Component(id) => {
                let component = program.component(id);
                let params: Vec<String> = (0..component.params.len())
                    .map(|i| self.component_param(id, i))
                    .collect();
                Symbol {
                    kind: SymbolKind::Component,
                    name: component.name.clone(),
                    detail: format!("component {}({})", component.name, params.join(", ")),
                    def: Some(component.span),
                }
            }
            Target::Field(id, index) => {
                let model = program.model(id);
                let field = &model.fields[index];
                Symbol {
                    kind: SymbolKind::Field,
                    name: field.name.clone(),
                    detail: format!("{}.{}: {}", model.name, field.name, self.ty(&field.ty)),
                    def: Some(field.span),
                }
            }
            Target::EventParam(id, index) => {
                let event = program.event(id);
                let param = &event.params[index];
                Symbol {
                    kind: SymbolKind::Field,
                    name: param.name.clone(),
                    detail: format!("{}.{}: {}", event.name, param.name, self.ty(&param.ty)),
                    def: Some(param.span),
                }
            }
            Target::ComponentParam(id, index) => {
                let param = &program.component(id).params[index];
                Symbol {
                    kind: SymbolKind::Param,
                    name: param.name.clone(),
                    detail: format!("(parameter) {}", self.component_param(id, index)),
                    def: Some(param.span),
                }
            }
        }
    }

    /// `name: Type` with ` = default` as written in the source.
    fn component_param(&self, id: ComponentId, index: usize) -> String {
        let param = &self.program.component(id).params[index];
        let ty = match &param.kind {
            ParamKind::Value(ty) => self.ty(ty),
            ParamKind::Children => "Children".to_owned(),
        };
        match &param.default {
            Some(default) => format!(
                "{}: {ty} = {}",
                param.name,
                self.sources.snippet(default.span)
            ),
            None => format!("{}: {ty}", param.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryLoader;

    const MODELS: &str = "model Todo { id: Int, name: String }\n\
                          model Model { todos: Todo[], draft: String? }\n\
                          event Remove(id: Int)\n";

    fn analyze_app(app: &str, probe: Option<&str>) -> IdeAnalysis {
        let mut loader = MemoryLoader::new();
        loader.insert("models.weft", MODELS);
        loader.insert("app.weft", app);
        analyze(&["app.weft"], &mut loader, probe)
    }

    fn at<'a>(analysis: &'a IdeAnalysis, app: &str, needle: &str) -> &'a Symbol {
        let offset = app.find(needle).expect("needle in source") as u32;
        &analysis
            .index
            .occurrence_at(FileId(0), offset)
            .unwrap_or_else(|| panic!("no occurrence at `{needle}`"))
            .symbol
    }

    const APP: &str = r##"import "./models.weft"
component Card(title: String, body: Children, footer: String = "end") {
  Column { Text(title) body }
}
component App(model: Model) {
  state count = 0
  Column(gap = 4) {
    for (todo in model.todos; key = todo.id) {
      Text(todo.name.trim(), color = "#fff")
      Button(onClick = Remove(id = todo.id)) { Text("x") }
    }
    Button(onClick = { v -> count += 1 }) { Text("${count}") }
    Card(title = "t") { Text(model.draft ?? "") }
  }
}
"##;

    #[test]
    fn resolves_names_to_signatures_and_definitions() {
        let analysis = analyze_app(APP, None);
        let todo = at(&analysis, APP, "todo in");
        assert_eq!(todo.detail, "(loop variable) todo: Todo");
        assert_eq!(todo.kind, SymbolKind::LoopVar);

        let use_site = at(&analysis, APP, "todo.name");
        assert_eq!(use_site.def, todo.def);

        let name = at(&analysis, APP, "name.trim");
        assert_eq!(name.detail, "Todo.name: String");
        let models = analysis.sources.files()[1].text();
        assert_eq!(
            name.def.map(|d| d.start as usize),
            models.find("name: String")
        );

        assert_eq!(at(&analysis, APP, "trim()").detail, "trim(): String");
        assert_eq!(at(&analysis, APP, "count +=").detail, "state count: Int");
        assert_eq!(at(&analysis, APP, "gap").detail, "Column.gap: Int");
        assert_eq!(at(&analysis, APP, "id = todo").detail, "Remove.id: Int");
        assert_eq!(
            at(&analysis, APP, "Card(title").detail,
            r#"component Card(title: String, body: Children, footer: String = "end")"#
        );
        assert_eq!(
            at(&analysis, APP, "title = ").detail,
            "(parameter) title: String"
        );
        assert_eq!(
            at(&analysis, APP, "Model)").detail,
            "model Model {\n    todos: Todo[],\n    draft: String?,\n}"
        );
        assert_eq!(at(&analysis, APP, "Remove(").kind, SymbolKind::Event);
        assert!(
            at(&analysis, APP, "Text(todo")
                .detail
                .starts_with("Text(value: String")
        );
    }

    #[test]
    fn records_calls_and_given_arguments() {
        let analysis = analyze_app(APP, None);
        let offset = APP.find("color =").expect("needle") as u32;
        let call = analysis
            .index
            .call_at(FileId(0), offset)
            .expect("call at offset");
        let names: Vec<&str> = call.params.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            ["value", "color", "size", "bold", "onMount", "onUnmount"]
        );
        assert_eq!(call.given, ["value", "color"]);
    }

    fn probe(app: &str) -> Probe {
        analyze_app(app, Some("__probe"))
            .index
            .probe
            .expect("probe resolved")
    }

    fn names(probe: &Probe) -> Vec<&str> {
        probe.candidates.iter().map(|c| c.name.as_str()).collect()
    }

    #[test]
    fn probes_report_names_valid_in_context() {
        let app = |body: &str| {
            format!(
                "import \"./models.weft\"\ncomponent App(model: Model, kids: Children) {{\n  state n = 0\n  {body}\n}}\n"
            )
        };

        let p = probe(&app(
            "for (todo in model.todos; key = todo.id) { Text(__probe) }",
        ));
        assert_eq!(p.context, ProbeContext::Value);
        assert_eq!(names(&p), ["model", "n", "todo"]);

        let p = probe(&app("Text(model.__probe)"));
        assert_eq!(p.context, ProbeContext::Member);
        assert_eq!(names(&p), ["todos", "draft"]);

        let p = probe(&app("Text(model.todos.__probe)"));
        assert_eq!(names(&p), ["length", "isEmpty"]);

        let p = probe(&app("Column { __probe }"));
        assert_eq!(p.context, ProbeContext::Node);
        let n = names(&p);
        assert!(n.contains(&"Text") && n.contains(&"App") && n.contains(&"kids"));
        assert!(!n.contains(&"model"));

        let p = probe(&app("Button(onClick = __probe)"));
        assert_eq!(p.context, ProbeContext::Handler);
        assert_eq!(names(&p), ["Remove"]);

        let p = probe(&app("Button(onClick = { __probe = 1 })"));
        assert_eq!(p.context, ProbeContext::AssignTarget);
        assert_eq!(names(&p), ["n"]);

        let p = probe("import \"./models.weft\"\nmodel X { a: __probe }\n");
        assert_eq!(p.context, ProbeContext::Type);
        let n = names(&p);
        assert!(n.contains(&"Int") && n.contains(&"Todo") && !n.contains(&"Remove"));
    }
}
