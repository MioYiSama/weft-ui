use weft_ui_ast as ast;
use weft_ui_ast::{AssignOp, Diagnostic, Span};

use crate::deps;
use crate::ir::{Assign, Dep, EmitArg, EventId, Handler, Type};
use crate::sema::record::{ProbeContext, Target};
use crate::sema::{DeclKind, article};

use super::Lowerer;
use super::expr::{assignable, error_expr};
use super::nodes::ArgSlot;
use super::scope::LocalKind;

impl Lowerer<'_, '_> {
    /// Lowers an event-handler value; returns the handler and its render-time deps.
    pub(super) fn handler(
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
}
