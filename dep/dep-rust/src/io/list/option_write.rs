//! The encoding of the optional elements of the lists, the arrays & the maps.
//!
//! The presence bits are the last bytes, one for each element, with the low bit first. An absent
//! element is zeros if it is fixed-size, & no bytes if it isn't. An element that can't be zeros has
//! no presence bit, since zeros are absent.

use crate::io::list::element::list_alignment;
use crate::io::list::map_ref::{entries_by_key_bytes, is_len_ordered};
use crate::io::list::view_write::{items_len, write_items};
use crate::io::{Element, align_up, write_end, write_padding, write_zeros};
use std::borrow::Borrow;
use std::collections::BTreeMap;

/// Gets the count of the bytes of the presence bits of the `count` of elements of the type `T`.
pub(crate) const fn bits_len<T>(count: usize) -> usize
where
    T: Element,
{
    if T::ZERO_IS_ABSENT {
        0
    } else {
        count.div_ceil(8)
    }
}

/// Gets the alignment of a list of optional elements of the type `T`.
///
/// # Note
/// - A list of fixed-size elements with presence bits has a count, of alignment 4.
#[must_use]
pub const fn option_list_alignment<T>() -> usize
where
    T: Element,
{
    if T::ZERO_IS_ABSENT || T::ALIGNMENT > 4 {
        list_alignment::<T>()
    } else {
        4
    }
}

/// Gets the length of the encoding of the `count` of optional `items` of the type `T`, with their
/// presence bits & without a count.
///
/// The length of a present variable-size item is from `len`.
pub(crate) fn option_items_len<T, I, B, X, F>(items: I, count: usize, len: &F) -> usize
where
    T: Element,
    I: Iterator<Item = Option<B>>,
    B: Borrow<X>,
    F: Fn(&X) -> usize,
{
    let elements: usize = items_len(items, count, T::SIZE, T::ALIGNMENT, &|item: &Option<B>| {
        item.as_ref().map_or(0, |item| len(item.borrow()))
    });
    elements + bits_len::<T>(count)
}

/// Appends the encoding of the `count` of optional `items` of the type `T` to the `out` bytes,
/// with their presence bits & without a count.
///
/// A present item is written by `write`.
///
/// # Panics
/// Panics if the count or an end offset does not fit in a `u32`, or the `items` are not the
/// `count`.
pub(crate) fn write_option_items<T, I, B, X, F>(
    items: I,
    count: usize,
    out: &mut Vec<u8>,
    write: &mut F,
) where
    T: Element,
    I: Iterator<Item = Option<B>>,
    B: Borrow<X>,
    F: FnMut(&X, &mut Vec<u8>),
{
    let mut bits: Vec<u8> = vec![0; bits_len::<T>(count)];
    let mut index: usize = 0;
    let mut write_item = |item: &Option<B>, out: &mut Vec<u8>| {
        match item {
            Some(item) => {
                if let Some(byte) = bits.get_mut(index / 8) {
                    *byte |= 1 << (index % 8);
                }
                write(item.borrow(), out);
            }
            None => write_zeros(out, T::SIZE.unwrap_or_default()),
        }
        index += 1;
    };
    write_items(items, count, out, T::SIZE, T::ALIGNMENT, &mut write_item);
    out.extend_from_slice(&bits);
}

/// Gets the length of the encoding of the `list` of optional elements, in bytes.
#[must_use]
pub fn option_list_encoded_len<T>(list: &[Option<T>]) -> usize
where
    T: Element,
{
    if list.is_empty() {
        return 0;
    }
    let items = list.iter().map(Option::as_ref);
    option_list_start::<T>() + option_items_len::<T, _, _, T, _>(items, list.len(), &T::encoded_len)
}

/// Appends the encoding of the `list` of optional elements to the `out` bytes, with elements that
/// can nest the `depth` of levels.
///
/// # Panics
/// Panics if the count or an end offset does not fit in a `u32`, or an element nests deeper.
pub fn encode_option_list<T>(list: &[Option<T>], out: &mut Vec<u8>, depth: usize)
where
    T: Element,
{
    if list.is_empty() {
        return;
    }
    write_option_list_start::<T>(list.len(), out);
    let items = list.iter().map(Option::as_ref);
    write_option_items::<T, _, _, T, _>(items, list.len(), out, &mut |item, out| {
        item.encode_to(out, depth);
    });
}

