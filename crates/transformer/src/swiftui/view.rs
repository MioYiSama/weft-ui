//! The SwiftUI view `V{n}` of a template: built-in elements map to SwiftUI
//! containers and WeftKit controls, mount points to the views of the child
//! scopes.
//!
//! | Element    | SwiftUI                                         |
//! | ---------- | ----------------------------------------------- |
//! | `Column`   | `VStack(alignment: .leading)` + `weftBox`       |
//! | `Row`      | `HStack(alignment: .top)` + `weftBox`           |
//! | `Text`     | `Text(verbatim:)` + `weftText`                  |
//! | `Button`   | `Button`                                        |
//! | `Input`    | `WeftInput` (controlled `NSTextField` / `UITextField`) |
//! | `Checkbox` | `WeftCheckbox` (`NSButton` checkbox / `UISwitch`) |
//! | `Image`    | `WeftImage` (`AsyncImage`)                      |
//!
//! Stack spacing defaults to 0 and children align to the start of the cross
//! axis, as the layout rules of GRAMMAR.md require.

use crate::ir::Element;
use crate::view::{Constant, PointKind, StaticProp, Template, ViewNodeKind};

use super::swift::{self, Code};

/// Writes the view struct `V{index}` of `template`.
pub(crate) fn view(out: &mut Code, index: usize, template: &Template) {
    out.open(format_args!("private struct V{index}: View {{"));
    out.writer.line(format_args!("let s: S{index}"));
    out.writer.blank();
    out.open("var body: some View {");
    let body = Body { template };
    for &root in &template.roots {
        body.node(out, root);
    }
    out.close("}");
    out.close("}");
}

struct Body<'t> {
    template: &'t Template,
}

impl Body<'_> {
    fn node(&self, out: &mut Code, node: u32) {
        match &self.template.nodes[node as usize].kind {
            ViewNodeKind::Element {
                element,
                props,
                children,
            } => self.element(out, node, *element, props, children),
            ViewNodeKind::Point(point) => self.point(out, *point),
        }
    }

    /// Code of property `name` of element `node`: its value slot, its
    /// constant, or `None` when it is not set.
    fn prop(&self, node: u32, props: &[StaticProp], name: &str) -> Option<String> {
        if let Some(slot) = self
            .template
            .values
            .iter()
            .position(|v| v.node == node && v.prop == name)
        {
            return Some(format!("s.v{slot}"));
        }
        props
            .iter()
            .find(|p| p.name == name)
            .map(|p| literal(&p.value))
    }

    /// The event slot of element `node` and the value slot it controls.
    fn event(&self, node: u32) -> Option<(usize, Option<u32>)> {
        self.template
            .events
            .iter()
            .enumerate()
            .find(|(_, e)| e.node == node)
            .map(|(slot, e)| (slot, e.controls))
    }

    fn element(
        &self,
        out: &mut Code,
        node: u32,
        element: Element,
        props: &[StaticProp],
        children: &[u32],
    ) {
        let prop = |name| self.prop(node, props, name);
        let optional = |name| prop(name).unwrap_or_else(|| "nil".to_owned());
        match element {
            Element::Column | Element::Row => {
                let gap = prop("gap").unwrap_or_else(|| "0".to_owned());
                let (stack, alignment) = match element {
                    Element::Column => ("VStack", "leading"),
                    _ => ("HStack", "top"),
                };
                out.open(format_args!(
                    "{stack}(alignment: .{alignment}, spacing: {gap}) {{"
                ));
                for &child in children {
                    self.node(out, child);
                }
                out.close("}");
                if ["padding", "width", "height", "background"]
                    .into_iter()
                    .any(|name| prop(name).is_some())
                {
                    out.writer.line(format_args!(
                        ".weftBox(padding: {}, width: {}, height: {}, background: {})",
                        optional("padding"),
                        optional("width"),
                        optional("height"),
                        optional("background"),
                    ));
                }
            }
            Element::Text => {
                let value = prop("value").expect("`Text` has a value");
                out.writer.line(format_args!("Text(verbatim: {value})"));
                if ["color", "size", "bold"]
                    .into_iter()
                    .any(|name| prop(name).is_some())
                {
                    out.writer.line(format_args!(
                        ".weftText(color: {}, size: {}, bold: {})",
                        optional("color"),
                        optional("size"),
                        optional("bold"),
                    ));
                }
            }
            Element::Button => {
                match self.event(node) {
                    Some((slot, _)) => {
                        out.open("Button {");
                        out.writer.line(format_args!("_ = s.dispatch({slot})"));
                        out.reopen("} label: {");
                    }
                    None => out.open("Button {} label: {"),
                }
                out.open("HStack(spacing: 0) {");
                for &child in children {
                    self.node(out, child);
                }
                out.close("}");
                out.close("}");
            }
            Element::Input => {
                let (event, slot) = self.controlled(node);
                out.writer.line(format_args!(
                    "WeftInput(text: s.v{slot}, placeholder: {}) {{ s.dispatch({event}, .string($0)) ? nil : s.v{slot} }}",
                    optional("placeholder"),
                ));
            }
            Element::Checkbox => {
                let (event, slot) = self.controlled(node);
                out.writer.line(format_args!(
                    "WeftCheckbox(checked: s.v{slot}) {{ s.dispatch({event}, .bool($0)) ? nil : s.v{slot} }}"
                ));
            }
            Element::Image => {
                let src = prop("src").expect("`Image` has a source");
                out.writer.line(format_args!(
                    "WeftImage(src: {src}, width: {}, height: {})",
                    optional("width"),
                    optional("height"),
                ));
            }
        }
        if let Some(disabled) = prop("disabled").filter(|d| d != "false") {
            out.writer.line(format_args!(".disabled({disabled})"));
        }
    }

    /// Event slot and controlled value slot of an `Input` or `Checkbox`.
    fn controlled(&self, node: u32) -> (usize, u32) {
        match self.event(node) {
            Some((event, Some(slot))) => (event, slot),
            _ => unreachable!("controlled elements have an edit event with a value slot"),
        }
    }

    fn point(&self, out: &mut Code, point: u32) {
        let property = format!("s.p{point}");
        match &self.template.points[point as usize].kind {
            PointKind::Branch { templates } => {
                out.writer.line(format_args!("switch {property} {{"));
                for template in templates {
                    let t = template.0;
                    out.writer
                        .line(format_args!("case let c as S{t}: V{t}(s: c).id(c.id)"));
                }
                out.writer.line("default: EmptyView()");
                out.writer.line("}");
            }
            PointKind::List { template } => {
                out.writer.line(format_args!(
                    "ForEach({property}) {{ c in V{}(s: c) }}",
                    template.0
                ));
            }
            PointKind::Instance { template } => {
                out.writer.line(format_args!(
                    "if let c = {property} {{ V{}(s: c) }}",
                    template.0
                ));
            }
            PointKind::Slot => {
                out.writer
                    .line(format_args!("if let c = {property} {{ c.view.id(c.id) }}"));
            }
        }
    }
}

/// Swift code of a constant property value.
fn literal(value: &Constant) -> String {
    match value {
        Constant::Int(v) => v.to_string(),
        Constant::Bool(v) => v.to_string(),
        Constant::String(s) => swift::string(s),
        Constant::Color(c) => format!(
            "Color(weftRGBA: 0x{:02x}{:02x}{:02x}{:02x})",
            c.r, c.g, c.b, c.a
        ),
    }
}
