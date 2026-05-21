use crate::parse::{TypeTagTree, parse_type_tag};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed message field.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MessageFieldTree {
    pub comments: Vec<Span>,
    pub field_name: Span,
    pub type_tag: TypeTagTree,
    pub tag_number: Span,
}

/// Parses a message field.
pub fn parse_message_field(p: &mut Parser<Kind>) -> Result<MessageFieldTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let field_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedFieldName))?
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
    Ok(MessageFieldTree {
        comments,
        field_name,
        type_tag,
        tag_number,
    })
}
