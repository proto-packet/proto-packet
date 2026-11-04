use crate::{Kind, ParseError, ParseErrorReason};
use lex::lexer::Span;
use lex::parser::Parser;

/// Gets the span of the last token before the current one, skipping whitespace & comments.
fn previous_span(p: &Parser<'_, Kind>) -> Option<Span> {
    p.tokens()[..p.pos()]
        .iter()
        .rev()
        .find(|token| !matches!(token.kind(), Kind::Whitespace | Kind::LineComment))
        .map(|token| token.span())
}

/// Gets the span to report an error at the `span`.
///
/// The end of the file has nothing to point at, so it's reported at the last token before it.
fn error_span(p: &Parser<'_, Kind>, span: Span) -> Span {
    if p.check(Kind::EndOfFile) {
        previous_span(p).unwrap_or(span)
    } else {
        span
    }
}

/// Expects a token of the `kind`, or fails with the `reason`.
pub(crate) fn expect(
    p: &mut Parser<'_, Kind>,
    kind: Kind,
    reason: ParseErrorReason,
) -> Result<Span, ParseError> {
    p.expect(kind)
        .map(|token| token.span())
        .map_err(|error| ParseError::new(error_span(p, error.span()), reason))
}

/// Expects a token of the `kind` ending what came before it, or fails with the `reason` at the last
/// token before it. (ex: a missing `;` is reported after the field, not at the next line)
pub(crate) fn expect_end(
    p: &mut Parser<'_, Kind>,
    kind: Kind,
    reason: ParseErrorReason,
) -> Result<Span, ParseError> {
    p.expect(kind)
        .map(|token| token.span())
        .map_err(|error| ParseError::new(previous_span(p).unwrap_or(error.span()), reason))
}

/// Parses the members of a type between braces, with the `parse_member` function.
pub(crate) fn parse_members<T>(
    p: &mut Parser<'_, Kind>,
    parse_member: fn(&mut Parser<'_, Kind>) -> Result<T, ParseError>,
) -> Result<Vec<T>, ParseError> {
    expect(p, Kind::LBrace, ParseErrorReason::ExpectedOpenBrace)?;
    let mut members: Vec<T> = Vec::default();
    while !p.check(Kind::RBrace) && !p.check(Kind::EndOfFile) {
        members.push(parse_member(p)?);
    }
    expect_end(p, Kind::RBrace, ParseErrorReason::ExpectedCloseBrace)?;
    Ok(members)
}

/// Expects the contextual `keyword`, or fails with the `reason`.
pub(crate) fn expect_keyword(
    p: &mut Parser<'_, Kind>,
    keyword: &str,
    reason: ParseErrorReason,
) -> Result<Span, ParseError> {
    match p.accept_text(Kind::Ident, keyword) {
        Some(token) => Ok(token.span()),
        None => Err(ParseError::new(error_span(p, p.peek().span()), reason)),
    }
}
