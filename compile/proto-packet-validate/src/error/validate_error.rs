use crate::ValidateErrorReason;
use lex::lexer::Span;
use std::fmt::{Display, Formatter};

/// A validation error.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ValidateError {
    span: Span,
    reason: ValidateErrorReason,
}

impl ValidateError {
    //! Construction

    /// Creates a new validation error.
    pub const fn new(span: Span, reason: ValidateErrorReason) -> Self {
        Self { span, reason }
    }
}

impl ValidateError {
    //! Properties

    /// Gets the span.
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Gets the reason.
    pub const fn reason(&self) -> &ValidateErrorReason {
        &self.reason
    }
}

impl Display for ValidateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.reason, f)
    }
}

impl std::error::Error for ValidateError {}
