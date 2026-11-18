//! Type checks one component body and lowers it to the view IR.

use std::collections::BTreeSet;
use std::ops::Range;

use crate::color::Color;
use crate::deps;
use crate::diagnostic::Diagnostic;
use crate::ide::{self, ProbeContext, RawCall, Symbol, SymbolKind, Target};
use crate::ir::{
    Arg, Assign, Binding, BindingId, BindingKind, Branch, Coercion, ComponentId, Dep, EmitArg,
    EventId, Expr, ExprKind, FieldId, Handler, InterpPart, Loop, LoopId, Node, NodeId, NodeKind,
    Param, ParamId, ParamKind, Prop, PropValue, State, StateId, Type,
};
use crate::sema::builtins::{self, MemberKind, PropKind, PropSpec};
use crate::sema::{Checker, DeclKind, article};
use crate::span::{FileId, Span, offset};
use crate::syntax::ast::{self, AssignOp, BinaryOp, UnaryOp};

/// A component instantiated inside another component's body.
#[derive(Debug, Clone)]
pub(crate) struct Instance {
    pub component: ComponentId,
    /// Inside an `if` or `for`, so recursion through it terminates.
    pub guarded: bool,
    pub span: Span,
}

pub(crate) struct LoweredBody {
    pub states: Vec<State>,
    pub nodes: Vec<Node>,
    pub roots: Vec<NodeId>,
    pub loops: Vec<Loop>,
    pub bindings: Vec<Binding>,
    pub emits: Vec<EventId>,
    pub instances: Vec<Instance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalKind {
    Param(ParamId),
    Slot(ParamId),
    State(StateId),
    Item(LoopId),
    LambdaArg,
}

struct Local {
    name: String,
    kind: LocalKind,
    ty: Type,
    span: Span,
}

/// Placeholder until the owning node is pushed and its bindings are patched.
const PENDING_NODE: NodeId = NodeId(u32::MAX);

/// One expected argument when matching call arguments.
struct ArgSlot<'n> {
    name: &'n str,
    positional: bool,
    required: bool,
}

pub(crate) struct Lowerer<'cx, 'd> {
    cx: &'cx Checker<'cx>,
    diags: &'d mut Vec<Diagnostic>,
    file: FileId,
    /// Component whose body is lowered; `None` for parameter defaults.
    component: Option<ComponentId>,
    params: &'cx [Param],
    states: Vec<State>,
    nodes: Vec<Node>,
    loops: Vec<Loop>,
    bindings: Vec<Binding>,
    scope: Vec<Local>,
    emits: BTreeSet<EventId>,
    instances: Vec<Instance>,
    /// Number of enclosing `if` / `for` nodes.
    guard_depth: u32,
}

impl<'cx, 'd> Lowerer<'cx, 'd> {
    pub(crate) fn new(
        cx: &'cx Checker<'cx>,
        diags: &'d mut Vec<Diagnostic>,
        file: FileId,
        component: Option<ComponentId>,
        params: &'cx [Param],
    ) -> Self {
        Self {
            cx,
            diags,
            file,
            component,
            params,
            states: Vec::new(),
            nodes: Vec::new(),
            loops: Vec::new(),
            bindings: Vec::new(),
            scope: Vec::new(),
            emits: BTreeSet::new(),
            instances: Vec::new(),
            guard_depth: 0,
        }
    }

    pub(crate) fn component(mut self, decl: &ast::ComponentDecl) -> LoweredBody {
        for (index, param) in self.params.iter().enumerate() {
            let id = ParamId(offset(index));
            let (kind, ty) = match &param.kind {
                ParamKind::Value(ty) => (LocalKind::Param(id), ty.clone()),
                ParamKind::Children => (LocalKind::Slot(id), Type::Error),
            };
            self.declare(&param.name, kind, ty, param.span);
        }
        for state in &decl.states {
            self.state(state);
        }
        let roots = self.nodes(&decl.body);
        LoweredBody {
            states: self.states,
            nodes: self.nodes,
            roots,
            loops: self.loops,
            bindings: self.bindings,
            emits: self.emits.into_iter().collect(),
            instances: self.instances,
        }
    }

    // ---- scope ---------------------------------------------------------

    fn error(&mut self, span: Span, message: impl Into<String>) {
        self.diags.push(Diagnostic::error(span, message));
    }

    fn declare(&mut self, name: &str, kind: LocalKind, ty: Type, span: Span) {
        if let Some(prev) = self.scope.iter().find(|l| l.name == name) {
            self.diags.push(
                Diagnostic::error(
                    span,
                    format!("`{name}` is already defined in this component"),
                )
                .with_secondary(prev.span, "previous definition here"),
            );
        }
        let local = Local {
            name: name.to_owned(),
            kind,
            ty,
            span,
        };
        // Parameters are recorded with the component signature.
        if !matches!(kind, LocalKind::Param(_) | LocalKind::Slot(_)) {
            self.cx.record(span, || self.local_target(&local));
        }
        self.scope.push(local);
    }

    fn local(&self, name: &str) -> Option<&Local> {
        self.scope.iter().rev().find(|l| l.name == name)
    }

    fn type_name(&self, ty: &Type) -> String {
        self.cx.type_name(ty)
    }

    // ---- editor index --------------------------------------------------

    fn display_type(&self, ty: &Type) -> String {
        ide::display_type(ty, &|id| self.cx.models[id.index()].name.clone())
    }

    fn local_target(&self, local: &Local) -> Target {
        let (kind, prefix) = match local.kind {
            LocalKind::Param(p) | LocalKind::Slot(p) => match self.component {
                Some(component) => return Target::ComponentParam(component, p.index()),
                None => (SymbolKind::Param, "(parameter) "),
            },
            LocalKind::State(_) => (SymbolKind::State, "state "),
            LocalKind::Item(_) => (SymbolKind::LoopVar, "(loop variable) "),
            LocalKind::LambdaArg => (SymbolKind::LambdaParam, "(lambda parameter) "),
        };
        Target::Symbol(Symbol {
            kind,
            name: local.name.clone(),
            detail: format!("{prefix}{}: {}", local.name, self.display_type(&local.ty)),
            def: Some(local.span),
        })
    }

