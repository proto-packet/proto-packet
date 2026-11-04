use crate::parse::expect::{expect, expect_keyword, parse_members};
use crate::{Kind, ParseError, ParseErrorReason, VariantCaseTree, parse_variant_case};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed variant.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct VariantTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub cases: Vec<VariantCaseTree>,
}

/// Parses a variant.
pub fn parse_variant(p: &mut Parser<'_, Kind>) -> Result<VariantTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    expect_keyword(p, "variant", ParseErrorReason::ExpectedTypeDeclaration)?;
    let type_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedTypeName)?;
    let cases: Vec<VariantCaseTree> = parse_members(p, parse_variant_case)?;
    Ok(VariantTree {
        comments,
        type_name,
        cases,
    })
}
