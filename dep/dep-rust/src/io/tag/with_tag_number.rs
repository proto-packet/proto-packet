use crate::io::TagNumber;

/// An element with a tag number.
pub trait WithTagNumber {
    /// Gets the tag number.
    fn tag_number(&self) -> TagNumber;
}
