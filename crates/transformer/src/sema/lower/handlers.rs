use weft_ui_ast as ast;
use weft_ui_ast::{AssignOp, Diagnostic, Span};

use crate::deps;
use crate::ir::{Assign, Dep, EmitArg, EmitTarget, Handler, ParamId, ParamKind, Stmt, Type};
use crate::sema::record::{ProbeContext, Target};
use crate::sema::{DeclKind, article};

use super::Lowerer;
use super::expr::{assignable, error_expr};
use super::nodes::ArgSlot;
use super::scope::LocalKind;

impl<'cx> Lowerer<'cx, '_> {
    /// Lowers an event-handler value; returns the handler and its render-time deps.
    pub(super) fn handler(
        &mut self,
        value: &ast::Expr,
        prop: &str,
        payload: Option<&Type>,
        lifecycle: bool,
    ) -> Option<(Handler, Vec<Dep>)> {
        match &value.kind {
            ast::ExprKind::Ident(name) => {
                let target = self.emit_target(name, value.span, prop, lifecycle)?;
                let args = self.forward_payload(target, name, value.span, prop, payload)?;
                Some((Handler::Emit { target, args }, Vec::new()))
            }
            ast::ExprKind::Call { callee, args } => {
                let ast::ExprKind::Ident(name) = &callee.kind else {
                    self.error(
                        value.span,
                        format!("expected an event or a lambda for `{prop}`"),
                    );
                    return None;
                };
                let target = self.emit_target(name, callee.span, prop, lifecycle)?;
                let args = self.emit_args(target, name, callee.span, args, value.span);
                let mut render_deps = Vec::new();
                for arg in &args {
                    if let EmitArg::Expr(expr) = arg {
                        deps::collect(expr, &mut render_deps);
                    }
                }
                Some((Handler::Emit { target, args }, render_deps))
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

    fn callback_payload(&self, id: ParamId) -> Option<&'cx Type> {
        let ParamKind::Callback { payload } = &self.params[id.index()].kind else {
            unreachable!("callback target refers to a callback parameter");
        };
        payload.as_ref()
    }

    fn forward_payload(
        &mut self,
        target: EmitTarget,
        name: &str,
        span: Span,
        prop: &str,
        payload: Option<&Type>,
    ) -> Option<Vec<EmitArg>> {
        let (what, count, expected) = match target {
            EmitTarget::Event(event) => {
                let params = &self.cx.events[event.index()].params;
                ("event", params.len(), params.first().map(|p| &p.ty))
            }
            EmitTarget::Callback(id) => {
                let ty = self.callback_payload(id);
                ("callback", usize::from(ty.is_some()), ty)
            }
        };
        match (payload, count, expected) {
            (None, 0, _) => Some(Vec::new()),
            (Some(ty), 1, Some(expected)) if assignable(ty, expected) => {
                Some(vec![EmitArg::Payload])
            }
            (Some(ty), 1, Some(expected)) => {
                self.error(
                    span,
                    format!(
                        "`{prop}` provides a `{}`, but {what} `{name}` expects `{}`",
                        self.type_name(ty),
                        self.type_name(expected)
                    ),
                );
                None
            }
            (None, n, _) => {
                let plural = if n == 1 { "" } else { "s" };
                self.diags.push(
                    Diagnostic::error(
                        span,
                        format!("{what} `{name}` takes {n} argument{plural}, but `{prop}` provides no value"),
                    )
                    .with_note(format!("bind the arguments explicitly: `{name}(...)`")),
                );
                None
            }
            (Some(ty), 0, _) => {
                let ty = self.type_name(ty);
                self.diags.push(
                    Diagnostic::error(
                        span,
                        format!(
                            "`{prop}` provides a `{ty}`, but {what} `{name}` takes no arguments"
                        ),
                    )
                    .with_note(format!("write `{name}()` to ignore the value")),
                );
                None
            }
            (Some(_), n, _) => {
                self.error(
                    span,
                    format!("{what} `{name}` takes {n} arguments, but `{prop}` provides one value"),
                );
                None
            }
        }
    }

    fn emit_args(
        &mut self,
        target: EmitTarget,
        name: &str,
        name_span: Span,
        args: &[ast::Arg],
        span: Span,
    ) -> Vec<EmitArg> {
        let slots: Vec<_> = match target {
            EmitTarget::Event(event) => self.cx.events[event.index()]
                .params
                .iter()
                .map(|p| (p.name.as_str(), &p.ty))
                .collect(),
            EmitTarget::Callback(id) => self
                .callback_payload(id)
                .map(|ty| ("value", ty))
                .into_iter()
                .collect(),
        };
        let specs: Vec<_> = slots
            .iter()
            .map(|&(name, _)| ArgSlot {
                name,
                positional: true,
                required: true,
            })
            .collect();
        let matched = self.match_args(name, &specs, args, span);
        if let EmitTarget::Event(event) = target {
            let call_span = Span::new(self.file, name_span.end, span.end);
            self.record_args(call_span, &matched, |i| Target::EventParam(event, i));
        }
        slots
            .into_iter()
            .zip(matched)
            .map(|((_, ty), arg)| {
                EmitArg::Expr(match arg {
                    Some(arg) => self.expr_as(&arg.value, ty),
                    None => error_expr(span),
                })
            })
            .collect()
    }

    fn emit_target(
        &mut self,
        name: &str,
        span: Span,
        prop: &str,
        lifecycle: bool,
    ) -> Option<EmitTarget> {
        if self.cx.is_probe(name) {
            let events = self
                .cx
                .visible_decls(self.file, |k| matches!(k, DeclKind::Event(_)));
            self.probe_locals(
                ProbeContext::Handler,
                |kind| !lifecycle && matches!(kind, LocalKind::Callback(_)),
                events,
            );
        }
        if let Some(local) = self.local(name) {
            self.cx.record(span, || self.local_target(local));
            if let LocalKind::Callback(id) = local.kind
                && !lifecycle
            {
                return Some(EmitTarget::Callback(id));
            }
            let what = if lifecycle {
                "an event"
            } else {
                "an event or callback"
            };
            self.error(span, format!("`{name}` is not {what}"));
            return None;
        }
        match self.cx.global(self.file, name, span, self.diags) {
            Some(DeclKind::Event(id)) => {
                self.emits.insert(id);
                Some(EmitTarget::Event(id))
            }
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

    fn lambda(&mut self, lambda: &ast::Lambda, prop: &str, payload: Option<&Type>) -> Handler {
        let scope_len = self.scope.len();
        if let Some(param) = &lambda.param {
            let ty = match payload {
                Some(ty) => ty.clone(),
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
            self.declare(&param.name, LocalKind::LambdaArg(ty), param.span);
        }
        let body = lambda
            .body
            .iter()
            .filter_map(|stmt| match stmt {
                ast::LambdaStmt::Assign(assign) => self.assign(assign).map(Stmt::Assign),
                ast::LambdaStmt::Call(call) => {
                    let name = &call.callee;
                    let target = self.emit_target(&name.name, name.span, prop, false)?;
                    let args = self.emit_args(target, &name.name, name.span, &call.args, call.span);
                    Some(Stmt::Emit { target, args })
                }
            })
            .collect();
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
                |k| matches!(k, LocalKind::State(..)),
                Vec::new(),
            );
        }
        let Some(local) = self.local(&target.name) else {
            self.error(target.span, format!("unknown state `{}`", target.name));
            return None;
        };
        self.cx.record(target.span, || self.local_target(local));
        let LocalKind::State(state, ty) = &local.kind else {
            let what = match local.kind {
                LocalKind::Let(_) => {
                    self.error(
                        target.span,
                        format!(
                            "cannot assign to `{}`: only `state` can be assigned",
                            target.name
                        ),
                    );
                    return None;
                }
                LocalKind::Param(..) | LocalKind::Callback(_) | LocalKind::Slot(_) => "parameter",
                LocalKind::Item(..) => "loop variable",
                LocalKind::LambdaArg(_) | LocalKind::State(..) => "lambda parameter",
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
        let (state, ty) = (*state, ty.clone());
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
}
