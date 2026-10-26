use crate::linker::schema_linker::{SchemaLinker, copy_comments};
use crate::{LinkError, Location, Member};
use proto_packet_tree::{Struct, StructField, TypeTag, WithFieldName, WithTypeName, WithTypeTag};

impl SchemaLinker<'_> {
    //! Structs

    /// Links the `structure`.
    pub(in crate::linker) fn link_struct(self, structure: &Struct) -> Result<Struct, LinkError> {
        let mut linked: Struct = Struct::from(structure.type_name().into_owned());
        copy_comments(structure, &mut linked);
        for field in structure.fields() {
            let member: Member = Member::Field(field.field_name().into_owned());
            let location: Location = self.location(structure.type_name(), member);
            let type_tag: TypeTag = self.resolver.link_type_tag(field.type_tag(), &location)?;
            let mut linked_field: StructField = StructField::new(field.field_name(), type_tag)
                .with_optional(field.is_optional())
                .with_boxed(field.is_boxed());
            copy_comments(field, &mut linked_field);
            linked
                .add_field(linked_field)
                .expect("linking keeps the field names unique");
        }
        Ok(linked)
    }
}
