//! Conversion between byte offsets and LSP positions (UTF-16 code units).

use lsp_types::{Position, Range};
use weft_ui_ast::Span;

pub(crate) struct LineIndex<'a> {
    text: &'a str,
    /// Byte offset where each line starts.
    line_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        let line_starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        Self { text, line_starts }
    }

    pub(crate) fn position(&self, offset: u32) -> Position {
        let offset = (offset as usize).min(self.text.len());
        let line = self.line_starts.partition_point(|&s| s <= offset) - 1;
        let start = self.line_starts[line];
        let character = self.text[start..offset]
            .chars()
            .map(char::len_utf16)
            .sum::<usize>();
        Position::new(to_u32(line), to_u32(character))
    }

    pub(crate) fn range(&self, span: Span) -> Range {
        Range::new(self.position(span.start), self.position(span.end))
    }

    /// Byte offset of `position`, clamped to the line and the text.
    pub(crate) fn offset(&self, position: Position) -> u32 {
        let Some(&start) = self.line_starts.get(position.line as usize) else {
            return to_u32(self.text.len());
        };
        let line = &self.text[start..];
        let line = &line[..line.find('\n').unwrap_or(line.len())];
        let mut units = 0;
        for (i, c) in line.char_indices() {
            if units >= position.character as usize {
                return to_u32(start + i);
            }
            units += c.len_utf16();
        }
        to_u32(start + line.trim_end_matches('\r').len())
    }

    /// Range covering the whole text.
    pub(crate) fn full_range(&self) -> Range {
        Range::new(Position::new(0, 0), self.position(to_u32(self.text.len())))
    }
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_utf16_units() {
        let index = LineIndex::new("ab\n\u{1F600}x\r\ny");
        let x = "ab\n\u{1F600}".len() as u32;
        assert_eq!(index.position(x), Position::new(1, 2));
        assert_eq!(index.offset(Position::new(1, 2)), x);
        assert_eq!(index.offset(Position::new(1, 99)), x + 1);
        assert_eq!(index.offset(Position::new(9, 0)), 11);
        assert_eq!(index.position(11), Position::new(2, 1));
    }
}
