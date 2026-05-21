use crate::{Kind, ParseError, ParseErrorReason};
use lex::lexer::Span;
use lex::parser::Parser;
use proto_packet_tree::PrimitiveType;

/// A parsed type tag.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TypeTagTree {
    /// A primitive type with its parsed value.
    Primitive(PrimitiveType),

    /// A named type referenced by the span of its identifier.
    Named(Span),
}

/// Parses a type tag.
pub fn parse_type_tag(p: &mut Parser<Kind>) -> Result<TypeTagTree, ParseError> {
    let span: Span = p.peek().span();
    if p.check(Kind::Ident) {
        let text: &str = p.text(span);
        if let Ok(primitive) = text.parse::<PrimitiveType>() {
            p.advance();
            return Ok(TypeTagTree::Primitive(primitive));
        }
        p.advance();
        return Ok(TypeTagTree::Named(span));
    }
    Err(ParseError::new(span, ParseErrorReason::UnrecognizedTypeTag))
}
