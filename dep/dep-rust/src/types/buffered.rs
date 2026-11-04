use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{Error, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, forward_to_deserialize_any};
use std::fmt::Formatter;
use std::marker::PhantomData;

/// A buffered value, for a value read before the type that reads it is known.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Buffered {
    Unit,
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Large(u128),
    Float(f64),
    String(String),
    Seq(Vec<Buffered>),
    Map(Vec<(Buffered, Buffered)>),
}

impl<'de> Deserialize<'de> for Buffered {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(BufferedVisitor)
    }
}

/// Visits a value to buffer.
struct BufferedVisitor;

impl<'de> Visitor<'de> for BufferedVisitor {
    type Value = Buffered;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("a value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Buffered, E> {
        Ok(Buffered::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Buffered, E> {
        Ok(Buffered::Signed(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Buffered, E> {
        Ok(Buffered::Unsigned(value))
    }

    fn visit_u128<E>(self, value: u128) -> Result<Buffered, E> {
        Ok(Buffered::Large(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Buffered, E> {
        Ok(Buffered::Float(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Buffered, E> {
        Ok(Buffered::String(value.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Buffered, E> {
        Ok(Buffered::String(value))
    }

    fn visit_unit<E>(self) -> Result<Buffered, E> {
        Ok(Buffered::Unit)
    }

    fn visit_none<E>(self) -> Result<Buffered, E> {
        Ok(Buffered::Unit)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Buffered, D::Error>
    where
        D: Deserializer<'de>,
    {
        Buffered::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Buffered, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values: Vec<Buffered> = Vec::default();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(Buffered::Seq(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Buffered, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries: Vec<(Buffered, Buffered)> = Vec::default();
        while let Some(entry) = map.next_entry()? {
            entries.push(entry);
        }
        Ok(Buffered::Map(entries))
    }
}

/// Deserializes a value from a buffered value.
pub(crate) struct BufferedDeserializer<E> {
    value: Buffered,
    error: PhantomData<E>,
}

impl<E> BufferedDeserializer<E> {
    //! Construction

    /// Creates a new deserializer of the `value`.
    pub(crate) const fn new(value: Buffered) -> Self {
        Self {
            value,
            error: PhantomData,
        }
    }
}

impl<'de, E> IntoDeserializer<'de, E> for Buffered
where
    E: Error,
{
    type Deserializer = BufferedDeserializer<E>;

    fn into_deserializer(self) -> Self::Deserializer {
        BufferedDeserializer::new(self)
    }
}

impl<'de, E> Deserializer<'de> for BufferedDeserializer<E>
where
    E: Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Buffered::Unit => visitor.visit_unit(),
            Buffered::Bool(value) => visitor.visit_bool(value),
            Buffered::Unsigned(value) => visitor.visit_u64(value),
            Buffered::Signed(value) => visitor.visit_i64(value),
            Buffered::Large(value) => visitor.visit_u128(value),
            Buffered::Float(value) => visitor.visit_f64(value),
            Buffered::String(value) => visitor.visit_string(value),
            Buffered::Seq(values) => visitor.visit_seq(SeqDeserializer::new(values.into_iter())),
            Buffered::Map(entries) => visitor.visit_map(MapDeserializer::new(entries.into_iter())),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Buffered::Unit => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf unit
        unit_struct seq tuple tuple_struct map struct enum identifier ignored_any
    }
}
