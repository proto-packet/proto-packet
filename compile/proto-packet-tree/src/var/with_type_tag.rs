use crate::TypeTag;

/// An element with a type tag.
pub trait WithTypeTag {
    /// Gets the type tag.
    fn type_tag(&self) -> &TypeTag;
}
