use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::{ValidateError, ValidateErrorReason, validate_variant_case};
use lex::lexer::Span;
use proto_packet_parse::VariantTree;
use proto_packet_tree::{TreeError, TypeName, Variant, VariantCase};

/// Validates the variant `tree`.
pub fn validate_variant(tree: &VariantTree, source: &str) -> Result<Variant, ValidateError> {
    let type_name: TypeName = validate_name(tree.type_name, source)?;
    let mut variant: Variant = Variant::from(type_name);
    add_comments(&mut variant, &tree.comments, source);
    for case_tree in &tree.cases {
        let case: VariantCase = validate_variant_case(case_tree, source)?;
        variant.add_case(case).map_err(|error| {
            let span: Span = match error {
                TreeError::DuplicateTagNumber { .. } => case_tree.tag_number,
                _ => case_tree.case_name,
            };
            ValidateError::new(span, ValidateErrorReason::Tree(error))
        })?;
    }
    Ok(variant)
}