    /// Reports a probe with the locals `keep` accepts, then `extra`.
    fn probe_locals(
        &self,
        context: ProbeContext,
        keep: impl Fn(LocalKind) -> bool,
        extra: Vec<Target>,
    ) {
        let locals = self
            .scope
            .iter()
            .filter(|l| keep(l.kind))
            .map(|l| self.local_target(l));
        self.cx.set_probe(context, locals.chain(extra).collect());
    }

    /// Records named arguments matched to `targets` (by slot index) and the
    /// argument list as a call site.
    fn record_args(
        &self,
        args_span: Span,
        matched: &[Option<&ast::Arg>],
        targets: impl Fn(usize) -> Target,
    ) {
        if !self.cx.recording() {
            return;
        }
        for (index, arg) in matched.iter().enumerate() {
            if let Some(name) = arg.and_then(|a| a.name.as_ref()) {
                self.cx.record(name.span, || targets(index));
            }
        }
        self.cx.record_call(|| RawCall {
            args: args_span,
            params: (0..matched.len()).map(&targets).collect(),
            given: matched.iter().map(Option::is_some).collect(),
        });
    }

    // ---- bindings ------------------------------------------------------

    fn bind(&mut self, kind: BindingKind, mut deps: Vec<Dep>) -> Option<BindingId> {
        if deps.is_empty() {
            return None;
        }
        deps::normalize(&mut deps);
        self.bindings.push(Binding {
            node: PENDING_NODE,
            kind,
            deps,
            bits: Vec::new(),
        });
        Some(BindingId(offset(self.bindings.len() - 1)))
    }

    fn push_node(&mut self, kind: NodeKind, span: Span, own_bindings: Range<usize>) -> NodeId {
        let id = NodeId(offset(self.nodes.len()));
        self.nodes.push(Node { kind, span });
        for binding in &mut self.bindings[own_bindings] {
            binding.node = id;
        }
        id
    }

    // ---- states --------------------------------------------------------

    fn state(&mut self, decl: &ast::StateDecl) {
        let annotated = decl
            .ty
            .as_ref()
            .map(|ty| self.cx.resolve_type(self.file, ty, self.diags));
        let init = self.expr(&decl.init);
        let (ty, init) = match annotated {
            Some(ty) => {
                let init = self.coerce_to(init, &ty);
                (ty, init)
            }
            None if init.ty == Type::Null => {
                self.error(
                    decl.init.span,
                    "cannot infer the type of `null`; add a type annotation like `state x: Int? = null`",
                );
                (Type::Error, init)
            }
            None => (init.ty.clone(), init),
        };
        let id = StateId(offset(self.states.len()));
        self.declare(
            &decl.name.name,
            LocalKind::State(id),
            ty.clone(),
            decl.name.span,
        );
        self.states.push(State {
            name: decl.name.name.clone(),
            ty,
            init,
            span: decl.name.span,
        });
    }

    // ---- nodes ---------------------------------------------------------

    fn nodes(&mut self, nodes: &[ast::Node]) -> Vec<NodeId> {
        nodes.iter().filter_map(|n| self.node(n)).collect()
    }

    fn guarded_nodes(&mut self, nodes: &[ast::Node]) -> Vec<NodeId> {
        self.guard_depth += 1;
        let ids = self.nodes(nodes);
        self.guard_depth -= 1;
        ids
    }

    fn node(&mut self, node: &ast::Node) -> Option<NodeId> {
        match &node.kind {
            ast::NodeKind::Element(e) => self.element(e, node.span),
            ast::NodeKind::If(i) => Some(self.if_node(i, node.span)),
            ast::NodeKind::For(f) => Some(self.for_node(f, node.span)),
            ast::NodeKind::Error => None,
        }
    }

    fn element(&mut self, e: &ast::ElementNode, span: Span) -> Option<NodeId> {
        let first = e.path.first()?;
        if e.path.len() > 1 {
            let name: Vec<&str> = e.path.iter().map(|i| i.name.as_str()).collect();
            let path_span = first.span.to(e.path[e.path.len() - 1].span);
            self.error(
                path_span,
                format!("unknown platform element `{}`", name.join(".")),
            );
            return None;
        }
        let name = first.name.as_str();
        if self.cx.is_probe(name) {
            let elements = ide::element_symbols().map(Target::Symbol);
            let components = self
                .cx
                .visible_decls(self.file, |k| matches!(k, DeclKind::Component(_)));
            self.probe_locals(
                ProbeContext::Node,
                |k| matches!(k, LocalKind::Slot(_)),
                elements.chain(components).collect(),
            );
        }
        if let Some(local) = self.local(name) {
            self.cx.record(first.span, || self.local_target(local));
            let (kind, local_name) = (local.kind, local.name.clone());
            let LocalKind::Slot(param) = kind else {
                self.error(
                    first.span,
                    format!("`{local_name}` is a value, not an element or component"),
                );
                return None;
            };
            if !e.args.is_empty() || e.children.is_some() {
                self.error(
                    span,
                    format!("slot `{local_name}` takes no arguments or children"),
                );
            }
            return Some(self.push_node(NodeKind::Slot { param }, span, 0..0));
        }
        if let Some(spec) = builtins::element(name) {
            self.cx
                .record(first.span, || Target::Symbol(ide::element_symbol(spec)));
            return Some(self.builtin_element(spec, e, span));
        }
        match self.cx.global(self.file, name, first.span, self.diags) {
            Some(DeclKind::Component(id)) => Some(self.component_call(id, e, span)),
            Some(kind) => {
                self.error(
                    first.span,
                    format!("`{name}` is {}, not an element or component", article(kind)),
                );
                None
            }
            None => {
                self.error(first.span, format!("unknown element or component `{name}`"));
                None
            }
        }
    }

