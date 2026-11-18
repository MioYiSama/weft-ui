//! Semantic analysis: module resolution, name resolution, type checking, and
//! lowering of component bodies to the view IR.

pub(crate) mod builtins;
mod lower;

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};

use weft_ui_ast as ast;
use weft_ui_ast::span::offset;
use weft_ui_ast::{Diagnostic, FileId, ParsedFile, ParsedProgram, SourceMap, Span};

use crate::deps;
use crate::ide::{self, ProbeContext, RawCall, RawProbe, Recorder, Target};
use crate::ir::{
    Component, ComponentId, Event, EventId, Field, Model, ModelId, Module, Param, ParamKind,
    Program, Type, type_display,
};
use lower::{Instance, Lowerer};

/// Name of the slot parameter type.
const CHILDREN: &str = "Children";
const PRIMITIVES: [&str; 4] = ["Int", "Float", "Bool", "String"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclKind {
    Model(ModelId),
    Event(EventId),
    Component(ComponentId),
}

impl DeclKind {
    fn describe(self) -> &'static str {
        match self {
            DeclKind::Model(_) => "model",
            DeclKind::Event(_) => "event",
            DeclKind::Component(_) => "component",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Decl {
    kind: DeclKind,
    file: FileId,
    span: Span,
}

/// Program-wide declarations shared by every component being checked.
pub(crate) struct Checker<'a> {
    sources: &'a SourceMap,
    decls: HashMap<String, Decl>,
    /// Per file: the file itself plus its direct imports.
    visible: Vec<Vec<FileId>>,
    pub models: Vec<Model>,
    pub events: Vec<Event>,
    /// Component signatures; bodies are filled in after all are checked.
    pub components: Vec<Component>,
    /// Present when analyzing for editor tooling.
    recorder: Option<RefCell<Recorder>>,
}

impl Checker<'_> {
    pub(crate) fn recording(&self) -> bool {
        self.recorder.is_some()
    }

    /// Records that the name at `span` refers to `target`.
    pub(crate) fn record(&self, span: Span, target: impl FnOnce() -> Target) {
        if let Some(recorder) = &self.recorder {
            recorder.borrow_mut().refs.push((span, target()));
        }
    }

    pub(crate) fn record_call(&self, call: impl FnOnce() -> RawCall) {
        if let Some(recorder) = &self.recorder {
            recorder.borrow_mut().calls.push(call());
        }
    }

    /// Whether `name` is the probe and no probe was resolved yet.
    pub(crate) fn is_probe(&self, name: &str) -> bool {
        self.recorder
            .as_ref()
            .is_some_and(|r| r.borrow().is_probe(name))
    }

    pub(crate) fn set_probe(&self, context: ProbeContext, candidates: Vec<Target>) {
        if let Some(recorder) = &self.recorder {
            recorder.borrow_mut().probe = Some(RawProbe {
                context,
                candidates,
            });
        }
    }

    /// Declarations visible from `file` that `keep` accepts, in source order.
    pub(crate) fn visible_decls(
        &self,
        file: FileId,
        keep: impl Fn(DeclKind) -> bool,
    ) -> Vec<Target> {
        let visible = &self.visible[file.index()];
        let mut decls: Vec<&Decl> = self
            .decls
            .values()
            .filter(|d| visible.contains(&d.file) && keep(d.kind))
            .collect();
        decls.sort_by_key(|d| (d.span.file, d.span.start));
        decls.into_iter().map(|d| decl_target(d.kind)).collect()
    }

    /// Resolves a global name as seen from `file`.
    ///
    /// Returns `None` for unknown names without reporting. A name declared in a
    /// file that is not imported is reported here but still returned, so callers
    /// do not emit follow-up errors.
    pub(crate) fn global(
        &self,
        file: FileId,
        name: &str,
        span: Span,
        diags: &mut Vec<Diagnostic>,
    ) -> Option<DeclKind> {
        let decl = self.decls.get(name)?;
        self.record(span, || decl_target(decl.kind));
        if !self.visible[file.index()].contains(&decl.file) {
            let path = self.sources.get(decl.file).path();
            diags.push(
                Diagnostic::error(
                    span,
                    format!("`{name}` is declared in `{path}`, which is not imported here"),
                )
                .with_secondary(decl.span, format!("{} declared here", decl.kind.describe()))
                .with_note("imports are not transitive; import the declaring file directly"),
            );
        }
        Some(decl.kind)
    }

    pub(crate) fn type_name(&self, ty: &Type) -> String {
        type_display(ty, &self.models).to_string()
    }

    /// Reports a type probe at `name`; `Children` is offered for parameters.
    pub(crate) fn probe_type(&self, file: FileId, name: &str, children: bool) {
        if !self.is_probe(name) {
            return;
        }
        let builtins = ide::BUILTIN_TYPES
            .iter()
            .filter(|&&t| children || t != CHILDREN)
            .map(|t| Target::Symbol(ide::type_symbol(t)));
        let models = self.visible_decls(file, |k| matches!(k, DeclKind::Model(_)));
        self.set_probe(ProbeContext::Type, builtins.chain(models).collect());
    }

    pub(crate) fn resolve_type(
        &self,
        file: FileId,
        ty: &ast::TypeExpr,
        diags: &mut Vec<Diagnostic>,
    ) -> Type {
        if let ast::TypeExprKind::Named(name) = &ty.kind {
            self.probe_type(file, &name.name, false);
            if PRIMITIVES.contains(&name.name.as_str()) {
                self.record(name.span, || Target::Symbol(ide::type_symbol(&name.name)));
            }
        }
        match &ty.kind {
            ast::TypeExprKind::Error => Type::Error,
            ast::TypeExprKind::Array(elem) => {
                Type::Array(Box::new(self.resolve_type(file, elem, diags)))
            }
            ast::TypeExprKind::Optional(inner) => match self.resolve_type(file, inner, diags) {
                Type::Optional(_) => {
                    diags.push(Diagnostic::error(ty.span, "nested optional type"));
                    Type::Error
                }
                inner => inner.optional(),
            },
            ast::TypeExprKind::Named(name) => match name.name.as_str() {
                "Int" => Type::Int,
                "Float" => Type::Float,
                "Bool" => Type::Bool,
                "String" => Type::String,
                CHILDREN => {
                    diags.push(Diagnostic::error(
                        name.span,
                        "`Children` can only be the type of a component parameter",
                    ));
                    Type::Error
                }
                other => match self.global(file, other, name.span, diags) {
                    Some(DeclKind::Model(id)) => Type::Model(id),
                    Some(kind) => {
                        diags.push(Diagnostic::error(
                            name.span,
                            format!("`{other}` is {}, not a type", article(kind)),
                        ));
                        Type::Error
                    }
                    None => {
                        diags.push(Diagnostic::error(
                            name.span,
                            format!("unknown type `{other}`"),
                        ));
                        Type::Error
                    }
                },
            },
        }
    }

    fn fields(
        &self,
        file: FileId,
        fields: &[ast::Field],
        what: &str,
        diags: &mut Vec<Diagnostic>,
    ) -> Vec<Field> {
        let mut out: Vec<Field> = Vec::with_capacity(fields.len());
        for field in fields {
            if let Some(prev) = out.iter().find(|f| f.name == field.name.name) {
                diags.push(
                    Diagnostic::error(
                        field.name.span,
                        format!("duplicate {what} `{}`", field.name.name),
                    )
                    .with_secondary(prev.span, "first declared here"),
                );
                continue;
            }
            out.push(Field {
                name: field.name.name.clone(),
                ty: self.resolve_type(file, &field.ty, diags),
                span: field.name.span,
            });
        }
        out
    }
}

fn decl_target(kind: DeclKind) -> Target {
    match kind {
        DeclKind::Model(id) => Target::Model(id),
        DeclKind::Event(id) => Target::Event(id),
        DeclKind::Component(id) => Target::Component(id),
    }
}

pub(crate) fn article(kind: DeclKind) -> &'static str {
    match kind {
        DeclKind::Model(_) => "a model",
        DeclKind::Event(_) => "an event",
        DeclKind::Component(_) => "a component",
    }
}

