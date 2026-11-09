use std::fmt::{Display, Formatter};

/// An error from an edit that would change a length in place. The bytes are left unchanged.
///
/// # Note
/// - Nothing moves in place, so a value must be the length of the one it replaces.
/// - The lengths are of what's being edited. This is a struct's string, a message or a variant.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CapacityError {
    /// The length needed, in bytes.
    pub required: usize,

    /// The length available, in bytes.
    pub capacity: usize,
}

impl Display for CapacityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the value needs {} bytes, with {} available",
            self.required, self.capacity
        )
    }
}

impl std::error::Error for CapacityError {}
