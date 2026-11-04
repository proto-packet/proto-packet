use std::fmt::{Display, Formatter};

/// An error from a value that doesn't fit in place. The bytes are left unchanged.
///
/// The lengths are of what's being edited: a struct's tail, a message or a variant. A variant's
/// string must fill the variant, so one that's too short fails with less required than available.
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
