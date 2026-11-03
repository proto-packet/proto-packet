use serde::Deserializer;
use serde::de::{Error, Unexpected, Visitor};
use std::fmt::Formatter;

/// Deserializes a generated enum from a case name or a tag number.
///
/// The `from_name` function gets the case with a name, & the `from_tag_number` function gets the
/// case with a tag number.
pub fn deserialize_enum<'de, D, E>(
    deserializer: D,
    type_name: &'static str,
    from_name: fn(&str) -> Option<E>,
    from_tag_number: fn(u32) -> Option<E>,
) -> Result<E, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(EnumVisitor {
        type_name,
        from_name,
        from_tag_number,
    })
}

/// Visits a case name or a tag number of a generated enum.
struct EnumVisitor<E> {
    type_name: &'static str,
    from_name: fn(&str) -> Option<E>,
    from_tag_number: fn(u32) -> Option<E>,
}

impl<E> Visitor<'_> for EnumVisitor<E> {
    type Value = E;

    fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "a `{}` case name or tag number", self.type_name)
    }

    fn visit_str<R>(self, name: &str) -> Result<E, R>
    where
        R: Error,
    {
        (self.from_name)(name).ok_or_else(|| R::invalid_value(Unexpected::Str(name), &self))
    }

    fn visit_u64<R>(self, tag_number: u64) -> Result<E, R>
    where
        R: Error,
    {
        u32::try_from(tag_number)
            .ok()
            .and_then(self.from_tag_number)
            .ok_or_else(|| R::invalid_value(Unexpected::Unsigned(tag_number), &self))
    }
}
