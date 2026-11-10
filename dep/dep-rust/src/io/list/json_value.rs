/// A value that can be written as JSON, when the `types` feature is on.
///
/// # Note
/// - Every type has it when the feature is off, so an element needs no JSON then.
#[cfg(feature = "types")]
pub trait JsonValue: serde::Serialize + serde::de::DeserializeOwned {}

#[cfg(feature = "types")]
impl<T> JsonValue for T where T: serde::Serialize + serde::de::DeserializeOwned {}

/// A value that can be written as JSON, when the `types` feature is on.
///
/// # Note
/// - Every type has it when the feature is off, so an element needs no JSON then.
#[cfg(not(feature = "types"))]
pub trait JsonValue {}

#[cfg(not(feature = "types"))]
impl<T> JsonValue for T {}
