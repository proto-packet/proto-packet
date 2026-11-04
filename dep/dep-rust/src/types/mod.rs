pub use deserialize_enum::*;
pub use variant_json::*;

/// A 16-byte universally unique identifier.
pub use ::uuid::Uuid;

mod buffered;
mod deserialize_enum;
mod variant_json;
