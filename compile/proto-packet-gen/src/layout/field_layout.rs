use crate::layout::{BitLayout, ValueLayout};

/// The layout of a field in the head of its struct's encoding.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FieldLayout {
    /// The place of the field's value.
    pub value: ValueLayout,

    /// The presence bit of an optional field whose type has no absent value. (ex: an `optional u32`)
    ///
    /// An optional enum is absent as the tag number 0, & an optional variable-size value as the
    /// length `u32::MAX`, so they have no presence bit.
    pub presence: Option<BitLayout>,
}
