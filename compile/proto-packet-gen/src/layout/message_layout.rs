use crate::layout::primitive_size;
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{Message, Project, SpecialType, TypeDec, TypeTag, WithTypeTag};

/// The layout of a message field.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MessageFieldLayout {
    /// The index of the field, in declaration order.
    pub index: usize,

    /// The tag number.
    pub tag: u16,

    /// The size of the value, if its type is a fixed-size scalar.
    pub size: Option<usize>,
}

/// The layout of a message's encoding. (see `format/binary.md`)
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MessageLayout {
    /// The fields, in tag order.
    pub fields: Vec<MessageFieldLayout>,
}

impl MessageLayout {
    //! Construction

    /// Gets the layout of the `message` of the linked `project`.
    pub fn new(project: &Project, message: &Message) -> Self {
        let mut fields: Vec<MessageFieldLayout> = message
            .fields()
            .iter()
            .enumerate()
            .map(|(index, field)| MessageFieldLayout {
                index,
                tag: field.tag_number().value(),
                size: scalar_size(project, field.type_tag()),
            })
            .collect();
        fields.sort_by_key(|field| field.tag);
        Self { fields }
    }
}

/// Gets the size of a message value or variant value of the `tag`, if it's a fixed-size scalar.
pub fn scalar_size(project: &Project, tag: &TypeTag) -> Option<usize> {
    match tag {
        TypeTag::Primitive(primitive) => Some(primitive_size(*primitive)),
        TypeTag::Special(SpecialType::Uuid) => Some(16),
        TypeTag::Special(SpecialType::String) => None,
        TypeTag::Named(name) => match project.type_dec_with_name(name.to_ref()) {
            Some(TypeDec::Enum(_)) => Some(2),
            _ => None,
        },
        TypeTag::List(_) => None,
        TypeTag::Map { .. } => todo!(),
    }
}
