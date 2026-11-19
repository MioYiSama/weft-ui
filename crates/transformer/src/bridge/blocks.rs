//! Block structure shared by the DOM and remote (mini program) modes:
//! dirty-bit conditions, watch shapes, module constants and handler code.
//!
//! Both modes split a component into blocks (component body, `if` branch,
//! `for` row, slot content) whose update functions re-run only the bindings
//! whose dirty bits are set.

use std::collections::{BTreeMap, HashMap};

use crate::ir::{
    Arg, Assign, AssignOp, BindingId, Branch, Component, Dep, DepRoot, EmitArg, EventId, Expr,
    ExprKind, FieldId, Handler, LoopId, NodeId, NodeKind, ParamId, ParamKind, Program, Prop,
    PropValue, SlotRef, StateId, Type,
};

use super::expr::{Env, expr};
use super::js::{self, Code};

/// Module-level constants shared by all components of a module.
#[derive(Debug, Default)]
pub struct Consts {
    templates: Vec<String>,
    template_ids: HashMap<String, usize>,
    shapes: Vec<String>,
    shape_ids: HashMap<String, usize>,
}

impl Consts {
    /// `$t{n}`: a view template constant initialized by the JS expression
    /// `init`, e.g. a lazily parsed DOM `<template>`. Equal initializers
    /// share one constant.
    pub fn template(&mut self, init: String) -> String {
        format!(
            "$t{}",
            intern(&mut self.templates, &mut self.template_ids, init)
        )
    }

    fn shape(&mut self, literal: String) -> String {
        format!(
            "$s{}",
            intern(&mut self.shapes, &mut self.shape_ids, literal)
        )
    }

    pub fn write(&self, out: &mut Code) {
        for (i, shape) in self.shapes.iter().enumerate() {
            out.line(format!("const $s{i} = {shape};"));
        }
        for (i, init) in self.templates.iter().enumerate() {
            out.line(format!("const $t{i} = {init};"));
        }
    }
}

fn intern(values: &mut Vec<String>, ids: &mut HashMap<String, usize>, value: String) -> usize {
    let next = values.len();
    let id = *ids.entry(value.clone()).or_insert(next);
    if id == next {
        values.push(value);
    }
    id
}

/// Converts an index into an IR id's `u32`.
///
/// # Panics
///
/// Panics past `u32::MAX`, which the compiler never produces.
pub(crate) fn offset(index: usize) -> u32 {
    u32::try_from(index).expect("IR indices fit in u32")
}

/// Dirty words needed for `slots` bits; scopes always have at least one.
pub fn words(slots: usize) -> usize {
    slots.div_ceil(32).max(1)
}

/// Loops enclosing the code being generated, outermost first.
#[derive(Debug, Clone, Default)]
pub struct Ctx {
    loops: Vec<LoopId>,
}

impl Ctx {
    /// The innermost scope: the current row, or the component instance.
    pub fn scope(&self) -> String {
        match self.loops.last() {
            Some(l) => format!("r{}", l.0),
            None => "c".to_owned(),
        }
    }

    fn sees(&self, slot: SlotRef) -> bool {
        match slot {
            SlotRef::Component(_) => true,
            SlotRef::Loop(l, _) => self.loops.contains(&l),
        }
    }

    pub fn with_loop(&self, l: LoopId) -> Ctx {
        let mut loops = self.loops.clone();
        loops.push(l);
        Ctx { loops }
    }
}

/// Code of one block: built once, re-run on dirty bits, and released.
#[derive(Default)]
pub struct Parts {
    pub create: Code,
    pub update: Code,
    pub cleanup: Code,
}

pub fn bits(comp: &Component, binding: BindingId) -> &[SlotRef] {
    &comp.bindings[binding.index()].bits
}

/// `scope.d[word] & mask || ...` for the given bits, or `None` if empty.
pub fn cond<'s>(bits: impl IntoIterator<Item = &'s SlotRef>) -> Option<String> {
    let mut masks: BTreeMap<(Option<LoopId>, u32), u32> = BTreeMap::new();
    for bit in bits {
        let (scope, index) = match *bit {
            SlotRef::Component(i) => (None, i),
            SlotRef::Loop(l, i) => (Some(l), i),
        };
        *masks.entry((scope, index / 32)).or_default() |= 1 << (index % 32);
    }
    if masks.is_empty() {
        return None;
    }
    let parts: Vec<String> = masks
        .into_iter()
        .map(|((scope, word), mask)| {
            let var = scope.map_or_else(|| "c".to_owned(), |l| format!("r{}", l.0));
            format!("{var}.d[{word}] & {mask}")
        })
        .collect();
    Some(parts.join(" || "))
}