/// Result of analyzing an import closure.
pub(crate) struct Analysis {
    pub sources: SourceMap,
    pub diagnostics: Vec<Diagnostic>,
    /// Present only when no errors were reported and nothing was recorded.
    pub program: Option<Program>,
    /// When recording: the recorder and the program, even with errors.
    pub recording: Option<(Recorder, Program)>,
}

pub(crate) fn analyze(parsed: ParsedProgram) -> Analysis {
    analyze_recording(parsed, None)
}

pub(crate) fn analyze_recording(
    parsed: ParsedProgram,
    recorder: Option<RefCell<Recorder>>,
) -> Analysis {
    let ParsedProgram {
        sources,
        files,
        entries,
        diagnostics: mut diags,
    } = parsed;
    let (program, recorder) = check(&files, entries, &sources, &mut diags, recorder);
    diags.sort_by_key(|d| (d.span.file, d.span.start));
    let (program, recording) = match recorder {
        Some(recorder) => (None, Some((recorder.into_inner(), program))),
        None => (
            (!diags.iter().any(Diagnostic::is_error)).then_some(program),
            None,
        ),
    };
    Analysis {
        sources,
        diagnostics: diags,
        program,
        recording,
    }
}

fn check(
    files: &[ParsedFile],
    entries: Vec<FileId>,
    sources: &SourceMap,
    diags: &mut Vec<Diagnostic>,
    recorder: Option<RefCell<Recorder>>,
) -> (Program, Option<RefCell<Recorder>>) {
    report_import_cycles(files, sources, diags);

    // Declarations: one namespace for the whole import closure.
    let mut cx = Checker {
        sources,
        decls: HashMap::new(),
        visible: files
            .iter()
            .enumerate()
            .map(|(i, f)| {
                std::iter::once(FileId(offset(i)))
                    .chain(f.imports.iter().map(|&(id, _)| id))
                    .collect()
            })
            .collect(),
        models: Vec::new(),
        events: Vec::new(),
        components: Vec::new(),
        recorder,
    };
    let mut item_ids: Vec<Vec<Option<DeclKind>>> = Vec::with_capacity(files.len());
    for (index, file) in files.iter().enumerate() {
        let file_id = FileId(offset(index));
        let ids = file
            .ast
            .items
            .iter()
            .map(|item| declare(&mut cx, file_id, item, diags))
            .collect();
        item_ids.push(ids);
    }
    report_handler_name_clashes(&cx.events, diags);

    let items = || {
        files.iter().enumerate().flat_map(|(fi, f)| {
            f.ast
                .items
                .iter()
                .zip(&item_ids[fi])
                .filter_map(move |(item, id)| id.map(|id| (FileId(offset(fi)), item, id)))
        })
    };

    // Model fields and event parameters.
    for (file, item, id) in items() {
        match (item, id) {
            (ast::Item::Model(m), DeclKind::Model(id)) => {
                let fields = cx.fields(file, &m.fields, "field", diags);
                for (index, field) in fields.iter().enumerate() {
                    cx.record(field.span, || Target::Field(id, index));
                }
                cx.models[id.index()].fields = fields;
            }
            (ast::Item::Event(e), DeclKind::Event(id)) => {
                let params = cx.fields(file, &e.params, "parameter", diags);
                for (index, param) in params.iter().enumerate() {
                    cx.record(param.span, || Target::EventParam(id, index));
                }
                cx.events[id.index()].params = params;
            }
            _ => {}
        }
    }
    report_model_cycles(&cx.models, diags);

    // Component signatures, then bodies.
    let mut bodies = Vec::new();
    for (file, item, id) in items() {
        if let (ast::Item::Component(c), DeclKind::Component(id)) = (item, id) {
            let params = signature(&cx, file, c, diags);
            for (index, param) in params.iter().enumerate() {
                cx.record(param.span, || Target::ComponentParam(id, index));
            }
            cx.components[id.index()].params = params;
            bodies.push((file, c, id));
        }
    }
    let mut instances: Vec<Vec<Instance>> = vec![Vec::new(); cx.components.len()];
    let mut lowered = Vec::with_capacity(bodies.len());
    for &(file, decl, id) in &bodies {
        let params = &cx.components[id.index()].params;
        let body = Lowerer::new(&cx, diags, file, Some(id), params).component(decl);
        lowered.push((id, body));
    }
    for (id, body) in lowered {
        let component = &mut cx.components[id.index()];
        component.states = body.states;
        component.nodes = body.nodes;
        component.roots = body.roots;
        component.loops = body.loops;
        component.bindings = body.bindings;
        component.emits = body.emits;
        instances[id.index()] = body.instances;
    }

    report_unguarded_recursion(&cx.components, &instances, diags);
    let reachable = reachable_events(&cx.components, &instances);
    for (component, events) in cx.components.iter_mut().zip(reachable) {
        component.reachable_events = events;
        deps::layout(component, &cx.models);
    }

    let modules = files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let file = FileId(offset(i));
            let mut module = Module {
                file,
                path: sources.get(file).path().to_owned(),
                imports: f.imports.iter().map(|&(id, _)| id).collect(),
                models: Vec::new(),
                events: Vec::new(),
                components: Vec::new(),
            };
            for id in item_ids[i].iter().flatten() {
                match *id {
                    DeclKind::Model(m) => module.models.push(m),
                    DeclKind::Event(e) => module.events.push(e),
                    DeclKind::Component(c) => module.components.push(c),
                }
            }
            module
        })
        .collect();

    let program = Program {
        entries,
        modules,
        models: cx.models,
        events: cx.events,
        components: cx.components,
    };
    (program, cx.recorder)
}

