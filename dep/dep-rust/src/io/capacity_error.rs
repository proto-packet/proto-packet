use std::fmt::{Display, Formatter};

/// An error from a value that doesn't fit in place. The value is left unchanged.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CapacityError {
    /// The length the tail needs, in bytes.
    pub required: usize,

    /// The length available to the tail, in bytes.
    pub capacity: usize,
}

impl Display for CapacityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the tail needs {} bytes, with {} available",
            self.required, self.capacity
        )
    }
}

impl std::error::Error for CapacityError {}
