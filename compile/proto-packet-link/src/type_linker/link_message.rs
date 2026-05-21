use crate::{LinkError, TypeLinker};
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{
    Message, MessageField, TypeName, TypeTag, WithComments, WithFieldName, WithTypeName,
    WithTypeTag,
};

impl TypeLinker<'_> {
    //! Messages

    /// Links the message `m`.
    pub(in crate::type_linker) fn link_message(self, m: &Message) -> Result<Message, LinkError> {
        let type_name: TypeName = m.type_name().into_owned();
        let mut linked: Message = Message::from(type_name);
        for comment in m.comments() {
            linked.add_comment(comment);
        }
        for field in m.fields() {
            let linked_field: MessageField = self.link_message_field(field)?;
            linked
                .add_field(linked_field)
                .expect("duplicate message field")
        }
        Ok(linked)
    }

    /// Links the message `field`.
    fn link_message_field(self, field: &MessageField) -> Result<MessageField, LinkError> {
        let type_tag: TypeTag = self.link_type_tag(field.type_tag())?;
        let mut linked: MessageField =
            MessageField::new(field.field_name(), type_tag, field.tag_number());
        for comment in field.comments() {
            linked.add_comment(comment);
        }
        Ok(linked)
    }
}
