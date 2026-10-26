use std::fmt::{Display, Formatter};

/// An error generating code.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum GenError {
    /// Two items of a module have the same generated module name. (ex: `FooBar` & `Foo_Bar`)
    ///
    /// The module name `mod` also conflicts, with the parent module's own `mod.rs` file.
    ModNameConflict { mod_path: String, mod_name: String },
}

impl Display for GenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModNameConflict { mod_path, mod_name } => write!(
                f,
                "the generated module name `{}` conflicts in `{}`",
                mod_name, mod_path
            ),
        }
    }
}

impl std::error::Error for GenError {}
