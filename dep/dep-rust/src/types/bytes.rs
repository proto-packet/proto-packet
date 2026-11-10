use crate::types::base_64::{append_base_64, encode_base_64};
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

/// The bytes of a `[]u8`.
///
/// # Note
/// - It is base-64 text in JSON, since a list of numbers is several times the size.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct Bytes {
    bytes: Vec<u8>,
}

impl Bytes {
    //! Construction

    /// Creates the [Bytes].
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Self::new(bytes.to_vec())
    }
}

impl<const N: usize> From<[u8; N]> for Bytes {
    fn from(bytes: [u8; N]) -> Self {
        Self::new(bytes.to_vec())
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(bytes: Bytes) -> Self {
        bytes.bytes
    }
}

impl Bytes {
    //! Properties

    /// Gets the bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Gets the bytes to change them.
    pub fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }

    /// Converts the bytes into a vec.
    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.bytes
    }
}

impl DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Debug for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.bytes, f)
    }
}

impl serde::Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let text: String = encode_base_64(&self.bytes).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&text)
    }
}

impl<'de> serde::Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor;

        impl serde::de::Visitor<'_> for BytesVisitor {
            type Value = Bytes;

            fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str("bytes as base-64 text")
            }

            fn visit_str<E>(self, text: &str) -> Result<Bytes, E>
            where
                E: serde::de::Error,
            {
                let mut bytes: Vec<u8> = Vec::default();
                append_base_64(text, &mut bytes).map_err(E::custom)?;
                Ok(Bytes::new(bytes))
            }
        }

        deserializer.deserialize_str(BytesVisitor)
    }
}
