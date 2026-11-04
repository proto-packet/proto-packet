use crate::parse::expect::{expect, expect_keyword, parse_members};
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
    let fields: Vec<StructFieldTree> = parse_members(p, parse_struct_field)?;
    Ok(StructTree {
        comments,
        type_name,
        fields,
    })
}
