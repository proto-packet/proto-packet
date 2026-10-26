use crate::parse::expect::expect;
use crate::{Kind, ParseError, ParseErrorReason, TypeTagTree, parse_tag_number, parse_type_tag};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed variant case. (ex: `One: string = 1;`)
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct VariantCaseTree {
    pub comments: Vec<Span>,
    pub case_name: Span,
    pub type_tag: TypeTagTree,
    pub tag_number: Span,
}

/// Parses a variant case.
pub fn parse_variant_case(p: &mut Parser<'_, Kind>) -> Result<VariantCaseTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let case_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedCaseName)?;
    expect(p, Kind::Colon, ParseErrorReason::ExpectedColon)?;
    let type_tag: TypeTagTree = parse_type_tag(p)?;
    let tag_number: Span = parse_tag_number(p)?;
    expect(p, Kind::Semi, ParseErrorReason::ExpectedSemicolon)?;
    Ok(VariantCaseTree {
        comments,
        case_name,
        type_tag,
        tag_number,
    })
}
