use std::fmt::{Display, Formatter};

/// A special type.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SpecialType {
    /// A 16-byte identifier.
    Uuid,

    /// A UTF-8 byte sequence.
    String,
}

impl SpecialType {
    //! Names

    /// The special types.
    const ALL: [Self; 2] = [Self::Uuid, Self::String];

    /// Gets the name. (ex: `uuid`)
    pub const fn name(self) -> &'static str {
        match self {
            Self::Uuid => "uuid",
            Self::String => "string",
        }
    }

    /// Gets the optional special type with the `name`.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|special| special.name() == name)
    }
}

impl Display for SpecialType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
