use crate::{Kind, ParseError, ParseErrorReason};
use lex::lexer::Span;
use lex::parser::Parser;

/// Expects a token of the `kind`, or fails with the `reason`.
pub(crate) fn expect(
    p: &mut Parser<'_, Kind>,
    kind: Kind,
    reason: ParseErrorReason,
) -> Result<Span, ParseError> {
    p.expect(kind)
        .map(|token| token.span())
        .map_err(|error| ParseError::new(error.span(), reason))
}
