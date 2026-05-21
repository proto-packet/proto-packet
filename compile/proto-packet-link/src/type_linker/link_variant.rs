use crate::{LinkError, TypeLinker};
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{
    TypeName, TypeTag, Variant, VariantCase, WithCaseName, WithComments, WithTypeName, WithTypeTag,
};

impl TypeLinker<'_> {
    //! Variants

    /// Links the variant `v`.
    pub(in crate::type_linker) fn link_variant(self, v: &Variant) -> Result<Variant, LinkError> {
        let type_name: TypeName = v.type_name().into_owned();
        let mut linked: Variant = Variant::from(type_name);
        for comment in v.comments() {
            linked.add_comment(comment);
        }
        for case in v.cases() {
            let linked_case: VariantCase = self.link_variant_case(case)?;
            linked
                .add_case(linked_case)
                .expect("duplicate variant case")
        }
        Ok(linked)
    }

    /// Links the variant `case`.
    fn link_variant_case(self, case: &VariantCase) -> Result<VariantCase, LinkError> {
        let type_tag: TypeTag = self.link_type_tag(case.type_tag())?;
        let mut linked: VariantCase =
            VariantCase::new(case.case_name(), type_tag, case.tag_number());
        for comment in case.comments() {
            linked.add_comment(comment);
        }
        Ok(linked)
    }
}
