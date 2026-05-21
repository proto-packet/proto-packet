use crate::io::WithTagNumber;
use std::fmt::{Debug, Display, Formatter};

/// A tag number.
#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
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
    #[must_use]
    pub const fn is_valid(value: u16) -> bool {
        value >= Self::MIN && value <= Self::MAX
    }
}

impl TagNumber {
    //! Construction

    /// Creates a new [TagNumber] from the `value`.
    ///
    /// Returns `None` when the `value` is invalid.
    pub const fn new(value: u16) -> Option<Self> {
        if Self::is_valid(value) {
            Some(unsafe { Self::new_unchecked(value) })
        } else {
            None
        }
    }

    /// Creates a new [TagNumber] from the `value`.
    ///
    /// # Safety
    /// The `value` must be valid.
    #[must_use]
    pub const unsafe fn new_unchecked(value: u16) -> Self {
        Self { value }
    }
}

impl TagNumber {
    //! Properties

    /// Gets the value.
    #[must_use]
    pub fn value(self) -> u16 {
        self.value
    }
}

impl From<TagNumber> for u16 {
    fn from(tag: TagNumber) -> Self {
        tag.value
    }
}

impl WithTagNumber for TagNumber {
    fn tag_number(&self) -> TagNumber {
        *self
    }
}

impl Debug for TagNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.value, f)
    }
}

impl Display for TagNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value, f)
    }
}
