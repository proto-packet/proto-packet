//! The equality of the forms of the generated types.
//!
//! A value has one encoding, so two encoded forms are equal when their bytes are.

use crate::rust::prelude::{Prelude, PreludeName};

/// Generates the equality of the native type with the `type_name` & its encoded forms.
///
/// # Note
/// - The native value is encoded to compare it, unless its length already differs.
pub(crate) fn gen_native_eq(
    prelude: Prelude,
    type_name: &str,
    ref_name: &str,
    buf_name: &str,
) -> String {
    format!(
        "impl {eq}<super::{ref_name}<'_>> for {type_name} {{
    fn eq(&self, other: &super::{ref_name}<'_>) -> bool {{
        self.encoded_len() == other.as_bytes().len() && self.encode() == other.as_bytes()
    }}
}}

impl {eq}<super::{buf_name}> for {type_name} {{
    fn eq(&self, other: &super::{buf_name}) -> bool {{
        *self == other.as_ref()
    }}
}}
",
        eq = prelude.path(PreludeName::PartialEq),
        type_name = type_name,
        ref_name = ref_name,
        buf_name = buf_name,
    )
}

/// Generates the equality of the `Ref` type with the `ref_name` & the other forms of its type.
pub(crate) fn gen_ref_eq(type_name: &str, ref_name: &str, buf_name: &str) -> String {
    format!(
        "impl PartialEq<super::{type_name}> for {ref_name}<'_> {{
    fn eq(&self, other: &super::{type_name}) -> bool {{
        other == self
    }}
}}

impl PartialEq<super::{buf_name}> for {ref_name}<'_> {{
    fn eq(&self, other: &super::{buf_name}) -> bool {{
        self.bytes == other.as_bytes()
    }}
}}
",
        type_name = type_name,
        ref_name = ref_name,
        buf_name = buf_name,
    )
}

/// Generates the equality of the `Buf` type with the `buf_name` & the other forms of its type.
pub(crate) fn gen_buf_eq(type_name: &str, ref_name: &str, buf_name: &str) -> String {
    format!(
        "impl PartialEq<super::{type_name}> for {buf_name} {{
    fn eq(&self, other: &super::{type_name}) -> bool {{
        other == self
    }}
}}

impl PartialEq<super::{ref_name}<'_>> for {buf_name} {{
    fn eq(&self, other: &super::{ref_name}<'_>) -> bool {{
        self.bytes == other.as_bytes()
    }}
}}
",
        type_name = type_name,
        ref_name = ref_name,
        buf_name = buf_name,
    )
}