    fn builtin_element(
        &mut self,
        spec: &builtins::ElementSpec,
        e: &ast::ElementNode,
        span: Span,
    ) -> NodeId {
        let props_spec: Vec<PropSpec> = spec
            .props
            .iter()
            .chain(&builtins::LIFECYCLE)
            .copied()
            .collect();
        let slots: Vec<ArgSlot<'_>> = props_spec
            .iter()
            .map(|p| ArgSlot {
                name: p.name,
                positional: p.positional,
                required: p.required,
            })
            .collect();
        let matched = self.match_args(spec.name, &slots, &e.args, span);
        self.record_args(args_span(e, span), &matched, |i| {
            let owner = (i < spec.props.len()).then_some(spec);
            Target::Symbol(ide::prop_symbol(owner, &props_spec[i]))
        });

        let first_binding = self.bindings.len();
        let mut props = Vec::new();
        for (prop, arg) in props_spec.iter().zip(matched) {
            let Some(arg) = arg else { continue };
            let (value, binding) = match prop.kind {
                PropKind::Value(ty) => {
                    let expr = self.expr_as(&arg.value, &ty.to_type());
                    if ty == builtins::PropType::Color
                        && let ExprKind::Str(value) = &expr.kind
                        && Color::parse(value).is_none()
                    {
                        self.error(
                            arg.value.span,
                            format!(
                                "invalid color `{value}`: expected `#RGB`, `#RRGGBB` or `#RRGGBBAA`"
                            ),
                        );
                    }
                    let binding = self.bind(BindingKind::Prop(prop.name), expr_deps(&expr));
                    (PropValue::Expr(expr), binding)
                }
                PropKind::Handler { payload } => {
                    let payload = payload.map(builtins::PropType::to_type);
                    let Some((handler, deps)) = self.handler(&arg.value, prop.name, payload, false)
                    else {
                        continue;
                    };
                    (
                        PropValue::Handler(handler),
                        self.bind(BindingKind::Handler(prop.name), deps),
                    )
                }
                PropKind::Lifecycle => {
                    let Some((handler, deps)) = self.handler(&arg.value, prop.name, None, true)
                    else {
                        continue;
                    };
                    (
                        PropValue::Handler(handler),
                        self.bind(BindingKind::Handler(prop.name), deps),
                    )
                }
            };
            props.push(Prop {
                name: prop.name,
                value,
                binding,
                span: arg.span,
            });
        }
        let own = first_binding..self.bindings.len();

        let children = match &e.children {
            Some(block) if !spec.children => {
                self.error(
                    block.span,
                    format!("`{}` does not accept children", spec.name),
                );
                Vec::new()
            }
            Some(block) => self.nodes(&block.nodes),
            None => Vec::new(),
        };
        self.push_node(
            NodeKind::Element {
                element: spec.element,
                props,
                children,
            },
            span,
            own,
        )
    }

    fn component_call(&mut self, id: ComponentId, e: &ast::ElementNode, span: Span) -> NodeId {
        let cx = self.cx;
        let callee = &cx.components[id.index()];
        self.instances.push(Instance {
            component: id,
            guarded: self.guard_depth > 0,
            span: e.path[0].span,
        });

        // Value parameters are positional in declaration order; lifecycle props follow.
        let value_params: Vec<(ParamId, &Param, &Type)> = callee
            .params
            .iter()
            .enumerate()
            .filter_map(|(i, p)| match &p.kind {
                ParamKind::Value(ty) => Some((ParamId(offset(i)), p, ty)),
                ParamKind::Children => None,
            })
            .collect();
        let mut slots: Vec<ArgSlot<'_>> = value_params
            .iter()
            .map(|(_, p, _)| ArgSlot {
                name: &p.name,
                positional: true,
                required: p.default.is_none(),
            })
            .collect();
        slots.extend(builtins::LIFECYCLE.iter().map(|p| ArgSlot {
            name: p.name,
            positional: false,
            required: false,
        }));
        let matched = self.match_args(&callee.name, &slots, &e.args, span);
        self.record_args(args_span(e, span), &matched, |i| {
            match value_params.get(i) {
                Some(&(param, _, _)) => Target::ComponentParam(id, param.index()),
                None => Target::Symbol(ide::prop_symbol(
                    None,
                    &builtins::LIFECYCLE[i - value_params.len()],
                )),
            }
        });

        let first_binding = self.bindings.len();
        let mut args = Vec::new();
        let mut props = Vec::new();
        for (index, arg) in matched.into_iter().enumerate() {
            let Some(arg) = arg else { continue };
            if let Some(&(param, _, ty)) = value_params.get(index) {
                let value = self.expr_as(&arg.value, ty);
                let binding = self.bind(BindingKind::Arg(param), expr_deps(&value));
                args.push(Arg {
                    param,
                    value,
                    binding,
                    span: arg.span,
                });
            } else {
                let name = builtins::LIFECYCLE[index - value_params.len()].name;
                if let Some((handler, deps)) = self.handler(&arg.value, name, None, true) {
                    let binding = self.bind(BindingKind::Handler(name), deps);
                    props.push(Prop {
                        name,
                        value: PropValue::Handler(handler),
                        binding,
                        span: arg.span,
                    });
                }
            }
        }
        let own = first_binding..self.bindings.len();

        let accepts_children = callee.params.iter().any(|p| p.kind == ParamKind::Children);
        let children = match &e.children {
            Some(block) if !accepts_children => {
                let name = callee.name.clone();
                self.diags.push(
                    Diagnostic::error(
                        block.span,
                        format!("component `{name}` does not accept children"),
                    )
                    .with_note("declare a parameter of type `Children` to accept a block"),
                );
                Vec::new()
            }
            Some(block) => self.nodes(&block.nodes),
            None => Vec::new(),
        };
        self.push_node(
            NodeKind::Component {
                component: id,
                args,
                props,
                children,
            },
            span,
            own,
        )
    }

