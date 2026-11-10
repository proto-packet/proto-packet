use crate::io::Element;
use serde::{Deserialize, Deserializer};

/// An element that deserializes from its JSON form.
///
/// # Note
/// - A map with a key that isn't text in JSON is a list of its entries, which a map of serde
///   can't read.
#[derive(Debug)]
pub struct JsonOwned<T> {
    value: T,
}

impl<T> JsonOwned<T>
where
    T: Element,
{
    //! Properties

    /// Converts the [JsonOwned] into its element.
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T> JsonOwned<T>
where
    T: Element,
{
    //! Serialization

    /// Deserializes an element from its JSON form.
    ///
    /// # Errors
    /// Returns an error if the JSON is not the element.
    pub fn deserialize_value<'de, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize_json(deserializer)
    }

    /// Deserializes an optional element from its JSON form.
    ///
    /// # Errors
    /// Returns an error if the JSON is not the element.
    pub fn deserialize_option<'de, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Option<Self> = Option::deserialize(deserializer)?;
        Ok(value.map(Self::into_value))
    }
}

impl<'de, T> Deserialize<'de> for JsonOwned<T>
where
    T: Element,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize_json(deserializer).map(|value| Self { value })
    }
}
