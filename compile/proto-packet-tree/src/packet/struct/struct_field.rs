use crate::{FieldName, FieldNameRef, TypeTag, WithFieldName, WithTypeTag};

/// A struct field.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StructField {
    pub(crate) comments: Vec<String>,
    field_name: FieldName,
    type_tag: TypeTag,
}

impl StructField {
    //! Construction

    /// Creates a new struct field.
    pub fn new<N, T>(field_name: N, type_tag: T) -> Self
    where
        N: Into<FieldName>,
        T: Into<TypeTag>,
    {
        let field_name: FieldName = field_name.into();
        let type_tag: TypeTag = type_tag.into();
        Self {
            comments: Vec::default(),
            field_name,
            type_tag,
        }
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
