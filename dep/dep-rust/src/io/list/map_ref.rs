use crate::io::list::element::list_alignment;
use crate::io::list::list_write::{elements_encoded_len, encode_elements};
use crate::io::{
    ArrayElement, DecodeError, Element, ListRef, MAX_DEPTH, align_up, check_padding, write_end,
};
use crate::io::{write_padding, write_zeros};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Range;

/// A read-only view of an encoded map.
///
/// # Note
/// - A map is the end of its keys, its keys as a list, then its values as a list.
/// - The keys are strictly increasing, so a key is found by a binary search.
/// - A map has one encoding, so two maps are equal when their bytes are.
pub struct MapRef<'a, K, V> {
    bytes: &'a [u8],
    key: PhantomData<K>,
    value: PhantomData<V>,
}

/// Gets the alignment of a map of the key `K` & the value `V`.
#[must_use]
pub const fn map_alignment<K, V>() -> usize
where
    K: Element,
    V: Element,
{
    let (keys, values): (usize, usize) = (list_alignment::<K>(), list_alignment::<V>());
    let lists: usize = if keys > values { keys } else { values };
    if lists > 4 { lists } else { 4 }
}

impl<'a, K, V> MapRef<'a, K, V>
where
    K: Element + Ord,
    V: Element,
{
    //! Construction

    /// Creates a new ref of the `bytes`.
    ///
    /// # Errors
    /// Returns an error if the `bytes` are not a valid encoding.
    pub fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        Self::new_with_depth(bytes, MAX_DEPTH)
    }

    /// Creates a new ref of the `bytes`, whose keys & values can nest the `depth` of levels.
    ///
    /// # Note
    /// - A map is not a level, so its keys & values can nest every level it can.
    ///
    /// # Errors
    /// Returns an error if the `bytes` are not a valid encoding.
    pub fn new_with_depth(bytes: &'a [u8], depth: usize) -> Result<Self, DecodeError> {
        let map: Self = Self {
            bytes,
            key: PhantomData,
            value: PhantomData,
        };
        if bytes.is_empty() {
            return Ok(map);
        }
        let Some(keys_end) = bytes.first_chunk::<4>() else {
            return Err(DecodeError::TooShort {
                expected: 4,
                actual: bytes.len(),
            });
        };
        let keys_end: usize = u32::from_le_bytes(*keys_end) as usize;
        let keys_start: usize = Self::keys_start();
        let values_start: usize = align_up(keys_end, list_alignment::<V>());
        if keys_start >= keys_end || values_start >= bytes.len() {
            return Err(DecodeError::InvalidMap);
        }
        check_padding(&bytes[4..keys_start])?;
        check_padding(&bytes[keys_end..values_start])?;
        let keys: ListRef<K> = ListRef::new_with_depth(&bytes[keys_start..keys_end], depth)?;
        let values: ListRef<V> = ListRef::new_with_depth(&bytes[values_start..], depth)?;
        if keys.len() != values.len() {
            return Err(DecodeError::InvalidMap);
        }
        check_keys(keys)?;
        Ok(map)
    }

    /// Creates a new ref of the `bytes`, without validating them.
    ///
    /// # Safety
    /// The `bytes` must be a valid encoding.
    #[must_use]
    pub const unsafe fn new_unchecked(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// Gets the offset where the keys start.
    const fn keys_start() -> usize {
        align_up(4, list_alignment::<K>())
    }
}

impl<K, V> Copy for MapRef<'_, K, V> {}

impl<K, V> Clone for MapRef<'_, K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Eq for MapRef<'_, K, V> {}

impl<K, V> PartialEq for MapRef<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<K, V> Hash for MapRef<'_, K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<'a, K, V> MapRef<'a, K, V>
where
    K: Element + Ord,
    V: Element,
{
    //! Properties

    /// Gets the encoded bytes.
    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.bytes
    }

    /// Gets the count of the entries.
    #[must_use]
    pub fn len(self) -> usize {
        self.keys().len()
    }

    /// Checks if the map has no entries.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    /// Gets the keys, in increasing order.
    #[must_use]
    pub fn keys(self) -> ListRef<'a, K> {
        if self.bytes.is_empty() {
            return unsafe { ListRef::new_unchecked(self.bytes) };
        }
        let end: usize = unsafe { crate::io::read_u32_unchecked(self.bytes, 0) as usize };
        unsafe { ListRef::new_unchecked(self.bytes.get_unchecked(Self::keys_start()..end)) }
    }

    /// Gets the values, in the order of their keys.
    #[must_use]
    pub fn values(self) -> ListRef<'a, V> {
        if self.bytes.is_empty() {
            return unsafe { ListRef::new_unchecked(self.bytes) };
        }
        let end: usize = unsafe { crate::io::read_u32_unchecked(self.bytes, 0) as usize };
        let start: usize = align_up(end, list_alignment::<V>());
        unsafe { ListRef::new_unchecked(self.bytes.get_unchecked(start..)) }
    }
}

