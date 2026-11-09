use crate::io::list::element::stride;
use crate::io::{Element, align_up, write_end, write_padding, write_zeros};

/// Gets the length of the encoding of the `list`, in bytes.
#[must_use]
pub fn list_encoded_len<T>(list: &[T]) -> usize
where
    T: Element,
{
    if list.is_empty() {
        return 0;
    }
    match T::SIZE {
        Some(size) => (list.len() - 1) * stride::<T>(size) + size,
        None => list.iter().fold(list.len() * 4, |len, element| {
            align_up(len, T::ALIGNMENT) + element.encoded_len()
        }),
    }
}

/// Appends the encoding of the `list` to the `out` bytes.
///
/// # Note
/// - The end table is zeros until each element is written, since writing an element finds its
///   length.
///
/// # Panics
/// Panics if the count or an end offset does not fit in a `u32`.
pub fn encode_list<T>(list: &[T], out: &mut Vec<u8>)
where
    T: Element,
{
    if list.is_empty() {
        return;
    }
    let start: usize = out.len();
    let Some(size) = T::SIZE else {
        let count: u32 = u32::try_from(list.len()).expect("the list is too long");
        out.extend_from_slice(&count.to_le_bytes());
        write_zeros(out, (list.len() - 1) * 4);
        for (index, element) in list.iter().enumerate() {
            write_padding(out, start, T::ALIGNMENT);
            element.encode_to(out);
            if index + 1 < list.len() {
                write_end(out, start + 4 + index * 4, start);
            }
        }
        return;
    };
    if stride::<T>(size) == size {
        T::encode_all_to(list, out);
    } else {
        for element in list {
            write_padding(out, start, T::ALIGNMENT);
            element.encode_to(out);
        }
    }
}
