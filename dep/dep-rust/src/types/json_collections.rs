//! The JSON forms of the lists & the maps.

use crate::io::{Element, ListRef, encode_list};
use crate::types::{Bytes, JsonOwned, JsonRef};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

/// Serializes the `list` of optional elements, with an absent element as `null`.
pub(crate) fn serialize_option_list<T, S>(
    list: &[Option<T>],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: Element,
    S: Serializer,
{
    serializer.collect_seq(
        list.iter()
            .map(|element| element.as_ref().map(JsonRef::new)),
    )
}

/// Deserializes a list of optional elements, with a `null` as an absent element.
pub(crate) fn deserialize_option_list<'de, T, D>(
    deserializer: D,
) -> Result<Vec<Option<T>>, D::Error>
where
    T: Element,
    D: Deserializer<'de>,
{
    struct OptionListVisitor<T> {
        element: PhantomData<T>,
    }

    impl<'de, T> Visitor<'de> for OptionListVisitor<T>
    where
        T: Element,
    {
        type Value = Vec<Option<T>>;

        fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str("a list")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Vec<Option<T>>, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut list: Vec<Option<T>> = Vec::default();
            while let Some(element) = seq.next_element::<Option<JsonOwned<T>>>()? {
                list.push(element.map(JsonOwned::into_value));
            }
            Ok(list)
        }
    }

    deserializer.deserialize_seq(OptionListVisitor {
        element: PhantomData,
    })
}

/// Serializes the `array` as a list of its elements, or as base-64 if its elements are bytes.
pub(crate) fn serialize_array<T, S, const N: usize>(
    array: &[T; N],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: Element,
    S: Serializer,
{
    if T::IS_BYTE {
        let mut bytes: Vec<u8> = Vec::with_capacity(N);
        encode_list(array, &mut bytes, 0);
        Bytes::new(bytes).serialize(serializer)
    } else {
        serialize_list(array, serializer)
    }
}

/// Deserializes an array from a list of exactly its length, or from base-64 if its elements are
/// bytes.
pub(crate) fn deserialize_array<'de, T, D, const N: usize>(
    deserializer: D,
) -> Result<[T; N], D::Error>
where
    T: Element,
    D: Deserializer<'de>,
{
    let list: Vec<T> = if T::IS_BYTE {
        let bytes: Bytes = Bytes::deserialize(deserializer)?;
        ListRef::<T>::new(bytes.as_bytes())
            .map_err(D::Error::custom)?
            .decode()
    } else {
        deserialize_list(deserializer)?
    };
    <[T; N]>::try_from(list).map_err(|_| D::Error::custom("an array of another length"))
}

/// Deserializes an array of optional elements from a list of exactly its length.
pub(crate) fn deserialize_option_array<'de, T, D, const N: usize>(
    deserializer: D,
) -> Result<[Option<T>; N], D::Error>
where
    T: Element,
    D: Deserializer<'de>,
{
    let list: Vec<Option<T>> = deserialize_option_list(deserializer)?;
    <[Option<T>; N]>::try_from(list).map_err(|_| D::Error::custom("an array of another length"))
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

/// Serializes the `map` with optional values as its JSON form, with an absent value as `null`.
pub(crate) fn serialize_option_map<K, V, S>(
    map: &BTreeMap<K, Option<V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    K: Element,
    V: Element,
    S: Serializer,
{
    let entries = map
        .iter()
        .map(|(key, value)| (JsonRef::new(key), value.as_ref().map(JsonRef::new)));
    if K::JSON_TEXT_KEY {
        serializer.collect_map(entries)
    } else {
        serializer.collect_seq(entries)
    }
}

/// Deserializes a map from its JSON form.
///
/// # Note
/// - A map can't repeat a key, in either form.
pub(crate) fn deserialize_map<'de, K, V, D>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
where
    K: Element + Ord,
    V: Element,
    D: Deserializer<'de>,
{
    deserialize_entries::<K, JsonOwned<V>, V, D>(deserializer, JsonOwned::into_value)
}

/// Deserializes a map with optional values from its JSON form, with a `null` as an absent value.
pub(crate) fn deserialize_option_map<'de, K, V, D>(
    deserializer: D,
) -> Result<BTreeMap<K, Option<V>>, D::Error>
where
    K: Element + Ord,
    V: Element,
    D: Deserializer<'de>,
{
    deserialize_entries::<K, Option<JsonOwned<V>>, Option<V>, D>(deserializer, |value| {
        value.map(JsonOwned::into_value)
    })
}

/// Deserializes the entries of a map from its JSON form, with each value read as a `W` & then
/// converted by `convert`.
fn deserialize_entries<'de, K, W, V, D>(
    deserializer: D,
    convert: fn(W) -> V,
) -> Result<BTreeMap<K, V>, D::Error>
where
    K: Element + Ord,
    W: Deserialize<'de>,
    D: Deserializer<'de>,
{
    struct MapVisitor<K, W, V> {
        key: PhantomData<K>,
        convert: fn(W) -> V,
    }

    impl<'de, K, W, V> Visitor<'de> for MapVisitor<K, W, V>
    where
        K: Element + Ord,
        W: Deserialize<'de>,
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
            while let Some((key, value)) = map.next_entry::<JsonOwned<K>, W>()? {
                if result
                    .insert(key.into_value(), (self.convert)(value))
                    .is_some()
                {
                    return Err(A::Error::custom("a repeated key"));
                }
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
            while let Some((key, value)) = seq.next_element::<(JsonOwned<K>, W)>()? {
                if result
                    .insert(key.into_value(), (self.convert)(value))
                    .is_some()
                {
                    return Err(A::Error::custom("a repeated key"));
                }
            }
            Ok(result)
        }
    }

    let visitor: MapVisitor<K, W, V> = MapVisitor {
        key: PhantomData,
        convert,
    };
    if K::JSON_TEXT_KEY {
        deserializer.deserialize_map(visitor)
    } else {
        deserializer.deserialize_seq(visitor)
    }
}