fn declare(
    cx: &mut Checker<'_>,
    file: FileId,
    item: &ast::Item,
    diags: &mut Vec<Diagnostic>,
) -> Option<DeclKind> {
    let name = item.name();
    let reserved = if builtins::element(&name.name).is_some() {
        Some("a built-in element")
    } else if PRIMITIVES.contains(&name.name.as_str()) || name.name == CHILDREN {
        Some("a built-in type")
    } else {
        None
    };
    if let Some(what) = reserved {
        diags.push(Diagnostic::error(
            name.span,
            format!("`{}` is {what} and cannot be redeclared", name.name),
        ));
        return None;
    }
    if let Some(prev) = cx.decls.get(&name.name) {
        diags.push(
            Diagnostic::error(
                name.span,
                format!("`{}` is declared more than once", name.name),
            )
            .with_label("redeclared here")
            .with_secondary(prev.span, "first declared here")
            .with_note(
                "models, events and components share one namespace across all imported files",
            ),
        );
        return None;
    }
    let kind = match item {
        ast::Item::Model(_) => {
            cx.models.push(Model {
                name: name.name.clone(),
                file,
                fields: Vec::new(),
                span: name.span,
            });
            DeclKind::Model(ModelId(offset(cx.models.len() - 1)))
        }
        ast::Item::Event(_) => {
            cx.events.push(Event {
                name: name.name.clone(),
                file,
                params: Vec::new(),
                span: name.span,
            });
            DeclKind::Event(EventId(offset(cx.events.len() - 1)))
        }
        ast::Item::Component(_) => {
            cx.components.push(Component {
                name: name.name.clone(),
                file,
                span: name.span,
                params: Vec::new(),
                states: Vec::new(),
                nodes: Vec::new(),
                roots: Vec::new(),
                loops: Vec::new(),
                bindings: Vec::new(),
                slots: Vec::new(),
                emits: Vec::new(),
                reachable_events: Vec::new(),
            });
            DeclKind::Component(ComponentId(offset(cx.components.len() - 1)))
        }
    };
    cx.record(name.span, || decl_target(kind));
    cx.decls.insert(
        name.name.clone(),
        Decl {
            kind,
            file,
            span: name.span,
        },
    );
    Some(kind)
}

