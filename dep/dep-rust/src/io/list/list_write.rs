use crate::io::list::element::stride;
use crate::io::{Element, align_up, trim_padding, write_end, write_padding, write_zeros};

/// Gets the length of the encoding of the `list`, in bytes.
#[must_use]
pub fn list_encoded_len<T>(list: &[T]) -> usize
where
    T: Element,
{
    elements_encoded_len(list.iter())
}

/// Gets the length of the encoding of the list of the `elements`, in bytes.
pub(crate) fn elements_encoded_len<'t, T, I>(elements: I) -> usize
where
    T: Element + 't,
    I: ExactSizeIterator<Item = &'t T>,
{
    let count: usize = elements.len();
    if count == 0 {
        return 0;
    }
    match T::SIZE {
        Some(size) => (count - 1) * stride::<T>(size) + size,
        None => elements.fold(count * 4, |len, element| match element.encoded_len() {
            0 => len,
            size => align_up(len, T::ALIGNMENT) + size,
        }),
    }
}

/// Appends the encoding of the `list` to the `out` bytes, with elements that can nest the `depth`
/// of levels.
///
/// # Panics
/// Panics if the count or an end offset does not fit in a `u32`, or an element nests deeper.
pub fn encode_list<T>(list: &[T], out: &mut Vec<u8>, depth: usize)
where
    T: Element,
{
    match T::SIZE {
        Some(size) if stride::<T>(size) == size => T::encode_all_to(list, out),
        _ => encode_elements(list.iter(), out, depth),
    }
}

/// Appends the encoding of the list of the `elements` to the `out` bytes.
///
/// # Note
/// - The end table is zeros until each element is written, since writing an element finds its
///   length.
///
/// # Panics
/// Panics if the count or an end offset does not fit in a `u32`, or an element nests deeper than
/// the `depth` of levels.
pub(crate) fn encode_elements<'t, T, I>(elements: I, out: &mut Vec<u8>, depth: usize)
where
    T: Element + 't,
    I: ExactSizeIterator<Item = &'t T>,
{
    let count: usize = elements.len();
    if count == 0 {
        return;
    }
    let start: usize = out.len();
    if T::SIZE.is_some() {
        for element in elements {
            write_padding(out, start, T::ALIGNMENT);
            element.encode_to(out, depth);
        }
        return;
    }
    let length: u32 = u32::try_from(count).expect("the list is too long");
    out.extend_from_slice(&length.to_le_bytes());
    write_zeros(out, (count - 1) * 4);
    for (index, element) in elements.enumerate() {
        let before: usize = out.len();
        write_padding(out, start, T::ALIGNMENT);
        let padded: usize = out.len();
        element.encode_to(out, depth);
        trim_padding(out, before, padded);
        if index + 1 < count {
            write_end(out, start + 4 + index * 4, start);
        }
    }
}
