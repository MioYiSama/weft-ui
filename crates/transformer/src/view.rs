//! View protocol layout: the contract between the JS logic layer and every
//! non-Web view layer.
//!
//! [`ViewLayout`] splits each component into block templates (component body,
//! `if` branch, `for` row, slot content), numbered across the whole app. A
//! template holds the static element tree with constants inlined, plus the
//! dynamic parts the logic layer drives at run time:
//!
//! - value slots: dynamic properties and text, written by `set` ops; color
//!   properties have semantic type `Color` and travel as strings;
//! - event slots: user events reported back with `dispatch`;
//! - mount points: where child scopes are inserted by `branch`, `rows` and
//!   `attach` ops.
//!
//! Template `i` for `i < components.len()` is the body of component `i`;
//! nested templates follow in discovery order. The encoding of ops is
//! specified in `docs/PROTOCOL.md`.

use std::fmt::{self, Write as _};

use crate::color::Color;
use crate::ir::{
    Component, ComponentId, Element, Expr, ExprKind, NodeId, NodeKind, Program, PropValue,
};
use crate::sema::builtins::{self, PropType};

/// Index into [`ViewLayout::templates`]; the same number the logic layer
/// sends in `create` ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TemplateId(pub u32);

impl TemplateId {
    #[must_use]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Block templates of a whole app.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewLayout {
    templates: Vec<Template>,
    /// Lookup from IR nodes to their place in the layout, by component.
    components: Vec<Vec<NodeMap>>,
    hash: u64,
}

/// One block template.
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    /// Human-readable name for dumps and diagnostics, e.g. `TodoList:if8.0`.
    pub label: String,
    /// Component whose IR nodes the template renders.
    pub component: ComponentId,
    pub kind: TemplateKind,
    /// Static tree in pre-order; node indices are local to the template.
    pub nodes: Vec<ViewNode>,
    /// Top-level nodes in source order.
    pub roots: Vec<u32>,
    pub values: Vec<ValueSlot>,
    pub events: Vec<EventSlot>,
    pub points: Vec<MountPoint>,
}

/// What a template renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateKind {
    /// The body of [`Template::component`].
    Component,
    /// Branch `index` of the `if` node `node`.
    Branch { node: NodeId, index: u32 },
    /// One row of the `for` node `node`.
    Row { node: NodeId },
    /// Children that the component call `node` passes to its callee's slot.
    SlotContent { node: NodeId },
}

/// A node of a template's static tree.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewNode {
    /// IR node of [`Template::component`] this node renders.
    pub source: NodeId,
    pub kind: ViewNodeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewNodeKind {
    Element {
        element: Element,
        /// Constant properties, inlined into the template.
        props: Vec<StaticProp>,
        children: Vec<u32>,
    },
    /// Placeholder replaced by the roots of the child scopes mounted at
    /// mount point `index` of the template.
    Point(u32),
}

/// A constant property value.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticProp {
    pub name: &'static str,
    pub value: Constant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant {
    Int(i64),
    Bool(bool),
    String(String),
    /// A validated color, formatted as normalized lowercase hex.
    Color(Color),
}

/// Type of the values a slot receives or an event carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Int,
    Bool,
    String,
    Color,
}

impl ValueType {
    fn of(ty: PropType) -> Self {
        match ty {
            PropType::Int => ValueType::Int,
            PropType::Bool => ValueType::Bool,
            PropType::String => ValueType::String,
            PropType::Color => ValueType::Color,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ValueType::Int => "Int",
            ValueType::Bool => "Bool",
            ValueType::String => "String",
            ValueType::Color => "Color",
        }
    }
}

/// A dynamic property or text of an element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueSlot {
    /// Local node index of the element.
    pub node: u32,
    pub prop: &'static str,
    pub ty: ValueType,
}

/// A user event of an element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSlot {
    /// Local node index of the element.
    pub node: u32,
    /// Handler property, e.g. `onClick`.
    pub event: &'static str,
    /// Type of the element's value sent with the event, if any.
    pub payload: Option<ValueType>,
    /// Value slot the user edits (controlled `Input` / `Checkbox`). The view
    /// restores it to the last value it received when `dispatch` reports
    /// that the edit was not accepted.
    pub controls: Option<u32>,
}

