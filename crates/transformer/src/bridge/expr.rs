//! Expressions to JavaScript.
//!
//! Generated names: `c` is the component instance (`c.p[i]` parameters,
//! `c.s[i]` states), `r{loop}` the row of a `for` loop (`.item`), `$a` the
//! handler payload and `$it` the item passed to a key function. Model fields
//! are read by index from `$v`.

use crate::ir::{BinaryOp, Expr, ExprKind, InterpPart, LoopId, Member, Type, UnaryOp};

use super::js;

/// Where expression roots come from.
#[derive(Debug, Clone, Copy, Default)]
pub struct Env {
    /// Loop whose item is the key function parameter `$it`.
    pub key_loop: Option<LoopId>,
}

/// Returns JS that is safe to use as an operand or member-access base:
/// compound expressions are parenthesized.
pub fn expr(e: &Expr, env: Env) -> String {
    match &e.kind {
        ExprKind::Int(v) => paren_negative(v.to_string()),
        ExprKind::Float(v) => paren_negative(format!("{v:?}")),
        ExprKind::Bool(v) => v.to_string(),
        ExprKind::Null => "null".to_owned(),
        ExprKind::Str(s) => js::string(s),
        ExprKind::Theme(name) => format!("\"@{name}\""),
        ExprKind::Interp(parts) => {
            if parts.is_empty() {
                return "\"\"".to_owned();
            }
            let parts: Vec<String> = parts
                .iter()
                .map(|part| match part {
                    InterpPart::Lit(s) => js::string(s),
                    InterpPart::Expr(e) => format!("$.str({})", expr(e, env)),
                })
                .collect();
            if parts.len() == 1 {
                parts.into_iter().next().unwrap_or_default()
            } else {
                format!("({})", parts.join(" + "))
            }
        }
        ExprKind::Param(p) => format!("c.p[{}]", p.0),
        ExprKind::State(s) => format!("c.s[{}]", s.0),
        ExprKind::Item(l) if env.key_loop == Some(*l) => "$it".to_owned(),
        ExprKind::Item(l) => format!("r{}.item", l.0),
        ExprKind::LambdaArg => "$a".to_owned(),
        ExprKind::Field { base, field, safe } => {
            let base = member_base(expr(base, env));
            if *safe {
                format!("({base}?.$v[{}] ?? null)", field.0)
            } else {
                format!("{base}.$v[{}]", field.0)
            }
        }
        ExprKind::Member {
            member,
            recv,
            args,
            safe,
        } => {
            let args: Vec<String> = args.iter().map(|a| expr(a, env)).collect();
            let recv = expr(recv, env);
            if *safe {
                format!(
                    "$.safe({recv}, ($r) => {})",
                    member_call(*member, "$r", &args)
                )
            } else {
                member_call(*member, &member_base(recv), &args)
            }
        }
        ExprKind::Unary { op, expr: inner } => {
            let op = match op {
                UnaryOp::Not => "!",
                UnaryOp::Neg => "-",
            };
            format!("({op}{})", expr(inner, env))
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let (l, r) = (expr(lhs, env), expr(rhs, env));
            let op = match op {
                BinaryOp::Div if e.ty == Type::Int => return format!("$.idiv({l}, {r})"),
                BinaryOp::Coalesce => "??",
                BinaryOp::Or => "||",
                BinaryOp::And => "&&",
                BinaryOp::Eq => "===",
                BinaryOp::Ne => "!==",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Rem => "%",
            };
            format!("({l} {op} {r})")
        }
        ExprKind::If { cond, then, else_ } => format!(
            "({} ? {} : {})",
            expr(cond, env),
            expr(then, env),
            expr(else_, env)
        ),
        ExprKind::Coerce { expr: inner, .. } => expr(inner, env),
        ExprKind::Error => unreachable!("error expressions never reach code generation"),
    }
}

fn paren_negative(text: String) -> String {
    if text.starts_with('-') {
        format!("({text})")
    } else {
        text
    }
}

/// Numeric literals need parentheses before `.member`.
fn member_base(text: String) -> String {
    if text.starts_with(|c: char| c.is_ascii_digit()) {
        format!("({text})")
    } else {
        text
    }
}

fn member_call(member: Member, recv: &str, args: &[String]) -> String {
    let arg = |i: usize| args.get(i).map_or("undefined", String::as_str);
    match member {
        Member::Length => format!("{recv}.length"),
        Member::IsEmpty => format!("({recv}.length === 0)"),
        Member::Trim => format!("{recv}.trim()"),
        Member::Uppercase => format!("{recv}.toUpperCase()"),
        Member::Lowercase => format!("{recv}.toLowerCase()"),
        Member::Contains => format!("{recv}.includes({})", arg(0)),
        Member::ToString => format!("$.str({recv})"),
        Member::ToFixed => format!("{recv}.toFixed({})", arg(0)),
    }
}
