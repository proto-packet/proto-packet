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

    /// The maximum valid tag number. (65,535)
    pub const MAX: u16 = u16::MAX;

    /// Checks if the `value` is valid.
    pub const fn is_valid(value: u16) -> bool {
        value >= Self::MIN
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

    /// Creates a new tag number from the `value`, without validating it.
    ///
    /// # Safety
    /// The `value` must be valid.
    pub const unsafe fn new_unchecked(value: u16) -> Self {
        Self { value }
    }

    /// Creates a new tag number from the `value`.
    ///
    /// Returns `None` if the `value` is invalid.
    pub const fn from_u32(value: u32) -> Option<Self> {
        if value <= Self::MAX as u32 {
            Self::new(value as u16)
        } else {
            None
        }
    }
}

impl From<TagNumber> for u16 {
    fn from(tag_number: TagNumber) -> Self {
        tag_number.value
    }
}

impl TagNumber {
    //! Properties

    /// Gets the value.
    pub const fn value(self) -> u16 {
        self.value
    }
}

impl Display for TagNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value, f)
    }
}
