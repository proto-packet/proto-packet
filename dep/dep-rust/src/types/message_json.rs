use crate::io::{DecodeError, MessageWriter, TagNumber, UnknownField};
use crate::types::base_64::{append_base_64, encode_base_64};
use serde::Deserializer;
use serde::de::{Error as _, IgnoredAny, MapAccess, Visitor};
use serde::ser::{Error as _, SerializeMap};
use std::collections::BTreeSet;
use std::fmt::Formatter;
use std::marker::PhantomData;

/// A generated message read from JSON.
pub trait JsonMessage: Default {
    /// The name of the message.
    const TYPE_NAME: &'static str;

    /// Reads the value of the field with the `name` from the `map`.
    ///
    /// Returns `false`, without reading it, if the message has no field with the `name`.
    fn read_field<'de, A>(&mut self, name: &str, map: &mut A) -> Result<bool, A::Error>
    where
        A: MapAccess<'de>;

    /// Decodes the message from its encoding, the `bytes`.
    fn from_encoded(bytes: &[u8]) -> Result<Self, DecodeError>;

    /// Moves the fields of the `other` message into the message.
    ///
    /// Returns the name of a field in both messages as an error.
    fn merge(&mut self, other: Self) -> Result<(), &'static str>;
}

/// Serializes the unknown `fields` of a generated message.
///
/// Each key is a tag number, which no field name can be, & each value is the field's encoded value
/// as base-64 text, so a reader that knows the field can decode it.
pub fn serialize_unknown_fields<M>(map: &mut M, fields: &[UnknownField]) -> Result<(), M::Error>
where
    M: SerializeMap,
{
    for field in fields {
        let value: String = encode_base_64(field.bytes()).map_err(M::Error::custom)?;
        map.serialize_entry(&field.tag_number().value().to_string(), &value)?;
    }
    Ok(())
}

/// Deserializes a generated message.
///
/// A key is a field name, or a tag number with the field's encoded value as base-64 text. The tag
/// number can be a field the message knows, since it can be written by a message that doesn't.
pub fn deserialize_message<'de, D, M>(deserializer: D) -> Result<M, D::Error>
where
    D: Deserializer<'de>,
    M: JsonMessage,
{
    deserializer.deserialize_map(MessageVisitor {
        message: PhantomData,
    })
}

/// Gets the tag number of the `key`, if it's one.
fn tag_of<E>(key: &str) -> Result<Option<u16>, E>
where
    E: serde::de::Error,
{
    if key.is_empty() || !key.bytes().all(|byte| byte.is_ascii_digit()) {
        return Ok(None);
    }
    key.parse::<u32>()
        .ok()
        .and_then(TagNumber::from_u32)
        .filter(|tag| tag.to_string() == key)
        .map(|tag| Some(tag.value()))
        .ok_or_else(|| E::custom("invalid tag number"))
}

/// Encodes the `fields` as a message.
fn encode_fields<E>(mut fields: Vec<(u16, Vec<u8>)>) -> Result<Vec<u8>, E>
where
    E: serde::de::Error,
{
    fields.sort_by_key(|(tag, _)| *tag);
    if fields.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(E::custom("duplicate tag number"));
    }
    let len: usize = fields.iter().map(|(_, value)| value.len()).sum();
    if len > u32::MAX as usize {
        return Err(E::custom("the fields are too long"));
    }
    let mut bytes: Vec<u8> = Vec::default();
    let mut writer: MessageWriter = MessageWriter::new(&mut bytes, fields.len(), &[]);
    for (tag, value) in &fields {
        writer.field(*tag).extend_from_slice(value);
        writer.end_field();
    }
    writer.finish();
    Ok(bytes)
}

/// Visits a generated message.
struct MessageVisitor<M> {
    message: PhantomData<M>,
}

impl<'de, M> Visitor<'de> for MessageVisitor<M>
where
    M: JsonMessage,
{
    type Value = M;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "a `{}` message", M::TYPE_NAME)
    }

    fn visit_map<A>(self, mut map: A) -> Result<M, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result: M = M::default();
        let mut encoded: Vec<(u16, Vec<u8>)> = Vec::default();
        let mut names: BTreeSet<String> = BTreeSet::default();
        while let Some(key) = map.next_key::<String>()? {
            if let Some(tag) = tag_of(&key)? {
                let text: String = map.next_value()?;
                let mut value: Vec<u8> = Vec::default();
                append_base_64(&text, &mut value).map_err(A::Error::custom)?;
                encoded.push((tag, value));
            } else if names.contains(&key) {
                return Err(A::Error::custom(format!("duplicate field `{}`", key)));
            } else if result.read_field(&key, &mut map)? {
                names.insert(key);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        if !encoded.is_empty() {
            let bytes: Vec<u8> = encode_fields(encoded)?;
            let decoded: M = M::from_encoded(&bytes).map_err(A::Error::custom)?;
            result.merge(decoded).map_err(A::Error::duplicate_field)?;
        }
        Ok(result)
    }
}
