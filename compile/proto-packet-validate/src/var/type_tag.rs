use crate::ValidateError;
use crate::naming::validate_name_text;
use proto_packet_parse::{NameTree, TypeTagTree};
use proto_packet_tree::{PrimitiveType, QualifiedName, SpecialType, TypeTag};

/// Validates the type tag `tree`.
pub fn validate_type_tag(tree: &TypeTagTree, source: &str) -> Result<TypeTag, ValidateError> {
    match tree {
        TypeTagTree::Name(name) => validate_named_type(name, source),
    }
}

/// Validates the type `name` as a primitive, special or named type.
fn validate_named_type(name: &NameTree, source: &str) -> Result<TypeTag, ValidateError> {
    let text: String = name
        .idents
        .iter()
        .map(|ident| ident.text(source))
        .collect::<Vec<&str>>()
        .join(".");
    if let Some(primitive) = PrimitiveType::from_name(&text) {
        Ok(primitive.into())
    } else if let Some(special) = SpecialType::from_name(&text) {
        Ok(special.into())
    } else {
        let name: QualifiedName = validate_name_text(&text, name.span())?;
        Ok(name.into())
    }
}
