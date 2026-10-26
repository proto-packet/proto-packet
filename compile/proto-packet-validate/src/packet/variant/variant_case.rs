use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::var::validate_boxed;
use crate::{ValidateError, validate_tag_number, validate_type_tag};
use proto_packet::io::TagNumber;
use proto_packet_parse::VariantCaseTree;
use proto_packet_tree::{CaseName, TypeTag, VariantCase};

/// Validates the variant case `tree`.
pub fn validate_variant_case(
    tree: &VariantCaseTree,
    source: &str,
) -> Result<VariantCase, ValidateError> {
    let case_name: CaseName = validate_name(tree.case_name, source)?;
    let type_tag: TypeTag = validate_type_tag(&tree.type_tag, source)?;
    let tag_number: TagNumber = validate_tag_number(tree.tag_number, source)?;
    validate_boxed(tree.is_boxed, &type_tag, tree.case_name)?;
    let mut case: VariantCase =
        VariantCase::new(case_name, type_tag, tag_number).with_boxed(tree.is_boxed);
    add_comments(&mut case, &tree.comments, source);
    Ok(case)
}
