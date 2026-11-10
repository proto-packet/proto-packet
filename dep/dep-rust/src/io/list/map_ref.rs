use crate::io::list::element::list_alignment;
use crate::io::list::list_write::{elements_encoded_len, encode_elements};
use crate::io::{DecodeError, Element, ListRef, align_up, check_padding, write_end};
use crate::io::{write_padding, write_zeros};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/// A read-only view of an encoded map.
///
/// # Note
/// - A map is the end of its keys, its keys as a list, then its values as a list.
/// - The keys are strictly increasing, so a key is found by a binary search.
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
        let keys: ListRef<K> = ListRef::new(&bytes[keys_start..keys_end])?;
        let values: ListRef<V> = ListRef::new(&bytes[values_start..])?;
        if keys.len() != values.len() {
            return Err(DecodeError::InvalidMap);
        }
        let mut iter = keys.iter();
        let mut previous: Option<K::Ref<'_>> = iter.next();
        for key in iter {
            if previous.is_some_and(|previous| K::cmp_refs(previous, key) != Ordering::Less) {
                return Err(DecodeError::InvalidMap);
            }
            previous = Some(key);
        }
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
        let keys: ListRef<'a, K> = self.keys();
        let (mut low, mut high): (usize, usize) = (0, keys.len());
        while low < high {
            let middle: usize = (low + high) / 2;
            match K::cmp_refs(unsafe { keys.get_unchecked(middle) }, key) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => return Some(unsafe { self.values().get_unchecked(middle) }),
            }
        }
        None
    }

    /// Gets the value of the `key`, given as the key's own type.
    ///
    /// Returns `None` if the map does not have the `key`.
    #[must_use]
    pub fn get_by_key(self, key: &K) -> Option<V::Ref<'a>> {
        let keys: ListRef<'a, K> = self.keys();
        let (mut low, mut high): (usize, usize) = (0, keys.len());
        while low < high {
            let middle: usize = (low + high) / 2;
            match key.cmp_ref(unsafe { keys.get_unchecked(middle) }) {
                Ordering::Greater => low = middle + 1,
                Ordering::Less => high = middle,
                Ordering::Equal => return Some(unsafe { self.values().get_unchecked(middle) }),
            }
        }
        None
    }

    /// Gets the iterator of the entries, in the order of their keys.
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

/// Gets the length of the encoding of the `map`, in bytes.
#[must_use]
pub fn map_encoded_len<K, V>(map: &BTreeMap<K, V>) -> usize
where
    K: Element,
    V: Element,
{
    if map.is_empty() {
        return 0;
    }
    let keys_end: usize = align_up(4, list_alignment::<K>()) + elements_encoded_len(map.keys());
    align_up(keys_end, list_alignment::<V>()) + elements_encoded_len(map.values())
}

/// Appends the encoding of the `map` to the `out` bytes.
///
/// # Note
/// - A map has its keys in increasing order, so its entries are written as they are.
///
/// # Panics
/// Panics if a count or an end offset does not fit in a `u32`.
pub fn encode_map<K, V>(map: &BTreeMap<K, V>, out: &mut Vec<u8>)
where
    K: Element,
    V: Element,
{
    if map.is_empty() {
        return;
    }
    let start: usize = out.len();
    write_zeros(out, 4);
    write_padding(out, start, list_alignment::<K>());
    encode_elements(map.keys(), out);
    write_end(out, start, start);
    write_padding(out, start, list_alignment::<V>());
    encode_elements(map.values(), out);
}

impl<K, V> Element for BTreeMap<K, V>
where
    K: Element + Ord,
    V: Element,
{
    const SIZE: Option<usize> = None;
    const ALIGNMENT: usize = map_alignment::<K, V>();
    const CAN_BE_INVALID: bool = true;
    type Ref<'a> = MapRef<'a, K, V>;
    type View<'a>
        = &'a BTreeMap<K, V>
    where
        K: 'a,
        V: 'a;

    fn check(bytes: &[u8]) -> Result<(), DecodeError> {
        MapRef::<K, V>::new(bytes).map(|_| ())
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

    fn encoded_len(&self) -> usize {
        map_encoded_len(self)
    }

    fn encode_to(&self, out: &mut Vec<u8>) {
        encode_map(self, out);
    }
}
