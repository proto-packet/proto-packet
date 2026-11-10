use crate::types::time::impl_time::impl_time;

/// A calendar date, as the days since `1970-01-01`.
///
/// # Note
/// - It has no time zone.
/// - The calendar is the proleptic Gregorian calendar.
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
