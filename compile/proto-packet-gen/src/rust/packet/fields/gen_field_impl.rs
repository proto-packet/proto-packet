use crate::rust::GenRust;
use code_gen::rust::{ImplBlock, WithComments, WithFunctions};
use proto_packet_tree::{WithFieldName, WithTypeName, WithTypeTag};

impl GenRust<'_> {
    //! Gen Field Impl

    /// Generates the impl block for the `field`.
    pub(in crate::rust) fn gen_field_impl<T, F>(
        self,
        type_dec: &T,
        field: &F,
        is_optional: bool,
    ) -> ImplBlock
    where
        T: WithTypeName,
        F: WithFieldName + WithTypeTag,
    {
        ImplBlock::from(self.type_name(type_dec))
            .with_comment(format!(" Field: `{}`", field.field_name()))
            .with_function(self.gen_field_getter(field, is_optional))
            .with_function(self.gen_field_setter(field, is_optional))
            .with_function(self.gen_field_builder(field, is_optional))
    }
}
