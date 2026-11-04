use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::var::validate_boxed;
use crate::{ValidateError, ValidateErrorReason, validate_tag_number, validate_type_tag};
use proto_packet::io::TagNumber;
use proto_packet_parse::VariantCaseTree;
use proto_packet_tree::{CaseName, TypeTag, VariantCase};

/// The name of the generated unrecognized case.
const UNRECOGNIZED: &str = "Unrecognized";

/// Validates the variant case `tree`.
///
/// No case can be named `Unrecognized`, since each language generates that case.
pub fn validate_variant_case(
    tree: &VariantCaseTree,
    source: &str,
) -> Result<VariantCase, ValidateError> {
    let case_name: CaseName = validate_name(tree.case_name, source)?;
    if case_name.value() == UNRECOGNIZED {
        let message: &'static str = "variant cases cannot be named `Unrecognized`";
        return Err(ValidateError::new(
            tree.case_name,
            ValidateErrorReason::InvalidName { message },
        ));
    }
    let type_tag: TypeTag = validate_type_tag(&tree.type_tag, source)?;
    let tag_number: TagNumber = validate_tag_number(tree.tag_number, source)?;
    validate_boxed(tree.is_boxed, &type_tag, tree.type_tag.span())?;
    let mut case: VariantCase =
        VariantCase::new(case_name, type_tag, tag_number).with_boxed(tree.is_boxed);
    add_comments(&mut case, &tree.comments, source);
    Ok(case)
}
