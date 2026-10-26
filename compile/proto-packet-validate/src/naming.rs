use crate::{ValidateError, ValidateErrorReason};
use custom_string::ValidationError;
use lex::lexer::Span;

/// Validates the name at the `span` of the `source`.
pub(crate) fn validate_name<N>(span: Span, source: &str) -> Result<N, ValidateError>
where
    for<'a> N: TryFrom<&'a str, Error = ValidationError>,
{
    validate_name_text(span.text(source), span)
}

/// Validates the name `text` at the `span`.
pub(crate) fn validate_name_text<N>(text: &str, span: Span) -> Result<N, ValidateError>
where
    for<'a> N: TryFrom<&'a str, Error = ValidationError>,
{
    N::try_from(text).map_err(|error| {
        let message: &'static str = error.message();
        ValidateError::new(span, ValidateErrorReason::InvalidName { message })
    })
}
