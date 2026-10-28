use code_gen::rust::{WithAttributes, WithDerives};

/// Adds the derives of a generated data type to the `result`.
///
/// The `Ord` & `PartialOrd` derives are skipped unless `is_ord`, for types ordered by hand. (ex: an
/// enum, by tag number)
pub(crate) fn add_derives<R>(result: &mut R, is_copy: bool, is_ord: bool, is_default: bool)
where
    R: WithDerives + WithAttributes,
{
    if is_copy {
        result.add_derive("Copy");
    }
    result.add_derive("Clone");
    if is_ord {
        result.add_derive("Ord");
        result.add_derive("PartialOrd");
    }
    for derive in ["Eq", "PartialEq", "Hash", "Debug"] {
        result.add_derive(derive);
    }
    if is_default {
        result.add_derive("Default");
    }
    result.add_derive("::proto_packet::serde::Serialize");
    result.add_derive("::proto_packet::serde::Deserialize");
    result.add_attribute("serde(crate = \"::proto_packet::serde\")");
}
