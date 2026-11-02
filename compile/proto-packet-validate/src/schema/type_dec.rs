use crate::{
    ValidateError, validate_enum, validate_message, validate_service, validate_struct,
    validate_variant,
};
use proto_packet_parse::TypeDecTree;
use proto_packet_tree::TypeDec;

/// Validates the type declaration `tree`.
pub fn validate_type_dec(tree: &TypeDecTree, source: &str) -> Result<TypeDec, ValidateError> {
    Ok(match tree {
        TypeDecTree::Struct(structure) => validate_struct(structure, source)?.into(),
        TypeDecTree::Message(message) => validate_message(message, source)?.into(),
        TypeDecTree::Enum(enumeration) => validate_enum(enumeration, source)?.into(),
        TypeDecTree::Variant(variant) => validate_variant(variant, source)?.into(),
        TypeDecTree::Service(service) => validate_service(service, source)?.into(),
    })
}