impl<'a, K, V> MapRef<'a, K, V>
where
    K: Element + Ord,
    V: Element,
{
    //! Entries

    /// Gets the value of the `key`.
    ///
    /// Returns `None` if the map does not have the `key`.
    #[must_use]
    pub fn get(self, key: K::Ref<'_>) -> Option<V::Ref<'a>> {
        find_key(self.keys(), key).map(|index| unsafe { self.values().get_unchecked(index) })
    }

    /// Gets the value of the `key`, given as the key's own type.
    ///
    /// Returns `None` if the map does not have the `key`.
    #[must_use]
    pub fn get_by_key(self, key: &K) -> Option<V::Ref<'a>> {
        find_native_key(self.keys(), key).map(|index| unsafe { self.values().get_unchecked(index) })
    }

    /// Gets the iterator of the entries, in the order of their encoded keys.
    pub fn iter(self) -> impl ExactSizeIterator<Item = (K::Ref<'a>, V::Ref<'a>)> {
        self.keys().iter().zip(self.values().iter())
    }

    /// Decodes the map.
    #[must_use]
    pub fn decode(self) -> BTreeMap<K, V> {
        self.iter()
            .map(|(key, value)| (K::decode(key), V::decode(value)))
            .collect()
    }
}

impl<K, V> Debug for MapRef<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapRef")
            .field("bytes", &self.bytes)
            .finish()
    }
}

