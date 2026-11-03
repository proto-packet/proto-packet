//! The Rust prelude names used by the generated code.

/// A Rust prelude name used by the generated code.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum PreludeName {
    Box,
    Into,
    Option,
    Ord,
    PartialOrd,
    Result,
    Send,
    String,
    Sync,
}

impl PreludeName {
    //! Names

    /// The prelude names.
    const ALL: [Self; 9] = [
        Self::Box,
        Self::Into,
        Self::Option,
        Self::Ord,
        Self::PartialOrd,
        Self::Result,
        Self::Send,
        Self::String,
        Self::Sync,
    ];

    /// Gets the full path. (ex: `::std::option::Option`)
    const fn full_path(self) -> &'static str {
        match self {
            Self::Box => "::std::boxed::Box",
            Self::Into => "::std::convert::Into",
            Self::Option => "::std::option::Option",
            Self::Ord => "::std::cmp::Ord",
            Self::PartialOrd => "::std::cmp::PartialOrd",
            Self::Result => "::std::result::Result",
            Self::Send => "::std::marker::Send",
            Self::String => "::std::string::String",
            Self::Sync => "::std::marker::Sync",
        }
    }

    /// Gets the name. (ex: `Option`)
    fn name(self) -> &'static str {
        let path: &'static str = self.full_path();
        path.rsplit("::").next().unwrap_or(path)
    }
}

/// The prelude of a generated file.
///
/// Each file declares one type & imports nothing, so only its own type can shadow a prelude name.
/// The prelude names are bare, unless the type is named like one of them. Then every prelude name
/// in the file has its full path. (ex: `::std::option::Option` in the file of a struct `Option`)
#[derive(Copy, Clone, Debug)]
pub(crate) struct Prelude {
    is_shadowed: bool,
}

impl Prelude {
    //! Construction

    /// Creates the prelude of the file of the type with the Rust `type_name`.
    pub(crate) fn new(type_name: &str) -> Self {
        let is_shadowed: bool = PreludeName::ALL.iter().any(|name| name.name() == type_name);
        Self { is_shadowed }
    }
}

impl Prelude {
    //! Paths

    /// Gets the path of the prelude `name` in the file. (ex: `Option` or `::std::option::Option`)
    pub(crate) fn path(self, name: PreludeName) -> &'static str {
        if self.is_shadowed {
            name.full_path()
        } else {
            name.name()
        }
    }
}
