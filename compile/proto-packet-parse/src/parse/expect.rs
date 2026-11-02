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

/// Expects the contextual `keyword`, or fails with the `reason`.
pub(crate) fn expect_keyword(
    p: &mut Parser<'_, Kind>,
    keyword: &str,
    reason: ParseErrorReason,
) -> Result<Span, ParseError> {
    match p.accept_text(Kind::Ident, keyword) {
        Some(token) => Ok(token.span()),
        None => Err(ParseError::new(p.peek().span(), reason)),
    }
}
