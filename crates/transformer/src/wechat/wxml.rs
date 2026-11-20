//! WXML templates of the mini program backend.
//!
//! Every component becomes a `<template>` that renders the data object of one
//! instance (keys are listed in [`super::elements`]); nested scopes are
//! reached through their key (`b3.n7_0`, `r0.n9_0`). Slot content is
//! rendered by a template of the calling component, chosen at run time
//! through `t`, so all templates of the app live in one file.

use std::fmt;

use crate::elements::Element;
use crate::ir::{Component, ComponentId, NodeId, NodeKind, ParamId, Program, first_slot};
use crate::style::{COLUMN_STYLE, ROW_STYLE};
use crate::view::ViewLayout;

use super::elements::{Target, Value, event_key, event_type, prop_value, target};

fn slot_name(base: fmt::Arguments<'_>, callee: &Component, param: ParamId) -> String {
    if first_slot(&callee.params) == Some(param) {
        base.to_string()
    } else {
        format!("{base}_{}", param.0)
    }
}

/// Data key for a rendered slot in the receiving component.
pub(crate) fn slot_key(comp: &Component, node: NodeId, param: ParamId) -> String {
    slot_name(format_args!("y{}", node.0), comp, param)
}

/// Template rendering the children a component call passes to a slot.
pub(crate) fn slot_template(
    program: &Program,
    comp: &Component,
    node: NodeId,
    param: ParamId,
) -> String {
    let NodeKind::Component { component, .. } = comp.node(node).kind else {
        unreachable!("slot content comes from a component call");
    };
    slot_name(
        format_args!("{}-s{}", comp.name, node.0),
        program.component(component),
        param,
    )
}

/// Templates for every component of the program.
pub(crate) fn templates(program: &Program, layout: &ViewLayout) -> String {
    let mut out = String::new();
    for (index, comp) in program.components.iter().enumerate() {
        let mut markup = Markup {
            program,
            layout,
            id: ComponentId(u32::try_from(index).expect("component ids fit in u32")),
            comp,
            slots: Vec::new(),
        };
        let mut body = String::new();
        markup.nodes(&comp.roots, "", &mut body);
        template(&comp.name, &body, &mut out);
        for (name, body) in markup.slots {
            template(&name, &body, &mut out);
        }
    }
    out
}

fn template(name: &str, body: &str, out: &mut String) {
    out.push_str("<template name=\"");
    out.push_str(name);
    out.push_str("\">");
    out.push_str(body);
    out.push_str("</template>\n");
}

/// Tag, fixed attributes and base style of an element.
fn tag(element: Element) -> (&'static str, &'static str, &'static str) {
    match element {
        Element::Column => ("view", "", COLUMN_STYLE),
        Element::Row => ("view", "", ROW_STYLE),
        Element::Text => ("text", "", ""),
        // Mini buttons are centered with auto margins by default.
        Element::Button => ("button", " size=\"mini\"", "margin:0"),
        Element::Input => ("input", "", "border:1px solid #ccc;padding:4px 8px"),
        Element::Checkbox => ("switch", " type=\"checkbox\"", ""),
        Element::Image => ("image", "", ""),
    }
}

fn is_void(element: Element) -> bool {
    matches!(element, Element::Input | Element::Checkbox | Element::Image)
}

struct Markup<'a> {
    program: &'a Program,
    layout: &'a ViewLayout,
    id: ComponentId,
    comp: &'a Component,
    /// Slot content templates found so far: name and body.
    slots: Vec<(String, String)>,
}