/// Checks the `keys` of a map are strictly increasing.
///
/// # Errors
/// Returns an error if a key is not after the key before it.
pub(crate) fn check_keys<K>(keys: ListRef<'_, K>) -> Result<(), DecodeError>
where
    K: Element + Ord,
{
    let mut iter = keys.iter();
    let mut previous: Option<K::Ref<'_>> = iter.next();
    for key in iter {
        if previous.is_some_and(|previous| K::cmp_refs(previous, key) != Ordering::Less) {
            return Err(DecodeError::InvalidMap);
        }
        previous = Some(key);
    }
    Ok(())
}

/// Finds the index of the read `key` in the `keys` of a map, by a binary search.
pub(crate) fn find_key<K>(keys: ListRef<'_, K>, key: K::Ref<'_>) -> Option<usize>
where
    K: Element + Ord,
{
    let (mut low, mut high): (usize, usize) = (0, keys.len());
    while low < high {
        let middle: usize = (low + high) / 2;
        match K::cmp_refs(unsafe { keys.get_unchecked(middle) }, key) {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => high = middle,
            Ordering::Equal => return Some(middle),
        }
    }
    None
}

/// Finds the index of the `key` in the `keys` of a map, by a binary search.
pub(crate) fn find_native_key<K>(keys: ListRef<'_, K>, key: &K) -> Option<usize>
where
    K: Element + Ord,
{
    if K::ORDERED_BY_BYTES {
        let mut bytes: Vec<u8> = Vec::with_capacity(key.encoded_len());
        key.encode_to(&mut bytes, usize::MAX);
        return find_key(keys, unsafe { K::read_unchecked(&bytes) });
    }
    let (mut low, mut high): (usize, usize) = (0, keys.len());
    while low < high {
        let middle: usize = (low + high) / 2;
        match key.cmp_ref(unsafe { keys.get_unchecked(middle) }) {
            Ordering::Greater => low = middle + 1,
            Ordering::Less => high = middle,
            Ordering::Equal => return Some(middle),
        }
    }
    None
}

/// Gets the length of the encoding of the `map`, in bytes.
///
/// # Note
/// - The entries are sorted first if their order changes the padding between them.
#[must_use]
pub fn map_encoded_len<K, V>(map: &BTreeMap<K, V>) -> usize
where
    K: Element,
    V: Element,
{
    if map.is_empty() {
        return 0;
    }
    if K::ORDERED_BY_BYTES && (is_len_ordered::<K>() || is_len_ordered::<V>()) {
        let entries: Vec<(&K, &V)> = entries_by_key_bytes(map);
        let keys = entries.iter().map(|entry| entry.0);
        let values = entries.iter().map(|entry| entry.1);
        return entries_encoded_len(keys, values);
    }
    entries_encoded_len(map.keys(), map.values())
}

/// Appends the encoding of the `map` to the `out` bytes, with keys & values that can nest the
/// `depth` of levels.
///
/// # Note
/// - A map has its keys in increasing order, so its entries are written as they are.
/// - Keys that are ordered by their bytes are sorted first, since the map has another order.
///
/// # Panics
/// Panics if a count or an end offset does not fit in a `u32`, or a key or a value nests deeper.
pub fn encode_map<K, V>(map: &BTreeMap<K, V>, out: &mut Vec<u8>, depth: usize)
where
    K: Element,
    V: Element,
{
    if map.is_empty() {
        return;
    }
    if K::ORDERED_BY_BYTES {
        let entries: Vec<(&K, &V)> = entries_by_key_bytes(map);
        let keys = entries.iter().map(|entry| entry.0);
        let values = entries.iter().map(|entry| entry.1);
        encode_entries(keys, values, out, depth);
    } else {
        encode_entries(map.keys(), map.values(), out, depth);
    }
}

/// Checks if the length of a list of the element `T` depends on the order of its elements.
///
/// # Note
/// - A variable-size element is padded by the length of the elements before it.
pub(crate) const fn is_len_ordered<T>() -> bool
where
    T: Element,
{
    T::SIZE.is_none() && T::ALIGNMENT > 1
}

/// Gets the entries of the `map`, in the order of the encoded bytes of their keys.
pub(crate) fn entries_by_key_bytes<K, V>(map: &BTreeMap<K, V>) -> Vec<(&K, &V)>
where
    K: Element,
{
    let mut bytes: Vec<u8> = Vec::default();
    let mut entries: Vec<(Range<usize>, &K, &V)> = Vec::with_capacity(map.len());
    for (key, value) in map {
        let start: usize = bytes.len();
        key.encode_to(&mut bytes, usize::MAX);
        entries.push((start..bytes.len(), key, value));
    }
    entries.sort_unstable_by(|a, b| bytes[a.0.clone()].cmp(&bytes[b.0.clone()]));
    entries
        .into_iter()
        .map(|(_, key, value)| (key, value))
        .collect()
}

/// Gets the length of the encoding of a map of the `keys` & their `values`, in bytes.
fn entries_encoded_len<'m, K, V, KI, VI>(keys: KI, values: VI) -> usize
where
    K: Element + 'm,
    V: Element + 'm,
    KI: ExactSizeIterator<Item = &'m K>,
    VI: ExactSizeIterator<Item = &'m V>,
{
    let keys_end: usize = align_up(4, list_alignment::<K>()) + elements_encoded_len(keys);
    align_up(keys_end, list_alignment::<V>()) + elements_encoded_len(values)
}

/// Appends the encoding of a map of the `keys` & their `values` to the `out` bytes.
fn encode_entries<'m, K, V, KI, VI>(keys: KI, values: VI, out: &mut Vec<u8>, depth: usize)
where
    K: Element + 'm,
    V: Element + 'm,
    KI: ExactSizeIterator<Item = &'m K>,
    VI: ExactSizeIterator<Item = &'m V>,
{
    let start: usize = out.len();
    write_zeros(out, 4);
    write_padding(out, start, list_alignment::<K>());
    encode_elements(keys, out, depth);
    write_end(out, start, start);
    write_padding(out, start, list_alignment::<V>());
    encode_elements(values, out, depth);
}

impl<K, V> Element for BTreeMap<K, V>
where
    K: Element + Ord,
    V: Element,
{
    const SIZE: Option<usize> = None;
    const ALIGNMENT: usize = map_alignment::<K, V>();
    const CAN_BE_INVALID: bool = true;
    const ORDERED_BY_BYTES: bool = K::ORDERED_BY_BYTES || V::ORDERED_BY_BYTES;
    const JSON_TEXT_KEY: bool = false;
    type Ref<'a> = MapRef<'a, K, V>;
    type View<'a>
        = &'a BTreeMap<K, V>
    where
        K: 'a,
        V: 'a;

    fn check(bytes: &[u8], depth: usize) -> Result<(), DecodeError> {
        MapRef::<K, V>::new_with_depth(bytes, depth).map(|_| ())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> MapRef<'_, K, V> {
        unsafe { MapRef::new_unchecked(bytes) }
    }

    fn decode(element: MapRef<'_, K, V>) -> BTreeMap<K, V> {
        element.decode()
    }

    fn view(&self) -> &BTreeMap<K, V> {
        self
    }

    fn cmp_refs(a: MapRef<'_, K, V>, b: MapRef<'_, K, V>) -> Ordering
    where
        Self: Ord,
    {
        if Self::ORDERED_BY_BYTES {
            a.as_bytes().cmp(b.as_bytes())
        } else {
            Self::decode(a).cmp(&Self::decode(b))
        }
    }

    fn encoded_len(&self) -> usize {
        map_encoded_len(self)
    }

    fn encode_to(&self, out: &mut Vec<u8>, depth: usize) {
        encode_map(self, out, depth);
    }

    #[cfg(feature = "types")]
    fn serialize_json<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::types::serialize_map(self, serializer)
    }

    #[cfg(feature = "types")]
    fn deserialize_json<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::types::deserialize_map(deserializer)
    }
}

impl<K, V> ArrayElement for BTreeMap<K, V>
where
    K: Element + Ord,
    V: Element,
{
}
