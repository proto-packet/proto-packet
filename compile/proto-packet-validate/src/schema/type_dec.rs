use crate::{ValidateError, validate_enum, validate_message, validate_struct, validate_variant};
use proto_packet_parse::TypeDecTree;
use proto_packet_tree::TypeDec;

/// Validates the type declaration `tree`.
pub fn validate_type_dec(tree: &TypeDecTree, source: &str) -> Result<TypeDec, ValidateError> {
    match tree {
        TypeDecTree::Struct(s) => Ok(TypeDec::from(validate_struct(s, source)?)),
        TypeDecTree::Message(m) => Ok(TypeDec::from(validate_message(m, source)?)),
        TypeDecTree::Variant(v) => Ok(TypeDec::from(validate_variant(v, source)?)),
        TypeDecTree::Enum(e) => Ok(TypeDec::from(validate_enum(e, source)?)),
        TypeDecTree::Service(_) => todo!(),
    }
}