/// Where child scopes are inserted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountPoint {
    /// Local node index of the [`ViewNodeKind::Point`] node.
    pub node: u32,
    pub kind: PointKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointKind {
    /// An `if` chain: at most one child, from the template of its branch.
    Branch { templates: Vec<TemplateId> },
    /// A `for` loop: ordered children of the row template.
    List { template: TemplateId },
    /// A component call: one child of the callee's body template.
    Instance { template: TemplateId },
    /// The component's `Children` parameter: at most one child, whose
    /// template is the caller's slot content.
    Slot,
}

/// Place of an IR node in the layout.
#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeMap {
    template: TemplateId,
    local: u32,
    role: Role,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Role {
    Element {
        /// Value slot of each IR property, by property index.
        values: Vec<Option<u32>>,
        event: Option<u32>,
    },
    Point {
        index: u32,
        /// Branch templates of an `if`, the row template of a `for`, or the
        /// slot content of a component call with children.
        children: Vec<TemplateId>,
    },
}

impl ViewLayout {
    /// Derives the layout of every component of `program`.
    #[must_use]
    pub fn new(program: &Program) -> Self {
        let mut builder = Builder {
            program,
            templates: program.components.iter().map(|_| None).collect(),
            components: program
                .components
                .iter()
                .map(|c| vec![None; c.nodes.len()])
                .collect(),
        };
        for (index, comp) in program.components.iter().enumerate() {
            let id = TemplateId(offset(index));
            let component = ComponentId(id.0);
            builder.build(
                id,
                component,
                comp.name.clone(),
                TemplateKind::Component,
                &comp.roots,
            );
        }
        let templates: Vec<Template> = builder
            .templates
            .into_iter()
            .map(|t| t.expect("every reserved template is built"))
            .collect();
        let components = builder
            .components
            .into_iter()
            .map(|nodes| {
                nodes
                    .into_iter()
                    .map(|n| n.expect("every IR node belongs to a template"))
                    .collect()
            })
            .collect();
        let mut layout = ViewLayout {
            templates,
            components,
            hash: 0,
        };
        layout.hash = fnv1a(layout.templates_json(None).as_bytes());
        layout
    }

    #[must_use]
    pub fn templates(&self) -> &[Template] {
        &self.templates
    }

    #[must_use]
    pub fn template(&self, id: TemplateId) -> &Template {
        &self.templates[id.index()]
    }

    /// Hash of the layout, compared at startup by the logic and view layers
    /// so a mismatched JS bundle and native project fail loudly.
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// The body template of `component`.
    #[must_use]
    pub fn body(component: ComponentId) -> TemplateId {
        TemplateId(component.0)
    }

    fn node(&self, component: ComponentId, node: NodeId) -> &NodeMap {
        &self.components[component.index()][node.index()]
    }

    /// The template holding `node` of `component` and the node's local index
    /// in it.
    #[must_use]
    pub fn location(&self, component: ComponentId, node: NodeId) -> (TemplateId, u32) {
        let map = self.node(component, node);
        (map.template, map.local)
    }

    /// Value slot of property `prop` (an index into the IR properties) of
    /// the element `node`; `None` for constants and handlers.
    ///
    /// # Panics
    ///
    /// Panics if `node` is not an element.
    #[must_use]
    pub fn value_slot(&self, component: ComponentId, node: NodeId, prop: usize) -> Option<u32> {
        match &self.node(component, node).role {
            Role::Element { values, .. } => values[prop],
            Role::Point { .. } => panic!("node {} is not an element", node.0),
        }
    }

    /// Event slot of the element `node`, if it has one.
    ///
    /// # Panics
    ///
    /// Panics if `node` is not an element.
    #[must_use]
    pub fn event_slot(&self, component: ComponentId, node: NodeId) -> Option<u32> {
        match &self.node(component, node).role {
            Role::Element { event, .. } => *event,
            Role::Point { .. } => panic!("node {} is not an element", node.0),
        }
    }

    /// Mount point of an `if`, `for`, component call or slot node within its
    /// template.
    ///
    /// # Panics
    ///
    /// Panics if `node` is an element.
    #[must_use]
    pub fn point(&self, component: ComponentId, node: NodeId) -> u32 {
        match &self.node(component, node).role {
            Role::Point { index, .. } => *index,
            Role::Element { .. } => panic!("node {} is an element", node.0),
        }
    }

