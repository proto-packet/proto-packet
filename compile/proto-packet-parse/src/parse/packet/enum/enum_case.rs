use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed enum case.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EnumCaseTree {
    pub comments: Vec<Span>,
    pub case_name: Span,
    pub tag_number: Span,
}

/// Parses an enum case.
pub fn parse_enum_case(p: &mut Parser<Kind>) -> Result<EnumCaseTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let case_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedCaseName))?
        .span();
    p.expect(Kind::Equal)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedEqual))?;
    let tag_number: Span = p
        .expect(Kind::Integer)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTagNumber))?
        .span();
    p.expect(Kind::Semi)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedSemicolon))?;
    Ok(EnumCaseTree {
        comments,
        case_name,
        tag_number,
    })
}
