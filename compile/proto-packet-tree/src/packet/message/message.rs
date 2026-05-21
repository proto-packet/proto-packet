use crate::{MessageField, TreeError, TypeName, TypeNameRef, WithFieldName, WithTypeName};
use proto_packet::io::{TagNumber, WithTagNumber};

/// A message.
///
/// # Invariants
/// 1. No two fields can have the same name.
/// 2. No two fields can have the same tag number.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Message {
    pub(crate) comments: Vec<String>,
    type_name: TypeName,
    fields: Vec<MessageField>,
}

impl From<TypeName> for Message {
    fn from(type_name: TypeName) -> Self {
        Self {
            comments: Vec::default(),
            type_name,
            fields: Vec::default(),
        }
    }
}

impl Message {
    //! Fields

    /// Gets the fields.
    pub fn fields(&self) -> &[MessageField] {
        self.fields.as_slice()
    }

    /// Gets the optional field with the `field_name`.
    pub fn field_with_name<S>(&self, field_name: S) -> Option<&MessageField>
    where
        S: AsRef<str>,
    {
        self.fields
            .iter()
            .find(|field| field.field_name() == field_name)
    }

    /// Gets the optional field with the `tag_number`.
    pub fn field_with_tag_number(&self, tag_number: TagNumber) -> Option<&MessageField> {
        self.fields
            .iter()
            .find(|field| field.tag_number() == tag_number)
    }

    /// Adds the `field`.
    pub fn add_field<F>(&mut self, field: F) -> Result<(), TreeError>
    where
        F: Into<MessageField>,
    {
        let field: MessageField = field.into();

        if self.field_with_name(field.field_name()).is_some() {
            return Err(TreeError::DuplicateFieldName {
                field_name: field.field_name().into_owned(),
                type_name: self.type_name.clone(),
            });
        }
        if self.field_with_tag_number(field.tag_number()).is_some() {
            return Err(TreeError::DuplicateTagNumber {
                type_name: self.type_name.clone(),
                tag_number: field.tag_number(),
            });
        }

        self.fields.push(field);

        Ok(())
    }

    /// Adds the `field`.
    pub fn with_field<F>(mut self, field: F) -> Result<Self, TreeError>
    where
        F: Into<MessageField>,
    {
        self.add_field(field)?;
        Ok(self)
    }
}

impl WithTypeName for Message {
    fn type_name(&self) -> TypeNameRef<'_> {
        self.type_name.to_ref()
    }
}
