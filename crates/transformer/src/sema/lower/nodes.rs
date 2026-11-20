use std::ops::Range;

use weft_ui_ast as ast;
use weft_ui_ast::span::offset;
use weft_ui_ast::{Diagnostic, Span};

use crate::color::Color;
use crate::deps;
use crate::elements::{
    self, Children, ELEMENTS as BUILTIN_ELEMENTS, ElementSpec, LIFECYCLE as CALL_HOOKS, PropKind,
    PropType,
};
use crate::ir::{
    Arg, Binding, BindingId, BindingKind, Branch, ComponentId, Dep, Expr, ExprKind, Loop, LoopId,
    Node, NodeId, NodeKind, Param, ParamId, ParamKind, Prop, PropValue, Type,
};
use crate::sema::record::{ProbeContext, RawCall, Target};
use crate::sema::{DeclKind, article};

use super::expr::error_expr;
use super::scope::LocalKind;
use super::{Instance, Lowerer};

/// Placeholder until the owning node is pushed and its bindings are patched.
const PENDING_NODE: NodeId = NodeId(u32::MAX);

/// One expected argument when matching call arguments.
pub(super) struct ArgSlot<'n> {
    pub(super) name: &'n str,
    pub(super) positional: bool,
    pub(super) required: bool,
}

impl Lowerer<'_, '_> {
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

    pub(super) fn nodes(&mut self, nodes: &[ast::Node]) -> Vec<NodeId> {
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
            let elements = BUILTIN_ELEMENTS.iter().map(Target::Element);
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
        if let Some(spec) = elements::element(name) {
            self.cx.record(first.span, || Target::Element(spec));
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
        spec: &'static ElementSpec,
        e: &ast::ElementNode,
        span: Span,
    ) -> NodeId {
        let props_spec: Vec<_> = spec.props.iter().chain(&CALL_HOOKS).collect();
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
            Target::Prop {
                owner,
                prop: props_spec[i],
            }
        });

        let first_binding = self.bindings.len();
        let mut props = Vec::new();
        for (prop, arg) in props_spec.iter().zip(matched) {
            let Some(arg) = arg else { continue };
            let (value, binding) = match prop.kind {
                PropKind::Value(ty) => {
                    let expr = self.expr_as(&arg.value, &ty.to_type());
                    if ty == PropType::Color
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
                    let payload = payload.map(PropType::to_type);
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
            Some(block) if spec.children == Children::None => {
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
        slots.extend(CALL_HOOKS.iter().map(|p| ArgSlot {
            name: p.name,
            positional: false,
            required: false,
        }));
        let matched = self.match_args(&callee.name, &slots, &e.args, span);
        self.record_args(args_span(e, span), &matched, |i| {
            match value_params.get(i) {
                Some(&(param, _, _)) => Target::ComponentParam(id, param.index()),
                None => Target::Prop {
                    owner: None,
                    prop: &CALL_HOOKS[i - value_params.len()],
                },
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
                let name = CALL_HOOKS[index - value_params.len()].name;
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
    pub(super) fn match_args<'a>(
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

    /// Records named arguments matched to `targets` (by slot index) and the
    /// argument list as a call site.
    pub(super) fn record_args(
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
}

/// From the end of the element's name to its children block or end.
fn args_span(e: &ast::ElementNode, span: Span) -> Span {
    let start = e.path.last().map_or(span.start, |p| p.span.end);
    let end = e.children.as_ref().map_or(span.end, |b| b.span.start);
    Span::new(span.file, start, end.max(start))
}

fn expr_deps(expr: &Expr) -> Vec<Dep> {
    let mut deps = Vec::new();
    deps::collect(expr, &mut deps);
    deps
}