/// Hosts derive registration methods from event names (`onFooBar`,
/// `on_foo_bar`), so names differing only in case or underscores would clash.
fn report_handler_name_clashes(events: &[Event], diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<String, &Event> = HashMap::new();
    for event in events {
        let key: String = event
            .name
            .chars()
            .filter(|&c| c != '_')
            .map(|c| c.to_ascii_lowercase())
            .collect();
        if let Some(prev) = seen.get(&key) {
            diags.push(
                Diagnostic::error(
                    event.span,
                    format!(
                        "event `{}` clashes with event `{}` in host bindings",
                        event.name, prev.name
                    ),
                )
                .with_secondary(prev.span, "other event declared here")
                .with_note(
                    "handler registration names ignore case and underscores, e.g. `onFooBar` / `on_foo_bar`",
                ),
            );
        } else {
            seen.insert(key, event);
        }
    }
}

fn signature(
    cx: &Checker<'_>,
    file: FileId,
    decl: &ast::ComponentDecl,
    diags: &mut Vec<Diagnostic>,
) -> Vec<Param> {
    let mut params: Vec<Param> = Vec::with_capacity(decl.params.len());
    for param in &decl.params {
        let name = &param.name;
        if let Some(prev) = params.iter().find(|p| p.name == name.name) {
            diags.push(
                Diagnostic::error(name.span, format!("duplicate parameter `{}`", name.name))
                    .with_secondary(prev.span, "first declared here"),
            );
            continue;
        }
        if builtins::LIFECYCLE.iter().any(|p| p.name == name.name) {
            diags.push(Diagnostic::error(
                name.span,
                format!("`{}` is reserved for lifecycle events", name.name),
            ));
            continue;
        }
        if let ast::TypeExprKind::Named(n) = &param.ty.kind {
            cx.probe_type(file, &n.name, true);
        }
        let is_children =
            matches!(&param.ty.kind, ast::TypeExprKind::Named(n) if n.name == CHILDREN);
        if is_children {
            cx.record(param.ty.span, || Target::Symbol(ide::type_symbol(CHILDREN)));
        }
        let kind = if is_children {
            if let Some(prev) = params.iter().find(|p| p.kind == ParamKind::Children) {
                diags.push(
                    Diagnostic::error(
                        name.span,
                        "a component can have only one `Children` parameter",
                    )
                    .with_secondary(prev.span, "first `Children` parameter"),
                );
            }
            if let Some(default) = &param.default {
                diags.push(Diagnostic::error(
                    default.span,
                    "a `Children` parameter cannot have a default value",
                ));
            }
            ParamKind::Children
        } else {
            ParamKind::Value(cx.resolve_type(file, &param.ty, diags))
        };
        let default = match (&kind, &param.default) {
            (ParamKind::Value(ty), Some(default)) => {
                let mut lowerer = Lowerer::new(cx, diags, file, None, &[]);
                Some(lowerer.expr_as(default, ty))
            }
            _ => None,
        };
        params.push(Param {
            name: name.name.clone(),
            kind,
            default,
            span: name.span,
        });
    }
    params
}

