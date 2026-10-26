use crate::{ModName, ModNameRef};
use custom_string::{ValidationError, custom_string};

custom_string!(
    #[doc = "A module path. (ex: the.mod.path)"],
    ModPath,
    validate_mod_path
);

/// Validates the `mod_path`.
fn validate_mod_path(mod_path: &str) -> Result<(), &'static str> {
    if mod_path.is_empty() {
        Err("mod paths cannot be empty")
    } else {
        for mod_name in mod_path.split('.') {
            ModName::validate(mod_name).map_err(ValidationError::message)?;
        }
        Ok(())
    }
}

impl ModPath {
    //! Properties

    /// Gets the mod names.
    pub fn mod_names(&self) -> impl Iterator<Item = ModNameRef<'_>> {
        self.to_ref().mod_names()
    }
}

impl<'a> ModPathRef<'a> {
    //! Properties

    /// Gets the mod names.
    pub fn mod_names(self) -> impl Iterator<Item = ModNameRef<'a>> {
        self.value
            .split('.')
            .map(|mod_name| ModNameRef::new(mod_name).expect("mod paths have valid mod names"))
    }
}

impl ModPath {
    //! Mutations

    /// Appends the `mod_name`.
    #[must_use]
    pub fn with_appended(mut self, mod_name: ModNameRef<'_>) -> Self {
        self.value.push('.');
        self.value.push_str(&mod_name);
        self
    }
}