/// Gets the offset of the elements of a list of optional elements of the type `T`.
///
/// # Note
/// - Fixed-size elements with presence bits follow a count, since nothing else has it.
pub(crate) const fn option_list_start<T>() -> usize
where
    T: Element,
{
    if T::SIZE.is_some() && !T::ZERO_IS_ABSENT {
        align_up(4, T::ALIGNMENT)
    } else {
        0
    }
}

/// Appends the count of a list of the `count` of optional elements of the type `T` to the `out`
/// bytes, if the list has one before its elements.
///
/// # Panics
/// Panics if the count does not fit in a `u32`.
pub(crate) fn write_option_list_start<T>(count: usize, out: &mut Vec<u8>)
where
    T: Element,
{
    if option_list_start::<T>() > 0 {
        let start: usize = out.len();
        let count: u32 = u32::try_from(count).expect("the list is too long");
        out.extend_from_slice(&count.to_le_bytes());
        write_padding(out, start, T::ALIGNMENT);
    }
}

/// Gets the length of the encoding of the `map` with optional values, in bytes.
#[must_use]
pub fn option_map_encoded_len<K, V>(map: &BTreeMap<K, Option<V>>) -> usize
where
    K: Element,
    V: Element,
{
    if map.is_empty() {
        return 0;
    }
    let keys_start: usize = align_up(4, list_alignment::<K>());
    let (keys, values): (usize, usize) =
        if K::ORDERED_BY_BYTES && (is_len_ordered::<K>() || is_len_ordered::<V>()) {
            let entries: Vec<(&K, &Option<V>)> = entries_by_key_bytes(map);
            (
                items_len(
                    entries.iter().map(|entry| entry.0),
                    map.len(),
                    K::SIZE,
                    K::ALIGNMENT,
                    &K::encoded_len,
                ),
                option_items_len::<V, _, _, V, _>(
                    entries.iter().map(|entry| entry.1.as_ref()),
                    map.len(),
                    &V::encoded_len,
                ),
            )
        } else {
            (
                items_len(
                    map.keys(),
                    map.len(),
                    K::SIZE,
                    K::ALIGNMENT,
                    &K::encoded_len,
                ),
                option_items_len::<V, _, _, V, _>(
                    map.values().map(Option::as_ref),
                    map.len(),
                    &V::encoded_len,
                ),
            )
        };
    align_up(keys_start + keys, list_alignment::<V>()) + values
}

/// Appends the encoding of the `map` with optional values to the `out` bytes, with keys & values
/// that can nest the `depth` of levels.
///
/// # Note
/// - The map is its keys, its values, then the presence bits of its values.
///
/// # Panics
/// Panics if a count or an end offset does not fit in a `u32`, or a key or a value nests deeper.
pub fn encode_option_map<K, V>(map: &BTreeMap<K, Option<V>>, out: &mut Vec<u8>, depth: usize)
where
    K: Element,
    V: Element,
{
    if map.is_empty() {
        return;
    }
    let entries: Vec<(&K, &Option<V>)> = if K::ORDERED_BY_BYTES {
        entries_by_key_bytes(map)
    } else {
        map.iter().collect()
    };
    let start: usize = out.len();
    write_zeros(out, 4);
    write_padding(out, start, list_alignment::<K>());
    write_items(
        entries.iter().map(|entry| entry.0),
        entries.len(),
        out,
        K::SIZE,
        K::ALIGNMENT,
        &mut |key: &K, out: &mut Vec<u8>| key.encode_to(out, depth),
    );
    write_end(out, start, start);
    write_padding(out, start, list_alignment::<V>());
    write_option_items::<V, _, _, V, _>(
        entries.iter().map(|entry| entry.1.as_ref()),
        entries.len(),
        out,
        &mut |value, out| value.encode_to(out, depth),
    );
}
