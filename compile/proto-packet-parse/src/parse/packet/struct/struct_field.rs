use crate::parse::{TypeTagTree, parse_type_tag};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed struct field.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StructFieldTree {
    pub comments: Vec<Span>,
    pub field_name: Span,
    pub type_tag: TypeTagTree,
}

/// Parses a struct field.
pub fn parse_struct_field(p: &mut Parser<Kind>) -> Result<StructFieldTree, ParseError> {
    let comments: Vec<Span> = p.leading_comments();
    let field_name: Span = p
        .expect(Kind::Ident)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedFieldName))?
        .span();
    p.expect(Kind::Colon)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedColon))?;
    let type_tag: TypeTagTree = parse_type_tag(p)?;
    p.expect(Kind::Semi)
        .ok_or_else(|| ParseError::new(p.peek().span(), ParseErrorReason::ExpectedSemicolon))?;
    Ok(StructFieldTree {
        comments,
        field_name,
        type_tag,
    })
}
