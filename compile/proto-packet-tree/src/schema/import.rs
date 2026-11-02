use crate::{ModPathRef, QualifiedName, TypeNameRef, WithTypeName};

/// An import. (ex: `import the.mod.path.TypeName;`)
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Import {
    name: QualifiedName,
}

impl Import {
    //! Construction

    /// Creates a new import of the `type_name` in the `mod_path`.
    pub fn new(mod_path: ModPathRef<'_>, type_name: TypeNameRef<'_>) -> Self {
        Self {
            name: QualifiedName::from_parts(mod_path, type_name),
        }
    }
}

impl Import {
    //! Properties

    /// Gets the qualified name. (always with a mod path)
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }

    /// Gets the effective name. (the type name)
    pub fn effective_name(&self) -> TypeNameRef<'_> {
        self.name.type_name()
    }
}
