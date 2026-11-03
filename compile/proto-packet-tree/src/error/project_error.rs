use crate::ModPath;
use std::fmt::{Display, Formatter};

/// An error constructing a project.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ProjectError {
    /// A schema file with the same mod path already exists.
    DuplicateModPath { mod_path: ModPath },
}

impl Display for ProjectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateModPath { mod_path } => write!(f, "duplicate mod path `{}`", mod_path),
        }
    }
}

impl std::error::Error for ProjectError {}
