use crate::io::TagNumber;

/// A message field the message doesn't know.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct UnknownField {
    tag_number: TagNumber,
    bytes: Vec<u8>,
}

impl UnknownField {
    //! Construction

    /// Creates a new unknown field.
    pub const fn new(tag_number: TagNumber, bytes: Vec<u8>) -> Self {
        Self { tag_number, bytes }
    }
}

impl UnknownField {
    //! Properties

    /// Gets the tag number.
    pub const fn tag_number(&self) -> TagNumber {
        self.tag_number
    }

    /// Gets the encoded value.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
