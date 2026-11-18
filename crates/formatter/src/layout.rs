use crate::{Indent, Options};

const TAB_WIDTH: usize = 4;

#[derive(Clone, Copy)]
pub(super) enum Line {
    Hard,
    Soft,
    Space,
}

impl Line {
    fn flat_text(self) -> Option<&'static str> {
        match self {
            Self::Hard => None,
            Self::Soft => Some(""),
            Self::Space => Some(" "),
        }
    }
}

enum Part<'src> {
    Text(&'src str, usize),
    Comment(&'src str),
    Line(Line, bool),
    IfFlat(&'static str),
    IfBroken(&'static str),
    Indent,
    Dedent,
    Group { width: Option<usize>, end: usize },
    End { following: usize },
}

pub(super) struct GroupStart {
    index: usize,
    width: usize,
    hard: usize,
}

#[derive(Default)]
pub(super) struct Doc<'src> {
    parts: Vec<Part<'src>>,
    width: usize,
    hard: usize,
}

impl<'src> Doc<'src> {
    pub(super) fn text(&mut self, text: &'src str) {
        let width = columns(text);
        self.width += width;
        self.parts.push(Part::Text(text, width));
    }

    pub(super) fn comment(&mut self, text: &'src str) {
        self.hard += 1;
        self.parts.push(Part::Comment(text));
    }

    pub(super) fn line(&mut self, line: Line, blank: bool) {
        if let Some(text) = line.flat_text() {
            self.width += text.len();
        } else {
            self.hard += 1;
        }
        self.parts.push(Part::Line(line, blank));
    }

    pub(super) fn if_flat(&mut self, text: &'static str) {
        self.width += text.len();
        self.parts.push(Part::IfFlat(text));
    }

    pub(super) fn if_broken(&mut self, text: &'static str) {
        self.parts.push(Part::IfBroken(text));
    }

    pub(super) fn indent(&mut self) {
        self.parts.push(Part::Indent);
    }

    pub(super) fn dedent(&mut self) {
        self.parts.push(Part::Dedent);
    }

    pub(super) fn begin_group(&mut self) -> GroupStart {
        let start = GroupStart {
            index: self.parts.len(),
            width: self.width,
            hard: self.hard,
        };
        self.parts.push(Part::Group {
            width: None,
            end: 0,
        });
        start
    }

    pub(super) fn end_group(&mut self, start: GroupStart) {
        self.parts[start.index] = Part::Group {
            width: (self.hard == start.hard).then(|| self.width - start.width),
            end: self.parts.len(),
        };
        self.parts.push(Part::End { following: 0 });
    }

    pub(super) fn render(mut self, options: &Options, capacity: usize) -> String {
        // A group must fit not just its own text but also what follows it on the
        // line. Measure that suffix with other, as-yet-undecided groups broken.
        let mut following = 0;
        for part in self.parts.iter_mut().rev() {
            match part {
                Part::Text(_, width) => following += *width,
                Part::Comment(text) => {
                    let (first, rest) = text.split_once('\n').unwrap_or((text, ""));
                    let width = columns(first.trim_end());
                    following = if rest.is_empty() {
                        following + width
                    } else {
                        width
                    };
                }
                Part::Line(..) => following = 0,
                Part::IfBroken(text) => following += text.len(),
                Part::End { following: suffix } => *suffix = following,
                _ => {}
            }
        }

        let mut out = Output {
            text: String::with_capacity(capacity),
            options,
            depth: 0,
            column: 0,
            newlines: 1,
        };
        // Once a group is flat, all groups inside it are flat as well.
        let mut flat_until = None;
        for (index, part) in self.parts.iter().enumerate() {
            let flat = flat_until.is_some();
            match part {
                Part::Text(text, width) => out.write(text, *width),
                Part::Comment(text) => out.comment(text),
                Part::Line(line, blank) => {
                    if let Some(text) = line.flat_text().filter(|_| flat) {
                        out.write(text, text.len());
                    } else {
                        out.newlines = out.newlines.max(1 + u8::from(*blank));
                    }
                }
                Part::IfFlat(text) if flat => out.write(text, text.len()),
                Part::IfBroken(text) if !flat => out.write(text, text.len()),
                Part::Indent => out.depth += 1,
                Part::Dedent => out.depth -= 1,
                Part::Group {
                    width: Some(width),
                    end,
                } if !flat => {
                    if let Part::End { following } = self.parts[*end]
                        && out.column() + width + following <= options.max_width
                    {
                        flat_until = Some(*end);
                    }
                }
                Part::End { .. } if flat_until == Some(index) => flat_until = None,
                _ => {}
            }
        }
        if !out.text.is_empty() {
            out.text.push('\n');
        }
        out.text
    }
}

struct Output<'options> {
    text: String,
    options: &'options Options,
    depth: usize,
    column: usize,
    newlines: u8,
}

impl Output<'_> {
    fn column(&self) -> usize {
        if self.newlines > 0 {
            self.depth
                * match self.options.indent {
                    Indent::Spaces(count) => usize::from(count),
                    Indent::Tab => TAB_WIDTH,
                }
        } else {
            self.column
        }
    }

    fn write(&mut self, text: &str, width: usize) {
        if text.is_empty() {
            return;
        }
        if self.newlines > 0 {
            if !self.text.is_empty() {
                self.text
                    .extend(std::iter::repeat_n('\n', usize::from(self.newlines)));
            }
            self.column = self.column();
            let (indent, count) = match self.options.indent {
                Indent::Spaces(_) => (' ', self.column),
                Indent::Tab => ('\t', self.depth),
            };
            self.text.extend(std::iter::repeat_n(indent, count));
            self.newlines = 0;
        }
        self.text.push_str(text);
        self.column += width;
    }

    fn comment(&mut self, text: &str) {
        let mut lines = text.lines().map(str::trim_end);
        if let Some(first) = lines.next() {
            self.write(first, columns(first));
        }
        // Interior lines are part of the comment token. Don't reindent them.
        for line in lines {
            self.text.push('\n');
            self.text.push_str(line);
            self.column = columns(line);
        }
    }
}

fn columns(text: &str) -> usize {
    if text.is_ascii() && !text.as_bytes().contains(&b'\t') {
        return text.len();
    }
    text.chars()
        .map(|ch| if ch == '\t' { TAB_WIDTH } else { 1 })
        .sum()
}
