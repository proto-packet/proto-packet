use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// A decimal number with `S` fraction digits.
///
/// # Note
/// - It holds the number times 10 to the `S`, as an integer. (ex: `1.50` is 150 units when `S` is 2)
/// - So it is exact, & equal numbers have equal bits.
/// - The scale `S` is from 0 to 18.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct Decimal<const S: u8> {
    units: i64,
}

impl<const S: u8> Decimal<S> {
    //! Constants

    /// The count of fraction digits.
    pub const SCALE: u8 = S;

    /// The largest scale.
    pub const MAX_SCALE: u8 = 18;

    /// The count of units in 1.
    const ONE: u128 = {
        assert!(S <= Self::MAX_SCALE, "a decimal's scale is from 0 to 18");
        10_u128.pow(S as u32)
    };
}

impl<const S: u8> Decimal<S> {
    //! Construction

    /// Creates the [Decimal] of the `units`. (ex: 150 for `1.50` when `S` is 2)
    #[must_use]
    pub const fn new(units: i64) -> Self {
        Self { units }
    }
}

impl<const S: u8> Decimal<S> {
    //! Properties

    /// Gets the units, the number times 10 to the `S`.
    #[must_use]
    pub const fn units(self) -> i64 {
        self.units
    }

    /// Gets the little-endian bytes.
    #[must_use]
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.units.to_le_bytes()
    }
}

impl<const S: u8> Decimal<S> {
    //! Math

    /// Adds the `other` decimal.
    ///
    /// Returns `None` if the sum is out of range.
    #[must_use]
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.units.checked_add(other.units) {
            Some(units) => Some(Self { units }),
            None => None,
        }
    }

    /// Subtracts the `other` decimal.
    ///
    /// Returns `None` if the difference is out of range.
    #[must_use]
    pub const fn checked_sub(self, other: Self) -> Option<Self> {
        match self.units.checked_sub(other.units) {
            Some(units) => Some(Self { units }),
            None => None,
        }
    }
}

impl<const S: u8> Debug for Decimal<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<const S: u8> Display for Decimal<S> {
    /// Writes the number with all `S` of its fraction digits. (ex: `1.50`)
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let units: u128 = self.units.unsigned_abs() as u128;
        if self.units < 0 {
            f.write_str("-")?;
        }
        write!(f, "{}", units / Self::ONE)?;
        if S > 0 {
            write!(f, ".{:0width$}", units % Self::ONE, width = S as usize)?;
        }
        Ok(())
    }
}

impl<const S: u8> FromStr for Decimal<S> {
    type Err = DecimalParseError;

    /// Parses the number from its digits, with up to `S` fraction digits.
    ///
    /// # Note
    /// - There is no exponent, no `+` & no space.
    fn from_str(text: &str) -> Result<Self, DecimalParseError> {
        let (is_negative, digits): (bool, &str) = match text.strip_prefix('-') {
            Some(digits) => (true, digits),
            None => (false, text),
        };
        let (whole, fraction): (&str, &str) = match digits.split_once('.') {
            Some((whole, fraction)) if !fraction.is_empty() => (whole, fraction),
            Some(_) => return Err(DecimalParseError::default()),
            None => (digits, ""),
        };
        let is_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty()
            || !is_digits(whole)
            || !is_digits(fraction)
            || fraction.len() > S as usize
        {
            return Err(DecimalParseError::default());
        }
        let mut units: u128 = 0;
        for digit in whole.bytes().chain(fraction.bytes()) {
            units = units
                .checked_mul(10)
                .and_then(|units| units.checked_add((digit - b'0') as u128))
                .filter(|units| *units <= i64::MIN.unsigned_abs() as u128 * 10)
                .ok_or_else(DecimalParseError::default)?;
        }
        units *= 10_u128.pow(S as u32 - fraction.len() as u32);
        let units: i128 = if is_negative {
            -(units as i128)
        } else {
            units as i128
        };
        i64::try_from(units)
            .map(Self::new)
            .map_err(|_| DecimalParseError::default())
    }
}

impl<const S: u8> serde::Serialize for Decimal<S> {
    /// Serializes the decimal as text.
    ///
    /// # Note
    /// - A JSON parser can round a plain number, so the text is quoted.
    fn serialize<R>(&self, serializer: R) -> Result<R::Ok, R::Error>
    where
        R: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de, const S: u8> serde::Deserialize<'de> for Decimal<S> {
    /// Deserializes the decimal from its text.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DecimalVisitor<const S: u8>;

        impl<const S: u8> serde::de::Visitor<'_> for DecimalVisitor<S> {
            type Value = Decimal<S>;

            fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "a decimal as text, with up to {} fraction digits", S)
            }

            fn visit_str<E>(self, text: &str) -> Result<Decimal<S>, E>
            where
                E: serde::de::Error,
            {
                text.parse()
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(text), &self))
            }
        }

        deserializer.deserialize_str(DecimalVisitor::<S>)
    }
}

/// An error from text that is not a decimal of its scale & range.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct DecimalParseError {}

impl Display for DecimalParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid decimal")
    }
}

impl std::error::Error for DecimalParseError {}
