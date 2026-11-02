use crate::Location;
use proto_packet_tree::QualifiedName;
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
        }
    }
}

impl std::error::Error for LinkError {}
