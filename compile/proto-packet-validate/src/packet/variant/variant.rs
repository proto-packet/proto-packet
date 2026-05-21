use crate::ValidateError;
use crate::packet::validate_variant_case;
use proto_packet_parse::VariantTree;
use proto_packet_tree::{TypeName, Variant, VariantCase, WithComments};

/// Validates the variant `tree`.
pub fn validate_variant(tree: &VariantTree, source: &str) -> Result<Variant, ValidateError> {
    let type_name: TypeName =
        TypeName::new(tree.type_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.type_name,
            message: e.message(),
        })?;

    let mut variant: Variant = Variant::from(type_name);
    for comment in &tree.comments {
        variant.add_comment(comment.text(source));
    }

    for case_tree in &tree.cases {
        let case: VariantCase = validate_variant_case(case_tree, source)?;
        variant
            .add_case(case)
            .map_err(|error| ValidateError::Tree { error })?;
    }

    Ok(variant)
}
