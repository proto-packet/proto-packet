use crate::parse::expect::expect;
use crate::{Kind, ParseError, ParseErrorReason, TypeTagTree, parse_type_tag};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed struct field. (ex: `name: optional string;`)
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StructFieldTree {
    pub comments: Vec<Span>,
    pub field_name: Span,
    pub is_optional: bool,
    pub type_tag: TypeTagTree,
}

/// Parses a struct field.
pub fn parse_struct_field(p: &mut Parser<'_, Kind>) -> Result<StructFieldTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let field_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedFieldName)?;
    expect(p, Kind::Colon, ParseErrorReason::ExpectedColon)?;
    let is_optional: bool = parse_optional(p);
    let type_tag: TypeTagTree = parse_type_tag(p)?;
    expect(p, Kind::Semi, ParseErrorReason::ExpectedSemicolon)?;
    Ok(StructFieldTree {
        comments,
        field_name,
        is_optional,
        type_tag,
    })
}

/// Parses the `optional` keyword if present.
///
/// It is only a keyword when a type follows, so `optional` is a mod name in `optional.Foo`.
fn parse_optional(p: &mut Parser<'_, Kind>) -> bool {
    let is_keyword: bool = p.check_text(Kind::Ident, "optional")
        && p.lookahead(1)
            .is_some_and(|next| matches!(next.kind(), Kind::Ident | Kind::LBracket));
    if is_keyword {
        p.advance();
    }
    is_keyword
}
