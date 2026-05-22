use crate::rust::GenRust;
use code_gen::rust::Access::Public;
use code_gen::rust::{
    Receiver, RustType, Signature, SignatureDec, Trait as RustTrait, WithAccess,
    WithComments as RustWithComments, WithReceiver, WithResult, WithTraitFunctions, WithVarParams,
};
use proto_packet_tree::{Service, ServiceCall, WithComments};

impl GenRust<'_> {
    //! Gen Service: Declaration

    pub(super) fn gen_service_dec(self, s: &Service) -> RustTrait {
        let mut result: RustTrait = RustTrait::from(self.type_name(s));
        for comment in s.comments() {
            result.add_comment(comment);
        }
        result.set_access(Public);
        for call in s.calls() {
            result.add_signature_dec(self.gen_service_call_signature_dec(call));
        }
        result
    }

    fn gen_service_call_signature_dec(self, call: &ServiceCall) -> SignatureDec {
        let request_type: RustType = self.field_type(call.request(), false, false);
        let response_type: RustType = self.field_type(call.response(), false, false);
        let signature: Signature = Signature::from(self.call_name(call))
            .with_receiver(Receiver::Borrowed)
            .with_param(("request", request_type))
            .with_result(response_type);
        let mut dec: SignatureDec = SignatureDec::from(signature);
        for comment in call.comments() {
            dec.add_comment(comment);
        }
        dec
    }
}
