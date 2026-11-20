//! The Compose view `V{n}` of a template: built-in elements call the shell's
//! composables, and mount points render child scopes.
//!
//! | Element    | Shell composable | Compose                                   |
//! | ---------- | ---------------- | ----------------------------------------- |
//! | `Column`   | `WeftColumn`     | `Column(horizontalAlignment = Start)`     |
//! | `Row`      | `WeftRow`        | `Row(verticalAlignment = Top)`            |
//! | `Text`     | `WeftText`       | Material 3 `Text`                         |
//! | `Button`   | `WeftButton`     | Material 3 `Button`                       |
//! | `Input`    | `WeftInput`      | controlled Material 3 `OutlinedTextField` |
//! | `Checkbox` | `WeftCheckbox`   | Material 3 `Checkbox`                     |
//! | `Image`    | `WeftImage`      | `Image` of a bitmap the shell downloads   |
//!
//! Stack spacing defaults to 0 and children align to the start of the cross
//! axis, as the layout rules of GRAMMAR.md require.

use std::fmt::{self, Display, Write as _};

use crate::elements::Element;
use crate::view::{Constant, PointKind, PropSource, StaticProp, Template, ViewNodeKind};

use super::kotlin::{self, Code};

/// Writes the composable `V{index}` of `template`.
pub(crate) fn view(out: &mut Code, index: usize, template: &Template) {
    out.writer.line("@Composable");
    out.open(format_args!("private fun V{index}(s: S{index}) {{"));
    let body = Body { template };
    for &root in &template.roots {
        body.node(out, root);
    }
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
        let optional = |mut args: CallArgs<'_>, names: &[&str]| {
            for &name in names {
                let Some(value) = prop(name) else {
                    continue;
                };
                if name == "disabled"
                    && matches!(value, PropSource::Constant(Constant::Bool(false)))
                {
                    continue;
                }
                let value = KotlinProp(value);
                args.push(format_args!("{name} = {value}"));
            }
            args.finish();
        };
        let line = out.writer.start_line();
        line.push_str("Weft");
        line.push_str(element.name());
        let mut args = CallArgs::new(line);
        match element {
            Element::Column | Element::Row => {
                optional(args, &["gap", "padding", "width", "height", "background"]);
            }
            Element::Button => {
                match self.template.event(node, "onClick") {
                    Some(slot) => {
                        args.push(format_args!("onClick = {{ s.dispatch({slot}) }}"));
                    }
                    None => args.push("onClick = {}"),
                }
                optional(args, &["disabled"]);
            }
            Element::Text => {
                args.push(KotlinProp(prop("value").expect("`Text` has a value")));
                optional(args, &["color", "size", "bold"]);
                out.writer.end_line("");
                return;
            }
            Element::Input => {
                let (event, slot) = self.template.controlled(node);
                args.push(format_args!("s.v{slot}"));
                optional(args, &["placeholder", "disabled"]);
                out.writer.end_line(format_args!(
                    " {{ if (s.dispatch({event}, WeftValue.Text(it))) null else s.v{slot} }}"
                ));
                return;
            }
            Element::Checkbox => {
                let (event, slot) = self.template.controlled(node);
                args.push(format_args!("s.v{slot}"));
                optional(args, &["disabled"]);
                out.writer.end_line(format_args!(
                    " {{ s.dispatch({event}, WeftValue.Bool(it)) }}"
                ));
                return;
            }
            Element::Image => {
                args.push(KotlinProp(prop("src").expect("`Image` has a source")));
                optional(args, &["width", "height"]);
                out.writer.end_line("");
                return;
            }
        }
        out.open_after(" {");
        for &child in children {
            self.node(out, child);
        }
        out.close("}");
    }

    fn point(&self, out: &mut Code, point: u32) {
        match &self.template.points[point as usize].kind {
            PointKind::Branch { templates } => {
                out.open(format_args!("when (val c = s.p{point}) {{"));
                for template in templates {
                    let t = template.0;
                    out.writer
                        .line(format_args!("is S{t} -> key(c.id) {{ V{t}(c) }}"));
                }
                out.close("}");
            }
            PointKind::List { template } => out.writer.line(format_args!(
                "for (c in s.p{point}) key(c.id) {{ V{}(c) }}",
                template.0
            )),
            PointKind::Instance { template } => out
                .writer
                .line(format_args!("s.p{point}?.let {{ V{}(it) }}", template.0)),
            PointKind::Slot => out.writer.line(format_args!(
                "s.p{point}?.let {{ key(it.id) {{ it.Content() }} }}"
            )),
        }
    }
}

/// Writes the argument list directly into the output, omitting empty parentheses.
struct CallArgs<'a> {
    line: &'a mut String,
    started: bool,
}

impl<'a> CallArgs<'a> {
    fn new(line: &'a mut String) -> Self {
        Self {
            line,
            started: false,
        }
    }

    fn push(&mut self, argument: impl Display) {
        if self.started {
            self.line.push_str(", ");
        } else {
            self.line.push('(');
            self.started = true;
        }
        write!(self.line, "{argument}").expect("writing to a String cannot fail");
    }

    fn finish(self) {
        if self.started {
            self.line.push(')');
        }
    }
}

struct KotlinProp<'t>(PropSource<'t>);

impl Display for KotlinProp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            PropSource::Slot(slot) => write!(f, "s.v{slot}"),
            PropSource::Constant(value) => match value {
                Constant::Int(v) => write!(f, "{v}f"),
                Constant::Bool(v) => v.fmt(f),
                Constant::String(s) => kotlin::string(f, s),
                Constant::Token(name) => {
                    f.write_str("WeftColor.token(")?;
                    kotlin::string(f, name)?;
                    f.write_char(')')
                }
                Constant::Color(c) => write!(f, "WeftColor.Fixed({})", kotlin::color(*c)),
            },
        }
    }
}
