use crate::common::impl_with_comments;
use crate::{StructField, TreeError, TypeName, TypeNameRef, WithFieldName, WithTypeName};

/// A struct.
///
/// # Invariants
/// 1. No two fields can have the same name.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Struct {
    comments: Vec<String>,
    type_name: TypeName,
    fields: Vec<StructField>,
}

impl_with_comments!(Struct);

impl From<TypeName> for Struct {
    fn from(type_name: TypeName) -> Self {
        Self {
            comments: Vec::default(),
            type_name,
            fields: Vec::default(),
        }
    }
}

impl Struct {
    //! Fields

    /// Gets the fields.
    pub fn fields(&self) -> &[StructField] {
        self.fields.as_slice()
    }

    /// Gets the optional field with the `field_name`.
    pub fn field_with_name<S>(&self, field_name: S) -> Option<&StructField>
    where
        S: AsRef<str>,
    {
        self.fields
            .iter()
            .find(|field| field.field_name() == field_name)
    }

    /// Adds the `field`.
    pub fn add_field(&mut self, field: StructField) -> Result<(), TreeError> {
        if self.field_with_name(field.field_name()).is_some() {
            return Err(TreeError::DuplicateFieldName {
                type_name: self.type_name.clone(),
                field_name: field.field_name().into_owned(),
            });
        }

        self.fields.push(field);
        Ok(())
    }

    /// Adds the `field`.
    pub fn with_field(mut self, field: StructField) -> Result<Self, TreeError> {
        self.add_field(field)?;
        Ok(self)
    }
}

impl WithTypeName for Struct {
    fn type_name(&self) -> TypeNameRef<'_> {
        self.type_name.to_ref()
    }
}
