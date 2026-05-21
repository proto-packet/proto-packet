use crate::ParseErrorReason;
use clerr::{Code, Report, Severity, TokenInfo};
use lex::lexer::Span;
use std::fmt::{Display, Formatter};

/// A parsing error.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ParseError {
    pub span: Span,
    pub reason: ParseErrorReason,
}

impl ParseError {
    //! Construction

    /// Creates a new [ParseError].
    #[must_use]
    pub const fn new(span: Span, reason: ParseErrorReason) -> Self {
        Self { span, reason }
    }
}

impl ParseError {
    //! Report

    /// Gets the command-line report.
    #[must_use]
    pub fn report(&self, file_name: &str, source: &str) -> Report {
        let message: &'static str = self.reason.message();
        Report::from(Code::error(format!("P_{}", self.reason.code()), message))
            .with_entry(self.token_info(file_name, source, message))
    }

    /// Builds the [TokenInfo] for this error's span.
    fn token_info<'a>(
        &self,
        file_name: &'a str,
        source: &'a str,
        message: &'a str,
    ) -> TokenInfo<'a> {
        // todo -- there should be a better way to construct the token info
        let (line, column): (usize, usize) = self.span.line_column(source);
        let line_text: &str = source.lines().nth(line).unwrap_or("");
        TokenInfo {
            severity: Severity::Error,
            file_name,
            line: line + 1,
            position: column + 1,
            line_text,
            token_len: self.span.len() as usize,
            message,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for ParseError {}
