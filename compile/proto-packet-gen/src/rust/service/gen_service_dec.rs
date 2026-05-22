use crate::rust::GenRust;
use code_gen::rust::RustType;
use proto_packet_tree::{Service, ServiceCall, WithCallName, WithComments};

impl GenRust<'_> {
    //! Gen Service: Trait Text

    pub(super) fn gen_service_trait_text(self, s: &Service) -> String {
        let mut out: String = String::new();
        for comment in s.comments() {
            out.push_str(&format!("///{}\n", comment));
        }
        out.push_str(&format!(
            "pub trait {}: Send + Sync {{\n",
            self.type_name(s)
        ));
        for (i, call) in s.calls().iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&self.gen_service_call_signature_text(call));
        }
        out.push_str("}\n");
        out
    }

    fn gen_service_call_signature_text(self, call: &ServiceCall) -> String {
        let request: RustType = self.field_type(call.request(), false, false);
        let response: RustType = self.field_type(call.response(), false, false);
        let mut out: String = String::new();
        for comment in call.comments() {
            out.push_str(&format!("    ///{}\n", comment));
        }
        out.push_str(&format!(
            "    fn {}(\n        &self,\n        request: {},\n    ) -> impl ::std::future::Future<\n        Output = ::std::result::Result<{}, ::proto_packet::service::ServiceError>,\n    > + Send + '_;\n",
            call.call_name(),
            request,
            response,
        ));
        out
    }
}
