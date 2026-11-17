//! Web DOM backend: one create function per component.
//!
//! Static structure is cloned from `<template>`s; every block (component body,
//! `if` branch, `for` row, slot content) gets an update function that re-runs
//! only the bindings whose dirty bits are set and walks into nested blocks
//! only when bits they depend on are set. See `@weft-ui/runtime` for the
//! runtime half of this contract.

use std::collections::{BTreeMap, HashMap};

use crate::ir::{
    Assign, AssignOp, BindingId, Component, ComponentId, Dep, DepRoot, Element, EmitArg, Expr,
    ExprKind, FieldId, Handler, LoopId, NodeId, NodeKind, ParamKind, Program, Prop, PropValue,
    SlotRef, Type,
};

use super::expr::{Env, expr};
use super::js::{self, Code};

/// Module-level constants shared by all components of a module.
#[derive(Debug, Default)]
pub(super) struct Consts {
    templates: Vec<String>,
    template_ids: HashMap<String, usize>,
    shapes: Vec<String>,
    shape_ids: HashMap<String, usize>,
}

impl Consts {
    fn template(&mut self, html: String) -> String {
        let next = self.templates.len();
        let id = *self.template_ids.entry(html.clone()).or_insert(next);
        if id == next {
            self.templates.push(html);
        }
        format!("$t{id}")
    }

    fn shape(&mut self, literal: String) -> String {
        let next = self.shapes.len();
        let id = *self.shape_ids.entry(literal.clone()).or_insert(next);
        if id == next {
            self.shapes.push(literal);
        }
        format!("$s{id}")
    }

    pub(super) fn write(&self, out: &mut Code) {
        for (i, shape) in self.shapes.iter().enumerate() {
            out.line(format!("const $s{i} = {shape};"));
        }
        for (i, html) in self.templates.iter().enumerate() {
            out.line(format!("const $t{i} = $.template({});", js::string(html)));
        }
    }
}