/// Finds cycles in a directed graph given as adjacency lists of `(target, edge span)`.
///
/// Each returned cycle lists the nodes from the re-entered node back to itself,
/// together with the span of the edge that closes it.
fn find_cycles(edges: &[Vec<(usize, Span)>]) -> Vec<(Vec<usize>, Span)> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        New,
        Active,
        Done,
    }

    fn visit(
        node: usize,
        edges: &[Vec<(usize, Span)>],
        marks: &mut [Mark],
        stack: &mut Vec<usize>,
        out: &mut Vec<(Vec<usize>, Span)>,
    ) {
        marks[node] = Mark::Active;
        stack.push(node);
        for &(next, span) in &edges[node] {
            match marks[next] {
                Mark::New => visit(next, edges, marks, stack, out),
                Mark::Active => {
                    let start = stack
                        .iter()
                        .position(|&n| n == next)
                        .expect("active nodes are on the stack");
                    let mut cycle = stack[start..].to_vec();
                    cycle.push(next);
                    out.push((cycle, span));
                }
                Mark::Done => {}
            }
        }
        stack.pop();
        marks[node] = Mark::Done;
    }

    let mut marks = vec![Mark::New; edges.len()];
    let mut out = Vec::new();
    for node in 0..edges.len() {
        if marks[node] == Mark::New {
            visit(node, edges, &mut marks, &mut Vec::new(), &mut out);
        }
    }
    out
}

