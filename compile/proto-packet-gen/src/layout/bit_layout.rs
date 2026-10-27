/// The place of a bit in the head of its struct's encoding: the `bit` of the byte at the `offset`.
///
/// Bit 0 is the least significant.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BitLayout {
    pub offset: usize,
    pub bit: u8,
}
