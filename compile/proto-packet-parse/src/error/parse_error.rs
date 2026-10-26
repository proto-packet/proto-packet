use crate::ParseErrorReason;
use clerr::{Code, Report, Severity, TokenInfo};
use lex::lexer::Span;
use std::fmt::{Display, Formatter};

/// A parse error.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ParseError {
    span: Span,
    reason: ParseErrorReason,
}

impl ParseError {
    //! Construction

    /// Creates a new parse error.
    pub const fn new(span: Span, reason: ParseErrorReason) -> Self {
        Self { span, reason }
    }
}

impl ParseError {
    //! Properties

    /// Gets the span.
    pub const fn span(self) -> Span {
        self.span
    }

    /// Gets the reason.
    pub const fn reason(self) -> ParseErrorReason {
        self.reason
    }
}

impl ParseError {
    //! Reports

    /// Creates the command-line report for the error in the `source` of the `file_name` file.
    pub fn report(self, file_name: &str, source: &str) -> Report {
        let message: &'static str = self.reason.message();
        let offset: usize = self.span.offset() as usize;
        let line_start: usize = source[..offset].rfind('\n').map_or(0, |i| i + 1);
        let line_text: &str = source[line_start..].lines().next().unwrap_or("");
        let position: usize = source[line_start..offset].chars().count();
        let token_len: usize = self
            .span
            .char_count(source)
            .min(line_text.chars().count().saturating_sub(position));
        Report::from(Code::error(self.reason.code(), message)).with_entry(TokenInfo {
            severity: Severity::Error,
            file_name,
            line: source[..offset].matches('\n').count() + 1,
            position: position + 1,
            line_text,
            token_len,
            message,
        })
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason.message())
    }
}

impl std::error::Error for ParseError {}
