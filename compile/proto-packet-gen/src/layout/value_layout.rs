use crate::layout::BitLayout;

/// The place of a field's value in a struct.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValueLayout {
    /// A `bool` bit in the bit group.
    Bit(BitLayout),

    /// A fixed-size value in the fixed block.
    Fixed { offset: usize, size: usize },

    /// A variable-size value, the entry at the `index` of the tail.
    Entry { index: usize },
}
