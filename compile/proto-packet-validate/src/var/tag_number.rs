use crate::{ValidateError, ValidateErrorReason};
use lex::lexer::Span;
use proto_packet::io::TagNumber;

/// Validates the tag number at the `span` of the `source`.
pub fn validate_tag_number(span: Span, source: &str) -> Result<TagNumber, ValidateError> {
    span.text(source)
        .parse::<u16>()
        .ok()
        .and_then(TagNumber::new)
        .ok_or_else(|| ValidateError::new(span, ValidateErrorReason::InvalidTagNumber))
}
