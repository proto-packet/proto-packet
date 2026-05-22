use crate::{Kind, ParseError, ParseErrorReason};
use lex::lexer::Span;
use lex::parser::Parser;
use proto_packet_tree::{PrimitiveType, SpecialType, TimeType};

/// A parsed type tag.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TypeTagTree {
    /// A primitive type with its parsed value.
    Primitive(PrimitiveType),

    /// A special type with its parsed value.
    Special(SpecialType),

    /// A time type with its parsed value.
    Time(TimeType),

    /// A named type referenced by the span of its identifier.
    Named(Span),

    /// A slice type wrapping another type tag (parsed `[]T`).
    Slice { base: Box<TypeTagTree> },
}

/// Parses a type tag.
pub fn parse_type_tag(p: &mut Parser<Kind>) -> Result<TypeTagTree, ParseError> {
    let span: Span = p.peek().span();
    if p.check(Kind::LBracket) {
        p.advance();
        p.expect(Kind::RBracket).ok_or_else(|| {
            ParseError::new(p.peek().span(), ParseErrorReason::ExpectedCloseBracket)
        })?;
        let base: TypeTagTree = parse_type_tag(p)?;
        return Ok(TypeTagTree::Slice {
            base: Box::new(base),
        });
    }
    if p.check(Kind::Ident) {
        let text: &str = p.text(span);
        if let Ok(primitive) = text.parse::<PrimitiveType>() {
            p.advance();
            return Ok(TypeTagTree::Primitive(primitive));
        }
        if let Ok(special) = text.parse::<SpecialType>() {
            p.advance();
            return Ok(TypeTagTree::Special(special));
        }
        if let Ok(time) = text.parse::<TimeType>() {
            p.advance();
            return Ok(TypeTagTree::Time(time));
        }
        p.advance();
        return Ok(TypeTagTree::Named(span));
    }
    Err(ParseError::new(span, ParseErrorReason::UnrecognizedTypeTag))
}