/// Condition for re-running `binding`, if it has any dirty bits.
pub fn binding_cond(comp: &Component, binding: Option<BindingId>) -> Option<String> {
    binding.and_then(|b| cond(bits(comp, b)))
}

/// Bits of the view bindings in the subtrees of `roots` that are visible
/// from `ctx`: the condition for walking into a nested block.
pub fn walk_cond(
    comp: &Component,
    roots: impl IntoIterator<Item = NodeId>,
    ctx: &Ctx,
) -> Option<String> {
    let mut bits = Vec::new();
    for root in roots {
        subtree_bits(comp, root, &mut bits);
    }
    bits.retain(|&bit| ctx.sees(bit));
    cond(&bits)
}

/// Event handler and key bindings are evaluated on demand by both backends,
/// so only values, arguments, conditions and lists count.
fn subtree_bits(comp: &Component, id: NodeId, out: &mut Vec<SlotRef>) {
    let kind = &comp.node(id).kind;
    let mut push = |binding: Option<BindingId>| {
        if let Some(b) = binding {
            out.extend_from_slice(bits(comp, b));
        }
    };
    match kind {
        NodeKind::Element { props, .. } | NodeKind::Component { props, .. } => {
            for prop in props {
                if let PropValue::Expr(_) = prop.value {
                    push(prop.binding);
                }
            }
        }
        NodeKind::If { binding, .. } => push(*binding),
        NodeKind::For { list_binding, .. } => push(*list_binding),
        NodeKind::Slot { .. } => {}
    }
    if let NodeKind::Component { args, .. } = kind {
        for arg in args {
            push(arg.binding);
        }
    }
    for child in kind.children() {
        subtree_bits(comp, child, out);
    }
}

/* ---- shapes ------------------------------------------------------------ */

/// `$s{n}`: the watch shape of a parameter, state or loop item.
pub fn root_shape(
    program: &Program,
    comp: &Component,
    consts: &mut Consts,
    root: DepRoot,
) -> String {
    let slots: &[Dep] = match root {
        DepRoot::Item(l) => &comp.loops[l.index()].slots,
        _ => &comp.slots,
    };
    let entries: Vec<(&[FieldId], usize)> = slots
        .iter()
        .enumerate()
        .filter(|(_, dep)| dep.root == root)
        .map(|(bit, dep)| (dep.path.as_slice(), bit))
        .collect();
    let literal = shape(program, comp.root_type(root), &entries);
    consts.shape(literal)
}

fn shape(program: &Program, ty: &Type, entries: &[(&[FieldId], usize)]) -> String {
    if let [(path, bit)] = entries
        && path.is_empty()
    {
        return match ty {
            Type::Optional(inner) if matches!(**inner, Type::Model(_)) => {
                format!("{{ d: {bit} }}")
            }
            _ => bit.to_string(),
        };
    }
    let Type::Model(model) = ty else {
        unreachable!("only models are flattened into several slots");
    };
    let fields: Vec<String> = program
        .model(*model)
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let nested: Vec<(&[FieldId], usize)> = entries
                .iter()
                .filter(|(path, _)| path.first().is_some_and(|f| f.index() == index))
                .map(|&(path, bit)| (&path[1..], bit))
                .collect();
            shape(program, &field.ty, &nested)
        })
        .collect();
    format!("[{}]", fields.join(", "))
}

/// Initializes the parameter and state watches of the instance `c`.
pub fn instance_watches(program: &Program, comp: &Component, consts: &mut Consts, out: &mut Code) {
    for (index, param) in comp.params.iter().enumerate() {
        if let ParamKind::Value(_) = param.kind {
            let root = DepRoot::Param(ParamId(offset(index)));
            let shape = root_shape(program, comp, consts, root);
            out.line(format!(
                "c.pw[{index}] = $.watch({shape}, c, args[{index}]);"
            ));
        }
    }
    for (index, state) in comp.states.iter().enumerate() {
        out.line(format!(
            "c.s[{index}] = {};",
            expr(&state.init, Env::default())
        ));
        let root = DepRoot::State(StateId(offset(index)));
        let shape = root_shape(program, comp, consts, root);
        out.line(format!(
            "c.sw[{index}] = $.watch({shape}, c, c.s[{index}]);"
        ));
    }
}

/* ---- expressions ------------------------------------------------------- */

/// A constant property value that can be written into static markup.
pub enum Literal<'e> {
    Str(&'e str),
    Int(i64),
    Bool(bool),
}

