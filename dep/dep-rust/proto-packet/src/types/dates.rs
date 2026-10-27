use crate::types::Date;
use chrono::Datelike;

/// The days from 0001-01-01 to 1970-01-01, counting 0001-01-01 as day 1.
const UNIX_EPOCH_DAYS_FROM_CE: i32 = 719_163;

/// Gets the days since 1970-01-01 of the `date`.
pub fn date_to_days(date: Date) -> i32 {
    date.num_days_from_ce() - UNIX_EPOCH_DAYS_FROM_CE
}

/// Gets the date the `days` since 1970-01-01.
///
/// Returns `None` if the date is outside the range of the dates.
pub fn date_from_days(days: i32) -> Option<Date> {
    Date::from_num_days_from_ce_opt(days.checked_add(UNIX_EPOCH_DAYS_FROM_CE)?)
}

#[cfg(test)]
mod tests {
    use crate::types::{Date, date_from_days, date_to_days};

    #[test]
    fn days_are_since_1970() {
        let test_cases: &[(i32, Date)] = &[
            (0, Date::from_ymd_opt(1970, 1, 1).unwrap()),
            (-1, Date::from_ymd_opt(1969, 12, 31).unwrap()),
            (19_782, Date::from_ymd_opt(2024, 2, 29).unwrap()),
        ];
        for (days, date) in test_cases {
            assert_eq!(date_to_days(*date), *days, "date: {}", date);
            assert_eq!(date_from_days(*days), Some(*date), "days: {}", days);
        }
    }

    #[test]
    fn days_out_of_range_are_none() {
        assert_eq!(date_from_days(date_to_days(Date::MAX)), Some(Date::MAX));
        assert_eq!(date_from_days(date_to_days(Date::MIN)), Some(Date::MIN));
        assert_eq!(date_from_days(date_to_days(Date::MAX) + 1), None);
        assert_eq!(date_from_days(date_to_days(Date::MIN) - 1), None);
        assert_eq!(date_from_days(i32::MAX), None);
        assert_eq!(date_from_days(i32::MIN), None);
    }
}
