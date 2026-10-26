use crate::{ValidateError, ValidateErrorReason};
use custom_string::ValidationError;
use lex::lexer::Span;

/// Validates the name `text` at the `span`.
pub(crate) fn validate_name<N>(text: &str, span: Span) -> Result<N, ValidateError>
where
    for<'a> N: TryFrom<&'a str, Error = ValidationError>,
{
    N::try_from(text).map_err(|error| {
        let message: &'static str = error.message();
        ValidateError::new(span, ValidateErrorReason::InvalidName { message })
    })
}
