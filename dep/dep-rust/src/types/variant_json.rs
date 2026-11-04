use crate::io::{DecodeError, TagNumber};
use crate::types::base_64::{append_base_64, encode_base_64};
use crate::types::buffered::{Buffered, BufferedDeserializer};
use serde::de::{DeserializeSeed, Error as _, IgnoredAny, MapAccess, Visitor};
use serde::ser::{Error as _, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::marker::PhantomData;

/// The key of a variant's case.
const VAR: &str = "var";

/// The key of a variant's value.
const VAL: &str = "val";

/// A generated variant read from JSON.
pub trait JsonVariant: Sized {
    /// The name of the variant.
    const TYPE_NAME: &'static str;

    /// Gets the case with the `name`, with its value from the `value` deserializer.
    fn from_case<'de, D>(name: &str, value: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;

    /// Decodes the variant from its encoding, the `bytes`.
    fn from_encoded(bytes: &[u8]) -> Result<Self, DecodeError>;
}

/// Serializes a case of a generated variant as its `name` & `value`.
pub fn serialize_case<S, T>(
    serializer: S,
    type_name: &'static str,
    name: &str,
    value: &T,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize + ?Sized,
{
    let mut result: S::SerializeStruct = serializer.serialize_struct(type_name, 2)?;
    result.serialize_field(VAR, name)?;
    result.serialize_field(VAL, value)?;
    result.end()
}

/// Serializes an unrecognized case of a generated variant as its `tag_number` & its encoded value.
///
/// The value is base-64 text, so a reader that knows the case can decode it.
pub fn serialize_unknown_case<S>(
    serializer: S,
    type_name: &'static str,
    tag_number: u16,
    bytes: &[u8],
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value: String = encode_base_64(bytes).map_err(S::Error::custom)?;
    let mut result: S::SerializeStruct = serializer.serialize_struct(type_name, 2)?;
    result.serialize_field(VAR, &tag_number)?;
    result.serialize_field(VAL, &value)?;
    result.end()
}

/// Deserializes a generated variant.
///
/// A case is a name with its value, or a tag number with its encoded value as base-64 text. The
/// tag number can be a case the variant knows, since it can be written by a variant that doesn't.
pub fn deserialize_variant<'de, D, V>(deserializer: D) -> Result<V, D::Error>
where
    D: Deserializer<'de>,
    V: JsonVariant,
{
    deserializer.deserialize_map(VariantVisitor {
        variant: PhantomData,
    })
}

/// A key of a variant.
enum Key {
    Var,
    Val,
    Other,
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(KeyVisitor)
    }
}

/// Visits a key of a variant.
struct KeyVisitor;

impl Visitor<'_> for KeyVisitor {
    type Value = Key;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` or `{}`", VAR, VAL)
    }

    fn visit_str<E>(self, key: &str) -> Result<Key, E> {
        Ok(match key {
            VAR => Key::Var,
            VAL => Key::Val,
            _ => Key::Other,
        })
    }
}

/// A selected case of a variant.
enum Selector {
    Name(String),
    Tag(u16),
}

impl<'de> Deserialize<'de> for Selector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SelectorVisitor)
    }
}

/// Visits a selected case of a variant.
struct SelectorVisitor;

impl SelectorVisitor {
    /// Gets the selector of the tag number `value`.
    fn tag<E>(value: Option<u32>) -> Result<Selector, E>
    where
        E: serde::de::Error,
    {
        value
            .and_then(TagNumber::from_u32)
            .map(|tag| Selector::Tag(tag.value()))
            .ok_or_else(|| E::custom("invalid tag number"))
    }
}

impl Visitor<'_> for SelectorVisitor {
    type Value = Selector;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("a case name or tag number")
    }

    fn visit_str<E>(self, name: &str) -> Result<Selector, E> {
        Ok(Selector::Name(name.to_string()))
    }

    fn visit_u64<E>(self, tag: u64) -> Result<Selector, E>
    where
        E: serde::de::Error,
    {
        Self::tag(u32::try_from(tag).ok())
    }

    fn visit_i64<E>(self, tag: i64) -> Result<Selector, E>
    where
        E: serde::de::Error,
    {
        Self::tag(u32::try_from(tag).ok())
    }
}

/// Reads the value of the selected case.
struct CaseSeed<'a, V> {
    selector: &'a Selector,
    variant: PhantomData<V>,
}

impl<'de, V> DeserializeSeed<'de> for CaseSeed<'_, V>
where
    V: JsonVariant,
{
    type Value = V;

    fn deserialize<D>(self, deserializer: D) -> Result<V, D::Error>
    where
        D: Deserializer<'de>,
    {
        match self.selector {
            Selector::Name(name) => V::from_case(name, deserializer),
            Selector::Tag(tag) => {
                let text: String = String::deserialize(deserializer)?;
                let mut bytes: Vec<u8> = tag.to_le_bytes().to_vec();
                append_base_64(&text, &mut bytes).map_err(D::Error::custom)?;
                V::from_encoded(&bytes).map_err(D::Error::custom)
            }
        }
    }
}

/// Visits a generated variant.
struct VariantVisitor<V> {
    variant: PhantomData<V>,
}

impl<'de, V> Visitor<'de> for VariantVisitor<V>
where
    V: JsonVariant,
{
    type Value = V;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "a `{}` variant", V::TYPE_NAME)
    }

    fn visit_map<A>(self, mut map: A) -> Result<V, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut selector: Option<Selector> = None;
        let mut buffered: Option<Buffered> = None;
        let mut result: Option<V> = None;
        while let Some(key) = map.next_key()? {
            match key {
                Key::Var if selector.is_some() => return Err(A::Error::duplicate_field(VAR)),
                Key::Var => selector = Some(map.next_value()?),
                Key::Val if result.is_some() || buffered.is_some() => {
                    return Err(A::Error::duplicate_field(VAL));
                }
                Key::Val => match &selector {
                    Some(selector) => {
                        result = Some(map.next_value_seed(CaseSeed {
                            selector,
                            variant: PhantomData,
                        })?);
                    }
                    None => buffered = Some(map.next_value()?),
                },
                Key::Other => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        if let Some(result) = result {
            return Ok(result);
        }
        let selector: Selector = selector.ok_or_else(|| A::Error::missing_field(VAR))?;
        let buffered: Buffered = buffered.ok_or_else(|| A::Error::missing_field(VAL))?;
        CaseSeed {
            selector: &selector,
            variant: PhantomData,
        }
        .deserialize(BufferedDeserializer::new(buffered))
    }
}
