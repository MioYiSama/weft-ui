use weft_ui_ast as ast;
use weft_ui_ast::span::offset;
use weft_ui_ast::{Diagnostic, Span};

use crate::ir::{Expr, LoopId, ParamId, State, StateId, Type};
use crate::sema::record::{LocalBinding, ProbeContext, Target};

use super::Lowerer;

#[derive(Debug)]
pub(super) enum LocalKind {
    Param(ParamId, Type),
    Callback(ParamId),
    Slot(ParamId),
    State(StateId, Type),
    Item(LoopId, Type),
    LambdaArg(Type),
    Let(Expr),
}

impl LocalKind {
    fn ty(&self) -> &Type {
        match self {
            Self::Param(_, ty) | Self::State(_, ty) | Self::Item(_, ty) | Self::LambdaArg(ty) => ty,
            Self::Slot(_) | Self::Callback(_) => &Type::Error,
            Self::Let(expr) => &expr.ty,
        }
    }
}

pub(super) struct Local {
    pub(super) name: String,
    pub(super) kind: LocalKind,
    span: Span,
}

impl Lowerer<'_, '_> {
    pub(super) fn declare(&mut self, name: &str, kind: LocalKind, span: Span) {
        if crate::sema::theme::reserved(name, span, self.diags) {
            return;
        }
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
            span,
        };
        // Parameters are recorded with the component signature.
        if !matches!(
            local.kind,
            LocalKind::Param(..) | LocalKind::Callback(_) | LocalKind::Slot(_)
        ) {
            self.cx.record(span, || self.local_target(&local));
        }
        self.scope.push(local);
    }

    pub(super) fn local(&self, name: &str) -> Option<&Local> {
        self.scope.iter().rev().find(|l| l.name == name)
    }

    pub(super) fn local_target(&self, local: &Local) -> Target {
        let binding = match local.kind {
            LocalKind::Param(p, _) | LocalKind::Callback(p) | LocalKind::Slot(p) => {
                match self.component {
                    Some(component) => return Target::ComponentParam(component, p.index()),
                    None => LocalBinding::Param,
                }
            }
            LocalKind::State(..) => LocalBinding::State,
            LocalKind::Item(..) => LocalBinding::LoopItem,
            LocalKind::LambdaArg(_) => LocalBinding::LambdaParam,
            LocalKind::Let(_) => LocalBinding::Let,
        };
        Target::Local {
            binding,
            name: local.name.clone(),
            ty: local.kind.ty().clone(),
            def: local.span,
        }
    }

    /// Reports a probe with the locals `keep` accepts, then `extra`.
    pub(super) fn probe_locals(
        &self,
        context: ProbeContext,
        keep: impl Fn(&LocalKind) -> bool,
        extra: Vec<Target>,
    ) {
        let locals = self
            .scope
            .iter()
            .filter(|l| keep(&l.kind))
            .map(|l| self.local_target(l));
        self.cx.set_probe(context, locals.chain(extra).collect());
    }

    pub(super) fn let_declarations(&mut self, declarations: &[ast::LetDecl]) {
        for decl in declarations {
            let value = if let Some(ty) = &decl.ty {
                let ty = self.cx.resolve_type(self.file, ty, self.diags);
                self.expr_as(&decl.value, &ty)
            } else {
                self.expr(&decl.value)
            };
            self.declare(&decl.name.name, LocalKind::Let(value), decl.name.span);
        }
    }

    pub(super) fn state(&mut self, decl: &ast::StateDecl) {
        let annotated = decl
            .ty
            .as_ref()
            .map(|ty| self.cx.resolve_type(self.file, ty, self.diags));
        let (ty, init) = if let Some(ty) = annotated {
            let init = self.expr_as(&decl.init, &ty);
            (ty, init)
        } else {
            let init = self.expr(&decl.init);
            let ty = if init.ty == Type::Null {
                self.error(
                    decl.init.span,
                    "cannot infer the type of `null`; add a type annotation like `state x: Int? = null`",
                );
                Type::Error
            } else {
                init.ty.clone()
            };
            (ty, init)
        };
        let id = StateId(offset(self.states.len()));
        self.declare(
            &decl.name.name,
            LocalKind::State(id, ty.clone()),
            decl.name.span,
        );
        self.states.push(State {
            name: decl.name.name.clone(),
            ty,
            init,
            span: decl.name.span,
        });
    }
}