/// Emits `export function {Name}$c(app, args, children, depth)`.
pub(super) fn component(program: &Program, id: ComponentId, consts: &mut Consts, out: &mut Code) {
    let comp = program.component(id);
    let mut generator = Gen {
        program,
        comp,
        consts,
    };
    out.open(format!(
        "export function {}$c(app, args, children, depth) {{",
        comp.name
    ));
    out.line(format!(
        "const c = new $.Instance(app, {}, depth, args, children);",
        words(comp.slots.len())
    ));
    for (index, param) in comp.params.iter().enumerate() {
        if let ParamKind::Value(_) = param.kind {
            let shape = generator.root_shape(DepRoot::Param(crate::ir::ParamId(offset(index))));
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
        let shape = generator.root_shape(DepRoot::State(crate::ir::StateId(offset(index))));
        out.line(format!(
            "c.sw[{index}] = $.watch({shape}, c, c.s[{index}]);"
        ));
    }
    let mut body = Code::default();
    generator.block(&comp.roots, &Ctx::default(), "c", &mut body);
    out.append(body);
    out.line("return c;");
    out.close("}");
}

fn offset(index: usize) -> u32 {
    crate::span::offset(index)
}

/// Dirty words needed for `slots` bits; scopes always have at least one.
fn words(slots: usize) -> usize {
    slots.div_ceil(32).max(1)
}

/// Loops enclosing the code being generated, outermost first.
#[derive(Debug, Clone, Default)]
struct Ctx {
    loops: Vec<LoopId>,
}

impl Ctx {
    /// The innermost scope: the current row, or the component instance.
    fn scope(&self) -> String {
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

    fn with_loop(&self, l: LoopId) -> Ctx {
        let mut loops = self.loops.clone();
        loops.push(l);
        Ctx { loops }
    }
}

#[derive(Default)]
struct Parts {
    create: Code,
    update: Code,
    cleanup: Code,
}

/// How a property value reaches the DOM.
#[derive(Debug, Clone, Copy)]
enum Target {
    Text,
    Value,
    Checked,
    Disabled,
    Attr(&'static str),
    Px(&'static str),
    Css(&'static str),
    Bold,
}

fn target(element: Element, prop: &str) -> Target {
    match (element, prop) {
        (Element::Text, "value") => Target::Text,
        (Element::Input, "value") => Target::Value,
        (Element::Checkbox, "checked") => Target::Checked,
        (_, "disabled") => Target::Disabled,
        (_, "placeholder") => Target::Attr("placeholder"),
        (_, "src") => Target::Attr("src"),
        (_, "padding") => Target::Px("padding"),
        (_, "gap") => Target::Px("gap"),
        (_, "width") => Target::Px("width"),
        (_, "height") => Target::Px("height"),
        (_, "size") => Target::Px("font-size"),
        (_, "background") => Target::Css("background"),
        (_, "color") => Target::Css("color"),
        (_, "bold") => Target::Bold,
        (element, prop) => unreachable!("`{}` has no property `{prop}`", element.name()),
    }
}

/// Tag, fixed attributes and base style of an element.
fn tag(element: Element) -> (&'static str, &'static str, &'static str) {
    match element {
        Element::Column => ("div", "", "display:flex;flex-direction:column"),
        Element::Row => ("div", "", "display:flex;flex-direction:row"),
        Element::Text => ("span", "", ""),
        Element::Button => ("button", " type=\"button\"", ""),
        Element::Input => ("input", "", ""),
        Element::Checkbox => ("input", " type=\"checkbox\"", ""),
        Element::Image => ("img", "", ""),
    }
}

fn is_void(element: Element) -> bool {
    matches!(element, Element::Input | Element::Checkbox | Element::Image)
}

enum Literal<'e> {
    Str(&'e str),
    Int(i64),
    Bool(bool),
}

fn literal(e: &Expr) -> Option<Literal<'_>> {
    match &e.kind {
        ExprKind::Str(s) => Some(Literal::Str(s)),
        ExprKind::Int(v) => Some(Literal::Int(*v)),
        ExprKind::Bool(v) => Some(Literal::Bool(*v)),
        ExprKind::Coerce { expr, .. } => literal(expr),
        _ => None,
    }
}

/// Whether a property is written into the template instead of by code.
/// Controlled values are always set by code so the runtime knows them.
fn is_template_prop(element: Element, prop: &Prop) -> bool {
    let PropValue::Expr(value) = &prop.value else {
        return false;
    };
    prop.binding.is_none()
        && literal(value).is_some()
        && !matches!(target(element, prop.name), Target::Value | Target::Checked)
}

fn apply(target: Target, node: &str, value: &str) -> String {
    match target {
        Target::Text => format!("{node}.textContent = {value};"),
        Target::Value => format!("$.value({node}, {value});"),
        Target::Checked => format!("$.checked({node}, {value});"),
        Target::Disabled => format!("{node}.disabled = {value};"),
        Target::Attr(name) => format!("{node}.setAttribute({}, {value});", js::string(name)),
        Target::Px(name) => format!(
            "{node}.style.setProperty({}, $.px({value}));",
            js::string(name)
        ),
        Target::Css(name) => format!("{node}.style.setProperty({}, {value});", js::string(name)),
        Target::Bold => format!("{node}.style.fontWeight = {value} ? \"bold\" : \"\";"),
    }
}

struct Gen<'a> {
    program: &'a Program,
    comp: &'a Component,
    consts: &'a mut Consts,
}

impl<'a> Gen<'a> {
    fn node_kind(&self, id: NodeId) -> &'a NodeKind {
        &self.comp.node(id).kind
    }

    fn is_element(&self, id: NodeId) -> bool {
        matches!(self.node_kind(id), NodeKind::Element { .. })
    }

    /* ---- dirty bits ---------------------------------------------------- */

    fn bits(&self, binding: BindingId) -> &[SlotRef] {
        &self.comp.bindings[binding.index()].bits
    }

    /// `scope.d[word] & mask || ...` for the given bits, or `None` if empty.
    fn cond<'s>(&self, bits: impl IntoIterator<Item = &'s SlotRef>) -> Option<String> {
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

    /// Bits of the DOM bindings in the subtrees of `roots` that are visible
    /// from `ctx`: the condition for walking into a nested block.
    fn walk_cond(&self, roots: &[NodeId], ctx: &Ctx) -> Option<String> {
        let mut bits = Vec::new();
        for &root in roots {
            self.subtree_bits(root, &mut bits);
        }
        bits.retain(|&bit| ctx.sees(bit));
        self.cond(&bits)
    }

    /// Event handler and key bindings are evaluated on demand by this
    /// backend, so only values, arguments, conditions and lists count.
    fn subtree_bits(&self, id: NodeId, out: &mut Vec<SlotRef>) {
        let kind = self.node_kind(id);
        let mut push = |binding: Option<BindingId>| {
            if let Some(b) = binding {
                out.extend_from_slice(self.bits(b));
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
            self.subtree_bits(child, out);
        }
    }

    /* ---- shapes -------------------------------------------------------- */

    fn root_shape(&mut self, root: DepRoot) -> String {
        let slots: &[Dep] = match root {
            DepRoot::Item(l) => &self.comp.loops[l.index()].slots,
            _ => &self.comp.slots,
        };
        let entries: Vec<(&[FieldId], usize)> = slots
            .iter()
            .enumerate()
            .filter(|(_, dep)| dep.root == root)
            .map(|(bit, dep)| (dep.path.as_slice(), bit))
            .collect();
        let literal = self.shape(self.comp.root_type(root), &entries);
        self.consts.shape(literal)
    }

    fn shape(&self, ty: &Type, entries: &[(&[FieldId], usize)]) -> String {
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
        let fields: Vec<String> = self
            .program
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
                self.shape(&field.ty, &nested)
            })
            .collect();
        format!("[{}]", fields.join(", "))
    }

    /* ---- blocks -------------------------------------------------------- */

    /// Builds the nodes `roots` into the block object `var` (already declared).
    fn block(&mut self, roots: &[NodeId], ctx: &Ctx, var: &str, out: &mut Code) {
        // The first node must be stable: dynamic content is inserted before
        // anchors, so a leading anchor gets a marker comment in front.
        let marker = roots.first().is_none_or(|&r| !self.is_element(r));
        let mut html = String::new();
        if marker {
            html.push_str("<!---->");
        }
        for &root in roots {
            self.html(root, &mut html);
        }
        let template = self.consts.template(html);
        out.line(format!("const f = {template}();"));
        self.refs(roots, "f", marker.then(|| "f.firstChild".to_owned()), out);

        let mut parts = Parts::default();
        for &root in roots {
            self.node(root, ctx, &mut parts);
        }
        out.append(parts.create);
        out.line(format!("{var}.init(f);"));
        if !parts.update.is_empty() {
            out.open(format!("{var}.u = () => {{"));
            out.append(parts.update);
            out.close("};");
        }
        if !parts.cleanup.is_empty() {
            out.open(format!("{var}.x = () => {{"));
            out.append(parts.cleanup);
            out.close("};");
        }
    }

    fn html(&self, id: NodeId, out: &mut String) {
        let NodeKind::Element {
            element,
            props,
            children,
        } = self.node_kind(id)
        else {
            out.push_str("<!---->");
            return;
        };
        let element = *element;
        let (tag, fixed, base_style) = tag(element);
        let mut style = base_style.to_owned();
        let mut attrs = String::from(fixed);
        let mut text = None;
        for prop in props.iter().filter(|p| is_template_prop(element, p)) {
            let PropValue::Expr(value) = &prop.value else {
                continue;
            };
            let Some(value) = literal(value) else {
                continue;
            };
            let mut add_style = |decl: String| {
                if !style.is_empty() {
                    style.push(';');
                }
                style.push_str(&decl);
            };
            match (target(element, prop.name), value) {
                (Target::Text, Literal::Str(s)) => text = Some(s),
                (Target::Disabled, Literal::Bool(true)) => attrs.push_str(" disabled"),
                (Target::Bold, Literal::Bool(true)) => add_style("font-weight:bold".to_owned()),
                (Target::Disabled | Target::Bold, _) => {}
                (Target::Attr(name), Literal::Str(s)) => {
                    attrs.push_str(&format!(" {name}=\""));
                    js::html_attr(s, &mut attrs);
                    attrs.push('"');
                }
                (Target::Px(name), Literal::Int(v)) => add_style(format!("{name}:{v}px")),
                (Target::Css(name), Literal::Str(s)) => add_style(format!("{name}:{s}")),
                (target, _) => unreachable!("literal of the wrong type for {target:?}"),
            }
        }
        out.push('<');
        out.push_str(tag);
        out.push_str(&attrs);
        if !style.is_empty() {
            out.push_str(" style=\"");
            js::html_attr(&style, out);
            out.push('"');
        }
        out.push('>');
        if is_void(element) {
            return;
        }
        if let Some(text) = text {
            js::html_text(text, out);
        }
        for &child in children {
            self.html(child, out);
        }
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
    }

    /// Declares `n{id}` for every template node of `nodes` and their element
    /// children.
    fn refs(&self, nodes: &[NodeId], parent: &str, mut prev: Option<String>, out: &mut Code) {
        for &id in nodes {
            let name = format!("n{}", id.0);
            let init = match &prev {
                Some(prev) => format!("{prev}.nextSibling"),
                None => format!("{parent}.firstChild"),
            };
            out.line(format!("const {name} = {init};"));
            if let NodeKind::Element { children, .. } = self.node_kind(id)
                && !children.is_empty()
            {
                self.refs(children, &name, None, out);
            }
            prev = Some(name);
        }
    }

    fn node(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
        match self.node_kind(id) {
            NodeKind::Element { .. } => self.element(id, ctx, parts),
            NodeKind::If { .. } => self.if_node(id, ctx, parts),
            NodeKind::For { .. } => self.for_node(id, ctx, parts),
            NodeKind::Component { .. } => self.component_node(id, ctx, parts),
            NodeKind::Slot { .. } => {
                let (y, n) = (format!("y{}", id.0), format!("n{}", id.0));
                parts.create.line(format!(
                    "const {y} = c.children ? c.children.create() : null;"
                ));
                parts
                    .create
                    .line(format!("if ({y}) {y}.mount({n}.parentNode, {n});"));
                parts.cleanup.line(format!("if ({y}) {y}.destroy(false);"));
            }
        }
    }

    fn element(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
        let NodeKind::Element {
            element,
            props,
            children,
        } = self.node_kind(id)
        else {
            unreachable!("caller checked the node kind");
        };
        let element = *element;
        let n = format!("n{}", id.0);
        let mut listened = false;
        let mut lifecycle = (None, None);
        for prop in props {
            match &prop.value {
                PropValue::Expr(value) => {
                    if is_template_prop(element, prop) {
                        continue;
                    }
                    let target = target(element, prop.name);
                    let value = expr(value, Env::default());
                    parts.create.line(apply(target, &n, &value));
                    if let Some(cond) = prop.binding.and_then(|b| self.cond(self.bits(b))) {
                        parts
                            .update
                            .line(format!("if ({cond}) {}", apply(target, &n, &value)));
                    }
                }
                PropValue::Handler(handler) => match prop.name {
                    "onMount" => lifecycle.0 = Some(handler),
                    "onUnmount" => lifecycle.1 = Some(handler),
                    name => {
                        listened = true;
                        let body = handler_body(self.program, handler);
                        parts.create.line(match (element, name) {
                            (Element::Input, "onChange") => {
                                format!("$.input({n}, ($a) => {{ {body} }});")
                            }
                            (Element::Checkbox, "onChange") => {
                                format!("$.checkbox({n}, ($a) => {{ {body} }});")
                            }
                            (_, "onClick") => {
                                format!("{n}.addEventListener(\"click\", () => {{ {body} }});")
                            }
                            (element, name) => {
                                unreachable!("`{}` has no handler `{name}`", element.name())
                            }
                        });
                    }
                },
            }
        }
        // Controlled inputs revert user edits the host did not accept.
        if !listened {
            match element {
                Element::Input => parts.create.line(format!("$.input({n}, null);")),
                Element::Checkbox => parts.create.line(format!("$.checkbox({n}, null);")),
                _ => {}
            }
        }
        for &child in children {
            self.node(child, ctx, parts);
        }
        self.lifecycle(lifecycle, parts);
    }

    /// Mount events are queued after the subtree is built and unmount events
    /// after the subtree is released, so children come before parents.
    fn lifecycle(&self, (mount, unmount): (Option<&Handler>, Option<&Handler>), parts: &mut Parts) {
        if let Some(handler) = mount {
            parts.create.line(lifecycle_call(self.program, handler));
        }
        if let Some(handler) = unmount {
            parts.cleanup.line(lifecycle_call(self.program, handler));
        }
    }

    fn if_node(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
        let NodeKind::If { branches, binding } = self.node_kind(id) else {
            unreachable!("caller checked the node kind");
        };
        let (var, n) = (format!("i{}", id.0), format!("n{}", id.0));
        let mut index = String::new();
        let mut has_else = false;
        for (i, branch) in branches.iter().enumerate() {
            match &branch.cond {
                Some(cond) => index.push_str(&format!("{} ? {i} : ", expr(cond, Env::default()))),
                None => {
                    index.push_str(&i.to_string());
                    has_else = true;
                }
            }
        }
        if !has_else {
            index.push_str("-1");
        }

        let mut create = Code::default();
        create.open(format!("const {var} = new $.Branches({n}, (i) => {{"));
        create.open("switch (i) {");
        let mut all_roots = Vec::new();
        for (i, branch) in branches.iter().enumerate() {
            create.open(format!("case {i}: {{"));
            create.line("const b = new $.Block();");
            self.block(&branch.children, ctx, "b", &mut create);
            create.line("return b;");
            create.close("}");
            all_roots.extend_from_slice(&branch.children);
        }
        create.line("default:");
        create.line("  return null;");
        create.close("}");
        create.close("});");
        create.line(format!("{var}.set({index});"));
        parts.create.append(create);

        if let Some(cond) = binding.and_then(|b| self.cond(self.bits(b))) {
            parts
                .update
                .line(format!("if ({cond}) {var}.set({index});"));
        }
        if let Some(cond) = self.walk_cond(&all_roots, ctx) {
            parts.update.line(format!("if ({cond}) {var}.update();"));
        }
        parts.cleanup.line(format!("{var}.destroy();"));
    }

    fn for_node(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
        let NodeKind::For {
            loop_id,
            list,
            key,
            children,
            list_binding,
            ..
        } = self.node_kind(id)
        else {
            unreachable!("caller checked the node kind");
        };
        let loop_id = *loop_id;
        let (var, n, row) = (
            format!("l{}", id.0),
            format!("n{}", id.0),
            format!("r{}", loop_id.0),
        );
        let list = expr(list, Env::default());
        let key = expr(
            key,
            Env {
                key_loop: Some(loop_id),
            },
        );
        let shape = self.root_shape(DepRoot::Item(loop_id));
        let words = words(self.comp.loops[loop_id.index()].slots.len());

        parts
            .create
            .open(format!("const {var} = new $.List({n}, (item) => {{"));
        parts.create.line(format!(
            "const {row} = new $.Row({words}, {}.depth + 1, {shape}, item);",
            ctx.scope()
        ));
        let mut body = Code::default();
        self.block(children, &ctx.with_loop(loop_id), &row, &mut body);
        parts.create.append(body);
        parts.create.line(format!("return {row};"));
        parts.create.close(format!("}}, ($it) => {key});"));
        parts.create.line(format!("{var}.set({list});"));

        if let Some(cond) = list_binding.and_then(|b| self.cond(self.bits(b))) {
            parts.update.line(format!("if ({cond}) {var}.set({list});"));
        }
        if let Some(cond) = self.walk_cond(children, ctx) {
            parts.update.line(format!("if ({cond}) {var}.update();"));
        }
        parts.cleanup.line(format!("{var}.destroy();"));
    }

    fn component_node(&mut self, id: NodeId, ctx: &Ctx, parts: &mut Parts) {
        let NodeKind::Component {
            component,
            args,
            props,
            children,
        } = self.node_kind(id)
        else {
            unreachable!("caller checked the node kind");
        };
        let callee = self.program.component(*component);
        let (var, n) = (format!("k{}", id.0), format!("n{}", id.0));

        let slot = if children.is_empty() {
            "null".to_owned()
        } else {
            let slot = format!("s{}", id.0);
            parts
                .create
                .open(format!("const {slot} = new $.Slot(() => {{"));
            parts.create.line("const b = new $.Block();");
            let mut body = Code::default();
            self.block(children, ctx, "b", &mut body);
            parts.create.append(body);
            parts.create.line("return b;");
            parts.create.close("});");
            slot
        };

        let values: Vec<String> = callee
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
            .collect();
        parts.create.line(format!(
            "const {var} = {}$c(c.app, [{}], {slot}, {}.depth + 1);",
            callee.name,
            values.join(", "),
            ctx.scope()
        ));
        parts
            .create
            .line(format!("{var}.mount({n}.parentNode, {n});"));

        for arg in args {
            if let Some(cond) = arg.binding.and_then(|b| self.cond(self.bits(b))) {
                parts.update.line(format!(
                    "if ({cond}) {var}.arg({}, {});",
                    arg.param.0,
                    expr(&arg.value, Env::default())
                ));
            }
        }
        if !children.is_empty()
            && let Some(cond) = self.walk_cond(children, ctx)
        {
            parts
                .update
                .line(format!("if ({cond}) s{}.update();", id.0));
        }
        parts.cleanup.line(format!("{var}.destroy(false);"));

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
        self.lifecycle(lifecycle, parts);
    }
}

/// `{ name: value, ... }` for an emitted event.
fn payload(program: &Program, event: crate::ir::EventId, args: &[EmitArg]) -> String {
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

fn handler_body(program: &Program, handler: &Handler) -> String {
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
