use crate::{ValidateError, ValidateErrorReason};
use lex::lexer::Span;
use proto_packet::io::TagNumber;
use proto_packet_tree::TreeError;

/// Validates the tag number at the `span` of the `source`.
pub fn validate_tag_number(span: Span, source: &str) -> Result<TagNumber, ValidateError> {
    span.text(source)
        .parse::<u16>()
        .ok()
        .and_then(TagNumber::new)
        .ok_or_else(|| ValidateError::new(span, ValidateErrorReason::InvalidTagNumber))
}

/// Creates the error for the tree `error` of adding a tagged member to its type. (ex: an enum case)
///
/// A duplicate tag number is at the member's `tag_number` span, & other errors at its `name` span.
pub(crate) fn tagged_member_error(error: TreeError, name: Span, tag_number: Span) -> ValidateError {
    let span: Span = match error {
        TreeError::DuplicateTagNumber { .. } => tag_number,
        _ => name,
    };
    ValidateError::new(span, ValidateErrorReason::Tree(error))
}
