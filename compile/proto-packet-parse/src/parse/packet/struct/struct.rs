use crate::parse::expect::{expect, expect_keyword};
use crate::{Kind, ParseError, ParseErrorReason, StructFieldTree, parse_struct_field};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed struct.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StructTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub fields: Vec<StructFieldTree>,
}

/// Parses a struct.
pub fn parse_struct(p: &mut Parser<'_, Kind>) -> Result<StructTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    expect_keyword(p, "struct", ParseErrorReason::ExpectedTypeDeclaration)?;
    let type_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedTypeName)?;
    expect(p, Kind::LBrace, ParseErrorReason::ExpectedOpenBrace)?;
    let mut fields: Vec<StructFieldTree> = Vec::default();
    while !p.check(Kind::RBrace) && !p.check(Kind::EndOfFile) {
        fields.push(parse_struct_field(p)?);
    }
    expect(p, Kind::RBrace, ParseErrorReason::ExpectedCloseBrace)?;
    Ok(StructTree {
        comments,
        type_name,
        fields,
    })
}
