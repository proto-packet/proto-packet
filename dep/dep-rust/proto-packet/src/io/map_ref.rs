use crate::io::{DecodeError, ListView, read_u32, read_u32_unchecked};
use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};

/// The size of a map's keys length.
const KEYS_LENGTH_SIZE: usize = 4;

/// Splits the encoding of a map, the `bytes`, into the encodings of its keys & values.
///
/// # Errors
/// Returns an error if the `bytes` are shorter than the keys length, or the keys length is more
/// than the bytes after it.
pub fn split_map(bytes: &[u8]) -> Result<(&[u8], &[u8]), DecodeError> {
    if bytes.len() < KEYS_LENGTH_SIZE {
        return Err(DecodeError::InvalidMap);
    }
    let end: usize = KEYS_LENGTH_SIZE.saturating_add(read_u32(bytes, 0) as usize);
    if end <= bytes.len() {
        Ok((&bytes[KEYS_LENGTH_SIZE..end], &bytes[end..]))
    } else {
        Err(DecodeError::InvalidMap)
    }
}

/// Splits the encoding of a map, the `bytes`, into the encodings of its keys & values, without
/// validating them.
///
/// # Safety
/// The `bytes` must start with the keys length, & the keys length must not be more than the bytes
/// after it.
pub unsafe fn split_map_unchecked(bytes: &[u8]) -> (&[u8], &[u8]) {
    let end: usize = KEYS_LENGTH_SIZE + unsafe { read_u32_unchecked(bytes, 0) } as usize;
    unsafe {
        (
            bytes.get_unchecked(KEYS_LENGTH_SIZE..end),
            bytes.get_unchecked(end..),
        )
    }
}

/// A borrowed map, read in place through the views of its keys & values. (see `FORMAT.md`)
///
/// The `n`th value is the value of the `n`th key. The keys can be in any order, so a key is found
/// by scanning the keys.
#[derive(Copy, Clone)]
pub struct MapRef<K, V> {
    keys: K,
    values: V,
}

impl<K, V> MapRef<K, V>
where
    K: ListView,
    V: ListView,
{
    //! Construction

    /// Creates the map of the `keys` & `values`.
    ///
    /// The counts & the uniqueness of the keys are validated, but not the keys & values.
    ///
    /// # Errors
    /// Returns an error if the `keys` & `values` have different counts, or two keys are equal.
    pub fn new(keys: K, values: V) -> Result<Self, DecodeError>
    where
        K::Item: Ord,
    {
        if keys.len() != values.len() {
            return Err(DecodeError::InvalidMap);
        }
        check_unique(&keys)?;
        Ok(Self { keys, values })
    }

    /// Creates the map of the `keys` & `values`, without validating them.
    ///
    /// # Safety
    /// The `keys` & `values` must have the same count, & no two keys can be equal.
    pub const unsafe fn new_unchecked(keys: K, values: V) -> Self {
        Self { keys, values }
    }
}

impl<K, V> MapRef<K, V>
where
    K: ListView,
    V: ListView,
{
    //! Entries

    /// Gets the keys.
    pub fn keys(&self) -> &K {
        &self.keys
    }

    /// Gets the values, in the order of their keys.
    pub fn values(&self) -> &V {
        &self.values
    }

    /// Gets the number of entries.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Checks if there are no entries.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Gets the optional value of the `key`, scanning the keys.
    pub fn get<Q>(&self, key: &Q) -> Option<V::Item>
    where
        K::Item: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        (0..self.len())
            .find(|index| {
                let item: K::Item = unsafe { self.keys.get_unchecked(*index) };
                <K::Item as Borrow<Q>>::borrow(&item) == key
            })
            .map(|index| unsafe { self.values.get_unchecked(index) })
    }

    /// Iterates the entries, in the order of their encoding.
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (K::Item, V::Item)> + DoubleEndedIterator + '_ {
        (0..self.len()).map(|index| unsafe {
            (
                self.keys.get_unchecked(index),
                self.values.get_unchecked(index),
            )
        })
    }
}

impl<K, V> Debug for MapRef<K, V>
where
    K: ListView,
    V: ListView,
    K::Item: Debug,
    V::Item: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

