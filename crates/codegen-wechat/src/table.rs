//! The WeChat sink's table embedded in `view.js`: for every layout template, the
//! data keys of its value slots, event slots and mount points.
//!
//! Entry fields, all optional:
//!
//! | Field | Value                                                          |
//! | ----- | -------------------------------------------------------------- |
//! | `n`   | WXML template rendering a component body as the view's root     |
//! | `t`   | WXML template of slot content, stored in the scope's data as `t` |
//! | `i`   | branch index of an `if` branch template                        |
//! | `v`   | `[key, unit]` per value slot; unit `1` appends `px`, `2` maps a flag to `font-weight`, `3` validates a color |
//! | `e`   | `[key, controlled value slot or -1]` per event slot; the key holds the handler id |
//! | `p`   | key per mount point; `[indexKey, dataKey]` for branches         |
//! | `d`   | constants that cannot be inlined into WXML, by key             |

use weft_ui_codegen_js::js::{self, Code};
use weft_ui_compiler::ir::{NodeKind, Program, Prop};
use weft_ui_compiler::view::{PointKind, Template, TemplateKind, ViewLayout, ViewNodeKind};

use crate::elements::{Target, inline, target, value_key};
use crate::wxml::slot_template;

/// The table object literal passed to the view runtime.
pub(crate) fn object(program: &Program, layout: &ViewLayout) -> String {
    let mut out = Code::default();
    out.open("{");
    out.line(format!("hash: \"{:016x}\",", layout.hash()));
    out.open("templates: [");
    for template in layout.templates() {
        out.line(format!("{},", entry(program, template)));
    }
    out.close("],");
    out.close("}");
    let mut text = String::new();
    out.write_to(&mut text);
    text
}

fn entry(program: &Program, template: &Template) -> String {
    let comp = program.component(template.component);
    let props_of = |local: u32| -> (&[Prop], _) {
        let source = template.nodes[local as usize].source;
        match &comp.node(source).kind {
            NodeKind::Element { element, props, .. } => (props.as_slice(), (*element, source)),
            _ => unreachable!("slots and events belong to elements"),
        }
    };
    let index_of = |props: &[Prop], name: &str| {
        props
            .iter()
            .position(|p| p.name == name)
            .expect("layout properties exist in the IR")
    };

    let mut fields = Vec::new();
    match template.kind {
        TemplateKind::Component => fields.push(format!("n: {}", js::string(&comp.name))),
        TemplateKind::SlotContent { node } => {
            fields.push(format!("t: {}", js::string(&slot_template(comp, node))));
        }
        TemplateKind::Branch { index, .. } => fields.push(format!("i: {index}")),
        TemplateKind::Row { .. } => {}
    }
    if !template.values.is_empty() {
        let values: Vec<String> = template
            .values
            .iter()
            .map(|slot| {
                let (props, (element, source)) = props_of(slot.node);
                let key = value_key(source, index_of(props, slot.prop));
                let unit = match target(element, slot.prop) {
                    Target::Style(_, unit) => unit.code(),
                    _ => 0,
                };
                format!("[\"{key}\", {unit}]")
            })
            .collect();
        fields.push(format!("v: [{}]", values.join(", ")));
    }
    if !template.events.is_empty() {
        let events: Vec<String> = template
            .events
            .iter()
            .map(|slot| {
                let n = template.nodes[slot.node as usize].source.0;
                let controls = slot.controls.map_or(-1, i64::from);
                format!("[\"e{n}\", {controls}]")
            })
            .collect();
        fields.push(format!("e: [{}]", events.join(", ")));
    }
    if !template.points.is_empty() {
        let points: Vec<String> = template
            .points
            .iter()
            .map(|point| {
                let n = template.nodes[point.node as usize].source.0;
                match point.kind {
                    PointKind::Branch { .. } => format!("[\"i{n}\", \"b{n}\"]"),
                    PointKind::List { .. } => format!("\"l{n}\""),
                    PointKind::Instance { .. } => format!("\"c{n}\""),
                    PointKind::Slot => format!("\"y{n}\""),
                }
            })
            .collect();
        fields.push(format!("p: [{}]", points.join(", ")));
    }
    let mut data = Vec::new();
    for (local, node) in template.nodes.iter().enumerate() {
        let ViewNodeKind::Element { element, props, .. } = &node.kind else {
            continue;
        };
        let local = u32::try_from(local).expect("node indices fit in u32");
        for prop in props {
            if inline(target(*element, prop.name), &prop.value).is_some() {
                continue;
            }
            let (ir_props, (_, source)) = props_of(local);
            let key = value_key(source, index_of(ir_props, prop.name));
            data.push(format!("{key}: {}", prop.value));
        }
    }
    if !data.is_empty() {
        fields.push(format!("d: {{ {} }}", data.join(", ")));
    }
    format!("{{ {} }}", fields.join(", "))
}
