use crate::{ModName, ModNameRef, QualifiedName, TypeNameRef};
use custom_string::custom_string;

custom_string!(
    #[doc = "A module path. (ex: the.mod.path)"],
    ModPath,
    validate_mod_path
);

/// Validates the `mod_path`.
pub fn validate_mod_path(mod_path: &str) -> Result<(), &'static str> {
    if mod_path.is_empty() {
        Err("mod paths cannot be empty")
    } else {
        for mod_name in mod_path.split('.') {
            ModName::validate(mod_name).map_err(|e| e.message())?;
        }
        Ok(())
    }
}

impl ModPath {
    //! Properties

    /// Gets the mod names.
    ///
    /// # Safety
    /// The `mod_path` must be valid.
    unsafe fn mod_names_internal(mod_path: &str) -> impl Iterator<Item = ModNameRef<'_>> {
        mod_path
            .split('.')
            .map(|s| unsafe { ModNameRef::new_unchecked(s) })
    }

    /// Gets the mod names.
    pub fn mod_names(&self) -> impl Iterator<Item = ModNameRef<'_>> {
        unsafe { Self::mod_names_internal(self.value()) }
    }
}

impl<'a> ModPathRef<'a> {
    //! Properties

    /// Gets the mod names.
    pub fn mod_names(&self) -> impl Iterator<Item = ModNameRef<'_>> {
        unsafe { ModPath::mod_names_internal(self.value) }
    }
}

impl ModPath {
    //! Conversions

    /// Converts the mod path to a qualified name with the `type_name`.
    #[must_use]
    pub fn to_qualified_name(self, type_name: TypeNameRef) -> QualifiedName {
        let mut qualified_name: String = self.value;
        qualified_name.reserve(1 + type_name.len());
        qualified_name.push('.');
        qualified_name.push_str(type_name.as_ref());
        unsafe { QualifiedName::new_unchecked(qualified_name) }
    }
}

impl<'a> ModPathRef<'a> {
    //! Conversions

    /// Converts the mod path to a qualified name with the `type_name`.
    #[must_use]
    pub fn to_qualified_name(self, type_name: TypeNameRef) -> QualifiedName {
        let mut qualified_name: String =
            String::with_capacity(self.value.len() + 1 + type_name.len());
        qualified_name.push_str(self.value);
        qualified_name.push('.');
        qualified_name.push_str(type_name.as_ref());
        unsafe { QualifiedName::new_unchecked(qualified_name) }
    }
}

impl ModPath {
    //! Mutations

    /// Appends the `mod_name`.
    pub fn append(&mut self, mod_name: ModNameRef) {
        self.value.reserve(1 + mod_name.len());
        self.value.push('.');
        self.value.push_str(mod_name.as_ref());
    }

    /// Appends the `mod_name`.
    #[must_use]
    pub fn with_appended(mut self, mod_name: ModNameRef) -> Self {
        self.append(mod_name);
        self
    }
}

impl<'a> ModPathRef<'a> {
    //! Mutations

    /// Appends the `mod_name`.
    #[must_use]
    pub fn with_appended(self, mod_name: ModNameRef) -> ModPath {
        let mut mod_path: String = String::with_capacity(self.value.len() + 1 + mod_name.len());
        mod_path.push_str(self.value);
        mod_path.push('.');
        mod_path.push_str(mod_name.as_ref());
        unsafe { ModPath::new_unchecked(mod_path) }
    }
}
