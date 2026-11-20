//! Typed view IR shared by every host binding and rendering backend.
//!
//! A [`Program`] is produced only when analysis reports no errors, so the
//! [`Type::Error`] placeholder never appears in it.

use std::fmt;

pub use weft_ui_ast::{AssignOp, BinaryOp, UnaryOp};
use weft_ui_ast::{FileId, Span};

use crate::elements::Element;

macro_rules! id_type {
    ($($(#[$doc:meta])* $name:ident;)*) => {$(
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u32);

        impl $name {
            #[must_use]
            pub fn index(self) -> usize {
                self.0 as usize
            }
        }
    )*};
}

id_type! {
    /// Index into [`Program::models`].
    ModelId;
    /// Index into [`Program::events`].
    EventId;
    /// Index into [`Program::components`].
    ComponentId;
    /// Index into [`Model::fields`] of the model being accessed.
    FieldId;
    /// Index into [`Component::params`].
    ParamId;
    /// Index into [`Component::states`].
    StateId;
    /// Index into [`Component::nodes`].
    NodeId;
    /// Index into [`Component::loops`].
    LoopId;
    /// Index into [`Component::bindings`].
    BindingId;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Entry modules in the order they were given, without duplicates.
    pub entries: Vec<FileId>,
    pub modules: Vec<Module>,
    pub models: Vec<Model>,
    pub events: Vec<Event>,
    pub components: Vec<Component>,
}

impl Program {
    /// The module of `file`; modules are stored in file order.
    ///
    /// # Panics
    ///
    /// Panics if `file` is not part of the program.
    #[must_use]
    pub fn module(&self, file: FileId) -> &Module {
        &self.modules[file.index()]
    }

    #[must_use]
    pub fn model(&self, id: ModelId) -> &Model {
        &self.models[id.index()]
    }

    #[must_use]
    pub fn event(&self, id: EventId) -> &Event {
        &self.events[id.index()]
    }

    #[must_use]
    pub fn component(&self, id: ComponentId) -> &Component {
        &self.components[id.index()]
    }

    /// Looks up a component by name; names are unique across the program.
    #[must_use]
    pub fn component_named(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.name == name)
    }

    /// Displays a type using the program's model names.
    #[must_use]
    pub fn display_type<'a>(&'a self, ty: &'a Type) -> impl fmt::Display + 'a {
        type_display(ty, &self.models)
    }
}

/// One `.weft` file of the import closure; hosts generate one module per file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub file: FileId,
    pub path: String,
    /// Directly imported modules.
    pub imports: Vec<FileId>,
    pub models: Vec<ModelId>,
    pub events: Vec<EventId>,
    pub components: Vec<ComponentId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub name: String,
    pub file: FileId,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub name: String,
    pub file: FileId,
    pub params: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Model(ModelId),
    Array(Box<Type>),
    Optional(Box<Type>),
    /// Type of the `null` literal before it is coerced to a `T?`.
    Null,
    /// Placeholder after a reported type error; suppresses follow-up errors.
    Error,
}

impl Type {
    #[must_use]
    pub fn optional(self) -> Type {
        match self {
            Type::Optional(_) | Type::Null | Type::Error => self,
            other => Type::Optional(Box::new(other)),
        }
    }

    /// Whether [`Type::Error`] appears anywhere in the type.
    #[must_use]
    pub fn has_error(&self) -> bool {
        match self {
            Type::Error => true,
            Type::Array(inner) | Type::Optional(inner) => inner.has_error(),
            _ => false,
        }
    }

    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    /// `Int`, `Float`, `Bool` or `String`.
    #[must_use]
    pub fn is_scalar(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::Bool | Type::String)
    }
}

struct TypeDisplay<'a> {
    ty: &'a Type,
    models: &'a [Model],
}

impl fmt::Display for TypeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = |ty| TypeDisplay {
            ty,
            models: self.models,
        };
        match self.ty {
            Type::Int => f.write_str("Int"),
            Type::Float => f.write_str("Float"),
            Type::Bool => f.write_str("Bool"),
            Type::String => f.write_str("String"),
            Type::Model(id) => match self.models.get(id.index()) {
                Some(model) => f.write_str(&model.name),
                None => write!(f, "<model {}>", id.0),
            },
            Type::Array(elem) => write!(f, "{}[]", inner(elem)),
            Type::Optional(elem) => write!(f, "{}?", inner(elem)),
            Type::Null => f.write_str("null"),
            Type::Error => f.write_str("{error}"),
        }
    }
}

pub(crate) fn type_display<'a>(ty: &'a Type, models: &'a [Model]) -> impl fmt::Display + 'a {
    TypeDisplay { ty, models }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: String,
    pub file: FileId,
    pub span: Span,
    pub params: Vec<Param>,
    pub states: Vec<State>,
    pub nodes: Vec<Node>,
    /// Top-level nodes of the component body, in source order.
    pub roots: Vec<NodeId>,
    pub loops: Vec<Loop>,
    pub bindings: Vec<Binding>,
    /// Dirty-bit layout of the component: bit `i` tracks `slots[i]`.
    pub slots: Vec<Dep>,
    /// Events emitted directly by this component's handlers.
    pub emits: Vec<EventId>,
    /// Events emitted by this component or any component it instantiates.
    pub reachable_events: Vec<EventId>,
}

