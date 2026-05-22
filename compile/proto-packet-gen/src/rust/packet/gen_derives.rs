use crate::rust::GenRust;
use code_gen::rust::{WithAttributes, WithDerives};

impl GenRust<'_> {
    //! Packet: Derives

    pub(super) fn gen_derives<R>(self, is_copy: bool, is_default: bool, result: &mut R)
    where
        R: WithDerives + WithAttributes,
    {
        if is_copy {
            result.add_derive("Copy");
        }
        result.add_derive("Clone");
        result.add_derive("Ord");
        result.add_derive("PartialOrd");
        result.add_derive("Eq");
        result.add_derive("PartialEq");
        result.add_derive("Hash");
        result.add_derive("Debug");
        if is_default {
            result.add_derive("Default");
        }
        result.add_derive("serde::Serialize");
        result.add_derive("serde::Deserialize");
    }
}
