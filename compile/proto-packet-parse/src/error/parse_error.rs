use crate::ParseErrorReason;
use clerr::{Code, Report, Severity, TokenInfo};
use lex::lexer::Span;
use std::fmt::{Display, Formatter};
use std::ops::Range;

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
        let range: Range<usize> = self.span.offset() as usize..self.span.end() as usize;
        let info: TokenInfo =
            TokenInfo::from_source(Severity::Error, file_name, source, range, message);
        Report::from(Code::error(self.reason.code(), message)).with_entry(info)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason.message())
    }
}

impl std::error::Error for ParseError {}
