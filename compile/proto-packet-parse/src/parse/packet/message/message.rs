use crate::parse::expect::{expect, expect_keyword};
use crate::{Kind, MessageFieldTree, ParseError, ParseErrorReason, parse_message_field};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed message.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MessageTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub fields: Vec<MessageFieldTree>,
}

/// Parses a message.
pub fn parse_message(p: &mut Parser<'_, Kind>) -> Result<MessageTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    expect_keyword(p, "message", ParseErrorReason::ExpectedTypeDeclaration)?;
    let type_name: Span = expect(p, Kind::Ident, ParseErrorReason::ExpectedTypeName)?;
    expect(p, Kind::LBrace, ParseErrorReason::ExpectedOpenBrace)?;
    let mut fields: Vec<MessageFieldTree> = Vec::default();
    while !p.check(Kind::RBrace) && !p.check(Kind::EndOfFile) {
        fields.push(parse_message_field(p)?);
    }
    expect(p, Kind::RBrace, ParseErrorReason::ExpectedCloseBrace)?;
    Ok(MessageTree {
        comments,
        type_name,
        fields,
    })
}
