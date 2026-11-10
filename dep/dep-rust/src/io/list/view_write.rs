//! The encoding of the lists & the maps of any form.
//!
//! A generated `View` trait writes its lists & maps through these functions, so a list or a map is
//! encoded from its view with no native list or map in between.

use crate::io::list::element::list_alignment;
use crate::io::{Element, ListView, MapView, align_up};
use crate::io::{trim_padding, write_end, write_padding, write_zeros};
use std::borrow::Borrow;
use std::ops::Range;

/// Gets the length of the encoding of the `list`, with each variable-size element measured by
/// `len`.
///
/// The elements have the `size` if they are fixed-size, & the `alignment`.
pub fn list_view_len<L, F>(list: L, size: Option<usize>, alignment: usize, len: F) -> usize
where
    L: ListView,
    F: Fn(&L::Item) -> usize,
{
    items_len(list.iter(), list.len(), size, alignment, &len)
}

/// Appends the encoding of the `list` to the `out` bytes, with each element written by `write`.
///
/// The elements have the `size` if they are fixed-size, & the `alignment`.
///
/// # Panics
/// Panics if the count or an end offset does not fit in a `u32`, or the list has another count of
/// elements than its length.
pub fn write_list_view<L, F>(
    list: L,
    out: &mut Vec<u8>,
    size: Option<usize>,
    alignment: usize,
    mut write: F,
) where
    L: ListView,
    F: FnMut(&L::Item, &mut Vec<u8>),
{
    write_items(list.iter(), list.len(), out, size, alignment, &mut write);
}

/// Gets the length of the encoding of the `map`, with each variable-size key & value measured by
/// `key_len` & `value_len`.
///
/// # Note
/// - The keys are written by `write_key` when their order changes the padding between the entries.
pub fn map_view_len<K, V, M, KL, VL, KW>(
    map: M,
    key_len: KL,
    value_len: VL,
    mut write_key: KW,
) -> usize
where
    K: Element,
    V: Element,
    M: MapView<K>,
    KL: Fn(&M::Key) -> usize,
    VL: Fn(&M::Value) -> usize,
    KW: FnMut(&M::Key, &mut Vec<u8>),
{
    let count: usize = map.len();
    if count == 0 {
        return 0;
    }
    let keys_start: usize = align_up(4, list_alignment::<K>());
    let is_len_ordered: bool =
        (K::SIZE.is_none() && K::ALIGNMENT > 1) || (V::SIZE.is_none() && V::ALIGNMENT > 1);
    let (keys, values): (usize, usize) = if K::ORDERED_BY_BYTES && is_len_ordered {
        let entries: Vec<(M::Key, M::Value)> = entries_by_key_bytes(map, &mut write_key);
        (
            items_len(
                entries.iter().map(|entry| &entry.0),
                count,
                K::SIZE,
                K::ALIGNMENT,
                &key_len,
            ),
            items_len(
                entries.iter().map(|entry| &entry.1),
                count,
                V::SIZE,
                V::ALIGNMENT,
                &value_len,
            ),
        )
    } else {
        (
            items_len(
                map.iter().map(|entry| entry.0),
                count,
                K::SIZE,
                K::ALIGNMENT,
                &key_len,
            ),
            items_len(
                map.iter().map(|entry| entry.1),
                count,
                V::SIZE,
                V::ALIGNMENT,
                &value_len,
            ),
        )
    };
    align_up(keys_start + keys, list_alignment::<V>()) + values
}

