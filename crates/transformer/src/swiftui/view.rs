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

use std::fmt::{self, Display};

use crate::ir::Element;
use crate::view::{Constant, PointKind, PropSource, StaticProp, Template, ViewNodeKind};

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

    fn element(
        &self,
        out: &mut Code,
        node: u32,
        element: Element,
        props: &[StaticProp],
        children: &[u32],
    ) {
        let prop = |name| self.template.prop(node, props, name);
        let optional = |source| match source {
            Some(source) => Argument::Property(source),
            None => Argument::Default("nil"),
        };
        match element {
            Element::Column | Element::Row => {
                let gap = prop("gap").map_or(Argument::Default("0"), Argument::Property);
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
                let properties = ["padding", "width", "height", "background"].map(prop);
                if properties.iter().any(Option::is_some) {
                    let [padding, width, height, background] = properties.map(optional);
                    out.writer.line(format_args!(
                        ".weftBox(padding: {padding}, width: {width}, height: {height}, background: {background})"
                    ));
                }
            }
            Element::Text => {
                let value = Argument::Property(prop("value").expect("`Text` has a value"));
                out.writer.line(format_args!("Text(verbatim: {value})"));
                let properties = ["color", "size", "bold"].map(prop);
                if properties.iter().any(Option::is_some) {
                    let [color, size, bold] = properties.map(optional);
                    out.writer.line(format_args!(
                        ".weftText(color: {color}, size: {size}, bold: {bold})"
                    ));
                }
            }
            Element::Button => {
                match self.template.event(node) {
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
                let (event, slot) = self.template.controlled(node);
                out.writer.line(format_args!(
                    "WeftInput(text: s.v{slot}, placeholder: {}) {{ s.dispatch({event}, .string($0)) ? nil : s.v{slot} }}",
                    optional(prop("placeholder")),
                ));
            }
            Element::Checkbox => {
                let (event, slot) = self.template.controlled(node);
                out.writer.line(format_args!(
                    "WeftCheckbox(checked: s.v{slot}) {{ s.dispatch({event}, .bool($0)) ? nil : s.v{slot} }}"
                ));
            }
            Element::Image => {
                let src = Argument::Property(prop("src").expect("`Image` has a source"));
                out.writer.line(format_args!(
                    "WeftImage(src: {src}, width: {}, height: {})",
                    optional(prop("width")),
                    optional(prop("height")),
                ));
            }
        }
        if let Some(disabled) = prop("disabled")
            && !matches!(disabled, PropSource::Constant(Constant::Bool(false)))
        {
            let disabled = Argument::Property(disabled);
            out.writer.line(format_args!(".disabled({disabled})"));
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

enum Argument<'t> {
    Property(PropSource<'t>),
    Default(&'static str),
}

impl Display for Argument<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default(value) => f.write_str(value),
            Self::Property(PropSource::Slot(slot)) => write!(f, "s.v{slot}"),
            Self::Property(PropSource::Constant(value)) => match value {
                Constant::Int(v) => v.fmt(f),
                Constant::Bool(v) => v.fmt(f),
                Constant::String(s) => swift::string(f, s),
                Constant::Color(c) => write!(
                    f,
                    "Color(weftRGBA: 0x{:02x}{:02x}{:02x}{:02x})",
                    c.r, c.g, c.b, c.a
                ),
            },
        }
    }
}
