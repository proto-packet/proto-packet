use crate::linker::schema_linker::{SchemaLinker, copy_comments};
use crate::{LinkError, Member};
use proto_packet_tree::{Service, ServiceCall, TypeTag, WithCallName, WithTypeName};

impl SchemaLinker<'_> {
    //! Services

    /// Links the `service`.
    pub(in crate::linker) fn link_service(self, service: &Service) -> Result<Service, LinkError> {
        let mut linked: Service = Service::from(service.type_name().into_owned());
        copy_comments(service, &mut linked);
        for call in service.calls() {
            let request: TypeTag = self.resolver.link_type_tag(
                call.request(),
                &self.location(
                    service.type_name(),
                    Member::Request(call.call_name().into_owned()),
                ),
            )?;
            let response: TypeTag = self.resolver.link_type_tag(
                call.response(),
                &self.location(
                    service.type_name(),
                    Member::Response(call.call_name().into_owned()),
                ),
            )?;
            let mut linked_call: ServiceCall =
                ServiceCall::new(call.call_name(), request, response);
            copy_comments(call, &mut linked_call);
            linked
                .add_call(linked_call)
                .expect("linking keeps the call names unique");
        }
        Ok(linked)
    }
}
