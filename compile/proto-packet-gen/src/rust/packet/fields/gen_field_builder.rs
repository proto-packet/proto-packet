use crate::rust::GenRust;
use code_gen::WithStatements;
use code_gen::rust::Access::Public;
use code_gen::rust::Receiver::OwnedMut;
use code_gen::rust::{
    Function, RustType, Signature, WithAccess, WithAttributes, WithComments, WithReceiver,
    WithResult, WithVarParams,
};
use proto_packet_tree::{WithFieldName, WithTypeTag};

impl GenRust<'_> {
    //! Gen Field Builder

    pub(in crate::rust) fn gen_field_builder<F>(self, field: &F, is_optional: bool) -> Function
    where
        F: WithFieldName + WithTypeTag,
    {
        let field_name: String = self.field_name(field);
        let field_type: RustType = self.field_type(field, is_optional);
        let signature: Signature = Signature::from(format!("with_{field_name}"))
            .with_receiver(OwnedMut)
            .with_param((field_name.clone(), field_type))
            .with_result(RustType::from("Self"));
        Function::from(signature)
            .with_comment(format!(
                " Sets the field: `{field_name}`. Returns the struct itself."
            ))
            .with_attribute("must_use")
            .with_access(Public)
            .with_semi(format!("self.set_{field_name}({field_name})"))
            .with_literal("self")
    }
}