pub fn literal(e: &Expr) -> Option<Literal<'_>> {
    match &e.kind {
        ExprKind::Str(s) => Some(Literal::Str(s)),
        ExprKind::Int(v) => Some(Literal::Int(*v)),
        ExprKind::Bool(v) => Some(Literal::Bool(*v)),
        ExprKind::Coerce { expr, .. } => literal(expr),
        _ => None,
    }
}

/// Index of the active branch of an `if` chain, `-1` when none matches.
pub fn branch_index(branches: &[Branch]) -> String {
    let mut index = String::new();
    for (i, branch) in branches.iter().enumerate() {
        match &branch.cond {
            Some(cond) => index.push_str(&format!("{} ? {i} : ", expr(cond, Env::default()))),
            None => {
                index.push_str(&i.to_string());
                return index;
            }
        }
    }
    index.push_str("-1");
    index
}

/// Argument values of a component call in parameter order; the `Children`
/// parameter is passed separately.
pub fn arg_values(callee: &Component, args: &[Arg]) -> Vec<String> {
    callee
        .params
        .iter()
        .enumerate()
        .map(|(index, param)| {
            if param.kind == ParamKind::Children {
                return "undefined".to_owned();
            }
            match args.iter().find(|a| a.param.index() == index) {
                Some(arg) => expr(&arg.value, Env::default()),
                None => param.default.as_ref().map_or_else(
                    || unreachable!("missing arguments are rejected by analysis"),
                    |d| expr(d, Env::default()),
                ),
            }
        })
        .collect()
}

/// Re-sends changed arguments to the instance `var`.
pub fn arg_updates(comp: &Component, var: &str, args: &[Arg], out: &mut Code) {
    for arg in args {
        if let Some(cond) = binding_cond(comp, arg.binding) {
            out.line(format!(
                "if ({cond}) {var}.arg({}, {});",
                arg.param.0,
                expr(&arg.value, Env::default())
            ));
        }
    }
}

/* ---- handlers ---------------------------------------------------------- */

/// `onMount` / `onUnmount` handlers of an element or component call.
pub fn lifecycle_handlers(props: &[Prop]) -> (Option<&Handler>, Option<&Handler>) {
    let mut lifecycle = (None, None);
    for prop in props {
        if let PropValue::Handler(handler) = &prop.value {
            match prop.name {
                "onMount" => lifecycle.0 = Some(handler),
                "onUnmount" => lifecycle.1 = Some(handler),
                _ => {}
            }
        }
    }
    lifecycle
}

/// Mount events are queued after the subtree is built and unmount events
/// after the subtree is released, so children come before parents.
pub fn lifecycle(
    program: &Program,
    (mount, unmount): (Option<&Handler>, Option<&Handler>),
    parts: &mut Parts,
) {
    if let Some(handler) = mount {
        parts.create.line(lifecycle_call(program, handler));
    }
    if let Some(handler) = unmount {
        parts.cleanup.line(lifecycle_call(program, handler));
    }
}

/// `{ name: value, ... }` for an emitted event.
fn payload(program: &Program, event: EventId, args: &[EmitArg]) -> String {
    let params = &program.event(event).params;
    if params.is_empty() {
        return "{}".to_owned();
    }
    let fields: Vec<String> = params
        .iter()
        .zip(args)
        .map(|(param, arg)| {
            let value = match arg {
                EmitArg::Expr(e) => expr(e, Env::default()),
                EmitArg::Payload => "$a".to_owned(),
            };
            format!("{}: {value}", js::object_key(&param.name))
        })
        .collect();
    format!("{{ {} }}", fields.join(", "))
}

/// Statements run when an element handler fires; the payload is `$a`.
pub fn handler_body(program: &Program, handler: &Handler) -> String {
    match handler {
        Handler::Emit { event, args } => format!(
            "c.app.$emit({}, {});",
            js::string(&program.event(*event).name),
            payload(program, *event, args)
        ),
        Handler::Lambda { body, .. } => body.iter().map(assign).collect::<Vec<_>>().join(" "),
    }
}

fn assign(assign: &Assign) -> String {
    let index = assign.state.0;
    let value = expr(&assign.value, Env::default());
    let value = match assign.op {
        AssignOp::Set => value,
        AssignOp::Add => format!("c.s[{index}] + {value}"),
        AssignOp::Sub => format!("c.s[{index}] - {value}"),
    };
    format!("c.state({index}, {value});")
}

fn lifecycle_call(program: &Program, handler: &Handler) -> String {
    let Handler::Emit { event, args } = handler else {
        unreachable!("lifecycle properties only accept events");
    };
    format!(
        "$.lifecycle(c.app, {}, {});",
        js::string(&program.event(*event).name),
        payload(program, *event, args)
    )
}
