use crate::common::impl_with_comments;
use crate::{FieldName, FieldNameRef, TypeTag, WithFieldName, WithTypeTag};

/// A struct field.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StructField {
    comments: Vec<String>,
    field_name: FieldName,
    type_tag: TypeTag,
    is_optional: bool,
}

impl_with_comments!(StructField);

impl StructField {
    //! Construction

    /// Creates a new struct field.
    pub fn new<N, T>(field_name: N, type_tag: T) -> Self
    where
        N: Into<FieldName>,
        T: Into<TypeTag>,
    {
        Self {
            comments: Vec::default(),
            field_name: field_name.into(),
            type_tag: type_tag.into(),
            is_optional: false,
        }
    }
}

impl StructField {
    //! Optionality

    /// Checks if the field is optional.
    pub fn is_optional(&self) -> bool {
        self.is_optional
    }

    /// Sets if the field is optional.
    pub fn set_optional(&mut self, is_optional: bool) {
        self.is_optional = is_optional;
    }

    /// Sets if the field is optional.
    #[must_use]
    pub fn with_optional(mut self, is_optional: bool) -> Self {
        self.set_optional(is_optional);
        self
    }
}

impl WithFieldName for StructField {
    fn field_name(&self) -> FieldNameRef<'_> {
        self.field_name.to_ref()
    }
}

impl WithTypeTag for StructField {
    fn type_tag(&self) -> &TypeTag {
        &self.type_tag
    }
}
