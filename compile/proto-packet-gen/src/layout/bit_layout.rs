/// The place of a bit in a struct's bit group.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BitLayout {
    /// The offset of the bit's byte in the fixed block.
    pub offset: usize,

    /// The bit of the byte. (bit 0 is the least significant)
    pub bit: u8,
}
