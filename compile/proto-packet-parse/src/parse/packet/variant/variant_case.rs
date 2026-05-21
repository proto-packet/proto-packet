use crate::parse::{TypeTagTree, parse_type_tag};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed variant case.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct VariantCaseTree {
    pub comments: Vec<Span>,
    pub case_name: Span,
    pub type_tag: TypeTagTree,
    pub tag_number: Span,
}

/// Parses a variant case.
pub fn parse_variant_case(p: &mut Parser<Kind>) -> Result<VariantCaseTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let case_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedCaseName))?
        .span();
    p.expect(Kind::Colon)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedColon))?;
    let type_tag: TypeTagTree = parse_type_tag(p)?;
    p.expect(Kind::Equal)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedEqual))?;
    let tag_number: Span = p
        .expect(Kind::Integer)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTagNumber))?
        .span();
    p.expect(Kind::Semi)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedSemicolon))?;
    Ok(VariantCaseTree {
        comments,
        case_name,
        type_tag,
        tag_number,
    })
}
