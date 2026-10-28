mod common;

use common::*;
use proto_packet_test::enums::cases::{EdgeTags, MultiCase, SingleCase, UnorderedTags};

/// Asserts the type `T` is `Copy`.
fn assert_copy<T: Copy>() {}

#[test]
fn enums_are_copy() {
    assert_copy::<SingleCase>();
    assert_copy::<MultiCase>();
}

#[test]
fn enums_order_by_tag_number() {
    let mut result: Vec<UnorderedTags> = vec![
        UnorderedTags::Middle,
        UnorderedTags::Last,
        UnorderedTags::First,
    ];
    result.sort();
    let expected: [UnorderedTags; 3] = [
        UnorderedTags::First,
        UnorderedTags::Middle,
        UnorderedTags::Last,
    ];
    assert_eq!(result, expected);
    assert!(UnorderedTags::First < UnorderedTags::Last);
}

#[test]
fn enums_json() {
    assert_json(&SingleCase::One, r#""One""#);
    assert_json(&MultiCase::One, r#""One""#);
    assert_json(&MultiCase::Two, r#""Two""#);
    assert_json(&MultiCase::Three, r#""Three""#);
}

#[test]
fn tags_do_not_change_the_json() {
    assert_json(&EdgeTags::Min, r#""Min""#);
    assert_json(&EdgeTags::Gap, r#""Gap""#);
    assert_json(&EdgeTags::Max, r#""Max""#);
}
