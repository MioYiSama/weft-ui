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

use weft_ui_compiler::ir::Element;
use weft_ui_compiler::view::{Constant, PointKind, StaticProp, Template, ViewNodeKind};

use crate::kotlin::{self, Code};

/// Writes the composable `V{index}` of `template`.
pub(crate) fn view(out: &mut Code, index: usize, template: &Template) {
    out.line("@Composable");
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

    /// A property is either read from its value slot or inlined as a constant.
    fn prop<'a>(&self, node: u32, props: &'a [StaticProp], name: &str) -> Option<Prop<'a>> {
        if let Some(slot) = self
            .template
            .values
            .iter()
            .position(|v| v.node == node && v.prop == name)
        {
            return Some(Prop::Slot(slot));
        }
        props
            .iter()
            .find(|p| p.name == name)
            .map(|p| Prop::Constant(&p.value))
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
        let prop = |name: &str| self.prop(node, props, name);
        let optional = |mut args: CallArgs<'_>, names: &[&str]| {
            for &name in names {
                let Some(value) = prop(name) else {
                    continue;
                };
                if name == "disabled" && matches!(value, Prop::Constant(Constant::Bool(false))) {
                    continue;
                }
                args.push(format_args!("{name} = {value}"));
            }
            args.finish();
        };
        let line = out.start_line();
        line.push_str("Weft");
        line.push_str(element.name());
        let mut args = CallArgs::new(line);
        match element {
            Element::Column | Element::Row => {
                optional(args, &["gap", "padding", "width", "height", "background"]);
            }
            Element::Button => {
                match self.event(node) {
                    Some((slot, _)) => {
                        args.push(format_args!("onClick = {{ s.dispatch({slot}) }}"));
                    }
                    None => args.push("onClick = {}"),
                }
                optional(args, &["disabled"]);
            }
            Element::Text => {
                args.push(prop("value").expect("`Text` has a value"));
                optional(args, &["color", "size", "bold"]);
                out.end_line("");
                return;
            }
            Element::Input => {
                let (event, slot) = self.controlled(node);
                args.push(format_args!("s.v{slot}"));
                optional(args, &["placeholder", "disabled"]);
                out.end_line(format_args!(
                    " {{ if (s.dispatch({event}, WeftValue.Text(it))) null else s.v{slot} }}"
                ));
                return;
            }
            Element::Checkbox => {
                let (event, slot) = self.controlled(node);
                args.push(format_args!("s.v{slot}"));
                optional(args, &["disabled"]);
                out.end_line(format_args!(
                    " {{ s.dispatch({event}, WeftValue.Bool(it)) }}"
                ));
                return;
            }
            Element::Image => {
                args.push(prop("src").expect("`Image` has a source"));
                optional(args, &["width", "height"]);
                out.end_line("");
                return;
            }
        }
        out.open_after(" {");
        for &child in children {
            self.node(out, child);
        }
        out.close("}");
    }

    /// Event slot and controlled value slot of an `Input` or `Checkbox`.
    fn controlled(&self, node: u32) -> (usize, u32) {
        match self.event(node) {
            Some((event, Some(slot))) => (event, slot),
            _ => unreachable!("controlled elements have an edit event with a value slot"),
        }
    }

    fn point(&self, out: &mut Code, point: u32) {
        match &self.template.points[point as usize].kind {
            PointKind::Branch { templates } => {
                out.open(format_args!("when (val c = s.p{point}) {{"));
                for template in templates {
                    let t = template.0;
                    out.line(format_args!("is S{t} -> key(c.id) {{ V{t}(c) }}"));
                }
                out.close("}");
            }
            PointKind::List { template } => out.line(format_args!(
                "for (c in s.p{point}) key(c.id) {{ V{}(c) }}",
                template.0
            )),
            PointKind::Instance { template } => {
                out.line(format_args!("s.p{point}?.let {{ V{}(it) }}", template.0))
            }
            PointKind::Slot => out.line(format_args!(
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

enum Prop<'a> {
    Slot(usize),
    Constant(&'a Constant),
}

impl Display for Prop<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Prop::Slot(slot) => write!(f, "s.v{slot}"),
            Prop::Constant(value) => match value {
                Constant::Int(v) => write!(f, "{v}f"),
                Constant::Bool(v) => v.fmt(f),
                Constant::String(s) => kotlin::string(f, s),
                Constant::Color(c) => {
                    write!(f, "Color(0x{:02x}{:02x}{:02x}{:02x}L)", c.a, c.r, c.g, c.b)
                }
            },
        }
    }
}
