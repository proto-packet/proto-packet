//! The order of the generated messages & variants.

use crate::rust::prelude::{Prelude, PreludeName};

/// Generates the `PartialOrd` impl of the type with the `type_name`, from its `Ord` impl.
fn gen_partial_ord(prelude: Prelude, type_name: &str) -> String {
    format!(
        "impl {partial_ord} for {type_name} {{
    fn partial_cmp(&self, other: &Self) -> {option}<std::cmp::Ordering> {{
        Some(self.cmp(other))
    }}
}}
",
        partial_ord = prelude.path(PreludeName::PartialOrd),
        option = prelude.path(PreludeName::Option),
        type_name = type_name,
    )
}

/// Generates the order of the message with the `type_name`, with its `fields` in tag number order.
///
/// # Note
/// - The order is by tag number, since the declaration order can change in the schema.
/// - An absent field comes before a present one, & the unknown fields are compared last.
pub(crate) fn gen_message_ord(prelude: Prelude, type_name: &str, fields: &[String]) -> String {
    let mut compare: String = match fields.first() {
        Some(field) => format!("self.{}.cmp(&other.{})", field, field),
        None => "self.unknown_fields.cmp(&other.unknown_fields)".to_string(),
    };
    for field in fields.iter().skip(1) {
        compare.push_str(&format!(
            ".then_with(|| self.{}.cmp(&other.{}))",
            field, field
        ));
    }
    if !fields.is_empty() {
        compare.push_str(".then_with(|| self.unknown_fields.cmp(&other.unknown_fields))");
    }
    format!(
        "impl {ord} for {type_name} {{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {{
        {compare}
    }}
}}

{partial_ord}",
        ord = prelude.path(PreludeName::Ord),
        type_name = type_name,
        compare = compare,
        partial_ord = gen_partial_ord(prelude, type_name),
    )
}

/// Generates the order of the variant with the `type_name` & the idents of its `cases`.
///
/// # Note
/// - The order is by tag number, then by the value, since the declaration order can change in the
///   schema.
pub(crate) fn gen_variant_ord(prelude: Prelude, type_name: &str, cases: &[String]) -> String {
    let compare: String = if cases.is_empty() {
        "let (Self::Unrecognized(a), Self::Unrecognized(b)) = (self, other);
        a.cmp(b)"
            .to_string()
    } else {
        let arms: Vec<String> = cases
            .iter()
            .map(|case| format!("(Self::{}(a), Self::{}(b)) => a.cmp(b),", case, case))
            .collect();
        format!(
            "match (self, other) {{
            {}
            (Self::Unrecognized(a), Self::Unrecognized(b)) => a.cmp(b),
            _ => self.tag_number().cmp(&other.tag_number()),
        }}",
            arms.join("\n            ")
        )
    };
    format!(
        "impl {ord} for {type_name} {{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {{
        {compare}
    }}
}}

{partial_ord}",
        ord = prelude.path(PreludeName::Ord),
        type_name = type_name,
        compare = compare,
        partial_ord = gen_partial_ord(prelude, type_name),
    )
}
