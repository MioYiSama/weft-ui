//! Human-readable dump of each component's dirty slots and binding dependencies.

use std::fmt::Write;

use crate::ir::{BindingKind, Component, Dep, NodeId, NodeKind, ParamKind, Program, SlotRef, Type};

/// Renders every component's slot layout and the dependencies of each binding.
#[must_use]
pub fn dependency_report(program: &Program) -> String {
    let mut out = String::new();
    for (index, component) in program.components.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        write_component(&mut out, program, component);
    }
    out
}

fn write_component(out: &mut String, program: &Program, component: &Component) {
    let params: Vec<String> = component
        .params
        .iter()
        .map(|p| match &p.kind {
            ParamKind::Value(ty) => format!("{}: {}", p.name, program.display_type(ty)),
            ParamKind::Children => format!("{}: Children", p.name),
            ParamKind::Callback { payload: None } => format!("{}: Event", p.name),
            ParamKind::Callback { payload: Some(ty) } => {
                format!("{}: Event<{}>", p.name, program.display_type(ty))
            }
        })
        .collect();
    let _ = writeln!(out, "component {}({})", component.name, params.join(", "));

    if !component.slots.is_empty() {
        let _ = writeln!(out, "  slots:");
        for (i, slot) in component.slots.iter().enumerate() {
            let label = format!("#{i}");
            let _ = writeln!(out, "    {label:<6} {}", dep_name(program, component, slot));
        }
    }
    for lp in &component.loops {
        let _ = writeln!(
            out,
            "  loop {}: {}",
            lp.var,
            program.display_type(&lp.item_ty)
        );
        for (i, slot) in lp.slots.iter().enumerate() {
            let label = format!("{}#{i}", lp.var);
            let _ = writeln!(out, "    {label:<6} {}", dep_name(program, component, slot));
        }
    }

    let labels = node_labels(program, component);
    let rows: Vec<[String; 4]> = component
        .bindings
        .iter()
        .map(|b| {
            let kind = match b.kind {
                BindingKind::Prop(name) | BindingKind::Handler(name) => name.to_owned(),
                BindingKind::Arg(p) => {
                    let NodeKind::Component {
                        component: callee, ..
                    } = component.node(b.node).kind
                    else {
                        unreachable!("argument bindings belong to component nodes");
                    };
                    format!("arg {}", program.component(callee).params[p.index()].name)
                }
                BindingKind::Condition => "condition".to_owned(),
                BindingKind::List => "list".to_owned(),
                BindingKind::Key => "key".to_owned(),
            };
            let deps: Vec<String> = b
                .deps
                .iter()
                .map(|d| dep_name(program, component, d))
                .collect();
            let bits: Vec<String> = b
                .bits
                .iter()
                .map(|bit| match *bit {
                    SlotRef::Component(i) => format!("#{i}"),
                    SlotRef::Loop(l, i) => format!("{}#{i}", component.loops[l.index()].var),
                })
                .collect();
            [
                labels[b.node.index()].clone(),
                kind,
                deps.join(", "),
                bits.join(" "),
            ]
        })
        .collect();
    if !rows.is_empty() {
        let width = |col: usize| {
            rows.iter()
                .map(|r| r[col].chars().count())
                .max()
                .unwrap_or(0)
        };
        let (w0, w1, w2) = (width(0), width(1), width(2));
        let _ = writeln!(out, "  bindings:");
        for (i, [node, kind, deps, bits]) in rows.iter().enumerate() {
            let id = format!("b{i}");
            let _ = writeln!(
                out,
                "    {id:<4} {node:<w0$}  {kind:<w1$}  {deps:<w2$}  {bits}"
            );
        }
    }

    let events = |ids: &[crate::ir::EventId]| {
        ids.iter()
            .map(|&e| program.event(e).name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    if !component.emits.is_empty() {
        let _ = writeln!(out, "  emits: {}", events(&component.emits));
    }
    if component.reachable_events != component.emits {
        let _ = writeln!(
            out,
            "  reachable events: {}",
            events(&component.reachable_events)
        );
    }
}

/// Dotted name of a dependency, e.g. `model.user.name`.
fn dep_name(program: &Program, component: &Component, dep: &Dep) -> String {
    let mut name = component.root_name(dep.root).to_owned();
    let mut ty = component.root_type(dep.root);
    for field in &dep.path {
        let Type::Model(model) = ty else {
            unreachable!("dependency paths only traverse models");
        };
        let field = &program.model(*model).fields[field.index()];
        name.push('.');
        name.push_str(&field.name);
        ty = &field.ty;
    }
    name
}

/// Path-like label for every node, e.g. `Column > if > for todo > Row > Text`.
fn node_labels(program: &Program, component: &Component) -> Vec<String> {
    fn visit(
        id: NodeId,
        prefix: &str,
        program: &Program,
        component: &Component,
        labels: &mut [String],
    ) {
        let node = component.node(id);
        let own = match &node.kind {
            NodeKind::Element { element, .. } => element.name().to_owned(),
            NodeKind::Component { component: c, .. } => program.component(*c).name.clone(),
            NodeKind::If { .. } => "if".to_owned(),
            NodeKind::For { loop_id, .. } => {
                format!("for {}", component.loops[loop_id.index()].var)
            }
            NodeKind::Slot { param } => component.params[param.index()].name.clone(),
        };
        let label = if prefix.is_empty() {
            own
        } else {
            format!("{prefix} > {own}")
        };
        for child in node.kind.children() {
            visit(child, &label, program, component, labels);
        }
        labels[id.index()] = label;
    }

    let mut labels = vec![String::new(); component.nodes.len()];
    for &root in &component.roots {
        visit(root, "", program, component, &mut labels);
    }
    labels
}
