use crate::{ValidateError, validate_type_tag};
use proto_packet::io::TagNumber;
use proto_packet_parse::VariantCaseTree;
use proto_packet_tree::{CaseName, TypeTag, VariantCase, WithComments};

/// Validates the variant case `tree`.
pub fn validate_variant_case(
    tree: &VariantCaseTree,
    source: &str,
) -> Result<VariantCase, ValidateError> {
    let case_name: CaseName =
        CaseName::new(tree.case_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.case_name,
            message: e.message(),
        })?;
    let type_tag: TypeTag = validate_type_tag(&tree.type_tag, source)?;
    let tag_number: TagNumber = tree
        .tag_number
        .text(source)
        .parse::<u16>()
        .ok()
        .and_then(TagNumber::new)
        .ok_or(ValidateError::InvalidTagNumber {
            span: tree.tag_number,
            message: "tag number must be in 1..=8191",
        })?;

    let mut case: VariantCase = VariantCase::new(case_name, type_tag, tag_number);
    for comment in &tree.comments {
        case.add_comment(comment.text(source));
    }

    Ok(case)
}
