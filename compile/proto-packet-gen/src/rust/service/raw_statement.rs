use code_gen::{CodeBuffer, Statement};

/// A raw multi-line text block written verbatim into the code buffer. Each non-empty input line
/// is indented to the requested `level`; blank input lines become bare newlines.
pub(super) struct RawStatement(pub String);

impl Statement for RawStatement {
    fn write(&self, b: &mut CodeBuffer, level: usize) {
        for line in self.0.lines() {
            if line.is_empty() {
                b.end_line();
            } else {
                b.indent(level);
                b.write(line);
                b.end_line();
            }
        }
    }
}
