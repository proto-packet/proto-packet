use crate::{ValidateError, ValidateErrorReason};
use custom_string::ValidationError;
use lex::lexer::Span;
use proto_packet_tree::CaseName;

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

/// Validates the case name at the `span`.
///
/// No case can be named `Unrecognized`, since each language generates that case. The `message` is
/// the error for that name.
pub(crate) fn validate_case_name(
    span: Span,
    source: &str,
    message: &'static str,
) -> Result<CaseName, ValidateError> {
    let case_name: CaseName = validate_name(span, source)?;
    if case_name.value() == "Unrecognized" {
        return Err(ValidateError::new(
            span,
            ValidateErrorReason::InvalidName { message },
        ));
    }
    Ok(case_name)
}
