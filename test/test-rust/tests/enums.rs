mod common;

use common::assert_json;
use proto_packet::io::TagNumber;
use proto_packet::serde::Deserialize;
use proto_packet::serde::de::IntoDeserializer;
use proto_packet::serde::de::value::Error;
use proto_packet_test::enums::cases::{EdgeTags, SingleCase, UnorderedTags};
use proto_packet_test::enums::empty::EmptyEnum;

#[test]
fn tag_numbers() {
    assert_eq!(SingleCase::Only.tag_number(), 1);
    assert_eq!(EdgeTags::Min.tag_number(), 1);
    assert_eq!(EdgeTags::Max.tag_number(), 8191);
    assert_eq!(UnorderedTags::Three.tag_number(), 3);
    assert_eq!(UnorderedTags::One.tag_number(), 1);
    assert_eq!(UnorderedTags::Two.tag_number(), 2);
}

/// Gets the tag number with the `value`.
fn tag(value: u16) -> TagNumber {
    TagNumber::new(value).unwrap()
}

#[test]
fn from_tag_numbers() {
    assert_eq!(EdgeTags::from_tag_number(tag(1)), EdgeTags::Min);
    assert_eq!(EdgeTags::from_tag_number(tag(8191)), EdgeTags::Max);
    assert!(matches!(
        EdgeTags::from_tag_number(tag(2)),
        EdgeTags::Unrecognized(_)
    ));
    assert_eq!(UnorderedTags::from_tag_number(tag(3)), UnorderedTags::Three);
    assert_eq!(EmptyEnum::from_tag_number(tag(1)).tag_number(), 1);
}

#[test]
fn from_tag_numbers_optional() {
    assert_eq!(EdgeTags::from_tag_number_optional(1), Some(EdgeTags::Min));
    assert!(matches!(
        EdgeTags::from_tag_number_optional(2),
        Some(EdgeTags::Unrecognized(_))
    ));
    for tag_number in [0, 65536, u32::MAX] {
        assert_eq!(EdgeTags::from_tag_number_optional(tag_number), None);
    }
}

#[test]
fn ordered_by_tag_number() {
    let mut cases: Vec<UnorderedTags> =
        vec![UnorderedTags::Three, UnorderedTags::One, UnorderedTags::Two];
    cases.sort();
    assert_eq!(
        cases,
        [UnorderedTags::One, UnorderedTags::Two, UnorderedTags::Three]
    );
}

#[test]
fn json() {
    assert_json(&SingleCase::Only, r#""Only""#);
    assert_json(&UnorderedTags::Three, r#""Three""#);
}

/// Gets the unrecognized case of `UnorderedTags` with the `tag_number`.
fn unrecognized(tag_number: u16) -> UnorderedTags {
    let case: UnorderedTags = UnorderedTags::from_tag_number(tag(tag_number));
    assert!(matches!(case, UnorderedTags::Unrecognized(_)));
    case
}

#[test]
fn unrecognized_tag_numbers() {
    assert_eq!(unrecognized(4).tag_number(), 4);
    assert_eq!(unrecognized(8191).tag_number(), 8191);
    assert_eq!(unrecognized(4), unrecognized(4));
    assert_ne!(unrecognized(4), unrecognized(5));
}

#[test]
fn unrecognized_ordered_by_tag_number() {
    let mut cases: Vec<UnorderedTags> = vec![
        unrecognized(4),
        UnorderedTags::Three,
        UnorderedTags::One,
        UnorderedTags::Two,
    ];
    cases.sort();
    assert_eq!(
        cases,
        [
            UnorderedTags::One,
            UnorderedTags::Two,
            UnorderedTags::Three,
            unrecognized(4)
        ]
    );

    let mut cases: Vec<EdgeTags> = vec![
        EdgeTags::Max,
        EdgeTags::from_tag_number(tag(2)),
        EdgeTags::Min,
    ];
    cases.sort();
    let tags: Vec<u16> = cases.iter().map(|case| case.tag_number()).collect();
    assert_eq!(tags, [1, 2, 8191]);
}

#[test]
fn unrecognized_json() {
    assert_json(&unrecognized(4), "4");
    assert_json(&EmptyEnum::from_tag_number(tag(1)), "1");
}

#[test]
fn json_tag_numbers() {
    let case: UnorderedTags = serde_json::from_str("3").unwrap();
    assert_eq!(case, UnorderedTags::Three);
    assert_eq!(serde_json::to_string(&case).unwrap(), r#""Three""#);

    for json in [r#""Four""#, "0", "65536", "-1", "true"] {
        assert!(
            serde_json::from_str::<UnorderedTags>(json).is_err(),
            "{}",
            json
        );
    }
}

#[test]
fn signed_tag_numbers() {
    let read = |tag_number: i64| {
        UnorderedTags::deserialize(IntoDeserializer::<Error>::into_deserializer(tag_number))
    };
    assert_eq!(read(3).unwrap(), UnorderedTags::Three);
    assert_eq!(read(4).unwrap(), unrecognized(4));
    for tag_number in [-1, 0, 65536, i64::MAX] {
        assert!(read(tag_number).is_err(), "{}", tag_number);
    }
}
