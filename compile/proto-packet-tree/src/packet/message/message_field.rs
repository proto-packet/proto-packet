use crate::common::impl_with_comments;
use crate::{FieldName, FieldNameRef, TypeTag, WithFieldName, WithTypeTag};
use proto_packet::io::{TagNumber, WithTagNumber};

/// A message field.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MessageField {
    comments: Vec<String>,
    field_name: FieldName,
    type_tag: TypeTag,
    tag_number: TagNumber,
}

impl_with_comments!(MessageField);

impl MessageField {
    //! Construction

    /// Creates a new message field.
    pub fn new<N, T>(field_name: N, type_tag: T, tag_number: TagNumber) -> Self
    where
        N: Into<FieldName>,
        T: Into<TypeTag>,
    {
        Self {
            comments: Vec::default(),
            field_name: field_name.into(),
            type_tag: type_tag.into(),
            tag_number,
        }
    }
}

impl WithFieldName for MessageField {
    fn field_name(&self) -> FieldNameRef<'_> {
        self.field_name.to_ref()
    }
}

impl WithTypeTag for MessageField {
    fn type_tag(&self) -> &TypeTag {
        &self.type_tag
    }
}

impl WithTagNumber for MessageField {
    fn tag_number(&self) -> TagNumber {
        self.tag_number
    }
}
