use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::{ValidateError, validate_type_tag};
use proto_packet_parse::ServiceCallTree;
use proto_packet_tree::{CallName, ServiceCall, TypeTag};

/// Validates the service call `tree`.
pub fn validate_service_call(
    tree: &ServiceCallTree,
    source: &str,
) -> Result<ServiceCall, ValidateError> {
    let call_name: CallName = validate_name(tree.call_name, source)?;
    let request: TypeTag = validate_type_tag(&tree.request, source)?;
    let response: TypeTag = validate_type_tag(&tree.response, source)?;
    let mut call: ServiceCall = ServiceCall::new(call_name, request, response);
    add_comments(&mut call, &tree.comments, source);
    Ok(call)
}
