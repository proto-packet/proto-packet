use std::fmt::{Display, Formatter};

/// A tag number.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct TagNumber {
    value: u16,
}

impl TagNumber {
    //! Validation

    /// The minimum valid tag number. (1)
    pub const MIN: u16 = 1;

    /// The maximum valid tag number. (8,191, 13-bits)
    pub const MAX: u16 = 0x1FFF;

    /// Checks if the `value` is valid.
    pub const fn is_valid(value: u16) -> bool {
        value >= Self::MIN && value <= Self::MAX
    }
}

impl TagNumber {
    //! Construction

    /// Creates a new tag number from the `value`.
    ///
    /// Returns `None` if the `value` is invalid.
    pub const fn new(value: u16) -> Option<Self> {
        if Self::is_valid(value) {
            Some(Self { value })
        } else {
            None
        }
    }
}

impl TagNumber {
    //! Properties

    /// Gets the value.
    pub const fn value(self) -> u16 {
        self.value
    }
}

impl From<TagNumber> for u16 {
    fn from(tag_number: TagNumber) -> Self {
        tag_number.value
    }
}

impl Display for TagNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value, f)
    }
}
