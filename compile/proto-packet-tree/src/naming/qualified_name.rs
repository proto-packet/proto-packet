use crate::{ModPath, ModPathRef, TypeName, TypeNameRef, WithTypeName};
use custom_string::{ValidationError, custom_string};

custom_string!(
    #[doc = "A qualified name. (ex: TypeName or the.mod.path.TypeName)"],
    QualifiedName,
    validate_qualified_name
);

/// Validates the `qualified_name`.
fn validate_qualified_name(qualified_name: &str) -> Result<(), &'static str> {
    let (mod_path, type_name) = split(qualified_name);
    if let Some(mod_path) = mod_path {
        ModPath::validate(mod_path).map_err(ValidationError::message)?;
    }
    TypeName::validate(type_name).map_err(ValidationError::message)?;
    Ok(())
}

/// Splits the `qualified_name` into the optional mod path & the type name.
fn split(qualified_name: &str) -> (Option<&str>, &str) {
    match qualified_name.rsplit_once('.') {
        Some((mod_path, type_name)) => (Some(mod_path), type_name),
        None => (None, qualified_name),
    }
}

impl QualifiedName {
    //! Construction

    /// Creates a new qualified name from the `mod_path` & `type_name`.
    pub fn from_parts(mod_path: ModPathRef<'_>, type_name: TypeNameRef<'_>) -> Self {
        Self {
            value: format!("{}.{}", mod_path, type_name),
        }
    }
}

impl QualifiedName {
    //! Properties

    /// Gets the optional mod path.
    pub fn mod_path(&self) -> Option<ModPathRef<'_>> {
        self.to_ref().mod_path()
    }
}

impl<'a> QualifiedNameRef<'a> {
    //! Properties

    /// Gets the optional mod path.
    pub fn mod_path(self) -> Option<ModPathRef<'a>> {
        let (mod_path, _) = split(self.value);
        mod_path.map(|mod_path| {
            ModPathRef::new(mod_path).expect("qualified names have valid mod paths")
        })
    }

    /// Gets the type name.
    pub fn type_name(self) -> TypeNameRef<'a> {
        let (_, type_name) = split(self.value);
        TypeNameRef::new(type_name).expect("qualified names have valid type names")
    }
}

impl WithTypeName for QualifiedName {
    fn type_name(&self) -> TypeNameRef<'_> {
        self.to_ref().type_name()
    }
}

impl WithTypeName for QualifiedNameRef<'_> {
    fn type_name(&self) -> TypeNameRef<'_> {
        QualifiedNameRef::type_name(*self)
    }
}
