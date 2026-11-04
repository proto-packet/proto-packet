use crate::layout::is_encodable;
use proto_packet::io::WithTagNumber;
use proto_packet_tree::{
    Message, PrimitiveType, Project, SpecialType, TypeDec, TypeTag, WithTypeTag,
};

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

/// The layout of a message's encoding. (see `FORMAT.md`)
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MessageLayout {
    /// The fields, in tag order.
    pub fields: Vec<MessageFieldLayout>,
}

impl MessageLayout {
    //! Construction

    /// Gets the layout of the `message` of the linked `project`.
    ///
    /// Returns `None` if the `message` contains a type with no encoding. (ex: a service)
    pub fn new(project: &Project, message: &Message) -> Option<Self> {
        let mut visited: Vec<_> = Vec::default();
        let is_encodable: bool = message
            .fields()
            .iter()
            .all(|field| is_encodable(project, field.type_tag(), &mut visited));
        if !is_encodable {
            return None;
        }
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
        Some(Self { fields })
    }
}

/// Gets the size of a message value or variant value of the `tag`, if it's a fixed-size scalar.
pub fn scalar_size(project: &Project, tag: &TypeTag) -> Option<usize> {
    match tag {
        TypeTag::Primitive(primitive) => Some(match primitive {
            PrimitiveType::Boolean | PrimitiveType::UnsignedInt8 => 1,
            PrimitiveType::UnsignedInt16 => 2,
            PrimitiveType::UnsignedInt32 => 4,
            PrimitiveType::UnsignedInt64 => 8,
            PrimitiveType::UnsignedInt128 => 16,
        }),
        TypeTag::Special(SpecialType::Uuid) => Some(16),
        TypeTag::Special(SpecialType::String) => None,
        TypeTag::Named(name) => match project.type_dec_with_name(name.to_ref()) {
            Some(TypeDec::Enum(_)) => Some(2),
            _ => None,
        },
    }
}