impl Markup<'_> {
    /// `prefix` leads from the template's data to the current block's data,
    /// e.g. `""`, `"b3."` or `"r0."`.
    fn nodes(&mut self, ids: &[NodeId], prefix: &str, out: &mut String) {
        for &id in ids {
            self.node(id, prefix, out);
        }
    }

    fn node(&mut self, id: NodeId, prefix: &str, out: &mut String) {
        let n = id.0;
        match &self.comp.node(id).kind {
            NodeKind::Element { .. } => self.element(id, prefix, out),
            NodeKind::If { branches, .. } => {
                for (i, branch) in branches.iter().enumerate() {
                    let directive = match (i, &branch.cond) {
                        (_, None) => "wx:else".to_owned(),
                        (0, Some(_)) => format!("wx:if=\"{{{{{prefix}i{n} === 0}}}}\""),
                        (i, Some(_)) => format!("wx:elif=\"{{{{{prefix}i{n} === {i}}}}}\""),
                    };
                    out.push_str(&format!("<block {directive}>"));
                    self.nodes(&branch.children, &format!("{prefix}b{n}."), out);
                    out.push_str("</block>");
                }
            }
            NodeKind::For {
                loop_id, children, ..
            } => {
                let row = format!("r{}", loop_id.0);
                out.push_str(&format!(
                    "<block wx:for=\"{{{{{prefix}l{n}}}}}\" wx:for-item=\"{row}\" wx:key=\"k\">"
                ));
                self.nodes(children, &format!("{row}."), out);
                out.push_str("</block>");
            }
            NodeKind::Component {
                component, slots, ..
            } => {
                let callee = &self.program.component(*component).name;
                out.push_str(&format!(
                    "<template is=\"{callee}\" data=\"{{{{...{prefix}c{n}}}}}\"/>"
                ));
                for fill in slots {
                    let mut body = String::new();
                    self.nodes(&fill.children, "", &mut body);
                    self.slots
                        .push((slot_template(self.program, self.comp, id, fill.param), body));
                }
            }
            NodeKind::Slot { param } => {
                let y = format!("{prefix}{}", slot_key(self.comp, id, *param));
                out.push_str(&format!(
                    "<block wx:if=\"{{{{{y}}}}}\"><template is=\"{{{{{y}.t}}}}\" data=\"{{{{...{y}}}}}\"/></block>"
                ));
            }
        }
    }

    fn element(&mut self, id: NodeId, prefix: &str, out: &mut String) {
        let NodeKind::Element {
            element,
            props,
            children,
        } = &self.comp.node(id).kind
        else {
            unreachable!("caller checked the node kind");
        };
        let element = *element;
        let (tag, fixed, base_style) = tag(element);
        let mut attrs = String::from(fixed);
        let mut style: Vec<String> = Vec::new();
        if !base_style.is_empty() {
            style.push(base_style.to_owned());
        }
        let mut text = String::new();
        for (index, prop) in props.iter().enumerate() {
            let Some(value) = prop_value(self.layout, self.id, element, id, index, prop) else {
                continue;
            };
            let value = match value {
                Value::Inline(text) => text,
                Value::Data(key) => format!("{{{{{prefix}{key}}}}}"),
            };
            match target(element, prop.name) {
                Target::Text => text = value,
                Target::Attr(name) => attrs.push_str(&format!(" {name}=\"{value}\"")),
                Target::Flag(_) if value == "false" => {}
                Target::Flag(name) if value == "true" => {
                    attrs.push_str(&format!(" {name}=\"{{{{true}}}}\""));
                }
                Target::Flag(name) => attrs.push_str(&format!(" {name}=\"{value}\"")),
                Target::Style(name, _) => {
                    style.push(format!("{name}:{value}"));
                }
            }
        }
        let (template, local) = self.layout.location(self.id, id);
        for (k, (_, event)) in self.layout.template(template).events(local).enumerate() {
            let kind = event_type(element, event.event);
            let key = event_key(id, k);
            attrs.push_str(&format!(
                " bind{kind}=\"$e\" data-e{kind}=\"{{{{{prefix}{key}}}}}\""
            ));
        }
        out.push('<');
        out.push_str(tag);
        out.push_str(&attrs);
        if !style.is_empty() {
            out.push_str(" style=\"");
            out.push_str(&style.join(";"));
            out.push('"');
        }
        if is_void(element) {
            out.push_str("/>");
            return;
        }
        out.push('>');
        out.push_str(&text);
        self.nodes(children, prefix, out);
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
    }
}
