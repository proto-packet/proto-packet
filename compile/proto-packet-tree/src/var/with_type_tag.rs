use crate::TypeTag;

/// An element with a type tag.
pub trait WithTypeTag: Sized {
    /// Gets the type tag.
    fn type_tag(&self) -> &TypeTag;

    /// Sets the type tag.
    fn set_type_tag(&mut self, type_tag: TypeTag);

    /// Sets the type tag.
    #[must_use]
    fn with_type_tag(mut self, type_tag: TypeTag) -> Self {
        self.set_type_tag(type_tag);
        self
    }
}
