use crate::error::TreeErrorHelper;
use clerr::{Code, Report, Severity, TokenInfo};
use lex::lexer::Span;
use proto_packet_parse::SchemaFileTree;
use proto_packet_tree::TreeError;
use std::fmt::{Display, Formatter};

/// An error validating a parsed schema tree.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValidateError {
    /// An error validating a name.
    InvalidName { span: Span, message: &'static str },

    /// An error validating a tag number.
    InvalidTagNumber { span: Span, message: &'static str },

    /// An error creating a semantic tree.
    Tree { error: TreeError },
}

impl ValidateError {
    //! Report

    /// Gets the command-line report.
    pub fn report(&self, file_name: &str, tree: &SchemaFileTree, source: &str) -> Report {
        match self {
            Self::InvalidName { span, message } => {
                Report::from(Code::error("V_INVALID_NAME", "invalid name"))
                    .with_entry(Self::token_info(*span, file_name, source, message))
            }
            Self::InvalidTagNumber { span, message } => {
                Report::from(Code::error("V_INVALID_TAG_NUMBER", "invalid tag number"))
                    .with_entry(Self::token_info(*span, file_name, source, message))
            }
            Self::Tree { error } => TreeErrorHelper::new(file_name, error, tree, source).report(),
        }
    }

    /// Builds a [TokenInfo] from `span`, `file_name`, `source`, and `message`.
    fn token_info<'a>(
        span: Span,
        file_name: &'a str,
        source: &'a str,
        message: &'a str,
    ) -> TokenInfo<'a> {
        let (line, column): (usize, usize) = span.line_column(source);
        let line_text: &str = source.lines().nth(line).unwrap_or("");
        TokenInfo {
            severity: Severity::Error,
            file_name,
            line: line + 1,
            position: column + 1,
            line_text,
            token_len: span.len() as usize,
            message,
        }
    }
}

impl Display for ValidateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for ValidateError {}
