use crate::rust::GenRust;
use code_gen::rust::Access::Public;
use code_gen::rust::Receiver::OwnedMut;
use code_gen::rust::{
    Function, RustType, Signature, Var, WithAccess, WithAttributes, WithComments, WithFnGenerics,
    WithReceiver, WithResult, WithVarParams,
};
use code_gen::{Expression, WithStatements};
use proto_packet_tree::{WithFieldName, WithTypeTag};

impl GenRust<'_> {
    //! Gen Field Builder

    pub(in crate::rust) fn gen_field_builder<F>(self, field: &F, is_optional: bool) -> Function
    where
        F: WithFieldName + WithTypeTag,
    {
        let field_name: String = self.field_name(field);
        let field_type: RustType = self.field_type(field, is_optional, false);
        let signature: Signature = if is_optional {
            let field_type_code: String = field_type.to_code();
            Signature::from(format!("with_{field_name}"))
                .with_receiver(OwnedMut)
                .with_generic(Var::from(("F", format!("Into<{field_type_code}>"))))
                .with_param((field_name.clone(), RustType::from("F")))
                .with_result(RustType::from("Self"))
        } else {
            Signature::from(format!("with_{field_name}"))
                .with_receiver(OwnedMut)
                .with_param((field_name.clone(), field_type))
                .with_result(RustType::from("Self"))
        };
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
