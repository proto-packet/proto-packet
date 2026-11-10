mod common;

use common::assert_json;
use proto_packet::types::{Date, Duration, Microtime, Nanotime, TimeRangeError, Timestamp};
use proto_packet_test::format::edges::{TimeKinds, TimeKindsMut, TimeKindsRef};
use proto_packet_test::format::messages::AllKinds;
use proto_packet_test::format::variants::AllCases;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Asserts the `value` of the `type` encodes to the `bytes`, & decodes back from them.
macro_rules! assert_encoding {
    ($type:ty, $value:expr, $bytes:expr) => {{
        let value: $type = $value;
        let bytes: &[u8] = $bytes;
        assert_eq!(value.encode(), bytes);
        assert_eq!(value.encoded_len(), bytes.len());
        assert_eq!(<$type>::decode(bytes), Ok(value.clone()));
        assert_eq!(unsafe { <$type>::decode_unchecked(bytes) }, value);
    }};
}

/// Gets a struct with a field of each time type.
fn time_kinds() -> TimeKinds {
    TimeKinds::new(
        Timestamp::new(1),
        Microtime::new(2),
        Nanotime::new(3),
        Date::new(-4),
        Duration::new(-5),
        None,
        Vec::default(),
        BTreeMap::default(),
    )
}

#[test]
fn times_are_their_integers() {
    let bytes: Vec<u8> = [
        &[1, 0, 0, 0, 0, 0, 0, 0][..],                     // at
        &[2, 0, 0, 0, 0, 0, 0, 0],                         // micro
        &[3, 0, 0, 0, 0, 0, 0, 0],                         // nano
        &[0xFB, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], // span
        &[0; 8],                                           // maybe, absent
        &[0xFC, 0xFF, 0xFF, 0xFF],                         // day
        &[49, 0, 0, 0],                                    // the end of the days
        &[0],                                              // bits
    ]
    .concat();
    assert_eq!(bytes.len(), 49);
    assert_encoding!(TimeKinds, time_kinds(), &bytes);

    let kinds: TimeKindsRef = TimeKindsRef::new(&bytes).unwrap();
    assert_eq!(kinds.at().millis(), 1);
    assert_eq!(kinds.micro().micros(), 2);
    assert_eq!(kinds.nano().nanos(), 3);
    assert_eq!(kinds.day().days(), -4);
    assert_eq!(kinds.span().nanos(), -5);
    assert_eq!(kinds.maybe(), None);
}

#[test]
fn times_keep_their_range() {
    let value: TimeKinds = TimeKinds::new(
        Timestamp::new(i64::MIN),
        Microtime::new(i64::MAX),
        Nanotime::new(i64::MIN),
        Date::new(i32::MAX),
        Duration::new(i64::MIN),
        Some(Timestamp::new(i64::MAX)),
        vec![Date::new(i32::MIN), Date::new(0)],
        BTreeMap::from([
            (Date::new(-1), Duration::new(1)),
            (Date::new(1), Duration::new(-1)),
        ]),
    );
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(TimeKinds, value, &bytes);
}

#[test]
fn mut_sets_times_in_place() {
    let mut bytes: Vec<u8> = time_kinds().encode();
    let mut kinds: TimeKindsMut = TimeKindsMut::new(&mut bytes).unwrap();
    kinds.set_at(Timestamp::new(9));
    kinds.set_day(Date::new(8));
    kinds.set_maybe(Timestamp::new(7));
    let decoded: TimeKinds = TimeKinds::decode(&bytes).unwrap();
    assert_eq!(decoded.at(), Timestamp::new(9));
    assert_eq!(decoded.day(), Date::new(8));
    assert_eq!(decoded.maybe(), Some(Timestamp::new(7)));
}

