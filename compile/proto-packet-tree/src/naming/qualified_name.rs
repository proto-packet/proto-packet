use crate::{ModPath, ModPathRef, TypeName, TypeNameRef, WithTypeName};
use custom_string::custom_string;

custom_string!(
    #[doc = "A qualified name. (ex: TypeName or the.mod.path.TypeName)"],
    QualifiedName,
    validate_qualified_name
);

/// Validates the `qualified_name`.
pub fn validate_qualified_name(qualified_name: &str) -> Result<(), &'static str> {
    if let Some(last_dot) = qualified_name.rfind('.') {
        ModPath::validate(&qualified_name[..last_dot]).map_err(|e| e.message())?;
        TypeName::validate(&qualified_name[(last_dot + 1)..]).map_err(|e| e.message())?;
    } else {
        TypeName::validate(qualified_name).map_err(|e| e.message())?;
    }
    Ok(())
}

impl QualifiedName {
    //! Properties

    /// Splits the `qualified_name` into the optional mod path and the type name.
    ///
    /// # Safety
    /// The `qualified_name` must be valid.
    unsafe fn mod_path_and_type_name_internal(
        qualified_name: &str,
    ) -> (Option<ModPathRef<'_>>, TypeNameRef<'_>) {
        if let Some(last_dot) = qualified_name.rfind('.') {
            let mod_path: &str = &qualified_name[..last_dot];
            let type_name: &str = &qualified_name[(last_dot + 1)..];
            unsafe {
                (
                    Some(ModPathRef::new_unchecked(mod_path)),
                    TypeNameRef::new_unchecked(type_name),
                )
            }
        } else {
            unsafe { (None, TypeNameRef::new_unchecked(qualified_name)) }
        }
    }

    /// Gets the optional mod path and the type name.
    pub fn mod_path_and_type_name(&self) -> (Option<ModPathRef<'_>>, TypeNameRef<'_>) {
        unsafe { Self::mod_path_and_type_name_internal(self.value.as_str()) }
    }

    /// Gets the optional mod path.
    pub fn mod_path(&self) -> Option<ModPathRef<'_>> {
        self.mod_path_and_type_name().0
    }
}

impl WithTypeName for QualifiedName {
    fn type_name(&self) -> TypeNameRef<'_> {
        self.mod_path_and_type_name().1
    }
}

impl<'a> QualifiedNameRef<'a> {
    //! Properties

    /// Gets the optional mod path and the type name.
    pub fn mod_path_and_type_name(&self) -> (Option<ModPathRef<'a>>, TypeNameRef<'a>) {
        unsafe { QualifiedName::mod_path_and_type_name_internal(self.value) }
    }

    /// Gets the optional mod path.
    pub fn mod_path(&self) -> Option<ModPathRef<'a>> {
        self.mod_path_and_type_name().0
    }

    /// Gets the type name.
    pub fn type_name(&self) -> TypeNameRef<'a> {
        self.mod_path_and_type_name().1
    }
}

impl WithTypeName for QualifiedNameRef<'_> {
    fn type_name(&self) -> TypeNameRef<'_> {
        self.mod_path_and_type_name().1
    }
}
