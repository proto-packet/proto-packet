use crate::rust::GenRust;
use code_gen::rust::Access::Public;
use code_gen::rust::{
    Function, Receiver, RustType, Signature, Var, WithAccess, WithComments, WithFnGenerics,
    WithReceiver, WithResult, WithVarParams,
};
use code_gen::{Expression, WithStatements};
use proto_packet_tree::{WithFieldName, WithTypeTag};

impl GenRust<'_> {
    //! Gen Field Setter

    /// Generates the setter function for the `field`.
    pub(in crate::rust) fn gen_field_setter<F>(self, field: &F, is_optional: bool) -> Function
    where
        F: WithFieldName + WithTypeTag,
    {
        let field_name: String = self.field_name(field);
        let field_type: RustType = self.field_type(field, is_optional, false);
        let field_type_code: String = field_type.to_code();
        let signature: Signature = if is_optional {
            Signature::from(format!("set_{}", field_name))
                .with_receiver(Receiver::BorrowedMut)
                .with_generic(Var::from(("F", format!("Into<{field_type_code}>"))))
                .with_param((field_name.clone(), RustType::from("F")))
                .with_result(field_type)
        } else {
            Signature::from(format!("set_{}", field_name))
                .with_receiver(Receiver::BorrowedMut)
                .with_param((field_name.clone(), field_type.clone()))
                .with_result(field_type)
        };
        let stored: String = self.into_expression(field, is_optional, &field_name);
        let replace: String = format!("std::mem::replace(&mut self.{field_name}, {stored})");
        let body: String = self.into_expression(field, is_optional, &replace);
        let mut func: Function =
            Function::from(signature)
                .with_access(Public)
                .with_comment(format!(
                    " Sets the field: `{}`. Returns the previous value.",
                    field.field_name()
                ));
        if is_optional {
            func = func.with_semi(format!(
                "let {field_name}: {field_type_code} = {field_name}.into()"
            ));
        }
        func.with_literal(body)
    }
}
