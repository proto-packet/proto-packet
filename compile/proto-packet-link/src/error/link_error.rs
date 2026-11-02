use crate::Location;
use proto_packet_tree::{ModPath, QualifiedName};
use std::fmt::{Display, Formatter};

/// An error linking a project.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum LinkError {
    /// A named type that is not declared in the project.
    UnresolvedType {
        location: Location,
        name: QualifiedName,
    },

    /// A named type that is a service instead of a data type.
    ServiceAsType {
        location: Location,
        name: QualifiedName,
    },

    /// An import of a type that is not declared in the project.
    UnresolvedImport {
        mod_path: ModPath,
        name: QualifiedName,
    },

    /// A loop of types without a `box`, closed at the location. (ex: `[a.A, a.B, a.A]`)
    UnboxedLoop {
        location: Location,
        names: Vec<QualifiedName>,
    },
}

impl Display for LinkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnresolvedType { location, name } => {
                write!(f, "unresolved type `{}` in {}", name, location)
            }
            Self::ServiceAsType { location, name } => {
                write!(
                    f,
                    "the service `{}` is used as a type in {}",
                    name, location
                )
            }
            Self::UnresolvedImport { mod_path, name } => {
                write!(f, "unresolved import `{}` in `{}`", name, mod_path)
            }
            Self::UnboxedLoop { location, names } => {
                let names: Vec<String> = names.iter().map(|name| format!("`{}`", name)).collect();
                write!(
                    f,
                    "the types {} form a loop without a `box`, closed by {}",
                    names.join(" -> "),
                    location
                )
            }
        }
    }
}

impl std::error::Error for LinkError {}
