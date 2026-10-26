use code_gen::rust::{WithAttributes, WithDerives};

/// Adds the derives of a generated data type to the `result`.
pub(crate) fn add_derives<R>(result: &mut R, is_copy: bool, is_default: bool)
where
    R: WithDerives + WithAttributes,
{
    if is_copy {
        result.add_derive("Copy");
    }
    for derive in [
        "Clone",
        "Ord",
        "PartialOrd",
        "Eq",
        "PartialEq",
        "Hash",
        "Debug",
    ] {
        result.add_derive(derive);
    }
    if is_default {
        result.add_derive("Default");
    }
    result.add_derive("::proto_packet::serde::Serialize");
    result.add_derive("::proto_packet::serde::Deserialize");
    result.add_attribute("serde(crate = \"::proto_packet::serde\")");
}
