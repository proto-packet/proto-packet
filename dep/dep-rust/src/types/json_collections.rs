//! The JSON forms of the lists & the maps.

use crate::io::Element;
use crate::types::{JsonOwned, JsonRef};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserializer, Serializer};
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::marker::PhantomData;

/// Serializes the `list` as a list of the JSON forms of its elements.
pub(crate) fn serialize_list<T, S>(list: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    T: Element,
    S: Serializer,
{
    serializer.collect_seq(list.iter().map(JsonRef::new))
}

/// Deserializes a list from a list of the JSON forms of its elements.
pub(crate) fn deserialize_list<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Element,
    D: Deserializer<'de>,
{
    struct ListVisitor<T> {
        element: PhantomData<T>,
    }

    impl<'de, T> Visitor<'de> for ListVisitor<T>
    where
        T: Element,
    {
        type Value = Vec<T>;

        fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str("a list")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Vec<T>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut list: Vec<T> = Vec::default();
            while let Some(element) = seq.next_element::<JsonOwned<T>>()? {
                list.push(element.into_value());
            }
            Ok(list)
        }
    }

    deserializer.deserialize_seq(ListVisitor {
        element: PhantomData,
    })
}

/// Serializes the `map` as its JSON form.
///
/// # Note
/// - A map with a key that is text in JSON is an object, & any other map is a list of its
///   entries, each a list of the key & the value.
pub(crate) fn serialize_map<K, V, S>(map: &BTreeMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Element,
    V: Element,
    S: Serializer,
{
    let entries = map
        .iter()
        .map(|(key, value)| (JsonRef::new(key), JsonRef::new(value)));
    if K::JSON_TEXT_KEY {
        serializer.collect_map(entries)
    } else {
        serializer.collect_seq(entries)
    }
}

/// Deserializes a map from its JSON form.
///
/// # Note
/// - A list of entries can't repeat a key.
pub(crate) fn deserialize_map<'de, K, V, D>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
where
    K: Element + Ord,
    V: Element,
    D: Deserializer<'de>,
{
    struct MapVisitor<K, V> {
        key: PhantomData<K>,
        value: PhantomData<V>,
    }

    impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
    where
        K: Element + Ord,
        V: Element,
    {
        type Value = BTreeMap<K, V>;

        fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if K::JSON_TEXT_KEY {
                f.write_str("a map")
            } else {
                f.write_str("a list of the entries of a map")
            }
        }

        fn visit_map<A>(self, mut map: A) -> Result<BTreeMap<K, V>, A::Error>
        where
            A: MapAccess<'de>,
        {
            if !K::JSON_TEXT_KEY {
                return Err(A::Error::custom("a map with keys that aren't text"));
            }
            let mut result: BTreeMap<K, V> = BTreeMap::default();
            while let Some((key, value)) = map.next_entry::<JsonOwned<K>, JsonOwned<V>>()? {
                result.insert(key.into_value(), value.into_value());
            }
            Ok(result)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<BTreeMap<K, V>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            if K::JSON_TEXT_KEY {
                return Err(A::Error::custom(
                    "a list of entries with keys that are text",
                ));
            }
            let mut result: BTreeMap<K, V> = BTreeMap::default();
            while let Some((key, value)) = seq.next_element::<(JsonOwned<K>, JsonOwned<V>)>()? {
                if result
                    .insert(key.into_value(), value.into_value())
                    .is_some()
                {
                    return Err(A::Error::custom("a repeated key"));
                }
            }
            Ok(result)
        }
    }

    let visitor: MapVisitor<K, V> = MapVisitor {
        key: PhantomData,
        value: PhantomData,
    };
    if K::JSON_TEXT_KEY {
        deserializer.deserialize_map(visitor)
    } else {
        deserializer.deserialize_seq(visitor)
    }
}