fn report_import_cycles(files: &[ParsedFile], sources: &SourceMap, diags: &mut Vec<Diagnostic>) {
    let edges: Vec<Vec<(usize, Span)>> = files
        .iter()
        .map(|f| {
            f.imports
                .iter()
                .map(|&(id, span)| (id.index(), span))
                .collect()
        })
        .collect();
    for (cycle, span) in find_cycles(&edges) {
        let chain = cycle
            .iter()
            .map(|&i| sources.get(FileId(offset(i))).path())
            .collect::<Vec<_>>()
            .join(" -> ");
        diags.push(
            Diagnostic::error(span, "import cycle")
                .with_label("this import closes the cycle")
                .with_note(chain),
        );
    }
}

fn report_model_cycles(models: &[Model], diags: &mut Vec<Diagnostic>) {
    let edges: Vec<Vec<(usize, Span)>> = models
        .iter()
        .map(|m| {
            m.fields
                .iter()
                .filter_map(|f| match f.ty {
                    Type::Model(id) => Some((id.index(), f.span)),
                    _ => None,
                })
                .collect()
        })
        .collect();
    for (cycle, span) in find_cycles(&edges) {
        let chain = cycle
            .iter()
            .map(|&i| models[i].name.as_str())
            .collect::<Vec<_>>()
            .join(" -> ");
        diags.push(
            Diagnostic::error(span, format!("recursive model has infinite size: {chain}"))
                .with_note("make the field optional (`T?`) or a list (`T[]`)"),
        );
    }
}

fn report_unguarded_recursion(
    components: &[Component],
    instances: &[Vec<Instance>],
    diags: &mut Vec<Diagnostic>,
) {
    let edges: Vec<Vec<(usize, Span)>> = instances
        .iter()
        .map(|list| {
            list.iter()
                .filter(|i| !i.guarded)
                .map(|i| (i.component.index(), i.span))
                .collect()
        })
        .collect();
    for (cycle, span) in find_cycles(&edges) {
        let chain = cycle
            .iter()
            .map(|&i| components[i].name.as_str())
            .collect::<Vec<_>>()
            .join(" -> ");
        diags.push(
            Diagnostic::error(
                span,
                format!("component renders itself unconditionally: {chain}"),
            )
            .with_note("recursive components must be instantiated inside `if` or `for`"),
        );
    }
}

fn reachable_events(components: &[Component], instances: &[Vec<Instance>]) -> Vec<Vec<EventId>> {
    (0..components.len())
        .map(|start| {
            let mut seen = vec![false; components.len()];
            let mut stack = vec![start];
            let mut events = BTreeSet::new();
            seen[start] = true;
            while let Some(c) = stack.pop() {
                events.extend(components[c].emits.iter().copied());
                for instance in &instances[c] {
                    let next = instance.component.index();
                    if !seen[next] {
                        seen[next] = true;
                        stack.push(next);
                    }
                }
            }
            events.into_iter().collect()
        })
        .collect()
}
