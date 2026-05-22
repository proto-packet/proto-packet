use crate::{ValidateError, validate_type_tag};
use proto_packet_parse::ServiceCallTree;
use proto_packet_tree::{CallName, ServiceCall, TypeTag, WithComments};

/// Validates the service call `tree`.
pub fn validate_service_call(
    tree: &ServiceCallTree,
    source: &str,
) -> Result<ServiceCall, ValidateError> {
    let call_name: CallName =
        CallName::new(tree.call_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.call_name,
            message: e.message(),
        })?;
    let request: TypeTag = validate_type_tag(&tree.request, source)?;
    let response: TypeTag = validate_type_tag(&tree.response, source)?;

    let mut call: ServiceCall = ServiceCall::new(call_name, request, response);
    for comment in &tree.comments {
        call.add_comment(comment.text(source));
    }

    Ok(call)
}
