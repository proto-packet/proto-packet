use crate::rust::GenRust;
use code_gen::WithStatements;
use code_gen::rust::Access::Public;
use code_gen::rust::{
    Function, ImplBlock, RustType, Signature, WithAccess, WithComments, WithFunctions, WithResult,
    WithVarParams,
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
        let mut sig: Signature = Signature::from("new").with_result(RustType::from("Self"));
        for field in s.fields() {
            let name: String = self.field_name(field);
            let tag: RustType = self.field_type(field, false);
            sig = sig.with_param((name, tag));
        }
        Function::from(sig)
            .with_comment(format!(" Creates a new [{}].", self.type_name(s)))
            .with_access(Public)
            .with_const(true)
            .with_literal(self.gen_struct_owned_construct_new_body(s))
    }

    fn gen_struct_owned_construct_new_body(self, s: &Struct) -> String {
        let inits: Vec<String> = s.fields().iter().map(|f| self.field_name(f)).collect();
        if inits.is_empty() {
            String::from("Self {}")
        } else {
            format!("Self {{ {} }}", inits.join(", "))
        }
    }

    fn gen_struct_owned_construct_from(self, s: &Struct) -> Function {
        let mut sig: Signature = Signature::from("from").with_result(RustType::from("Self"));
        for field in s.fields() {
            let name: String = self.field_name(field);
            let tag: RustType = self.field_type(field, false);
            let into_tag: RustType = RustType::from("impl Into").with_generic(tag);
            sig = sig.with_param((name, into_tag));
        }
        Function::from(sig)
            .with_comment(format!(" Creates a new [{}].", self.type_name(s)))
            .with_access(Public)
            .with_literal(self.gen_struct_owned_construct_from_body(s))
    }

    fn gen_struct_owned_construct_from_body(self, s: &Struct) -> String {
        let inits: Vec<String> = s
            .fields()
            .iter()
            .map(|f| {
                let name: String = self.field_name(f);
                format!("{name}: {name}.into()")
            })
            .collect();
        if inits.is_empty() {
            String::from("Self {}")
        } else {
            format!("Self {{ {} }}", inits.join(", "))
        }
    }
}
