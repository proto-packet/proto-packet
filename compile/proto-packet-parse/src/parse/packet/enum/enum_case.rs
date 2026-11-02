use crate::parse::expect::expect;
use crate::{Kind, ParseError, ParseErrorReason, parse_tag_number};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed enum case. (ex: `One = 1;`)
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EnumCaseTree {
    pub comments: Vec<Span>,
    pub case_name: Span,
    pub tag_number: Span,
}

/// Parses an enum case.
pub fn parse_enum_case(p: &mut Parser<'_, Kind>) -> Result<EnumCaseTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let case_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedCaseName)?;
    let tag_number: Span = parse_tag_number(p)?;
    expect(p, Kind::Semi, ParseErrorReason::ExpectedSemicolon)?;
    Ok(EnumCaseTree {
        comments,
        case_name,
        tag_number,
    })
}
