//! The text of the numbers that JSON has as strings.

use serde::Deserializer;
use serde::de::{Error, Unexpected, Visitor};
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::str::FromStr;

/// Checks if the `text` is one or more digits.
fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// Checks if the `text` is the digits of a whole number, with no leading zeros.
pub(crate) fn is_whole(text: &str) -> bool {
    is_digits(text) && (text == "0" || !text.starts_with('0'))
}

/// Checks if the `text` is an integer as JSON writes a number. (ex: `-12`)
///
/// # Note
/// - It has no `+`, no leading zeros & no space.
pub(crate) fn is_json_integer(text: &str) -> bool {
    is_whole(text.strip_prefix('-').unwrap_or(text))
}

/// Checks if the `text` is a number as JSON writes one. (ex: `-1.5e-7`)
///
/// # Note
/// - It has no `+` before it, no leading zeros, no space & a digit on each side of a `.`.
pub(crate) fn is_json_number(text: &str) -> bool {
    let text: &str = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent): (&str, Option<&str>) = match text.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (text, None),
    };
    let (whole, fraction): (&str, Option<&str>) = match mantissa.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (mantissa, None),
    };
    is_whole(whole)
        && fraction.is_none_or(is_digits)
        && exponent
            .is_none_or(|exponent| is_digits(exponent.strip_prefix(['+', '-']).unwrap_or(exponent)))
}

/// Deserializes an integer from its text.
///
/// # Note
/// - A large integer is text in JSON, since many JSON parsers read a number as a float.
pub(crate) fn deserialize_integer<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    D: Deserializer<'de>,
{
    struct IntegerVisitor<T> {
        integer: PhantomData<T>,
    }

    impl<T> Visitor<'_> for IntegerVisitor<T>
    where
        T: FromStr,
    {
        type Value = T;

        fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str("an integer as text")
        }

        fn visit_str<E>(self, text: &str) -> Result<T, E>
        where
            E: Error,
        {
            text.parse::<T>()
                .ok()
                .filter(|_| is_json_integer(text))
                .ok_or_else(|| E::invalid_value(Unexpected::Str(text), &self))
        }
    }

    deserializer.deserialize_str(IntegerVisitor {
        integer: PhantomData,
    })
}
