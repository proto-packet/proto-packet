use crate::types::time::impl_time::impl_time;

/// A calendar date, as the days since `1970-01-01`.
///
/// # Note
/// - It has no time zone.
/// - The calendar is the proleptic Gregorian calendar.
/// - Its text is its ISO 8601 calendar date. (ex: `2026-11-11`) A year before 0 or after 9999 has a
///   sign & at least 4 digits. (ex: `-0001-01-01`, `+10000-01-01`)
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct Date {
    days: i32,
}

impl_time!(Date, days, i32);

impl Date {
    //! Calendar

    /// Creates the date of the `year`, the `month` from 1 & the `day` from 1.
    ///
    /// Returns `None` if there is no such day, or the date is out of range.
    #[must_use]
    pub const fn from_ymd(year: i32, month: u32, day: u32) -> Option<Self> {
        if month < 1 || month > 12 || day < 1 || day > Self::days_in_month(year, month) {
            return None;
        }
        let (month, day): (i64, i64) = (month as i64, day as i64);
        let year: i64 = if month <= 2 {
            year as i64 - 1
        } else {
            year as i64
        };
        let era: i64 = year.div_euclid(400);
        let year_of_era: i64 = year.rem_euclid(400);
        let shifted_month: i64 = if month > 2 { month - 3 } else { month + 9 };
        let day_of_year: i64 = (153 * shifted_month + 2) / 5 + day - 1;
        let day_of_era: i64 = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        let days: i64 = era * 146_097 + day_of_era - 719_468;
        if days < i32::MIN as i64 || days > i32::MAX as i64 {
            None
        } else {
            Some(Self { days: days as i32 })
        }
    }

    /// Gets the year, the month from 1 & the day from 1.
    #[must_use]
    pub const fn to_ymd(self) -> (i32, u32, u32) {
        let days: i64 = self.days as i64 + 719_468;
        let era: i64 = days.div_euclid(146_097);
        let day_of_era: i64 = days.rem_euclid(146_097);
        let year_of_era: i64 =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year: i64 =
            day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month: i64 = (5 * day_of_year + 2) / 153;
        let day: i64 = day_of_year - (153 * shifted_month + 2) / 5 + 1;
        let month: i64 = if shifted_month < 10 {
            shifted_month + 3
        } else {
            shifted_month - 9
        };
        let year: i64 = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
        (year as i32, month as u32, day as u32)
    }

    /// Gets the count of days in the `month` of the `year`.
    const fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            4 | 6 | 9 | 11 => 30,
            2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
            2 => 28,
            _ => 31,
        }
    }
}

impl Date {
    //! Text

    /// Parses the date from its ISO 8601 `text`. (none if it isn't one, or there is no such day)
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (signed, rest) = match text.as_bytes().first()? {
            b'+' | b'-' => (true, &text[1..]),
            _ => (false, text),
        };
        let mut parts = rest.split('-');
        let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
        let digits = |part: &str, length: usize| {
            part.len() == length && part.bytes().all(|b| b.is_ascii_digit())
        };
        let year_ok: bool = match signed {
            true => year.len() >= 4 && year.bytes().all(|b| b.is_ascii_digit()),
            false => digits(year, 4),
        };
        if parts.next().is_some() || !year_ok || !digits(month, 2) || !digits(day, 2) {
            return None;
        }
        let year: i32 = match text.starts_with('-') {
            true => -year.parse::<i32>().ok()?,
            false => year.parse().ok()?,
        };
        Self::from_ymd(year, month.parse().ok()?, day.parse().ok()?)
    }
}

impl std::fmt::Display for Date {
    /// Writes the date's ISO 8601 calendar date.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (year, month, day) = self.to_ymd();
        match year {
            0..=9999 => write!(f, "{year:04}-{month:02}-{day:02}"),
            ..0 => write!(f, "-{:04}-{month:02}-{day:02}", year.unsigned_abs()),
            _ => write!(f, "+{year:04}-{month:02}-{day:02}"),
        }
    }
}

impl serde::Serialize for Date {
    /// Serializes the date as its ISO 8601 calendar date.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Date {
    /// Deserializes the date from its ISO 8601 calendar date.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DateVisitor;

        impl serde::de::Visitor<'_> for DateVisitor {
            type Value = Date;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an ISO 8601 calendar date")
            }

            fn visit_str<E>(self, text: &str) -> Result<Date, E>
            where
                E: serde::de::Error,
            {
                Date::parse(text)
                    .ok_or_else(|| E::invalid_value(serde::de::Unexpected::Str(text), &self))
            }
        }

        deserializer.deserialize_str(DateVisitor)
    }
}
