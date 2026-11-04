use crate::linker::schema_linker::{SchemaLinker, copy_comments};
use crate::{LinkError, Location, Member};
use proto_packet_tree::{TypeTag, Variant, WithCaseName, WithTypeName, WithTypeTag};

impl SchemaLinker<'_> {
    //! Variants

    /// Links the `variant`.
    pub fn link_variant(self, variant: &Variant) -> Result<Variant, LinkError> {
        let mut linked: Variant = Variant::from(variant.type_name().into_owned());
        copy_comments(variant, &mut linked);
        for case in variant.cases() {
            let member: Member = Member::Case(case.case_name().into_owned());
            let location: Location = self.location(variant.type_name(), member);
            let type_tag: TypeTag = self.resolver.link_type_tag(case.type_tag(), &location)?;
            linked
                .add_case(case.clone().with_type_tag(type_tag))
                .expect("linking keeps the case names & tag numbers unique");
        }
        Ok(linked)
    }
}
