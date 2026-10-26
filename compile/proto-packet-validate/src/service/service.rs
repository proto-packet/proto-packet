use crate::comments::add_comments;
use crate::naming::validate_name;
use crate::{ValidateError, ValidateErrorReason, validate_service_call};
use proto_packet_parse::ServiceTree;
use proto_packet_tree::{Service, ServiceCall, TypeName};

/// Validates the service `tree`.
pub fn validate_service(tree: &ServiceTree, source: &str) -> Result<Service, ValidateError> {
    let type_name: TypeName = validate_name(tree.type_name, source)?;
    let mut service: Service = Service::from(type_name);
    add_comments(&mut service, &tree.comments, source);
    for call_tree in &tree.calls {
        let call: ServiceCall = validate_service_call(call_tree, source)?;
        service.add_call(call).map_err(|error| {
            ValidateError::new(call_tree.call_name, ValidateErrorReason::Tree(error))
        })?;
    }
    Ok(service)
}
