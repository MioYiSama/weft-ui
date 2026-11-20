//! Signatures of built-in members.

use crate::ir::{Member, Type};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemberKind {
    Property,
    Method,
}

#[derive(Debug, Clone)]
pub(crate) struct MemberSig {
    pub member: Member,
    pub kind: MemberKind,
    pub params: Vec<Type>,
    pub ret: Type,
}

/// Looks up a built-in member on a non-optional receiver type.
pub(crate) fn member(recv: &Type, name: &str) -> Option<MemberSig> {
    use MemberKind::{Method, Property};
    let sig = |member, kind, params, ret| MemberSig {
        member,
        kind,
        params,
        ret,
    };
    let recv = if matches!(recv, Type::Union(_)) {
        &Type::String
    } else {
        recv
    };
    Some(match (recv, name) {
        (Type::String | Type::Array(_), "length") => {
            sig(Member::Length, Property, vec![], Type::Int)
        }
        (Type::String | Type::Array(_), "isEmpty") => {
            sig(Member::IsEmpty, Property, vec![], Type::Bool)
        }
        (Type::String, "trim") => sig(Member::Trim, Method, vec![], Type::String),
        (Type::String, "uppercase") => sig(Member::Uppercase, Method, vec![], Type::String),
        (Type::String, "lowercase") => sig(Member::Lowercase, Method, vec![], Type::String),
        (Type::String, "contains") => sig(Member::Contains, Method, vec![Type::String], Type::Bool),
        (Type::Array(elem), "contains") if elem.is_scalar() => {
            sig(Member::Contains, Method, vec![(**elem).clone()], Type::Bool)
        }
        (Type::Int | Type::Float | Type::Bool, "toString") => {
            sig(Member::ToString, Method, vec![], Type::String)
        }
        (Type::Float, "toFixed") => sig(Member::ToFixed, Method, vec![Type::Int], Type::String),
        _ => return None,
    })
}
