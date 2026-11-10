//! The list element impls of the generated types.

use crate::rust::encoding::IO;
use crate::rust::prelude::{Prelude, PreludeName};

/// Generates the list element impl of the struct, message or variant with the `type_name`.
///
/// The element is read through the `ref_name` type. A fixed-size type has a `size`.
///
/// # Note
/// - A type that is `ordered_by_bytes` as the key of a map compares its encoded bytes.
pub(crate) fn gen_element(
    prelude: Prelude,
    type_name: &str,
    ref_name: &str,
    size: Option<usize>,
    alignment: usize,
    can_be_invalid: bool,
    ordered_by_bytes: bool,
) -> String {
    let size: String = match size {
        Some(size) => format!("Some({})", size),
        None => "None".to_string(),
    };
    let (ordered, cmp_refs): (&str, String) = if ordered_by_bytes {
        (
            "\n    const ORDERED_BY_BYTES: bool = true;",
            format!(
                "
    fn cmp_refs(a: super::{ref_name}<'_>, b: super::{ref_name}<'_>) -> std::cmp::Ordering {{
        a.as_bytes().cmp(b.as_bytes())
    }}
",
                ref_name = ref_name
            ),
        )
    } else {
        ("", String::default())
    };
    format!(
        "impl {io}::Element for {type_name} {{
    const SIZE: {option}<usize> = {size};
    const ALIGNMENT: usize = {alignment};
    const CAN_BE_INVALID: bool = {can_be_invalid};{ordered}
    type Ref<'a> = super::{ref_name}<'a>;
    type View<'a> = &'a Self;

    fn check(bytes: &[u8]) -> {result}<(), {io}::DecodeError> {{
        super::{ref_name}::new(bytes).map(|_| ())
    }}

    unsafe fn read_unchecked(bytes: &[u8]) -> super::{ref_name}<'_> {{
        unsafe {{ super::{ref_name}::new_unchecked(bytes) }}
    }}

    fn decode(element: super::{ref_name}<'_>) -> Self {{
        element.decode()
    }}

    fn view(&self) -> &Self {{
        self
    }}
{cmp_refs}
    fn encoded_len(&self) -> usize {{
        {type_name}::encoded_len(self)
    }}

    fn encode_to(&self, out: &mut {vec}<u8>) {{
        {type_name}::encode_to(self, out);
    }}
}}
",
        io = IO,
        type_name = type_name,
        ref_name = ref_name,
        option = prelude.path(PreludeName::Option),
        result = prelude.path(PreludeName::Result),
        vec = prelude.path(PreludeName::Vec),
        size = size,
        alignment = alignment,
        can_be_invalid = can_be_invalid,
        ordered = ordered,
        cmp_refs = cmp_refs,
    )
}

/// Generates the list element impl of the enum with the `type_name`.
pub(crate) fn gen_enum_element(prelude: Prelude, type_name: &str) -> String {
    format!(
        "impl {io}::Element for {type_name} {{
    const SIZE: {option}<usize> = Some(2);
    const ALIGNMENT: usize = 2;
    const CAN_BE_INVALID: bool = true;
    type Ref<'a> = Self;
    type View<'a> = Self;

    fn check(bytes: &[u8]) -> {result}<(), {io}::DecodeError> {{
        {io}::check_exact_len(bytes, 2)?;
        {io}::check_tag_number(unsafe {{ {io}::read_u16_unchecked(bytes, 0) }})
    }}

    unsafe fn read_unchecked(bytes: &[u8]) -> Self {{
        Self::from_tag_number(unsafe {{
            {io}::TagNumber::new_unchecked({io}::read_u16_unchecked(bytes, 0))
        }})
    }}

    fn decode(element: Self) -> Self {{
        element
    }}

    fn view(&self) -> Self {{
        *self
    }}

    fn encoded_len(&self) -> usize {{
        2
    }}

    fn encode_to(&self, out: &mut {vec}<u8>) {{
        out.extend_from_slice(&self.tag_number().to_le_bytes());
    }}
}}
",
        io = IO,
        type_name = type_name,
        option = prelude.path(PreludeName::Option),
        result = prelude.path(PreludeName::Result),
        vec = prelude.path(PreludeName::Vec),
    )
}
