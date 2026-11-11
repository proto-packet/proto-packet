use lex::lexer::Span;
use lsp_types::{Position, Range};

/// The starts of the lines of a source, which give the position of an offset.
pub(crate) struct LineIndex<'a> {
    source: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    //! Construction

    /// Creates the [LineIndex].
    pub(crate) fn new(source: &'a str) -> Self {
        let mut line_starts: Vec<usize> = vec![0];
        line_starts.extend(
            source
                .bytes()
                .enumerate()
                .filter(|(_, byte)| *byte == b'\n')
                .map(|(offset, _)| offset + 1),
        );
        Self {
            source,
            line_starts,
        }
    }
}

impl LineIndex<'_> {
    //! Positions

    /// Gets the range of the `span`.
    pub(crate) fn range(&self, span: Span) -> Range {
        Range::new(
            self.position(span.offset() as usize),
            self.position(span.end() as usize),
        )
    }

    /// Gets the position of the byte `offset`.
    ///
    /// # Note
    /// - The character of a position counts UTF-16 code units, as the protocol does by default.
    /// - An offset past the end or inside a character is moved back to the nearest character.
    fn position(&self, offset: usize) -> Position {
        let mut offset: usize = offset.min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset -= 1;
        }
        let line: usize = self.line_starts.partition_point(|start| *start <= offset) - 1;
        let character: usize = self.source[self.line_starts[line]..offset]
            .encode_utf16()
            .count();
        Position::new(line as u32, character as u32)
    }
}