    /// Templates created for the children of a mount point: one per branch
    /// of an `if`, the row of a `for`, or the slot content of a component
    /// call (empty without children).
    ///
    /// # Panics
    ///
    /// Panics if `node` is an element.
    #[must_use]
    pub fn child_templates(&self, component: ComponentId, node: NodeId) -> &[TemplateId] {
        match &self.node(component, node).role {
            Role::Point { children, .. } => children,
            Role::Element { .. } => panic!("node {} is an element", node.0),
        }
    }

    /// The layout as JSON, consumed by JS tools such as the headless renderer.
    #[must_use]
    pub fn to_json(&self) -> String {
        self.templates_json(Some(self.hash))
    }

    fn templates_json(&self, hash: Option<u64>) -> String {
        let mut out = Json::default();
        out.open('{');
        if let Some(hash) = hash {
            out.key("hash");
            out.string(&format!("{hash:016x}"));
        }
        out.key("templates");
        out.open('[');
        for template in &self.templates {
            out.item();
            template.write_json(&mut out);
        }
        out.close(']');
        out.close('}');
        out.text.push('\n');
        out.text
    }
}

impl Template {
    fn write_json(&self, out: &mut Json) {
        out.open('{');
        out.key("label");
        out.string(&self.label);
        out.key("kind");
        out.string(match self.kind {
            TemplateKind::Component => "component",
            TemplateKind::Branch { .. } => "branch",
            TemplateKind::Row { .. } => "row",
            TemplateKind::SlotContent { .. } => "slot",
        });
        out.key("roots");
        out.inline(&numbers(&self.roots));
        out.key("nodes");
        out.open('[');
        for node in &self.nodes {
            out.item();
            out.inline(&node.json());
        }
        out.close(']');
        out.key("values");
        out.open('[');
        for slot in &self.values {
            out.item();
            out.inline(&format!(
                "{{\"node\": {}, \"prop\": {}, \"type\": {}}}",
                slot.node,
                quote(slot.prop),
                quote(slot.ty.as_str())
            ));
        }
        out.close(']');
        out.key("events");
        out.open('[');
        for slot in &self.events {
            out.item();
            let payload = slot
                .payload
                .map_or("null".to_owned(), |t| quote(t.as_str()));
            let controls = slot.controls.map_or("null".to_owned(), |c| c.to_string());
            out.inline(&format!(
                "{{\"node\": {}, \"event\": {}, \"payload\": {payload}, \"controls\": {controls}}}",
                slot.node,
                quote(slot.event)
            ));
        }
        out.close(']');
        out.key("points");
        out.open('[');
        for point in &self.points {
            out.item();
            let kind = match &point.kind {
                PointKind::Branch { templates } => {
                    let ids: Vec<u32> = templates.iter().map(|t| t.0).collect();
                    format!("\"kind\": \"branch\", \"templates\": {}", numbers(&ids))
                }
                PointKind::List { template } => {
                    format!("\"kind\": \"list\", \"template\": {}", template.0)
                }
                PointKind::Instance { template } => {
                    format!("\"kind\": \"instance\", \"template\": {}", template.0)
                }
                PointKind::Slot => "\"kind\": \"slot\"".to_owned(),
            };
            out.inline(&format!("{{\"node\": {}, {kind}}}", point.node));
        }
        out.close(']');
        out.close('}');
    }
}

impl ViewNode {
    fn json(&self) -> String {
        match &self.kind {
            ViewNodeKind::Element {
                element,
                props,
                children,
            } => {
                let props: Vec<String> = props
                    .iter()
                    .map(|p| format!("{}: {}", quote(p.name), p.value))
                    .collect();
                format!(
                    "{{\"element\": {}, \"props\": {{{}}}, \"children\": {}}}",
                    quote(element.name()),
                    props.join(", "),
                    numbers(children)
                )
            }
            ViewNodeKind::Point(index) => format!("{{\"point\": {index}}}"),
        }
    }
}

impl fmt::Display for Constant {
    /// JSON form of the constant.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Int(v) => write!(f, "{v}"),
            Constant::Bool(v) => write!(f, "{v}"),
            Constant::String(s) => f.write_str(&quote(s)),
            Constant::Color(color) => write!(f, "\"{color}\""),
        }
    }
}

/* ---- construction -------------------------------------------------------- */

/// Converts an index into a layout `u32`.
fn offset(index: usize) -> u32 {
    u32::try_from(index).expect("layout indices fit in u32")
}

/// The property a user edits on a controlled element.
fn controlled_prop(element: Element) -> Option<&'static str> {
    match element {
        Element::Input => Some("value"),
        Element::Checkbox => Some("checked"),
        _ => None,
    }
}

