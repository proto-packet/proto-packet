use crate::parse::expect::{expect, expect_end};
use crate::parse::var::{parse_type_modifier, reject_type_modifier};
use crate::{Kind, ParseError, ParseErrorReason, TypeTagTree, parse_type_tag};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed struct field. (ex: `child: optional box Node;`)
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StructFieldTree {
    pub comments: Vec<Span>,
    pub field_name: Span,
    pub is_optional: bool,
    pub is_boxed: bool,
    pub type_tag: TypeTagTree,
}

/// Parses a struct field.
pub fn parse_struct_field(p: &mut Parser<'_, Kind>) -> Result<StructFieldTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let field_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedFieldName)?;
    expect(p, Kind::Colon, ParseErrorReason::ExpectedColon)?;
    let is_optional: bool = parse_type_modifier(p, "optional");
    if is_optional {
        reject_type_modifier(p, "optional", ParseErrorReason::DuplicateOptional)?;
    }
    let is_boxed: bool = parse_type_modifier(p, "box");
    reject_type_modifier(p, "optional", ParseErrorReason::OptionalAfterBox)?;
    let type_tag: TypeTagTree = parse_type_tag(p)?;
    expect_end(p, Kind::Semi, ParseErrorReason::ExpectedSemicolon)?;
    Ok(StructFieldTree {
        comments,
        field_name,
        is_optional,
        is_boxed,
        type_tag,
    })
}
