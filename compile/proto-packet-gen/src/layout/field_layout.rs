/// The place of a field in the head of its struct's encoding.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FieldLayout {
    /// A fixed-size value, of the `size`, at the `offset`.
    Fixed { offset: usize, size: usize },

    /// The `u32` length of a variable-size value, at the `offset`. (its value is in the data)
    Length { offset: usize },

    /// A `bool`, the `bit` of the byte at the `offset`. (bit 0 is the least significant)
    Bit { offset: usize, bit: u8 },
}
