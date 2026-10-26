use crate::parse::expect::expect;
use crate::{Kind, ParseError, ParseErrorReason, TypeTagTree, parse_tag_number, parse_type_tag};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed message field. (ex: `name: string = 1;`)
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MessageFieldTree {
    pub comments: Vec<Span>,
    pub field_name: Span,
    pub type_tag: TypeTagTree,
    pub tag_number: Span,
}

/// Parses a message field.
pub fn parse_message_field(p: &mut Parser<'_, Kind>) -> Result<MessageFieldTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let field_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedFieldName)?;
    expect(p, Kind::Colon, ParseErrorReason::ExpectedColon)?;
    let type_tag: TypeTagTree = parse_type_tag(p)?;
    let tag_number: Span = parse_tag_number(p)?;
    expect(p, Kind::Semi, ParseErrorReason::ExpectedSemicolon)?;
    Ok(MessageFieldTree {
        comments,
        field_name,
        type_tag,
        tag_number,
    })
}
