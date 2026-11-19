use crate::color::Color;
use crate::elements::{self, PropType};
use crate::ir::{
    Component, ComponentId, Element, Expr, ExprKind, NodeId, NodeKind, Program, PropValue,
};

use super::json::fnv1a;
use super::{
    Constant, EventSlot, MountPoint, NodeMap, PointKind, Role, StaticProp, Template, TemplateId,
    TemplateKind, ValueSlot, ValueType, ViewLayout, ViewNode, ViewNodeKind,
};

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
}

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
                            let ty = elements::prop_type(*element, prop.name);
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
