use crate::parse::expect::{expect, expect_keyword};
use crate::{EnumCaseTree, Kind, ParseError, ParseErrorReason, parse_enum_case};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed enum.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EnumTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub cases: Vec<EnumCaseTree>,
}

/// Parses an enum.
pub fn parse_enum(p: &mut Parser<'_, Kind>) -> Result<EnumTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    expect_keyword(p, "enum", ParseErrorReason::ExpectedTypeDeclaration)?;
    let type_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedTypeName)?;
    expect(p, Kind::LBrace, ParseErrorReason::ExpectedOpenBrace)?;
    let mut cases: Vec<EnumCaseTree> = Vec::default();
    while !p.check(Kind::RBrace) && !p.check(Kind::EndOfFile) {
        cases.push(parse_enum_case(p)?);
    }
    expect(p, Kind::RBrace, ParseErrorReason::ExpectedCloseBrace)?;
    Ok(EnumTree {
        comments,
        type_name,
        cases,
    })
}
