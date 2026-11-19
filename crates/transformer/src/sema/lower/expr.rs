use weft_ui_ast as ast;
use weft_ui_ast::span::offset;
use weft_ui_ast::{BinaryOp, Diagnostic, Span, UnaryOp};

use crate::ir::{Coercion, Expr, ExprKind, FieldId, InterpPart, Type};
use crate::sema::DeclKind;
use crate::sema::builtins::{self, MemberKind};
use crate::sema::record::{ProbeContext, Target};

use super::Lowerer;
use super::scope::LocalKind;

impl Lowerer<'_, '_> {
    /// Lowers `e` and checks that it can be used where `ty` is expected.
    pub(crate) fn expr_as(&mut self, e: &ast::Expr, ty: &Type) -> Expr {
        let expr = self.expr(e);
        self.coerce_to(expr, ty)
    }

    pub(super) fn coerce_to(&mut self, expr: Expr, ty: &Type) -> Expr {
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

    pub(super) fn expr(&mut self, e: &ast::Expr) -> Expr {
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
        self.cx.record(name.span, || Target::Member {
            name: name.name.clone(),
            signature: sig.clone(),
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
            ty => builtins::MEMBER_NAMES
                .into_iter()
                .filter_map(|name| {
                    Some(Target::Member {
                        signature: builtins::member(ty, name)?,
                        name: name.to_owned(),
                    })
                })
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

pub(super) fn error_expr(span: Span) -> Expr {
    Expr {
        kind: ExprKind::Error,
        ty: Type::Error,
        span,
    }
}

fn is_comparable(ty: &Type) -> bool {
    match ty {
        Type::Optional(inner) => inner.is_scalar(),
        Type::Null => true,
        ty => ty.is_scalar(),
    }
}

/// Whether a value of type `from` can be used where `to` is expected.
pub(super) fn assignable(from: &Type, to: &Type) -> bool {
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