/// The handler property reporting user edits of a controlled element.
const EDIT_EVENT: &str = "onChange";

fn constant(expr: &Expr, ty: PropType) -> Option<Constant> {
    match &expr.kind {
        ExprKind::Str(s) if ty == PropType::Color => Some(Constant::Color(
            Color::parse(s).expect("color literals were validated during semantic analysis"),
        )),
        ExprKind::Str(s) => Some(Constant::String(s.clone())),
        ExprKind::Int(v) => Some(Constant::Int(*v)),
        ExprKind::Bool(v) => Some(Constant::Bool(*v)),
        ExprKind::Coerce { expr, .. } => constant(expr, ty),
        _ => None,
    }
}

struct Builder<'p> {
    program: &'p Program,
    /// Reserved templates are `None` until built.
    templates: Vec<Option<Template>>,
    components: Vec<Vec<Option<NodeMap>>>,
}

/// A template under construction.
struct Draft {
    id: TemplateId,
    component: ComponentId,
    nodes: Vec<ViewNode>,
    values: Vec<ValueSlot>,
    events: Vec<EventSlot>,
    points: Vec<MountPoint>,
}

impl Builder<'_> {
    fn reserve(&mut self) -> TemplateId {
        let id = TemplateId(offset(self.templates.len()));
        self.templates.push(None);
        id
    }

    fn build(
        &mut self,
        id: TemplateId,
        component: ComponentId,
        label: String,
        kind: TemplateKind,
        roots: &[NodeId],
    ) {
        let mut draft = Draft {
            id,
            component,
            nodes: Vec::new(),
            values: Vec::new(),
            events: Vec::new(),
            points: Vec::new(),
        };
        let roots = roots.iter().map(|&n| self.node(&mut draft, n)).collect();
        self.templates[id.index()] = Some(Template {
            label,
            component,
            kind,
            nodes: draft.nodes,
            roots,
            values: draft.values,
            events: draft.events,
            points: draft.points,
        });
    }

    fn comp(&self, id: ComponentId) -> &Component {
        self.program.component(id)
    }

    /// Adds `id` and its subtree to the template; returns its local index.
    fn node(&mut self, draft: &mut Draft, id: NodeId) -> u32 {
        let program = self.program;
        let comp = program.component(draft.component);
        let local = offset(draft.nodes.len());
        let role = match &comp.node(id).kind {
            NodeKind::Element {
                element,
                props,
                children,
            } => {
                draft.nodes.push(ViewNode {
                    source: id,
                    kind: ViewNodeKind::Element {
                        element: *element,
                        props: Vec::new(),
                        children: Vec::new(),
                    },
                });
                let controlled = controlled_prop(*element);
                let mut statics = Vec::new();
                let mut values = Vec::with_capacity(props.len());
                let mut handler = None;
                let mut edited = None;
                for prop in props {
                    let slot = match &prop.value {
                        PropValue::Handler(_) => {
                            if !matches!(prop.name, "onMount" | "onUnmount") {
                                handler = Some(prop.name);
                            }
                            None
                        }
                        PropValue::Expr(value) => {
                            let ty = builtins::prop_type(*element, prop.name);
                            let fixed = (prop.binding.is_none() && controlled != Some(prop.name))
                                .then(|| constant(value, ty))
                                .flatten();
                            match fixed {
                                Some(value) => {
                                    statics.push(StaticProp {
                                        name: prop.name,
                                        value,
                                    });
                                    None
                                }
                                None => {
                                    let slot = offset(draft.values.len());
                                    draft.values.push(ValueSlot {
                                        node: local,
                                        prop: prop.name,
                                        ty: ValueType::of(ty),
                                    });
                                    if controlled == Some(prop.name) {
                                        edited = Some((slot, ValueType::of(ty)));
                                    }
                                    Some(slot)
                                }
                            }
                        }
                    };
                    values.push(slot);
                }
                let event = match (handler, edited) {
                    (_, Some((slot, ty))) => Some(EventSlot {
                        node: local,
                        event: EDIT_EVENT,
                        payload: Some(ty),
                        controls: Some(slot),
                    }),
                    (Some(name), None) => Some(EventSlot {
                        node: local,
                        event: name,
                        payload: None,
                        controls: None,
                    }),
                    (None, None) => None,
                };
                let event = event.map(|slot| {
                    draft.events.push(slot);
                    offset(draft.events.len() - 1)
                });
                let children: Vec<u32> = children.iter().map(|&c| self.node(draft, c)).collect();
                if let ViewNodeKind::Element {
                    props, children: c, ..
                } = &mut draft.nodes[local as usize].kind
                {
                    *props = statics;
                    *c = children;
                }
                Role::Element { values, event }
            }
            kind => {
                let index = offset(draft.points.len());
                draft.nodes.push(ViewNode {
                    source: id,
                    kind: ViewNodeKind::Point(index),
                });
                let (point, children) = self.point(draft.component, id, kind);
                draft.points.push(MountPoint {
                    node: local,
                    kind: point,
                });
                Role::Point { index, children }
            }
        };
        self.components[draft.component.index()][id.index()] = Some(NodeMap {
            template: draft.id,
            local,
            role,
        });
        local
    }

    /// Builds the child templates of a mount point node.
    fn point(
        &mut self,
        component: ComponentId,
        id: NodeId,
        kind: &NodeKind,
    ) -> (PointKind, Vec<TemplateId>) {
        let name = self.comp(component).name.clone();
        let n = id.0;
        match kind {
            NodeKind::If { branches, .. } => {
                let mut templates = Vec::with_capacity(branches.len());
                for (index, branch) in branches.iter().enumerate() {
                    let child = self.reserve();
                    let index = offset(index);
                    self.build(
                        child,
                        component,
                        format!("{name}:if{n}.{index}"),
                        TemplateKind::Branch { node: id, index },
                        &branch.children,
                    );
                    templates.push(child);
                }
                (
                    PointKind::Branch {
                        templates: templates.clone(),
                    },
                    templates,
                )
            }
            NodeKind::For { children, .. } => {
                let child = self.reserve();
                self.build(
                    child,
                    component,
                    format!("{name}:for{n}"),
                    TemplateKind::Row { node: id },
                    children,
                );
                (PointKind::List { template: child }, vec![child])
            }
            NodeKind::Component {
                component: callee,
                children,
                ..
            } => {
                let content = if children.is_empty() {
                    Vec::new()
                } else {
                    let child = self.reserve();
                    self.build(
                        child,
                        component,
                        format!("{name}:slot{n}"),
                        TemplateKind::SlotContent { node: id },
                        children,
                    );
                    vec![child]
                };
                (
                    PointKind::Instance {
                        template: ViewLayout::body(*callee),
                    },
                    content,
                )
            }
            NodeKind::Slot { .. } => (PointKind::Slot, Vec::new()),
            NodeKind::Element { .. } => unreachable!("elements are not mount points"),
        }
    }
}