impl Component {
    #[must_use]
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.index()]
    }

    /// Type of the value a dependency root refers to.
    #[must_use]
    pub fn root_type(&self, root: DepRoot) -> &Type {
        match root {
            DepRoot::Param(p) => match &self.params[p.index()].kind {
                ParamKind::Value(ty) => ty,
                ParamKind::Children => &Type::Error,
            },
            DepRoot::State(s) => &self.states[s.index()].ty,
            DepRoot::Item(l) => &self.loops[l.index()].item_ty,
        }
    }

    #[must_use]
    pub fn root_name(&self, root: DepRoot) -> &str {
        match root {
            DepRoot::Param(p) => &self.params[p.index()].name,
            DepRoot::State(s) => &self.states[s.index()].name,
            DepRoot::Item(l) => &self.loops[l.index()].var,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub kind: ParamKind,
    /// Constant default value; arguments without a default are required.
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamKind {
    Value(Type),
    /// Slot filled by the caller's trailing block.
    Children,
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub name: String,
    pub ty: Type,
    /// Evaluated once when the component instance is created.
    pub init: Expr,
    pub span: Span,
}

/// Scope introduced by a `for` node.
#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    pub var: String,
    pub item_ty: Type,
    pub node: NodeId,
    /// Per-item dirty-bit layout: bit `i` tracks `slots[i]`.
    pub slots: Vec<Dep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Element {
        element: Element,
        props: Vec<Prop>,
        children: Vec<NodeId>,
    },
    Component {
        component: ComponentId,
        /// Arguments as written; omitted parameters use their default.
        args: Vec<Arg>,
        /// Lifecycle handlers (`onMount` / `onUnmount`).
        props: Vec<Prop>,
        children: Vec<NodeId>,
    },
    If {
        branches: Vec<Branch>,
        /// Selects the active branch; depends on every condition.
        binding: Option<BindingId>,
    },
    For {
        loop_id: LoopId,
        list: Expr,
        key: Expr,
        children: Vec<NodeId>,
        list_binding: Option<BindingId>,
        key_binding: Option<BindingId>,
    },
    /// Renders the nodes passed to a `Children` parameter.
    Slot { param: ParamId },
}

impl NodeKind {
    #[must_use]
    pub fn children(&self) -> Vec<NodeId> {
        match self {
            NodeKind::Element { children, .. }
            | NodeKind::Component { children, .. }
            | NodeKind::For { children, .. } => children.clone(),
            NodeKind::If { branches, .. } => branches
                .iter()
                .flat_map(|b| b.children.iter().copied())
                .collect(),
            NodeKind::Slot { .. } => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    /// `None` for the final `else`.
    pub cond: Option<Expr>,
    pub children: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prop {
    pub name: &'static str,
    pub value: PropValue,
    /// Present when the value depends on model, state, or loop items.
    pub binding: Option<BindingId>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    Expr(Expr),
    Handler(Handler),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub param: ParamId,
    pub value: Expr,
    pub binding: Option<BindingId>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Handler {
    /// Sends an event to the host; arguments are bound at render time.
    Emit { event: EventId, args: Vec<EmitArg> },
    /// Inline lambda that only assigns component state.
    Lambda {
        /// Name of the parameter receiving the element's value, if declared.
        param: Option<String>,
        body: Vec<Assign>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum EmitArg {
    Expr(Expr),
    /// The value produced by the element (e.g. the input's text).
    Payload,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub state: StateId,
    pub op: AssignOp,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Str(String),
    Interp(Vec<InterpPart>),
    Param(ParamId),
    State(StateId),
    /// Current item of a `for` loop.
    Item(LoopId),
    /// Parameter of the enclosing handler lambda.
    LambdaArg,
    Field {
        base: Box<Expr>,
        field: FieldId,
        /// `?.`: yields `null` when `base` is null.
        safe: bool,
    },
    Member {
        member: Member,
        recv: Box<Expr>,
        args: Vec<Expr>,
        safe: bool,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    /// Implicit conversion inserted by the type checker.
    Coerce {
        coercion: Coercion,
        expr: Box<Expr>,
    },
    /// Placeholder after a reported error; never present in a [`Program`].
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpPart {
    Lit(String),
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Coercion {
    IntToFloat,
    /// `T` to `T?`.
    WrapOptional,
}

/// Whitelisted built-in properties and methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Member {
    Length,
    IsEmpty,
    Trim,
    Uppercase,
    Lowercase,
    Contains,
    ToString,
    ToFixed,
}

/// A value an update depends on: a root plus a field path through models.
///
/// Paths stop at arrays, optionals and scalars; a dependency covers every
/// slot whose path it prefixes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dep {
    pub root: DepRoot,
    pub path: Vec<FieldId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DepRoot {
    Param(ParamId),
    State(StateId),
    Item(LoopId),
}

/// A dynamic point in the view that must be recomputed when its slots change.
#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub node: NodeId,
    pub kind: BindingKind,
    /// Normalized: sorted, deduplicated, no dependency prefixed by another.
    pub deps: Vec<Dep>,
    /// Dirty bits that trigger this binding.
    pub bits: Vec<SlotRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingKind {
    Prop(&'static str),
    /// Argument for a parameter of the component instantiated by the node.
    Arg(ParamId),
    Handler(&'static str),
    Condition,
    List,
    Key,
}

/// A dirty bit: either a component slot or a per-item slot of a loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SlotRef {
    Component(u32),
    Loop(LoopId, u32),
}
