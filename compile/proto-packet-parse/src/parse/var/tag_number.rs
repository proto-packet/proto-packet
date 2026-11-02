use crate::parse::expect::expect;
use crate::{Kind, ParseError, ParseErrorReason};
use lex::lexer::Span;
use lex::parser::Parser;

/// Parses a tag number assignment. (ex: `= 1`)
///
/// Returns the span of the tag number.
pub fn parse_tag_number(p: &mut Parser<'_, Kind>) -> Result<Span, ParseError> {
    expect(p, Kind::Equal, ParseErrorReason::ExpectedEqual)?;
    expect(p, Kind::Integer, ParseErrorReason::ExpectedTagNumber)
}
