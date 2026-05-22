use crate::ValidateError;
use crate::service::validate_service_call;
use proto_packet_parse::ServiceTree;
use proto_packet_tree::{Service, ServiceCall, TypeName, WithComments};

/// Validates the service `tree`.
pub fn validate_service(tree: &ServiceTree, source: &str) -> Result<Service, ValidateError> {
    let type_name: TypeName =
        TypeName::new(tree.type_name.text(source)).map_err(|e| ValidateError::InvalidName {
            span: tree.type_name,
            message: e.message(),
        })?;

    let mut service: Service = Service::from(type_name);
    for comment in &tree.comments {
        service.add_comment(comment.text(source));
    }

    for call_tree in &tree.calls {
        let call: ServiceCall = validate_service_call(call_tree, source)?;
        service
            .add_call(call)
            .map_err(|error| ValidateError::Tree { error })?;
    }

    Ok(service)
}