    fn if_node(&mut self, node: &ast::IfNode, span: Span) -> NodeId {
        let first_binding = self.bindings.len();
        let conds: Vec<Expr> = node
            .branches
            .iter()
            .map(|b| self.expr_as(&b.cond, &Type::Bool))
            .collect();
        let mut deps = Vec::new();
        for cond in &conds {
            deps::collect(cond, &mut deps);
        }
        let binding = self.bind(BindingKind::Condition, deps);
        let own = first_binding..self.bindings.len();

        let mut branches: Vec<Branch> = conds
            .into_iter()
            .zip(&node.branches)
            .map(|(cond, b)| Branch {
                cond: Some(cond),
                children: self.guarded_nodes(&b.block.nodes),
            })
            .collect();
        if let Some(block) = &node.else_block {
            branches.push(Branch {
                cond: None,
                children: self.guarded_nodes(&block.nodes),
            });
        }
        self.push_node(NodeKind::If { branches, binding }, span, own)
    }

    fn for_node(&mut self, node: &ast::ForNode, span: Span) -> NodeId {
        let first_binding = self.bindings.len();
        let list = self.expr(&node.iter);
        let item_ty = match &list.ty {
            Type::Array(elem) => (**elem).clone(),
            Type::Error => Type::Error,
            other => {
                let found = self.type_name(other);
                self.error(
                    node.iter.span,
                    format!("`for` expects a list, found `{found}`"),
                );
                Type::Error
            }
        };
        let list_binding = self.bind(BindingKind::List, expr_deps(&list));

        let loop_id = LoopId(offset(self.loops.len()));
        self.loops.push(Loop {
            var: node.var.name.clone(),
            item_ty: item_ty.clone(),
            node: PENDING_NODE,
            slots: Vec::new(),
        });
        let scope_len = self.scope.len();
        self.declare(
            &node.var.name,
            LocalKind::Item(loop_id),
            item_ty,
            node.var.span,
        );

        let key = match &node.key {
            Some(key) => {
                let key = self.expr(key);
                if !matches!(key.ty, Type::Int | Type::String | Type::Error) {
                    let found = self.type_name(&key.ty);
                    self.error(
                        key.span,
                        format!("`for` key must be `Int` or `String`, found `{found}`"),
                    );
                }
                key
            }
            None => {
                let header = span.to(node.iter.span);
                self.diags.push(
                    Diagnostic::error(header, "`for` requires a key")
                        .with_label(format!("add `; key = {}.<id>`", node.var.name))
                        .with_note("keys identify items so list updates can move nodes instead of recreating them"),
                );
                error_expr(header)
            }
        };
        let key_binding = self.bind(BindingKind::Key, expr_deps(&key));
        let own = first_binding..self.bindings.len();

        let children = self.guarded_nodes(&node.body.nodes);
        self.scope.truncate(scope_len);
        let id = self.push_node(
            NodeKind::For {
                loop_id,
                list,
                key,
                children,
                list_binding,
                key_binding,
            },
            span,
            own,
        );
        self.loops[loop_id.index()].node = id;
        id
    }

