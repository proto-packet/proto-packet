use crate::linker::schema_linker::{SchemaLinker, copy_comments};
use crate::{LinkError, Location, Member};
use proto_packet_tree::{Message, TypeTag, WithFieldName, WithTypeName, WithTypeTag};

impl SchemaLinker<'_> {
    //! Messages

    /// Links the `message`.
    pub fn link_message(self, message: &Message) -> Result<Message, LinkError> {
        let mut linked: Message = Message::from(message.type_name().into_owned());
        copy_comments(message, &mut linked);
        for field in message.fields() {
            let member: Member = Member::Field(field.field_name().into_owned());
            let location: Location = self.location(message.type_name(), member);
            let type_tag: TypeTag = self.resolver.link_type_tag(field.type_tag(), &location)?;
            linked
                .add_field(field.clone().with_type_tag(type_tag))
                .expect("linking keeps the field names & tag numbers unique");
        }
        Ok(linked)
    }
}
