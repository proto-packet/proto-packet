use crate::io::Element;
use serde::{Serialize, Serializer};

/// A borrowed element that serializes as its JSON form.
///
/// # Note
/// - A map with a key that isn't text in JSON is a list of its entries, which a map of serde
///   can't write.
#[derive(Debug)]
pub struct JsonRef<'a, T> {
    value: &'a T,
}

impl<'a, T> JsonRef<'a, T>
where
    T: Element,
{
    //! Construction

    /// Creates the [JsonRef].
    #[must_use]
    pub const fn new(value: &'a T) -> Self {
        Self { value }
    }
}

impl<T> Copy for JsonRef<'_, T> {}

impl<T> Clone for JsonRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> JsonRef<'_, T>
where
    T: Element,
{
    //! Serialization

    /// Serializes the `value` as its JSON form.
    ///
    /// # Errors
    /// Returns an error if the `serializer` does.
    pub fn serialize_value<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.serialize_json(serializer)
    }

    /// Serializes the optional `value` as its JSON form.
    ///
    /// # Errors
    /// Returns an error if the `serializer` does.
    pub fn serialize_option<S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer.serialize_some(&JsonRef::new(value)),
            None => serializer.serialize_none(),
        }
    }
}

impl<T> Serialize for JsonRef<'_, T>
where
    T: Element,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize_json(serializer)
    }
}
