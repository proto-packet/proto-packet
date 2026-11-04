//! The serde scaffolding shared by the types with hand-written JSON.

/// The path of the runtime's `types` module.
pub(crate) const TYPES: &str = "proto_packet::types";

/// The path of serde.
pub(crate) const SERDE: &str = "proto_packet::serde";

/// Generates the serde impls of the type with the `type_name`.
///
/// The `serialize` code is the body of `serialize`, & `deserialize` is the runtime function that
/// reads the type. The `result` is the path of the prelude `Result`.
pub(crate) fn gen_serde_impls(
    result: &str,
    type_name: &str,
    serialize: &str,
    deserialize: &str,
) -> String {
    format!(
        "impl {serde}::Serialize for {type_name} {{
    fn serialize<S>(&self, serializer: S) -> {result}<S::Ok, S::Error>
    where
        S: {serde}::Serializer,
    {{
        {serialize}
    }}
}}

impl<'de> {serde}::Deserialize<'de> for {type_name} {{
    fn deserialize<D>(deserializer: D) -> {result}<Self, D::Error>
    where
        D: {serde}::Deserializer<'de>,
    {{
        {types}::{deserialize}(deserializer)
    }}
}}
",
        serde = SERDE,
        types = TYPES,
        type_name = type_name,
        result = result,
        serialize = serialize,
        deserialize = deserialize,
    )
}