    /// Matches call arguments to parameters, reporting every mismatch.
    fn match_args<'a>(
        &mut self,
        callee: &str,
        params: &[ArgSlot<'_>],
        args: &'a [ast::Arg],
        span: Span,
    ) -> Vec<Option<&'a ast::Arg>> {
        let mut out: Vec<Option<&ast::Arg>> = vec![None; params.len()];
        let positional: Vec<usize> = (0..params.len())
            .filter(|&i| params[i].positional)
            .collect();
        let mut next_positional = 0;
        let mut seen_named = false;
        for arg in args {
            let index = match &arg.name {
                None if seen_named => {
                    self.error(arg.span, "positional argument after a named argument");
                    continue;
                }
                None => {
                    let Some(&index) = positional.get(next_positional) else {
                        let n = positional.len();
                        let plural = if n == 1 { "" } else { "s" };
                        self.error(
                            arg.span,
                            format!("`{callee}` takes {n} positional argument{plural}"),
                        );
                        continue;
                    };
                    next_positional += 1;
                    index
                }
                Some(name) => {
                    seen_named = true;
                    let Some(index) = params.iter().position(|p| p.name == name.name) else {
                        self.error(
                            name.span,
                            format!("`{callee}` has no argument named `{}`", name.name),
                        );
                        continue;
                    };
                    index
                }
            };
            if out[index].is_some() {
                self.error(
                    arg.span,
                    format!("argument `{}` is given more than once", params[index].name),
                );
                continue;
            }
            out[index] = Some(arg);
        }
        for (param, arg) in params.iter().zip(&out) {
            if param.required && arg.is_none() {
                self.error(
                    span,
                    format!("missing argument `{}` for `{callee}`", param.name),
                );
            }
        }
        out
    }

    // ---- handlers ------------------------------------------------------

    /// Lowers an event-handler value; returns the handler and its render-time deps.
    fn handler(
        &mut self,
        value: &ast::Expr,
        prop: &str,
        payload: Option<Type>,
        lifecycle: bool,
    ) -> Option<(Handler, Vec<Dep>)> {
        let cx = self.cx;
        match &value.kind {
            ast::ExprKind::Ident(name) if self.local(name).is_none() => {
                self.probe_events(name);
                let event = self.event_named(name, value.span, prop)?;
                self.emits.insert(event);
                let params = &cx.events[event.index()].params;
                let args = match (payload, params.len()) {
                    (None, 0) => Vec::new(),
                    (Some(ty), 1) if assignable(&ty, &params[0].ty) => vec![EmitArg::Payload],
                    (Some(ty), 1) => {
                        let msg = format!(
                            "`{prop}` provides a `{}`, but event `{name}` expects `{}`",
                            self.type_name(&ty),
                            self.type_name(&params[0].ty)
                        );
                        self.error(value.span, msg);
                        return None;
                    }
                    (None, n) => {
                        let plural = if n == 1 { "" } else { "s" };
                        self.diags.push(
                            Diagnostic::error(
                                value.span,
                                format!("event `{name}` takes {n} argument{plural}, but `{prop}` provides no value"),
                            )
                            .with_note(format!("construct the event explicitly: `{name}(...)`")),
                        );
                        return None;
                    }
                    (Some(ty), 0) => {
                        let ty = self.type_name(&ty);
                        self.diags.push(
                            Diagnostic::error(
                                value.span,
                                format!("`{prop}` provides a `{ty}`, but event `{name}` takes no arguments"),
                            )
                            .with_note(format!("write `{name}()` to ignore the value")),
                        );
                        return None;
                    }
                    (Some(_), n) => {
                        self.error(
                            value.span,
                            format!("event `{name}` takes {n} arguments, but `{prop}` provides one value"),
                        );
                        return None;
                    }
                };
                Some((Handler::Emit { event, args }, Vec::new()))
            }
            ast::ExprKind::Call { callee, args } => {
                let ast::ExprKind::Ident(name) = &callee.kind else {
                    self.error(
                        value.span,
                        format!("expected an event or a lambda for `{prop}`"),
                    );
                    return None;
                };
                if self.local(name).is_some() {
                    self.error(callee.span, format!("`{name}` is not an event"));
                    return None;
                }
                self.probe_events(name);
                let event = self.event_named(name, callee.span, prop)?;
                self.emits.insert(event);
                let params = &cx.events[event.index()].params;
                let slots: Vec<ArgSlot<'_>> = params
                    .iter()
                    .map(|p| ArgSlot {
                        name: &p.name,
                        positional: true,
                        required: true,
                    })
                    .collect();
                let matched = self.match_args(name, &slots, args, value.span);
                let call_span = Span::new(self.file, callee.span.end, value.span.end);
                self.record_args(call_span, &matched, |i| Target::EventParam(event, i));
                let mut deps = Vec::new();
                let mut emit_args = Vec::with_capacity(params.len());
                for (param, arg) in params.iter().zip(matched) {
                    let expr = match arg {
                        Some(arg) => self.expr_as(&arg.value, &param.ty),
                        None => error_expr(value.span),
                    };
                    deps::collect(&expr, &mut deps);
                    emit_args.push(EmitArg::Expr(expr));
                }
                Some((
                    Handler::Emit {
                        event,
                        args: emit_args,
                    },
                    deps,
                ))
            }
            ast::ExprKind::Lambda(_) if lifecycle => {
                self.error(
                    value.span,
                    format!("`{prop}` only accepts an event, not a lambda"),
                );
                None
            }
            ast::ExprKind::Lambda(lambda) => Some((self.lambda(lambda, prop, payload), Vec::new())),
            ast::ExprKind::Error => None,
            _ => {
                let what = if lifecycle {
                    "an event"
                } else {
                    "an event or a lambda"
                };
                self.error(value.span, format!("expected {what} for `{prop}`"));
                None
            }
        }
    }

    fn probe_events(&self, name: &str) {
        if self.cx.is_probe(name) {
            let events = self
                .cx
                .visible_decls(self.file, |k| matches!(k, DeclKind::Event(_)));
            self.cx.set_probe(ProbeContext::Handler, events);
        }
    }

    fn event_named(&mut self, name: &str, span: Span, prop: &str) -> Option<EventId> {
        match self.cx.global(self.file, name, span, self.diags) {
            Some(DeclKind::Event(id)) => Some(id),
            Some(kind) => {
                self.error(
                    span,
                    format!(
                        "`{name}` is {}, but `{prop}` expects an event",
                        article(kind)
                    ),
                );
                None
            }
            None => {
                self.error(span, format!("unknown event `{name}`"));
                None
            }
        }
    }

    fn lambda(&mut self, lambda: &ast::Lambda, prop: &str, payload: Option<Type>) -> Handler {
        let scope_len = self.scope.len();
        if let Some(param) = &lambda.param {
            let ty = match payload {
                Some(ty) => ty,
                None => {
                    self.error(
                        param.span,
                        format!(
                            "`{prop}` provides no value; remove the parameter `{}`",
                            param.name
                        ),
                    );
                    Type::Error
                }
            };
            self.declare(&param.name, LocalKind::LambdaArg, ty, param.span);
        }
        let body = lambda.body.iter().filter_map(|a| self.assign(a)).collect();
        self.scope.truncate(scope_len);
        Handler::Lambda {
            param: lambda.param.as_ref().map(|p| p.name.clone()),
            body,
        }
    }

    fn assign(&mut self, assign: &ast::Assign) -> Option<Assign> {
        let target = &assign.target;
        if self.cx.is_probe(&target.name) {
            self.probe_locals(
                ProbeContext::AssignTarget,
                |k| matches!(k, LocalKind::State(_)),
                Vec::new(),
            );
        }
        let Some(local) = self.local(&target.name) else {
            self.error(target.span, format!("unknown state `{}`", target.name));
            return None;
        };
        self.cx.record(target.span, || self.local_target(local));
        let (kind, ty) = (local.kind, local.ty.clone());
        let LocalKind::State(state) = kind else {
            let what = match kind {
                LocalKind::Param(_) | LocalKind::Slot(_) => "parameter",
                LocalKind::Item(_) => "loop variable",
                LocalKind::LambdaArg | LocalKind::State(_) => "lambda parameter",
            };
            self.diags.push(
                Diagnostic::error(
                    target.span,
                    format!("cannot assign to {what} `{}`", target.name),
                )
                .with_label("only `state` can be assigned in a view")
                .with_note("to change the model, send an event and update it in the host"),
            );
            return None;
        };
        let valid_op = match assign.op {
            AssignOp::Set => true,
            AssignOp::Add => ty.is_numeric() || ty == Type::String || ty == Type::Error,
            AssignOp::Sub => ty.is_numeric() || ty == Type::Error,
        };
        if !valid_op {
            let op = if assign.op == AssignOp::Add {
                "+="
            } else {
                "-="
            };
            let ty_name = self.type_name(&ty);
            self.error(
                assign.span,
                format!("cannot apply `{op}` to state of type `{ty_name}`"),
            );
        }
        let value = self.expr_as(&assign.value, &ty);
        Some(Assign {
            state,
            op: assign.op,
            value,
            span: assign.span,
        })
    }

    // ---- expressions ---------------------------------------------------

    /// Lowers `e` and checks that it can be used where `ty` is expected.
    pub(crate) fn expr_as(&mut self, e: &ast::Expr, ty: &Type) -> Expr {
        let expr = self.expr(e);
        self.coerce_to(expr, ty)
    }

    fn coerce_to(&mut self, expr: Expr, ty: &Type) -> Expr {
        match coerce(expr, ty) {
            Ok(expr) => expr,
            Err(expr) => {
                let msg = format!(
                    "mismatched types: expected `{}`, found `{}`",
                    self.type_name(ty),
                    self.type_name(&expr.ty)
                );
                self.error(expr.span, msg);
                Expr {
                    ty: Type::Error,
                    ..expr
                }
            }
        }
    }

    fn expr(&mut self, e: &ast::Expr) -> Expr {
        let span = e.span;
        let (kind, ty) = match &e.kind {
            ast::ExprKind::Int(v) => (ExprKind::Int(*v), Type::Int),
            ast::ExprKind::Float(v) => (ExprKind::Float(*v), Type::Float),
            ast::ExprKind::Bool(v) => (ExprKind::Bool(*v), Type::Bool),
            ast::ExprKind::Null => (ExprKind::Null, Type::Null),
            ast::ExprKind::Str(parts) => (self.string(parts), Type::String),
            ast::ExprKind::Ident(name) => return self.ident(name, span),
            ast::ExprKind::Field { base, name, safe } => {
                let base = self.expr(base);
                return self.access(base, name, *safe, None, span);
            }
            ast::ExprKind::Call { callee, args } => return self.call(callee, args, span),
            ast::ExprKind::Unary { op, expr } => return self.unary(*op, expr, span),
            ast::ExprKind::Binary { op, lhs, rhs } => return self.binary(*op, lhs, rhs, span),
            ast::ExprKind::If { cond, then, else_ } => {
                let cond = self.expr_as(cond, &Type::Bool);
                let then = self.expr(then);
                let else_ = self.expr(else_);
                let Some(ty) = unify(&then.ty, &else_.ty) else {
                    let msg = format!(
                        "`if` branches have different types: `{}` and `{}`",
                        self.type_name(&then.ty),
                        self.type_name(&else_.ty)
                    );
                    self.error(span, msg);
                    return error_expr(span);
                };
                let then = self.coerce_to(then, &ty);
                let else_ = self.coerce_to(else_, &ty);
                (
                    ExprKind::If {
                        cond: Box::new(cond),
                        then: Box::new(then),
                        else_: Box::new(else_),
                    },
                    ty,
                )
            }
            ast::ExprKind::Lambda(_) => {
                self.error(span, "a lambda can only be used as an event handler");
                return error_expr(span);
            }
            ast::ExprKind::Error => return error_expr(span),
        };
        Expr { kind, ty, span }
    }

    fn string(&mut self, parts: &[ast::StrPart]) -> ExprKind {
        if let [] | [ast::StrPart::Lit(_)] = parts {
            let text = match parts {
                [ast::StrPart::Lit(s)] => s.clone(),
                _ => String::new(),
            };
            return ExprKind::Str(text);
        }
        let parts = parts
            .iter()
            .map(|part| match part {
                ast::StrPart::Lit(s) => InterpPart::Lit(s.clone()),
                ast::StrPart::Expr(e) => {
                    let expr = self.expr(e);
                    if !expr.ty.is_scalar() && !expr.ty.has_error() {
                        let ty = self.type_name(&expr.ty);
                        self.error(
                            expr.span,
                            format!("cannot interpolate a value of type `{ty}`"),
                        );
                    }
                    InterpPart::Expr(expr)
                }
            })
            .collect();
        ExprKind::Interp(parts)
    }

    fn ident(&mut self, name: &str, span: Span) -> Expr {
        if self.cx.is_probe(name) {
            self.probe_locals(
                ProbeContext::Value,
                |k| !matches!(k, LocalKind::Slot(_)),
                Vec::new(),
            );
        }
        if let Some(local) = self.local(name) {
            self.cx.record(span, || self.local_target(local));
        }
        if let Some((kind, ty)) = self.local(name).map(|l| (l.kind, l.ty.clone())) {
            let kind = match kind {
                LocalKind::Param(p) => ExprKind::Param(p),
                LocalKind::State(s) => ExprKind::State(s),
                LocalKind::Item(l) => ExprKind::Item(l),
                LocalKind::LambdaArg => ExprKind::LambdaArg,
                LocalKind::Slot(_) => {
                    self.error(
                        span,
                        format!("`{name}` is a `Children` slot; use it as a node, not a value"),
                    );
                    return error_expr(span);
                }
            };
            return Expr { kind, ty, span };
        }
        let msg = match self.cx.global(self.file, name, span, self.diags) {
            Some(DeclKind::Event(_)) => {
                format!("event `{name}` can only be used as an event handler value")
            }
            Some(DeclKind::Model(_)) => format!("`{name}` is a model type, not a value"),
            Some(DeclKind::Component(_)) => {
                format!("component `{name}` can only be used as a node")
            }
            None => format!("unknown name `{name}`"),
        };
        self.error(span, msg);
        error_expr(span)
    }

    fn call(&mut self, callee: &ast::Expr, args: &[ast::Arg], span: Span) -> Expr {
        match &callee.kind {
            ast::ExprKind::Field { base, name, safe } => {
                let base = self.expr(base);
                self.access(base, name, *safe, Some(args), span)
            }
            ast::ExprKind::Ident(name) => {
                let msg = match self.cx.global(self.file, name, callee.span, self.diags) {
                    Some(DeclKind::Event(_)) if self.local(name).is_none() => {
                        format!("event `{name}` can only be constructed as an event handler value")
                    }
                    _ => format!("unknown function `{name}`; only built-in members can be called"),
                };
                self.error(callee.span, msg);
                error_expr(span)
            }
            _ => unreachable!("parser only produces calls on identifiers and fields"),
        }
    }

    /// Field access or built-in member access; `call` holds method arguments.
    fn access(
        &mut self,
        recv: Expr,
        name: &ast::Ident,
        safe: bool,
        call: Option<&[ast::Arg]>,
        span: Span,
    ) -> Expr {
        if self.cx.is_probe(&name.name) {
            self.probe_members(&recv.ty);
        }
        let (inner, nullable) = match &recv.ty {
            Type::Error => return error_expr(span),
            Type::Optional(inner) if safe => ((**inner).clone(), true),
            Type::Optional(_) | Type::Null => {
                let ty = self.type_name(&recv.ty);
                self.error(
                    name.span,
                    format!("value of type `{ty}` may be null; use `?.{}`", name.name),
                );
                return error_expr(span);
            }
            ty => {
                if safe {
                    let ty_name = self.type_name(ty);
                    self.diags.push(Diagnostic::warning(
                        name.span,
                        format!("unnecessary `?.`: a value of type `{ty_name}` is never null"),
                    ));
                }
                (ty.clone(), false)
            }
        };
        let wrap = |ty: Type| if nullable { ty.optional() } else { ty };

        if let Type::Model(model_id) = inner {
            let cx = self.cx;
            let model = &cx.models[model_id.index()];
            let Some(index) = model.fields.iter().position(|f| f.name == name.name) else {
                let msg = format!("model `{}` has no field `{}`", model.name, name.name);
                self.error(name.span, msg);
                return error_expr(span);
            };
            if call.is_some() {
                self.error(span, format!("field `{}` cannot be called", name.name));
                return error_expr(span);
            }
            self.cx.record(name.span, || Target::Field(model_id, index));
            let ty = wrap(model.fields[index].ty.clone());
            return Expr {
                kind: ExprKind::Field {
                    base: Box::new(recv),
                    field: FieldId(offset(index)),
                    safe: nullable,
                },
                ty,
                span,
            };
        }

        let Some(sig) = builtins::member(&inner, &name.name) else {
            let ty = self.type_name(&inner);
            self.error(
                name.span,
                format!("type `{ty}` has no member `{}`", name.name),
            );
            return error_expr(span);
        };
        self.cx.record(name.span, || {
            let member = ide::member_symbol(&inner, &name.name, |t| self.display_type(t));
            Target::Symbol(member.expect("the member exists"))
        });
        let args = match (sig.kind, call) {
            (MemberKind::Property, None) => Vec::new(),
            (MemberKind::Property, Some(_)) => {
                self.error(
                    span,
                    format!("`{}` is a property; remove the parentheses", name.name),
                );
                return error_expr(span);
            }
            (MemberKind::Method, None) => {
                self.error(
                    name.span,
                    format!("`{0}` is a method; call it as `{0}()`", name.name),
                );
                return error_expr(span);
            }
            (MemberKind::Method, Some(args)) => {
                if args.len() != sig.params.len() {
                    let n = sig.params.len();
                    let plural = if n == 1 { "" } else { "s" };
                    self.error(
                        span,
                        format!(
                            "`{}` takes {n} argument{plural}, but {} were given",
                            name.name,
                            args.len()
                        ),
                    );
                    return error_expr(span);
                }
                let mut lowered = Vec::with_capacity(args.len());
                for (arg, ty) in args.iter().zip(&sig.params) {
                    if let Some(arg_name) = &arg.name {
                        self.error(
                            arg_name.span,
                            "built-in methods take positional arguments only",
                        );
                    }
                    lowered.push(self.expr_as(&arg.value, ty));
                }
                lowered
            }
        };
        Expr {
            kind: ExprKind::Member {
                member: sig.member,
                recv: Box::new(recv),
                args,
                safe: nullable,
            },
            ty: wrap(sig.ret),
            span,
        }
    }

    /// Reports a member probe on a receiver of type `recv`.
    fn probe_members(&self, recv: &Type) {
        let inner = match recv {
            Type::Optional(inner) => inner,
            ty => ty,
        };
        let candidates = match inner {
            Type::Model(id) => (0..self.cx.models[id.index()].fields.len())
                .map(|i| Target::Field(*id, i))
                .collect(),
            ty => ide::MEMBER_NAMES
                .iter()
                .filter_map(|m| ide::member_symbol(ty, m, |t| self.display_type(t)))
                .map(Target::Symbol)
                .collect(),
        };
        self.cx.set_probe(ProbeContext::Member, candidates);
    }

    fn unary(&mut self, op: UnaryOp, operand: &ast::Expr, span: Span) -> Expr {
        let expr = self.expr(operand);
        let ty = match (op, &expr.ty) {
            (_, Type::Error) => Type::Error,
            (UnaryOp::Not, Type::Bool) => Type::Bool,
            (UnaryOp::Neg, ty @ (Type::Int | Type::Float)) => ty.clone(),
            (_, ty) => {
                let symbol = if op == UnaryOp::Not { "!" } else { "-" };
                let ty = self.type_name(ty);
                self.error(span, format!("cannot apply `{symbol}` to `{ty}`"));
                Type::Error
            }
        };
        Expr {
            kind: ExprKind::Unary {
                op,
                expr: Box::new(expr),
            },
            ty,
            span,
        }
    }

    fn binary(&mut self, op: BinaryOp, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Expr {
        let lhs = self.expr(lhs);
        let rhs = self.expr(rhs);
        let make = |lhs: Expr, rhs: Expr, ty: Type| Expr {
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty,
            span,
        };
        if lhs.ty.has_error() || rhs.ty.has_error() {
            return make(lhs, rhs, Type::Error);
        }
        let numeric = lhs.ty.is_numeric() && rhs.ty.is_numeric();
        let both_strings = lhs.ty == Type::String && rhs.ty == Type::String;
        let float_or_int = if lhs.ty == Type::Float || rhs.ty == Type::Float {
            Type::Float
        } else {
            Type::Int
        };
        let ok = match op {
            BinaryOp::Add if both_strings => Some((Type::String, Type::String)),
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem
                if numeric =>
            {
                Some((float_or_int.clone(), float_or_int))
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge if numeric => {
                Some((float_or_int, Type::Bool))
            }
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge if both_strings => {
                Some((Type::String, Type::Bool))
            }
            BinaryOp::Eq | BinaryOp::Ne => unify(&lhs.ty, &rhs.ty)
                .filter(is_comparable)
                .map(|ty| (ty, Type::Bool)),
            BinaryOp::And | BinaryOp::Or if lhs.ty == Type::Bool && rhs.ty == Type::Bool => {
                Some((Type::Bool, Type::Bool))
            }
            BinaryOp::Coalesce => return self.coalesce(lhs, rhs, span),
            _ => None,
        };
        let Some((operand_ty, result_ty)) = ok else {
            let msg = format!(
                "cannot apply `{}` to `{}` and `{}`",
                op.symbol(),
                self.type_name(&lhs.ty),
                self.type_name(&rhs.ty)
            );
            self.error(span, msg);
            return make(lhs, rhs, Type::Error);
        };
        let lhs = self.coerce_to(lhs, &operand_ty);
        let rhs = self.coerce_to(rhs, &operand_ty);
        make(lhs, rhs, result_ty)
    }

    fn coalesce(&mut self, lhs: Expr, rhs: Expr, span: Span) -> Expr {
        let ty = match &lhs.ty {
            // A nullable fallback keeps the result nullable.
            Type::Optional(_) if matches!(rhs.ty, Type::Optional(_) | Type::Null) => lhs.ty.clone(),
            Type::Optional(inner) => (**inner).clone(),
            ty => {
                let ty_name = self.type_name(ty);
                self.diags.push(Diagnostic::warning(
                    lhs.span,
                    format!("left side of `??` has type `{ty_name}` and is never null"),
                ));
                ty.clone()
            }
        };
        let rhs = self.coerce_to(rhs, &ty);
        Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Coalesce,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty,
            span,
        }
    }
}

