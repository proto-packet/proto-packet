use std::fmt::{Display, Formatter};

/// An error generating code.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum GenError {
    /// Two items of a module have the same generated module name. (ex: `FooBar` & `Foo_Bar`)
    ///
    /// The module name `mod` also conflicts, with the parent module's own `mod.rs` file.
    ModNameConflict { mod_path: String, mod_name: String },

    /// A field is named like a generated method of its type.
    ///
    /// (ex: the constructor `new`, or the setter `set_x` & builder `with_x` of the field `x`)
    FieldNameConflict {
        type_name: String,
        field_name: String,
    },

    /// A variant case gets the name of a generated method of its type.
    ///
    /// (ex: the getter `as_bytes` of the case `Bytes`, or the cases `FooBar` & `Foo_Bar`)
    CaseNameConflict {
        type_name: String,
        case_name: String,
    },

    /// A struct's fixed block is larger than an end offset can reach.
    StructTooLarge { type_name: String },

    /// A type is named like a generated type of its schema file. (ex: `PointRef` with `Point`)
    TypeNameConflict { mod_path: String, type_name: String },
}

impl GenError {
    //! Properties

    /// Gets the error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ModNameConflict { .. } => "G_MOD_NAME_CONFLICT",
            Self::FieldNameConflict { .. } => "G_FIELD_NAME_CONFLICT",
            Self::CaseNameConflict { .. } => "G_CASE_NAME_CONFLICT",
            Self::StructTooLarge { .. } => "G_STRUCT_TOO_LARGE",
            Self::TypeNameConflict { .. } => "G_TYPE_NAME_CONFLICT",
        }
    }
}

impl Display for GenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModNameConflict { mod_path, mod_name } => write!(
                f,
                "the generated module name `{}` conflicts in `{}`",
                mod_name, mod_path
            ),
            Self::FieldNameConflict {
                type_name,
                field_name,
            } => write!(
                f,
                "the field name `{}` conflicts with a generated method of `{}`",
                field_name, type_name
            ),
            Self::CaseNameConflict {
                type_name,
                case_name,
            } => write!(
                f,
                "the case name `{}` conflicts with a generated method of `{}`",
                case_name, type_name
            ),
            Self::StructTooLarge { type_name } => {
                write!(f, "the struct `{}` is too large to encode", type_name)
            }
            Self::TypeNameConflict {
                mod_path,
                type_name,
            } => write!(
                f,
                "the type name `{}` conflicts with a generated type in `{}`",
                type_name, mod_path
            ),
        }
    }
}

impl std::error::Error for GenError {}
