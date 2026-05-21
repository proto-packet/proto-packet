use clerr::{Code, Report};
use proto_packet_tree::{ModPath, TypeName};
use std::fmt::{Display, Formatter};

/// An error linking a schema file.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum LinkError {
    /// The `type_name` was not resolvable.
    UnresolvableName {
        mod_path: ModPath,
        type_name: TypeName,
    },
}

impl LinkError {
    //! Report

    /// Gets the command-line report.
    #[must_use]
    pub fn to_report(&self) -> Report {
        match self {
            Self::UnresolvableName {
                mod_path,
                type_name,
            } => Report::from(Code::error(
                "L_UNRESOLVABLE_NAME",
                format!("could not resolve type '{}' in '{}'", type_name, mod_path),
            )),
        }
    }
}

impl Display for LinkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for LinkError {}
