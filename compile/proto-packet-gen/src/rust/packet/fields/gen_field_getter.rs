use crate::rust::GenRust;
use code_gen::WithStatements;
use code_gen::rust::Access::Public;
use code_gen::rust::{
    Function, Receiver, Reference, RustType, Signature, WithAccess, WithAttributes, WithComments,
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
        let is_copy: bool = self.is_copy(field);
        let result_type: RustType = if is_copy {
            self.field_type(field, is_optional, false)
        } else {
            let inner: RustType = self
                .field_type(field, false, false)
                .to_ref(Reference::default());
            if is_optional {
                RustType::from("Option").with_generic(inner)
            } else {
                inner
            }
        };
        let body: String = if is_copy {
            self.reference_expression(field, is_optional)
        } else if is_optional {
            format!("self.{field_name}.as_ref()")
        } else {
            format!("&self.{field_name}")
        };
        let signature: Signature = Signature::from(field_name.clone())
            .with_receiver(Receiver::Borrowed)
            .with_result(result_type);
        Function::from(signature)
            .with_attribute("must_use")
            .with_access(Public)
            .with_comment(format!(" Gets the field: `{}`.", field.field_name()))
            .with_literal(body)
    }
}
