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