/// Appends the encoding of the `map` to the `out` bytes, with each key & value written by
/// `write_key` & `write_value`.
///
/// # Note
/// - Keys that are ordered by their bytes are sorted first, since the map has another order.
///
/// # Panics
/// Panics if a count or an end offset does not fit in a `u32`.
pub fn write_map_view<K, V, M, KW, VW>(
    map: M,
    out: &mut Vec<u8>,
    mut write_key: KW,
    mut write_value: VW,
) where
    K: Element,
    V: Element,
    M: MapView<K>,
    KW: FnMut(&M::Key, &mut Vec<u8>),
    VW: FnMut(&M::Value, &mut Vec<u8>),
{
    let count: usize = map.len();
    if count == 0 {
        return;
    }
    let start: usize = out.len();
    write_zeros(out, 4);
    write_padding(out, start, list_alignment::<K>());
    if K::ORDERED_BY_BYTES {
        let entries: Vec<(M::Key, M::Value)> = entries_by_key_bytes(map, &mut write_key);
        let keys = entries.iter().map(|entry| &entry.0);
        write_items(keys, count, out, K::SIZE, K::ALIGNMENT, &mut write_key);
        write_end(out, start, start);
        write_padding(out, start, list_alignment::<V>());
        let values = entries.iter().map(|entry| &entry.1);
        write_items(values, count, out, V::SIZE, V::ALIGNMENT, &mut write_value);
    } else {
        let keys = map.iter().map(|entry| entry.0);
        write_items(keys, count, out, K::SIZE, K::ALIGNMENT, &mut write_key);
        write_end(out, start, start);
        write_padding(out, start, list_alignment::<V>());
        let values = map.iter().map(|entry| entry.1);
        write_items(values, count, out, V::SIZE, V::ALIGNMENT, &mut write_value);
    }
}

/// Gets the entries of the `map`, in the order of the encoded bytes of their keys.
fn entries_by_key_bytes<K, M, KW>(map: M, write_key: &mut KW) -> Vec<(M::Key, M::Value)>
where
    M: MapView<K>,
    KW: FnMut(&M::Key, &mut Vec<u8>),
{
    let mut bytes: Vec<u8> = Vec::default();
    let mut entries: Vec<(Range<usize>, M::Key, M::Value)> = Vec::with_capacity(map.len());
    for (key, value) in map.iter() {
        let start: usize = bytes.len();
        write_key(&key, &mut bytes);
        entries.push((start..bytes.len(), key, value));
    }
    entries.sort_unstable_by(|a, b| bytes[a.0.clone()].cmp(&bytes[b.0.clone()]));
    entries
        .into_iter()
        .map(|(_, key, value)| (key, value))
        .collect()
}

/// Gets the length of the encoding of a list of the `count` of `items`.
fn items_len<I, B, T, F>(
    items: I,
    count: usize,
    size: Option<usize>,
    alignment: usize,
    len: &F,
) -> usize
where
    I: Iterator<Item = B>,
    B: Borrow<T>,
    F: Fn(&T) -> usize,
{
    if count == 0 {
        return 0;
    }
    match size {
        Some(size) => (count - 1) * align_up(size, alignment) + size,
        None => items.fold(count * 4, |total, item| match len(item.borrow()) {
            0 => total,
            size => align_up(total, alignment) + size,
        }),
    }
}

/// Appends the encoding of a list of the `count` of `items` to the `out` bytes.
///
/// # Panics
/// Panics if the count or an end offset does not fit in a `u32`, or the `items` are not the
/// `count`.
fn write_items<I, B, T, F>(
    items: I,
    count: usize,
    out: &mut Vec<u8>,
    size: Option<usize>,
    alignment: usize,
    write: &mut F,
) where
    I: Iterator<Item = B>,
    B: Borrow<T>,
    F: FnMut(&T, &mut Vec<u8>),
{
    if count == 0 {
        return;
    }
    let start: usize = out.len();
    let mut written: usize = 0;
    if size.is_some() {
        for item in items {
            write_padding(out, start, alignment);
            write(item.borrow(), out);
            written += 1;
        }
    } else {
        let length: u32 = u32::try_from(count).expect("the list is too long");
        out.extend_from_slice(&length.to_le_bytes());
        write_zeros(out, (count - 1) * 4);
        for item in items {
            let before: usize = out.len();
            write_padding(out, start, alignment);
            let padded: usize = out.len();
            write(item.borrow(), out);
            trim_padding(out, before, padded);
            written += 1;
            if written < count {
                write_end(out, start + written * 4, start);
            }
        }
    }
    assert_eq!(written, count, "the list has another count than its length");
}
