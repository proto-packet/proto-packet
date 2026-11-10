use crate::io::list::element::list_alignment;
use crate::io::list::map_ref::{check_keys, find_key, find_native_key, map_alignment};
use crate::io::list::option_write::{encode_option_map, option_map_encoded_len};
use crate::io::{ArrayElement, DecodeError, Element, ListRef, MAX_DEPTH, OptionListRef};
use crate::io::{align_up, check_padding, read_u32_unchecked};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A read-only view of an encoded map with optional values.
///
/// # Note
/// - The map is the end of its keys, its keys as a list, its values, then the presence bits of its
///   values.
/// - The values have no count of their own, since the keys have it.
pub struct OptionMapRef<'a, K, V> {
    bytes: &'a [u8],
    key: PhantomData<K>,
    value: PhantomData<V>,
}

impl<'a, K, V> OptionMapRef<'a, K, V>
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
    /// # Errors
    /// Returns an error if the `bytes` are not a valid encoding.
    pub fn new_with_depth(bytes: &'a [u8], depth: usize) -> Result<Self, DecodeError> {
        let map: Self = unsafe { Self::new_unchecked(bytes) };
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
        OptionListRef::<V>::of_len_with_depth(&bytes[values_start..], keys.len(), depth).map_err(
            |error| match error {
                DecodeError::InvalidList => DecodeError::InvalidMap,
                error => error,
            },
        )?;
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

impl<K, V> Copy for OptionMapRef<'_, K, V> {}

impl<K, V> Clone for OptionMapRef<'_, K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Eq for OptionMapRef<'_, K, V> {}

impl<K, V> PartialEq for OptionMapRef<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<K, V> Hash for OptionMapRef<'_, K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<'a, K, V> OptionMapRef<'a, K, V>
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
        let end: usize = unsafe { read_u32_unchecked(self.bytes, 0) as usize };
        unsafe { ListRef::new_unchecked(self.bytes.get_unchecked(Self::keys_start()..end)) }
    }

    /// Gets the values, in the order of their keys, each `None` if it is absent.
    #[must_use]
    pub fn values(self) -> OptionListRef<'a, V> {
        if self.bytes.is_empty() {
            return unsafe { OptionListRef::of_len_unchecked(self.bytes, 0) };
        }
        let end: usize = unsafe { read_u32_unchecked(self.bytes, 0) as usize };
        let start: usize = align_up(end, list_alignment::<V>());
        unsafe {
            OptionListRef::of_len_unchecked(self.bytes.get_unchecked(start..), self.keys().len())
        }
    }
}

impl<'a, K, V> OptionMapRef<'a, K, V>
where
    K: Element + Ord,
    V: Element,
{
    //! Entries

    /// Gets the value of the `key`, which is `None` inside if it is absent.
    ///
    /// Returns `None` if the map does not have the `key`.
    #[must_use]
    pub fn get(self, key: K::Ref<'_>) -> Option<Option<V::Ref<'a>>> {
        find_key(self.keys(), key).map(|index| unsafe { self.values().get_unchecked(index) })
    }

    /// Gets the value of the `key`, given as the key's own type.
    ///
    /// Returns `None` if the map does not have the `key`.
    #[must_use]
    pub fn get_by_key(self, key: &K) -> Option<Option<V::Ref<'a>>> {
        find_native_key(self.keys(), key).map(|index| unsafe { self.values().get_unchecked(index) })
    }

    /// Gets the iterator of the entries, in the order of their encoded keys.
    pub fn iter(self) -> impl ExactSizeIterator<Item = (K::Ref<'a>, Option<V::Ref<'a>>)> {
        self.keys().iter().zip(self.values().iter())
    }

    /// Decodes the map.
    #[must_use]
    pub fn decode(self) -> BTreeMap<K, Option<V>> {
        self.iter()
            .map(|(key, value)| (K::decode(key), value.map(V::decode)))
            .collect()
    }
}

impl<K, V> Debug for OptionMapRef<'_, K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptionMapRef")
            .field("bytes", &self.bytes)
            .finish()
    }
}

impl<K, V> Element for BTreeMap<K, Option<V>>
where
    K: Element + Ord,
    V: Element,
{
    const SIZE: Option<usize> = None;
    const ALIGNMENT: usize = map_alignment::<K, V>();
    const CAN_BE_INVALID: bool = true;
    const ORDERED_BY_BYTES: bool = K::ORDERED_BY_BYTES || V::ORDERED_BY_BYTES;
    const JSON_TEXT_KEY: bool = false;
    type Ref<'a> = OptionMapRef<'a, K, V>;
    type View<'a>
        = &'a BTreeMap<K, Option<V>>
    where
        K: 'a,
        V: 'a;

    fn check(bytes: &[u8], depth: usize) -> Result<(), DecodeError> {
        OptionMapRef::<K, V>::new_with_depth(bytes, depth).map(|_| ())
    }

    unsafe fn read_unchecked(bytes: &[u8]) -> OptionMapRef<'_, K, V> {
        unsafe { OptionMapRef::new_unchecked(bytes) }
    }

    fn decode(element: OptionMapRef<'_, K, V>) -> BTreeMap<K, Option<V>> {
        element.decode()
    }

    fn view(&self) -> &BTreeMap<K, Option<V>> {
        self
    }

    fn encoded_len(&self) -> usize {
        option_map_encoded_len(self)
    }

    fn encode_to(&self, out: &mut Vec<u8>, depth: usize) {
        encode_option_map(self, out, depth);
    }

    fn cmp_refs(a: OptionMapRef<'_, K, V>, b: OptionMapRef<'_, K, V>) -> Ordering
    where
        Self: Ord,
    {
        if Self::ORDERED_BY_BYTES {
            a.as_bytes().cmp(b.as_bytes())
        } else {
            Self::decode(a).cmp(&Self::decode(b))
        }
    }

    #[cfg(feature = "types")]
    fn serialize_json<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::types::serialize_option_map(self, serializer)
    }

    #[cfg(feature = "types")]
    fn deserialize_json<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::types::deserialize_option_map(deserializer)
    }
}

impl<K, V> ArrayElement for BTreeMap<K, Option<V>>
where
    K: Element + Ord,
    V: Element,
{
}