/* ---- JSON ---------------------------------------------------------------- */

/// Minimal pretty JSON writer: objects and arrays nest by indentation,
/// scalars and small records are written inline.
#[derive(Default)]
struct Json {
    text: String,
    depth: usize,
    /// Whether the current container already has an entry.
    started: Vec<bool>,
}

impl Json {
    fn open(&mut self, bracket: char) {
        self.text.push(bracket);
        self.depth += 1;
        self.started.push(false);
    }

    fn close(&mut self, bracket: char) {
        self.depth -= 1;
        if self.started.pop() == Some(true) {
            self.newline();
        }
        self.text.push(bracket);
    }

    fn newline(&mut self) {
        self.text.push('\n');
        for _ in 0..self.depth {
            self.text.push_str("  ");
        }
    }

    /// Starts an array entry.
    fn item(&mut self) {
        let started = self.started.last_mut().expect("inside a container");
        if *started {
            self.text.push(',');
        }
        *started = true;
        self.newline();
    }

    fn key(&mut self, key: &str) {
        self.item();
        self.text.push_str(&quote(key));
        self.text.push_str(": ");
    }

    fn string(&mut self, value: &str) {
        self.text.push_str(&quote(value));
    }

    fn inline(&mut self, json: &str) {
        self.text.push_str(json);
    }
}

fn numbers(values: &[u32]) -> String {
    let items: Vec<String> = values.iter().map(u32::to_string).collect();
    format!("[{}]", items.join(", "))
}

/// A JSON string literal.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// 64-bit FNV-1a: stable across platforms and toolchains.
fn fnv1a(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes
        .iter()
        .fold(OFFSET, |hash, &b| (hash ^ u64::from(b)).wrapping_mul(PRIME))
}
