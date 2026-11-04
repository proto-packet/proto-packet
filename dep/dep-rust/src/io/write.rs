/// Rounds the `value` up to a multiple of the `alignment`, a power of 2.
#[must_use]
pub const fn align_up(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

/// Appends the `count` of zeros to the `out` bytes.
pub fn write_zeros(out: &mut Vec<u8>, count: usize) {
    out.resize(out.len() + count, 0);
}

/// Pads the `out` bytes with zeros, so the bytes after the `start` are a multiple of the
/// `alignment`.
pub fn write_padding(out: &mut Vec<u8>, start: usize, alignment: usize) {
    out.resize(start + align_up(out.len() - start, alignment), 0);
}

/// Writes the end offset of the tail entry that ends at the end of the `out` bytes, for the tail at
/// the `tail`, to the end table item at the `offset`.
///
/// # Panics
/// Panics if the end offset does not fit in a `u32`.
pub fn write_end(out: &mut [u8], offset: usize, tail: usize) {
    let end: u32 = u32::try_from(out.len() - tail).expect("the tail is too long");
    out[offset..offset + 4].copy_from_slice(&end.to_le_bytes());
}
