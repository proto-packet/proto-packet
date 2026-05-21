use crate::parse::{MessageFieldTree, parse_message_field};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed message.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MessageTree {
    pub comments: Vec<Span>,
    pub type_name: Span,
    pub fields: Vec<MessageFieldTree>,
}

impl MessageTree {
    //! Fields

    /// Gets the fields with the `field_name`.
    pub fn fields_with_name(&self, field_name: &str, source: &str) -> Vec<&MessageFieldTree> {
        self.fields
            .iter()
            .filter(|f| f.field_name.text(source) == field_name)
            .collect()
    }
}

/// Parses a message.
pub fn parse_message(p: &mut Parser<Kind>) -> Result<MessageTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    p.expect(Kind::Message).ok_or_else(|| {
        ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDeclaration)
    })?;
    let type_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedTypeDecName))?
        .span();
    p.expect(Kind::LBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedOpenBrace))?;
    let mut fields: Vec<MessageFieldTree> = Vec::default();
    while !p.check(Kind::RBrace) && !p.check(Kind::EndOfFile) {
        fields.push(parse_message_field(p)?);
    }
    p.expect(Kind::RBrace)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedCloseBrace))?;
    Ok(MessageTree {
        comments,
        type_name,
        fields,
    })
}
