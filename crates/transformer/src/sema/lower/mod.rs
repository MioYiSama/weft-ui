//! Type checks one component body and lowers it to the view IR.

mod expr;
mod handlers;
mod nodes;
mod scope;

use std::collections::BTreeSet;

use weft_ui_ast as ast;
use weft_ui_ast::span::offset;
use weft_ui_ast::{Diagnostic, FileId, Span};

use crate::ir::{
    Binding, ComponentId, EventId, Loop, Node, NodeId, Param, ParamId, ParamKind, State, Type,
};
use crate::sema::Checker;

use self::scope::{Local, LocalKind};

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
            let kind = match &param.kind {
                ParamKind::Value(ty) => LocalKind::Param(id, ty.clone()),
                ParamKind::Children => LocalKind::Slot(id),
            };
            self.declare(&param.name, kind, param.span);
        }
        for state in &decl.states {
            self.state(state);
        }
        self.let_declarations(&decl.lets);
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

    fn error(&mut self, span: Span, message: impl Into<String>) {
        self.diags.push(Diagnostic::error(span, message));
    }

    fn type_name(&self, ty: &Type) -> String {
        self.cx.type_name(ty)
    }
}