/// From the end of the element's name to its children block or end.
fn args_span(e: &ast::ElementNode, span: Span) -> Span {
    let start = e.path.last().map_or(span.start, |p| p.span.end);
    let end = e.children.as_ref().map_or(span.end, |b| b.span.start);
    Span::new(span.file, start, end.max(start))
}

fn error_expr(span: Span) -> Expr {
    Expr {
        kind: ExprKind::Error,
        ty: Type::Error,
        span,
    }
}

fn expr_deps(expr: &Expr) -> Vec<Dep> {
    let mut deps = Vec::new();
    deps::collect(expr, &mut deps);
    deps
}

fn is_comparable(ty: &Type) -> bool {
    match ty {
        Type::Optional(inner) => inner.is_scalar(),
        Type::Null => true,
        ty => ty.is_scalar(),
    }
}

/// Whether a value of type `from` can be used where `to` is expected.
fn assignable(from: &Type, to: &Type) -> bool {
    match (from, to) {
        _ if from == to => true,
        _ if from.has_error() || to.has_error() => true,
        (Type::Null, Type::Optional(_)) | (Type::Int, Type::Float) => true,
        (Type::Optional(_), _) => false,
        (_, Type::Optional(inner)) => assignable(from, inner),
        _ => false,
    }
}

/// Converts `expr` to type `to`, inserting explicit coercions; `Err` returns
/// the expression unchanged if the types are incompatible.
fn coerce(expr: Expr, to: &Type) -> Result<Expr, Expr> {
    if expr.ty == *to || expr.ty.has_error() || to.has_error() {
        return Ok(expr);
    }
    let wrap = |coercion, expr: Expr, ty: Type| Expr {
        span: expr.span,
        kind: ExprKind::Coerce {
            coercion,
            expr: Box::new(expr),
        },
        ty,
    };
    match (&expr.ty, to) {
        (Type::Null, Type::Optional(_)) => Ok(Expr {
            ty: to.clone(),
            ..expr
        }),
        (Type::Int, Type::Float) => Ok(wrap(Coercion::IntToFloat, expr, Type::Float)),
        (Type::Optional(_), _) => Err(expr),
        (_, Type::Optional(inner)) => {
            coerce(expr, inner).map(|e| wrap(Coercion::WrapOptional, e, to.clone()))
        }
        _ => Err(expr),
    }
}

/// Smallest type both `a` and `b` can be coerced to.
fn unify(a: &Type, b: &Type) -> Option<Type> {
    match (a, b) {
        _ if a == b => Some(a.clone()),
        _ if a.has_error() || b.has_error() => Some(Type::Error),
        (Type::Null, other) | (other, Type::Null) => Some(other.clone().optional()),
        (Type::Int, Type::Float) | (Type::Float, Type::Int) => Some(Type::Float),
        (Type::Optional(x), Type::Optional(y)) => {
            // Optionals only unify when their contents are identical; a coercion
            // under `?` would require mapping the value.
            (x == y).then(|| a.clone())
        }
        (Type::Optional(x), y) | (y, Type::Optional(x)) => {
            assignable(y, x).then(|| Type::Optional(x.clone()))
        }
        _ => None,
    }
}
