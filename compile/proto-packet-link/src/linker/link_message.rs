use crate::linker::schema_linker::{SchemaLinker, copy_comments};
use crate::{LinkError, Location, Member};
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{Message, MessageField, TypeTag, WithFieldName, WithTypeName, WithTypeTag};

impl SchemaLinker<'_> {
    //! Messages

    /// Links the `message`.
    pub(in crate::linker) fn link_message(self, message: &Message) -> Result<Message, LinkError> {
        let mut linked: Message = Message::from(message.type_name().into_owned());
        copy_comments(message, &mut linked);
        for field in message.fields() {
            let member: Member = Member::Field(field.field_name().into_owned());
            let location: Location = self.location(message.type_name(), member);
            let type_tag: TypeTag = self.resolver.link_type_tag(field.type_tag(), &location)?;
            let mut linked_field: MessageField =
                MessageField::new(field.field_name(), type_tag, field.tag_number())
                    .with_boxed(field.is_boxed());
            copy_comments(field, &mut linked_field);
            linked
                .add_field(linked_field)
                .expect("linking keeps the field names & tag numbers unique");
        }
        Ok(linked)
    }
}
