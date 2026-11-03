use crate::naming::validate_name_text;
use crate::{ValidateError, ValidateErrorReason};
use lex::lexer::Span;
use proto_packet_parse::{NameTree, TypeTagTree, join_idents};
use proto_packet_tree::{PrimitiveType, QualifiedName, SpecialType, TypeTag};

/// Validates the type tag `tree`.
pub fn validate_type_tag(tree: &TypeTagTree, source: &str) -> Result<TypeTag, ValidateError> {
    match tree {
        TypeTagTree::Name(name) => validate_named_type(name, source),
    }
}

/// Validates the type `name` as a primitive, special or named type.
///
/// A single lowercase name that is not a primitive or special type is unknown, since type names
/// start with an uppercase letter. (ex: `i32`)
fn validate_named_type(name: &NameTree, source: &str) -> Result<TypeTag, ValidateError> {
    let text: String = join_idents(&name.idents, source);
    if let Some(primitive) = PrimitiveType::from_name(&text) {
        Ok(primitive.into())
    } else if let Some(special) = SpecialType::from_name(&text) {
        Ok(special.into())
    } else if name.idents.len() == 1 && text.starts_with(|c: char| c.is_ascii_lowercase()) {
        Err(ValidateError::new(
            name.span(),
            ValidateErrorReason::UnknownType { name: text },
        ))
    } else {
        let name: QualifiedName = validate_name_text(&text, name.span())?;
        Ok(name.into())
    }
}

/// Validates that a boxed `type_tag` is a named type. (the `span` is of its field or case name)
pub(crate) fn validate_boxed(
    is_boxed: bool,
    type_tag: &TypeTag,
    span: Span,
) -> Result<(), ValidateError> {
    if is_boxed && !matches!(type_tag, TypeTag::Named(_)) {
        Err(ValidateError::new(
            span,
            ValidateErrorReason::BoxedNonNamedType,
        ))
    } else {
        Ok(())
    }
}
