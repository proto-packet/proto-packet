use crate::ValidateErrorReason;
use clerr::{Code, Report, Severity, TokenInfo};
use lex::lexer::Span;
use std::fmt::{Display, Formatter};
use std::ops::Range;

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
        let range: Range<usize> = self.span.offset() as usize..self.span.end() as usize;
        let info: TokenInfo =
            TokenInfo::from_source(Severity::Error, file_name, source, range, &message);
        Report::from(Code::error(self.reason.code(), message.as_str())).with_entry(info)
    }
}

impl Display for ValidateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.reason, f)
    }
}

impl std::error::Error for ValidateError {}
