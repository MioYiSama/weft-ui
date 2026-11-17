//! WXML templates of the mini program backend.
//!
//! Every component becomes a `<template>` that renders the data object of one
//! instance (keys are listed in [`weft_ui_codegen_js::remote`]); nested
//! blocks are reached through their key (`b3.n7_0`, `r0.n9_0`). Slot content
//! is rendered by a template of the calling component, chosen at run time
//! through `t`, so all templates of the app live in one file.

use weft_ui_codegen_js::remote::{
    Target, Value, controlled_event, event_attr, handler_prop, prop_value, slot_template, target,
};
use weft_ui_compiler::ir::{Component, Element, NodeId, NodeKind, Program};

/// Templates for every component of the program.
pub(crate) fn templates(program: &Program) -> String {
    let mut out = String::new();
    for comp in &program.components {
        let mut markup = Markup {
            program,
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
        Element::Column => ("view", "", "display:flex;flex-direction:column"),
        Element::Row => ("view", "", "display:flex;flex-direction:row"),
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
                component,
                children,
                ..
            } => {
                let callee = &self.program.component(*component).name;
                out.push_str(&format!(
                    "<template is=\"{callee}\" data=\"{{{{...{prefix}c{n}}}}}\"/>"
                ));
                if !children.is_empty() {
                    let mut body = String::new();
                    self.nodes(children, "", &mut body);
                    self.slots.push((slot_template(self.comp, id), body));
                }
            }
            NodeKind::Slot { .. } => {
                let y = format!("{prefix}y{n}");
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
            let Some(value) = prop_value(element, id, index, prop) else {
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
        let event = handler_prop(props)
            .map(|prop| event_attr(element, prop.name))
            .or_else(|| controlled_event(element));
        if let Some(event) = event {
            attrs.push_str(&format!(
                " {event}=\"$e\" data-h=\"{{{{{prefix}e{}}}}}\"",
                id.0
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
