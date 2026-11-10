use crate::io::list::SortedKeys;
use crate::io::{Element, MapRef, OptionMapRef};
use std::collections::BTreeMap;

/// The entries of any form of a map with the key `K`.
///
/// # Note
/// - A map of the entries has it, & so does an encoded map.
/// - No other type can have it, since a writer trusts the order of the keys.
pub trait MapView<K>: Copy + SortedKeys {
    /// A key of the map.
    type Key;

    /// A value of the map.
    type Value;

    /// Gets the count of the entries.
    fn len(&self) -> usize;

    /// Checks if the map has no entries.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the value of the `key`.
    ///
    /// Returns `None` if the map does not have the `key`.
    fn get(&self, key: &K) -> Option<Self::Value>;

    /// Gets the iterator of the entries, in the order of their keys.
    ///
    /// # Note
    /// - An encoded map orders some keys by their bytes, so two forms of a map can differ in
    ///   their order. (see [Element::ORDERED_BY_BYTES])
    fn iter(&self) -> impl Iterator<Item = (Self::Key, Self::Value)>;
}

impl<'a, K, V> MapView<K> for &'a BTreeMap<K, V>
where
    K: Element + Ord,
    V: Element,
{
    type Key = K::View<'a>;
    type Value = V::View<'a>;

    fn len(&self) -> usize {
        BTreeMap::len(self)
    }

    fn get(&self, key: &K) -> Option<V::View<'a>> {
        BTreeMap::get(self, key).map(V::view)
    }

    fn iter(&self) -> impl Iterator<Item = (K::View<'a>, V::View<'a>)> {
        BTreeMap::iter(self).map(|(key, value)| (key.view(), value.view()))
    }
}

impl<'a, K, V> MapView<K> for MapRef<'a, K, V>
where
    K: Element + Ord,
    V: Element,
{
    type Key = K::Ref<'a>;
    type Value = V::Ref<'a>;

    fn len(&self) -> usize {
        MapRef::len(*self)
    }

    fn get(&self, key: &K) -> Option<V::Ref<'a>> {
        MapRef::get_by_key(*self, key)
    }

    fn iter(&self) -> impl Iterator<Item = (K::Ref<'a>, V::Ref<'a>)> {
        MapRef::iter(*self)
    }
}

impl<'a, K, V> MapView<K> for &'a BTreeMap<K, Option<V>>
where
    K: Element + Ord,
    V: Element,
{
    type Key = K::View<'a>;
    type Value = Option<V::View<'a>>;

    fn len(&self) -> usize {
        BTreeMap::len(self)
    }

    fn get(&self, key: &K) -> Option<Option<V::View<'a>>> {
        BTreeMap::get(self, key).map(|value| value.as_ref().map(V::view))
    }

    fn iter(&self) -> impl Iterator<Item = (K::View<'a>, Option<V::View<'a>>)> {
        BTreeMap::iter(self).map(|(key, value)| (key.view(), value.as_ref().map(V::view)))
    }
}

impl<'a, K, V> MapView<K> for OptionMapRef<'a, K, V>
where
    K: Element + Ord,
    V: Element,
{
    type Key = K::Ref<'a>;
    type Value = Option<V::Ref<'a>>;

    fn len(&self) -> usize {
        OptionMapRef::len(*self)
    }

    fn get(&self, key: &K) -> Option<Option<V::Ref<'a>>> {
        OptionMapRef::get_by_key(*self, key)
    }

    fn iter(&self) -> impl Iterator<Item = (K::Ref<'a>, Option<V::Ref<'a>>)> {
        OptionMapRef::iter(*self)
    }
}
