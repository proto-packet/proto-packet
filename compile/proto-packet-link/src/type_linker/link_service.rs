use crate::{LinkError, TypeLinker};
use proto_packet_tree::{
    Service, ServiceCall, TypeName, TypeTag, WithCallName, WithComments, WithTypeName,
};

impl TypeLinker<'_> {
    //! Services

    /// Links the service `s`.
    pub(in crate::type_linker) fn link_service(self, s: &Service) -> Result<Service, LinkError> {
        let type_name: TypeName = s.type_name().into_owned();
        let mut linked: Service = Service::from(type_name);
        for comment in s.comments() {
            linked.add_comment(comment);
        }
        for call in s.calls() {
            let linked_call: ServiceCall = self.link_service_call(call)?;
            linked
                .add_call(linked_call)
                .expect("duplicate service call")
        }
        Ok(linked)
    }

    /// Links the service `call`.
    fn link_service_call(self, call: &ServiceCall) -> Result<ServiceCall, LinkError> {
        let request: TypeTag = self.link_type_tag(call.request())?;
        let response: TypeTag = self.link_type_tag(call.response())?;
        let mut linked: ServiceCall = ServiceCall::new(call.call_name(), request, response);
        for comment in call.comments() {
            linked.add_comment(comment);
        }
        Ok(linked)
    }
}
