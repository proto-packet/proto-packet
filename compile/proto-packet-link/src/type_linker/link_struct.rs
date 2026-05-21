use crate::{LinkError, TypeLinker};
use proto_packet_tree::{
    Struct, StructField, TypeName, TypeTag, WithComments, WithFieldName, WithTypeName, WithTypeTag,
};

impl TypeLinker<'_> {
    //! Structs

    /// Links the struct `s`.
    pub(in crate::type_linker) fn link_struct(self, s: &Struct) -> Result<Struct, LinkError> {
        let type_name: TypeName = s.type_name().into_owned();
        let mut linked: Struct = Struct::from(type_name);
        for comment in s.comments() {
            linked.add_comment(comment);
        }
        for field in s.fields() {
            let linked_field: StructField = self.link_struct_field(field)?;
            linked
                .add_field(linked_field)
                .expect("duplicate struct field")
        }
        Ok(linked)
    }

    /// Links the struct `field`.
    fn link_struct_field(self, field: &StructField) -> Result<StructField, LinkError> {
        let type_tag: TypeTag = self.link_type_tag(field.type_tag())?;
        let mut linked: StructField = StructField::new(field.field_name(), type_tag);
        for comment in field.comments() {
            linked.add_comment(comment);
        }
        Ok(linked)
    }
}