/// Checks that no two of the `keys` are equal.
///
/// Writers write the keys in increasing order, which is checked in one pass. Keys in any other
/// order are sorted to find the equal keys.
fn check_unique<K>(keys: &K) -> Result<(), DecodeError>
where
    K: ListView,
    K::Item: Ord,
{
    let key = |index: usize| unsafe { keys.get_unchecked(index) };
    if (1..keys.len()).all(|index| key(index - 1) < key(index)) {
        return Ok(());
    }
    let mut sorted: Vec<K::Item> = (0..keys.len()).map(key).collect();
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(DecodeError::DuplicateKey)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{DecodeError, FixedListRef, MapRef, VarListRef, read_u32, split_map};

    /// Encodes the map of the encodings of the `keys` & `values`.
    fn encode(keys: &[u8], values: &[u8]) -> Vec<u8> {
        [&(keys.len() as u32).to_le_bytes()[..], keys, values].concat()
    }

    /// Encodes the `u32` `numbers` back to back.
    fn numbers(numbers: &[u32]) -> Vec<u8> {
        numbers.iter().flat_map(|n| n.to_le_bytes()).collect()
    }

    /// Reads a `u32` key.
    fn number(bytes: &[u8]) -> u32 {
        read_u32(bytes, 0)
    }

    /// Reads a `string` key.
    fn text(bytes: &[u8]) -> &str {
        std::str::from_utf8(bytes).unwrap()
    }

    #[test]
    fn split_map_splits_the_keys_from_the_values() {
        let bytes: Vec<u8> = encode(&[1, 2], &[3]);
        assert_eq!(split_map(&bytes), Ok((&[1, 2][..], &[3][..])));
        assert_eq!(split_map(&encode(&[], &[])), Ok((&[][..], &[][..])));

        let test_cases: Vec<Vec<u8>> = vec![
            vec![0; 3],
            [&5_u32.to_le_bytes()[..], &[0; 4]].concat(),
            u32::MAX.to_le_bytes().to_vec(),
        ];
        for bytes in test_cases {
            assert_eq!(
                split_map(&bytes),
                Err(DecodeError::InvalidMap),
                "{:?}",
                bytes
            );
        }
    }

    #[test]
    fn entries_are_read_in_encoding_order() {
        let keys: Vec<u8> = numbers(&[3, 1, 2]);
        let keys = FixedListRef::new(&keys, 4, number).unwrap();
        let map = MapRef::new(keys, &[30_u8, 10, 20][..]).unwrap();
        assert_eq!(map.len(), 3);
        assert!(!map.is_empty());
        assert_eq!(map.get(&1), Some(10));
        assert_eq!(map.get(&3), Some(30));
        assert_eq!(map.get(&4), None);
        assert_eq!(map.iter().collect::<Vec<_>>(), [(3, 30), (1, 10), (2, 20)]);
        assert_eq!(map.keys().to_vec(), [3, 1, 2]);
        assert_eq!(map.values(), &[30, 10, 20]);
        assert_eq!(format!("{:?}", map), "{3: 30, 1: 10, 2: 20}");

        let keys = FixedListRef::new(&[], 4, number).unwrap();
        assert!(MapRef::new(keys, &[][..]).unwrap().is_empty());
    }

    #[test]
    fn keys_are_found_by_their_borrowed_form() {
        let keys: Vec<u8> = [&numbers(&[2, 1, 2])[..], b"ba"].concat();
        let keys = VarListRef::new(&keys, text).unwrap();
        let map = MapRef::new(keys, &[2_u8, 1][..]).unwrap();
        assert_eq!(map.get("a"), Some(1));
        assert_eq!(map.get(&String::from("b")[..]), Some(2));
        assert_eq!(map.get("c"), None);
    }

    #[test]
    fn keys_must_be_unique() {
        for keys in [&[1, 1][..], &[1, 2, 1], &[2, 1, 2]] {
            let keys: Vec<u8> = numbers(keys);
            let keys = FixedListRef::new(&keys, 4, number).unwrap();
            let values: Vec<u8> = vec![0; keys.len()];
            assert!(matches!(
                MapRef::new(keys, &values[..]),
                Err(DecodeError::DuplicateKey)
            ));
        }
    }

    #[test]
    fn keys_and_values_must_have_the_same_count() {
        let keys: Vec<u8> = numbers(&[1, 2]);
        let keys = FixedListRef::new(&keys, 4, number).unwrap();
        assert!(matches!(
            MapRef::new(keys, &[0_u8][..]),
            Err(DecodeError::InvalidMap)
        ));
    }
}
