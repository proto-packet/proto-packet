use crate::parse::{
    EnumTree, MessageTree, ServiceTree, StructTree, VariantTree, parse_enum, parse_message,
    parse_service, parse_struct, parse_variant,
};
use crate::{Kind, ParseError, ParseErrorReason};
use ::lex::lexer::Span;
use ::lex::parser::Parser;

/// A parsed type declaration.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TypeDecTree {
    Struct(StructTree),
    Message(MessageTree),
    Variant(VariantTree),
    Enum(EnumTree),
    Service(ServiceTree),
}

impl TypeDecTree {
    //! Properties

    /// Gets the type name.
    pub fn type_name(&self) -> Span {
        match self {
            Self::Struct(s) => s.type_name,
            Self::Message(m) => m.type_name,
            Self::Variant(v) => v.type_name,
            Self::Enum(e) => e.type_name,
            Self::Service(s) => s.type_name,
        }
    }
}

/// Parses a type declaration.
pub fn parse_type_dec(p: &mut Parser<Kind>) -> Result<TypeDecTree, ParseError> {
    if p.check(Kind::Struct) {
        Ok(TypeDecTree::Struct(parse_struct(p)?))
    } else if p.check(Kind::Message) {
        Ok(TypeDecTree::Message(parse_message(p)?))
    } else if p.check(Kind::Variant) {
        Ok(TypeDecTree::Variant(parse_variant(p)?))
    } else if p.check(Kind::Enum) {
        Ok(TypeDecTree::Enum(parse_enum(p)?))
    } else if p.check(Kind::Service) {
        Ok(TypeDecTree::Service(parse_service(p)?))
    } else {
        Err(ParseError::new(
            p.peek().span(),
            ParseErrorReason::ExpectedTypeDeclaration,
        ))
    }
}
