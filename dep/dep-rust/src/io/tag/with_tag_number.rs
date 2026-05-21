use crate::io::TagNumber;

/// An element with a tag number.
pub trait WithTagNumber {
    /// Gets the tag number.
    #[must_use]
    fn tag_number(&self) -> TagNumber;
}
