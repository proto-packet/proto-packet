use crate::ValidateError;
use crate::naming::validate_name;
use proto_packet_parse::{NameTree, TypeTagTree};
use proto_packet_tree::{PrimitiveType, QualifiedName, SpecialType, TimeType, TypeTag};

/// Validates the type tag `tree`.
pub fn validate_type_tag(tree: &TypeTagTree, source: &str) -> Result<TypeTag, ValidateError> {
    match tree {
        TypeTagTree::Name(name) => validate_named_type(name, source),
        TypeTagTree::Slice { base } => Ok(validate_type_tag(base, source)?.into_slice()),
    }
}

/// Validates the type `name` as a primitive, special, time or named type.
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
    } else if let Some(time) = TimeType::from_name(&text) {
        Ok(time.into())
    } else {
        let name: QualifiedName = validate_name(&text, name.span())?;
        Ok(name.into())
    }
}
