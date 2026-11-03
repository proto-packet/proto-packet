mod common;

use common::assert_json;
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

#[test]
fn from_tag_numbers() {
    assert_eq!(EdgeTags::from_tag_number(1), Some(EdgeTags::Min));
    assert_eq!(EdgeTags::from_tag_number(8191), Some(EdgeTags::Max));
    assert_eq!(EdgeTags::from_tag_number(0), None);
    assert_eq!(EdgeTags::from_tag_number(8192), None);
    assert!(matches!(
        EdgeTags::from_tag_number(2),
        Some(EdgeTags::Unrecognized(_))
    ));
    assert_eq!(
        UnorderedTags::from_tag_number(3),
        Some(UnorderedTags::Three)
    );
    assert!(matches!(
        EmptyEnum::from_tag_number(1),
        Some(EmptyEnum::Unrecognized(_))
    ));
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
    let case: UnorderedTags = UnorderedTags::from_tag_number(tag_number).unwrap();
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
}

#[test]
fn unrecognized_json() {
    assert_json(&unrecognized(4), "4");
    assert_json(&EmptyEnum::from_tag_number(1).unwrap(), "1");
}

#[test]
fn json_tag_numbers() {
    let case: UnorderedTags = serde_json::from_str("3").unwrap();
    assert_eq!(case, UnorderedTags::Three);
    assert_eq!(serde_json::to_string(&case).unwrap(), r#""Three""#);

    for json in [r#""Four""#, "0", "8192", "65536", "-1", "true"] {
        assert!(
            serde_json::from_str::<UnorderedTags>(json).is_err(),
            "{}",
            json
        );
    }
}
