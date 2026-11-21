//! The SwiftUI view `V{n}` of a template maps built-in elements to WeftKit
//! layouts and native controls, and mount points to the views of child scopes.

use std::fmt::{self, Display};

use crate::elements::Element;
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
        let prop = |name: &str| self.template.prop(node, props, name);
        let optional = |source| with_default(source, "nil");
        let padding = Properties::new(PADDING, |name| prop(name).map(Argument::Property));
        let container_padding = fmt::from_fn(|f| {
            if padding.is_empty() {
                Ok(())
            } else {
                write!(f, ", padding: .init({padding})")
            }
        });
        let leaf = matches!(
            element,
            Element::Text | Element::Button | Element::Input | Element::Checkbox
        );
        if leaf {
            if padding.is_empty() {
                out.open("weftLeaf {");
            } else {
                out.open(format_args!("weftLeaf(padding: .init({padding})) {{"));
            }
        }
        match element {
            Element::Column | Element::Row | Element::Stack | Element::Scroll => {
                let gap = with_default(prop("gap"), "0");
                let align = Argument::Union(prop("align"), "CrossAlign", ".start");
                match element {
                    Element::Column | Element::Row => {
                        let axis = if element == Element::Column {
                            "vertical"
                        } else {
                            "horizontal"
                        };
                        let justify = Argument::Union(prop("justify"), "Justify", ".start");
                        out.open(format_args!(
                            "weftLinear(axis: .{axis}, gap: {gap}, align: {align}, justify: {justify}{container_padding}) {{"
                        ));
                    }
                    Element::Stack => {
                        let alignment =
                            Argument::Union(prop("alignment"), "Alignment", ".topStart");
                        out.open(format_args!(
                            "weftStack(alignment: {alignment}{container_padding}) {{"
                        ));
                    }
                    Element::Scroll => {
                        let direction = Argument::Union(prop("direction"), "Axis", ".vertical");
                        let indicators = with_default(prop("indicators"), "true");
                        out.open(format_args!(
                            "weftScroll(direction: {direction}, gap: {gap}, align: {align}, indicators: {indicators}{container_padding}) {{"
                        ));
                    }
                    _ => unreachable!("only container elements reach this branch"),
                }
                for &child in children {
                    self.node(out, child);
                }
                out.close("}");
            }
            Element::Spacer => match prop("size") {
                Some(size) => out.writer.line(format_args!(
                    "WeftSpacer(size: {})",
                    Argument::Property(size)
                )),
                None => out.writer.line("WeftSpacer()"),
            },
            Element::Span => unreachable!("platform support rejects `Span` before generation"),
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
                match self.template.event(node, "onClick") {
                    Some(slot) => {
                        out.open("Button {");
                        out.writer.line(format_args!("_ = s.dispatch({slot})"));
                        out.reopen("} label: {");
                    }
                    None => out.open("Button {} label: {"),
                }
                out.open("weftButtonLabel {");
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
                out.writer.line(format_args!("WeftImage(src: {src})"));
            }
        }
        if leaf {
            out.close("}");
        }
        let decoration = Properties::new(DECORATION, |name| {
            let source = prop(name)?;
            Some(match name {
                "shadow" => Argument::Union(Some(source), "Shadow", ".none"),
                _ => Argument::Property(source),
            })
        });
        if !decoration.is_empty() {
            out.writer
                .line(format_args!(".weftDecoration({decoration})"));
        }
        if let Some(opacity) = prop("opacity") {
            let opacity = Argument::Property(opacity);
            out.writer
                .line(format_args!(".weftOpacity(opacity: {opacity})"));
        }
        if let Some(hidden) = prop("hidden") {
            let hidden = Argument::Property(hidden);
            out.writer
                .line(format_args!(".weftVisibility(hidden: {hidden})"));
        }
        if let Some(disabled) = prop("disabled")
            && !matches!(disabled, PropSource::Constant(Constant::Bool(false)))
        {
            let disabled = Argument::Property(disabled);
            out.writer.line(format_args!(".disabled({disabled})"));
        }
        let frame = Properties::new(FRAME, |name| prop(name).map(Argument::Property));
        if !frame.is_empty() {
            out.writer.line(format_args!(".weftFrame({frame})"));
        }
        let child = Properties::new(CHILD, |name| {
            prop(name)
                .map(Argument::Property)
                .or(match (element, name) {
                    (Element::Scroll, "grow") => Some(Argument::Default("1")),
                    (Element::Scroll, "fill") => Some(Argument::Default("true")),
                    _ => None,
                })
        });
        if !child.is_empty() {
            out.writer.line(format_args!(".weftChild({child})"));
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

const PADDING: [&str; 7] = [
    "padding",
    "paddingX",
    "paddingY",
    "paddingTop",
    "paddingBottom",
    "paddingStart",
    "paddingEnd",
];
const DECORATION: [&str; 6] = [
    "background",
    "radius",
    "border",
    "borderColor",
    "shadow",
    "clip",
];
const FRAME: [&str; 7] = [
    "width",
    "height",
    "minWidth",
    "maxWidth",
    "minHeight",
    "maxHeight",
    "aspectRatio",
];
const CHILD: [&str; 9] = [
    "grow",
    "fill",
    "margin",
    "marginX",
    "marginY",
    "marginTop",
    "marginBottom",
    "marginStart",
    "marginEnd",
];

struct Properties<'t, const N: usize>([(&'static str, Option<Argument<'t>>); N]);

impl<'t, const N: usize> Properties<'t, N> {
    fn new(names: [&'static str; N], prop: impl Fn(&str) -> Option<Argument<'t>>) -> Self {
        Self(names.map(|name| (name, prop(name))))
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|(_, value)| value.is_none())
    }
}

impl<const N: usize> Display for Properties<'_, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut separator = "";
        for (name, argument) in &self.0 {
            if let Some(argument) = argument {
                write!(f, "{separator}{name}: {argument}")?;
                separator = ", ";
            }
        }
        Ok(())
    }
}

fn with_default<'t>(source: Option<PropSource<'t>>, default: &'static str) -> Argument<'t> {
    source.map_or(Argument::Default(default), Argument::Property)
}

enum Argument<'t> {
    Property(PropSource<'t>),
    Union(Option<PropSource<'t>>, &'static str, &'static str),
    Default(&'static str),
}

impl Display for Argument<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default(value) | Self::Union(None, _, value) => f.write_str(value),
            Self::Union(Some(PropSource::Slot(slot)), name, _) => {
                write!(f, "Weft{name}(weft: s.v{slot})")
            }
            Self::Union(Some(PropSource::Constant(Constant::String(case))), _, _) => {
                write!(f, ".{case}")
            }
            Self::Union(Some(PropSource::Constant(_)), name, _) => {
                unreachable!("{name} is a string union")
            }
            Self::Property(PropSource::Slot(slot)) => write!(f, "s.v{slot}"),
            Self::Property(PropSource::Constant(value)) => match value {
                Constant::Int(v) => v.fmt(f),
                Constant::Float(v) => write!(f, "{v:?}"),
                Constant::Bool(v) => v.fmt(f),
                Constant::String(s) => swift::string(f, s),
                Constant::Token(name) => {
                    f.write_str("WeftColor.token(")?;
                    swift::string(f, name)?;
                    f.write_str(")")
                }
                Constant::Color(c) => write!(f, "WeftColor.rgba({})", swift::rgba(*c)),
            },
        }
    }
}
