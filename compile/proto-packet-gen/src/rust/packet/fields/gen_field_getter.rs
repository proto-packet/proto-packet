use crate::rust::GenRust;
use code_gen::WithStatements;
use code_gen::rust::Access::Public;
use code_gen::rust::{
    Function, Receiver, RustType, Signature, WithAccess, WithAttributes, WithComments,
    WithReceiver, WithResult,
};
use proto_packet_tree::{WithFieldName, WithTypeTag};

impl GenRust<'_> {
    //! Gen Field Getter

    /// Generates the getter function for the `field`.
    pub(in crate::rust) fn gen_field_getter<F>(self, field: &F, is_optional: bool) -> Function
    where
        F: WithFieldName + WithTypeTag,
    {
        let field_name: String = self.field_name(field);
        let field_type: RustType = self.field_type(field, is_optional);
        let signature: Signature = Signature::from(field_name.clone())
            .with_receiver(Receiver::Borrowed)
            .with_result(field_type);
        Function::from(signature)
            .with_attribute("must_use")
            .with_access(Public)
            .with_comment(format!(" Gets the field: `{}`.", field.field_name()))
            .with_literal(self.reference_expression(field, is_optional))
    }
}
