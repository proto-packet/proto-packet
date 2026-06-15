use crate::rust::GenRust;
use code_gen::WithStatements;
use code_gen::rust::Access::Public;
use code_gen::rust::{
    Function, ImplBlock, RustType, Signature, WithAccess, WithComments, WithFnGenerics,
    WithFunctions, WithResult, WithVarParams,
};
use proto_packet_tree::Struct;

impl GenRust<'_> {
    //! Gen Struct: Owned Construct

    pub(super) fn gen_struct_owned_construct(self, s: &Struct) -> ImplBlock {
        ImplBlock::from(RustType::from(self.type_name(s)))
            .with_comment(" Construction")
            .with_function(self.gen_struct_owned_construct_new(s))
            .with_function(self.gen_struct_owned_construct_from(s))
    }

    fn gen_struct_owned_construct_new(self, s: &Struct) -> Function {
        let mut signature: Signature = Signature::from("new").with_result(RustType::from("Self"));
        for field in s.fields() {
            let name: String = self.field_name(field);
            let tag: RustType = self.field_type(field, field.is_optional(), false);
            signature = signature.with_param((name, tag));
        }
        Function::from(signature)
            .with_comment(format!(" Creates a new [{}].", self.type_name(s)))
            .with_access(Public)
            .with_const(true)
            .with_literal(format!(
                "Self {{ {} }}",
                s.fields()
                    .iter()
                    .map(|field| {
                        let name: String = self.field_name(field);
                        let stored: String =
                            self.into_expression_const(field, field.is_optional(), &name);
                        if stored == name {
                            name
                        } else {
                            format!("{name}: {stored}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
    }

    fn gen_struct_owned_construct_from(self, s: &Struct) -> Function {
        let mut signature: Signature = Signature::from("from").with_result(RustType::from("Self"));
        s.fields().iter().enumerate().for_each(|(i, field)| {
            let generic: String = format!("F{}", i + 1);
            let name: String = self.field_name(field);
            let tag: RustType = self.field_type(field, field.is_optional(), false);
            signature.add_generic((generic.clone(), RustType::from("Into").with_generic(tag)));
            signature.add_param((name, generic));
        });
        Function::from(signature)
            .with_comment(format!(" Creates a new [{}].", self.type_name(s)))
            .with_access(Public)
            .with_literal(format!(
                "Self::new({})",
                s.fields()
                    .iter()
                    .map(|field| self.field_name(field))
                    .map(|name| format!("{}.into()", name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
    }
}
