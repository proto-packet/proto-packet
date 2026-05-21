use crate::{QualifiedName, TypeNameRef, WithTypeName};

/// An import.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Import {
    name: QualifiedName,
}

impl From<QualifiedName> for Import {
    fn from(name: QualifiedName) -> Self {
        Self { name }
    }
}

impl Import {
    //! Properties

    /// Gets the qualified name.
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }

    /// Gets the effective name. (the qualified name's type name)
    pub fn effective_name(&self) -> TypeNameRef<'_> {
        self.name.type_name()
    }
}
