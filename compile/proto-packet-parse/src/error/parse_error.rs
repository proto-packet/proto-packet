use crate::ParseErrorReason;
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

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason.message())
    }
}

impl std::error::Error for ParseError {}
