use crate::rust::GenRust;
use code_gen::WithStatements;
use code_gen::rust::Access::Public;
use code_gen::rust::{
    Function, Receiver, RustType, Signature, WithAccess, WithComments, WithReceiver, WithResult,
    WithVarParams,
};
use proto_packet_tree::{WithFieldName, WithTypeTag};

impl GenRust<'_> {
    //! Gen Field Setter

    /// Generates the setter function for the `field`.
    pub(in crate::rust) fn gen_field_setter<F>(self, field: &F, is_optional: bool) -> Function
    where
        F: WithFieldName + WithTypeTag,
    {
        let field_name: String = self.field_name(field);
        let field_type: RustType = self.field_type(field, is_optional);
        let signature: Signature = Signature::from(format!("set_{}", field_name))
            .with_receiver(Receiver::BorrowedMut)
            .with_param((field_name.clone(), field_type.clone()))
            .with_result(field_type);
        Function::from(signature)
            .with_access(Public)
            .with_comment(format!(
                " Sets the field: `{}`. Returns the previous value.",
                field.field_name()
            ))
            .with_literal(format!(
                "std::mem::replace(&mut self.{}, {})",
                field_name, field_name
            ))
    }
}
