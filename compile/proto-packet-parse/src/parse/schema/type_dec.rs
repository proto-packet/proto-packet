use crate::{
    EnumTree, Kind, MessageTree, ParseError, ParseErrorReason, ServiceTree, StructTree,
    VariantTree, parse_enum, parse_message, parse_service, parse_struct, parse_variant,
};
use lex::lexer::Span;
use lex::parser::Parser;

/// A parsed type declaration.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TypeDecTree {
    Struct(StructTree),
    Message(MessageTree),
    Enum(EnumTree),
    Variant(VariantTree),
    Service(ServiceTree),
}

impl TypeDecTree {
    //! Properties

    /// Gets the type name.
    pub fn type_name(&self) -> Span {
        match self {
            Self::Struct(structure) => structure.type_name,
            Self::Message(message) => message.type_name,
            Self::Enum(enumeration) => enumeration.type_name,
            Self::Variant(variant) => variant.type_name,
            Self::Service(service) => service.type_name,
        }
    }
}

/// Parses a type declaration.
pub fn parse_type_dec(p: &mut Parser<'_, Kind>) -> Result<TypeDecTree, ParseError> {
    if p.check_text(Kind::Ident, "struct") {
        Ok(TypeDecTree::Struct(parse_struct(p)?))
    } else if p.check_text(Kind::Ident, "message") {
        Ok(TypeDecTree::Message(parse_message(p)?))
    } else if p.check_text(Kind::Ident, "enum") {
        Ok(TypeDecTree::Enum(parse_enum(p)?))
    } else if p.check_text(Kind::Ident, "variant") {
        Ok(TypeDecTree::Variant(parse_variant(p)?))
    } else if p.check_text(Kind::Ident, "service") {
        Ok(TypeDecTree::Service(parse_service(p)?))
    } else {
        Err(ParseError::new(
            p.peek().span(),
            ParseErrorReason::ExpectedTypeDeclaration,
        ))
    }
}
