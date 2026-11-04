use crate::ValidateErrorReason;
use clerr::Report;
use lex::lexer::Span;
use proto_packet_parse::span_report;
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

impl ValidateError {
    //! Reports

    /// Creates the command-line report for the error in the `source` of the `file_name` file.
    pub fn report(&self, file_name: &str, source: &str) -> Report {
        let message: String = self.reason.to_string();
        span_report(self.reason.code(), &message, self.span, file_name, source)
    }
}

impl Display for ValidateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.reason, f)
    }
}

impl std::error::Error for ValidateError {}