#[test]
fn messages_and_variants_hold_times() {
    let bytes: [u8; 12] = [1, 0, 13, 0, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    assert_encoding!(
        AllKinds,
        AllKinds::default().with_at(Timestamp::new(-2)),
        &bytes
    );
    let bytes: [u8; 10] = [14, 0, 7, 0, 0, 0, 0, 0, 0, 0];
    assert_encoding!(AllCases, AllCases::At(Timestamp::new(7)), &bytes);
}

#[test]
fn times_are_text_in_json() {
    let mut value: TimeKinds = time_kinds();
    value.set_days(vec![Date::new(6)]);
    value.set_spans(BTreeMap::from([(Date::new(7), Duration::new(8))]));
    assert_json(
        &value,
        r#"{"at":"1","micro":"2","nano":"3","day":"-4","span":"-5","days":["6"],"spans":{"7":"8"}}"#,
    );
    assert_json(
        &AllCases::At(Timestamp::new(7)),
        r#"{"var":"At","val":"7"}"#,
    );
    assert_json(
        &AllCases::At(Timestamp::new(i64::MIN)),
        r#"{"var":"At","val":"-9223372036854775808"}"#,
    );
}

#[test]
fn json_rejects_invalid_times() {
    for json in [
        r#"{"var":"At","val":7}"#,                     // a plain number
        r#"{"var":"At","val":"7.0"}"#,                 // not an integer
        r#"{"var":"At","val":"+7"}"#,                  // a plus sign
        r#"{"var":"At","val":" 7"}"#,                  // text around the integer
        r#"{"var":"At","val":""}"#,                    // empty text
        r#"{"var":"At","val":"-"}"#,                   // no digits
        r#"{"var":"At","val":"9223372036854775808"}"#, // out of range
    ] {
        assert!(serde_json::from_str::<AllCases>(json).is_err(), "{}", json);
    }
}

#[test]
fn times_convert_to_the_system_time() {
    let after: SystemTime = UNIX_EPOCH + std::time::Duration::from_nanos(1_234_567_891);
    assert_eq!(Timestamp::try_from(after), Ok(Timestamp::new(1_234)));
    assert_eq!(Microtime::try_from(after), Ok(Microtime::new(1_234_567)));
    assert_eq!(Nanotime::try_from(after), Ok(Nanotime::new(1_234_567_891)));
    assert_eq!(
        SystemTime::try_from(Nanotime::new(1_234_567_891)),
        Ok(after)
    );

    let before: SystemTime = UNIX_EPOCH - std::time::Duration::from_nanos(1_500_000);
    assert_eq!(Timestamp::try_from(before), Ok(Timestamp::new(-2)));
    assert_eq!(Microtime::try_from(before), Ok(Microtime::new(-1_500)));
    assert_eq!(SystemTime::try_from(Microtime::new(-1_500)), Ok(before));
}

#[test]
fn times_out_of_range_do_not_convert() {
    let far: SystemTime = UNIX_EPOCH + std::time::Duration::from_secs(10_000_000_000);
    assert_eq!(Nanotime::try_from(far), Err(TimeRangeError::default()));
    assert!(Timestamp::try_from(far).is_ok());
    assert!(SystemTime::try_from(Timestamp::new(i64::MIN)).is_err() || cfg!(unix));
}

#[test]
fn durations_convert_to_the_standard_duration() {
    let standard: std::time::Duration = std::time::Duration::from_nanos(1_500);
    assert_eq!(Duration::try_from(standard), Ok(Duration::new(1_500)));
    assert_eq!(
        std::time::Duration::try_from(Duration::new(1_500)),
        Ok(standard)
    );
    assert_eq!(
        std::time::Duration::try_from(Duration::new(-1)),
        Err(TimeRangeError::default())
    );
    assert!(Duration::try_from(std::time::Duration::from_secs(u64::MAX)).is_err());
}

#[test]
fn dates_convert_to_the_calendar() {
    for (days, ymd) in [
        (0, (1970, 1, 1)),
        (-1, (1969, 12, 31)),
        (59, (1970, 3, 1)),
        (11_016, (2000, 2, 29)),
        (20_766, (2026, 11, 9)),
        (-719_468, (0, 3, 1)),
        (-719_528, (0, 1, 1)),
        (-141_427, (1582, 10, 15)),
    ] {
        assert_eq!(Date::new(days).to_ymd(), ymd, "{}", days);
        assert_eq!(Date::from_ymd(ymd.0, ymd.1, ymd.2), Some(Date::new(days)));
    }
    for (year, month, day) in [
        (2026, 2, 29),
        (2026, 13, 1),
        (2026, 0, 1),
        (2026, 4, 31),
        (2026, 1, 0),
    ] {
        assert_eq!(Date::from_ymd(year, month, day), None);
    }
    assert_eq!(Date::from_ymd(i32::MAX, 1, 1), None);
    let (far, _, _) = Date::new(i32::MAX).to_ymd();
    assert_eq!(
        Date::from_ymd(far, 1, 1).map(|date| date.to_ymd().0),
        Some(far)
    );
}
