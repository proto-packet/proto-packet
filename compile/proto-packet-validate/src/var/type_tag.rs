use crate::ValidateError;
use proto_packet_parse::TypeTagTree;
use proto_packet_tree::{QualifiedName, TypeTag};

/// Validates the type tag `tree`.
pub fn validate_type_tag(tree: &TypeTagTree, source: &str) -> Result<TypeTag, ValidateError> {
    match tree {
        TypeTagTree::Primitive(primitive) => Ok(TypeTag::Primitive(*primitive)),
        TypeTagTree::Special(special) => Ok(TypeTag::Special(*special)),
        TypeTagTree::Named(span) => {
            let text: &str = span.text(source);
            let name: QualifiedName =
                QualifiedName::new(text).map_err(|e| ValidateError::InvalidName {
                    span: *span,
                    message: e.message(),
                })?;
            Ok(TypeTag::Named(name))
        }
    }
}
