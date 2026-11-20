//! Semantic occurrences, calls and completion candidates collected during checking.

use weft_ui_ast::Span;

use crate::elements::{ElementSpec, PropSpec};
use crate::ir::{ComponentId, EventId, ModelId, Type};

use super::builtins::MemberSig;

/// The part of the language in which a completion probe was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeContext {
    /// A node in a component body: elements, components and slots.
    Node,
    /// A named slot label in a component call's child block.
    SlotFill,
    /// An expression: let bindings, parameters, states, loop variables, lambda parameters.
    Value,
    /// An event handler value: events.
    Handler,
    /// The value of `style =`: visible style declarations.
    Style,
    /// After `.` / `?.`: fields and built-in members of the receiver.
    Member,
    /// A type annotation: built-in types, models and type aliases.
    Type,
    /// The target of an assignment in a lambda: states.
    AssignTarget,
}

/// A resolved entity, independent of how the editor displays it.
#[derive(Debug, Clone)]
pub(crate) enum Target {
    Model(ModelId),
    Event(EventId),
    Component(ComponentId),
    Field(ModelId, usize),
    EventParam(EventId, usize),
    ComponentParam(ComponentId, usize),
    Local {
        binding: LocalBinding,
        name: String,
        ty: Type,
        def: Span,
    },
    Element(&'static ElementSpec),
    Prop {
        owner: Option<&'static ElementSpec>,
        prop: &'static PropSpec,
    },
    Member {
        name: String,
        signature: MemberSig,
    },
    Type(String),
    TypeAlias(usize),
    Style(usize),
}

#[derive(Debug)]
pub(crate) struct AliasSymbol {
    pub name: String,
    pub ty: Type,
    pub def: Span,
}

#[derive(Debug)]
pub(crate) struct StyleSymbol {
    pub def: Span,
    pub declaration: Span,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum LocalBinding {
    Param,
    State,
    LoopItem,
    LambdaParam,
    Let,
}

#[derive(Debug)]
pub(crate) struct RawCall {
    pub args: Span,
    pub params: Vec<Target>,
    /// Per parameter: whether an argument was matched to it.
    pub given: Vec<bool>,
}

#[derive(Debug)]
pub(crate) struct RawProbe {
    pub context: ProbeContext,
    pub candidates: Vec<Target>,
}

/// Collects the index during analysis; owned by the checker in a `RefCell`
/// because the checker is shared immutably while bodies are lowered.
#[derive(Debug, Default)]
pub(crate) struct Recorder {
    pub probe_name: Option<String>,
    pub aliases: Vec<AliasSymbol>,
    pub styles: Vec<StyleSymbol>,
    pub refs: Vec<(Span, Target)>,
    pub calls: Vec<RawCall>,
    pub probe: Option<RawProbe>,
}

impl Recorder {
    pub(crate) fn is_probe(&self, name: &str) -> bool {
        self.probe.is_none() && self.probe_name.as_deref() == Some(name)
    }
}
