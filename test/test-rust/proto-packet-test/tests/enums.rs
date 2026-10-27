mod common;

use common::*;
use proto_packet_test::enums::cases::{EdgeTags, MultiCase, SingleCase};

/// Asserts the type `T` is `Copy`.
fn assert_copy<T: Copy>() {}

#[test]
fn enums_are_copy() {
    assert_copy::<SingleCase>();
    assert_copy::<MultiCase>();
}

#[test]
fn enums_order_by_declaration() {
    let mut result: Vec<MultiCase> = vec![MultiCase::Three, MultiCase::One, MultiCase::Two];
    result.sort();
    assert_eq!(result, [MultiCase::One, MultiCase::Two, MultiCase::Three]);
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
