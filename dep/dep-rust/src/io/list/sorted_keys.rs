use crate::io::MapRef;
use std::collections::BTreeMap;

/// A map of this crate, with its keys in their order.
///
/// # Note
/// - A writer trusts the order of the keys of a map, so only these maps can be written.
pub trait SortedKeys {}

impl<K, V> SortedKeys for &BTreeMap<K, V> {}

impl<K, V> SortedKeys for MapRef<'_, K, V> {}
