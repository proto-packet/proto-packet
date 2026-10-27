use crate::layout::BitLayout;

/// The place of a field's value in the head of its struct's encoding.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValueLayout {
    /// A fixed-size value, of the `size`, at the `offset`.
    Fixed { offset: usize, size: usize },

    /// The `u32` length of a variable-size value, at the `offset`. (its value is in the data)
    Length { offset: usize },

    /// A `bool` bit.
    Bit(BitLayout),
}
