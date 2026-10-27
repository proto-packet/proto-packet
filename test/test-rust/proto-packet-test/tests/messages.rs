mod common;

use common::*;
use proto_packet::types::{Float32, Float64};
use proto_packet_test::messages::boxed::BoxedFields;
use proto_packet_test::messages::empty::Empty;
use proto_packet_test::messages::maps::MapFields;
use proto_packet_test::messages::named::{NamedFields, NamedSlices};
use proto_packet_test::messages::tags::{EdgeTags, UnorderedTags};
use proto_packet_test::messages::{fields, slices};
use proto_packet_test::types::{Color, Label, Path, Point, Shape};
use std::collections::BTreeMap;

#[test]
fn fields_default_to_none() {
    let value: fields::UnsignedIntFields = fields::UnsignedIntFields::default();
    assert_eq!(value.one(), None);
    assert_eq!(value.five(), None);

    let value: fields::SpecialFields = fields::SpecialFields::default();
    assert_eq!(value.one(), None);
    assert_eq!(value.two(), None);
}

#[test]
fn fields_getters() {
    let boolean: fields::BooleanField = fields::BooleanField::default().with_one(true);
    assert_eq!(boolean.one(), Some(true));

    let unsigned: fields::UnsignedIntFields = fields::UnsignedIntFields::default()
        .with_one(u8::MAX)
        .with_two(u16::MAX)
        .with_three(u32::MAX)
        .with_four(u64::MAX)
        .with_five(u128::MAX);
    assert_eq!(unsigned.one(), Some(u8::MAX));
    assert_eq!(unsigned.two(), Some(u16::MAX));
    assert_eq!(unsigned.three(), Some(u32::MAX));
    assert_eq!(unsigned.four(), Some(u64::MAX));
    assert_eq!(unsigned.five(), Some(u128::MAX));

    let signed: fields::SignedIntFields = fields::SignedIntFields::default()
        .with_one(i8::MIN)
        .with_two(i16::MIN)
        .with_three(i32::MIN)
        .with_four(i64::MIN)
        .with_five(i128::MIN);
    assert_eq!(signed.one(), Some(i8::MIN));
    assert_eq!(signed.two(), Some(i16::MIN));
    assert_eq!(signed.three(), Some(i32::MIN));
    assert_eq!(signed.four(), Some(i64::MIN));
    assert_eq!(signed.five(), Some(i128::MIN));

    let floats: fields::FloatFields = fields::FloatFields::default()
        .with_one(1.5_f32)
        .with_two(-2.5);
    assert_eq!(floats.one(), Some(1.5));
    assert_eq!(floats.two(), Some(-2.5));

    let special: fields::SpecialFields = fields::SpecialFields::default()
        .with_one(id())
        .with_two("text".to_string());
    assert_eq!(special.one(), Some(id()));
    assert_eq!(special.two(), Some("text"));

    let time: fields::TimeFields = fields::TimeFields::default()
        .with_one(-1_i64)
        .with_two(day());
    assert_eq!(time.one(), Some(-1));
    assert_eq!(time.two(), Some(day()));
}

#[test]
fn fields_setters_return_the_previous_value() {
    let mut value: fields::SpecialFields = fields::SpecialFields::default();
    let result: Option<String> = value.set_two("old".to_string());
    assert_eq!(result, None);

    let result: Option<String> = value.set_two(Some("new".to_string()));
    assert_eq!(result.as_deref(), Some("old"));

    let result: Option<String> = value.set_two(None);
    assert_eq!(result.as_deref(), Some("new"));
    assert_eq!(value.two(), None);
}

#[test]
fn fields_json() {
    assert_json(&fields::BooleanField::default(), "{}");
    assert_json(
        &fields::BooleanField::default().with_one(false),
        r#"{"one":false}"#,
    );
    assert_json(
        &fields::UnsignedIntFields::default()
            .with_one(u8::MAX)
            .with_five(u128::MAX),
        &format!(r#"{{"one":{},"five":{}}}"#, u8::MAX, u128::MAX),
    );
    assert_json(
        &fields::SignedIntFields::default()
            .with_two(i16::MIN)
            .with_five(i128::MIN),
        &format!(r#"{{"two":{},"five":{}}}"#, i16::MIN, i128::MIN),
    );
    assert_json(
        &fields::FloatFields::default().with_two(-2.5),
        r#"{"two":-2.5}"#,
    );
    assert_json(
        &fields::SpecialFields::default()
            .with_one(id())
            .with_two("text".to_string()),
        &format!(r#"{{"one":{},"two":"text"}}"#, ID_JSON),
    );
    assert_json(
        &fields::TimeFields::default()
            .with_one(-1_i64)
            .with_two(day()),
        &format!(r#"{{"one":-1,"two":{}}}"#, DAY_JSON),
    );
}

#[test]
fn slices_getters() {
    let value: slices::UnsignedIntFields = slices::UnsignedIntFields::default()
        .with_one(vec![1_u8])
        .with_five(vec![5_u128, 6]);
    assert_eq!(value.one(), Some([1].as_slice()));
    assert_eq!(value.two(), None);
    assert_eq!(value.five(), Some([5, 6].as_slice()));

    let value: slices::FloatFields =
        slices::FloatFields::default().with_one(vec![Float32::new(1.5)]);
    assert_eq!(value.one(), Some([Float32::new(1.5)].as_slice()));

    let value: slices::SpecialFields =
        slices::SpecialFields::default().with_two(vec!["a".to_string()]);
    assert_eq!(value.two(), Some(["a".to_string()].as_slice()));

    let value: slices::NestedSlices =
        slices::NestedSlices::default().with_one(vec![vec![1_u8, 2], vec![]]);
    assert_eq!(value.one(), Some([vec![1, 2], vec![]].as_slice()));
}

#[test]
fn slices_json() {
    assert_json(&slices::BooleanField::default(), "{}");
    assert_json(
        &slices::BooleanField::default().with_one(vec![]),
        r#"{"one":[]}"#,
    );
    assert_json(
        &slices::UnsignedIntFields::default().with_three(vec![3_u32, 4]),
        r#"{"three":[3,4]}"#,
    );
    assert_json(
        &slices::SignedIntFields::default().with_four(vec![-4_i64]),
        r#"{"four":[-4]}"#,
    );
    assert_json(
        &slices::FloatFields::default()
            .with_one(vec![Float32::new(1.5)])
            .with_two(vec![Float64::new(-2.5)]),
        r#"{"one":[1.5],"two":[-2.5]}"#,
    );
    assert_json(
        &slices::SpecialFields::default().with_one(vec![id()]),
        &format!(r#"{{"one":[{}]}}"#, ID_JSON),
    );
    assert_json(
        &slices::TimeFields::default().with_two(vec![day()]),
        &format!(r#"{{"two":[{}]}}"#, DAY_JSON),
    );
    assert_json(
        &slices::NestedSlices::default()
            .with_one(vec![vec![1_u8, 2], vec![]])
            .with_two(vec![vec!["a".to_string()]]),
        r#"{"one":[[1,2],[]],"two":[["a"]]}"#,
    );
}

#[test]
fn named_fields_getters() {
    let value: NamedFields = NamedFields::default()
        .with_point(point())
        .with_path(path())
        .with_label(label())
        .with_color(Color::Green)
        .with_shape(shape());
    let result: Option<Point> = value.point();
    assert_eq!(result, Some(point()));
    let result: Option<&Path> = value.path();
    assert_eq!(result, Some(&path()));
    let result: Option<&Label> = value.label();
    assert_eq!(result, Some(&label()));
    let result: Option<Color> = value.color();
    assert_eq!(result, Some(Color::Green));
    let result: Option<&Shape> = value.shape();
    assert_eq!(result, Some(&shape()));
}

#[test]
fn named_fields_json() {
    assert_json(&NamedFields::default(), "{}");
    assert_json(
        &NamedFields::default()
            .with_point(point())
            .with_path(path())
            .with_label(label())
            .with_color(Color::Green)
            .with_shape(shape()),
        &format!(
            r#"{{"point":{},"path":{},"label":{},"color":"Green","shape":{}}}"#,
            POINT_JSON, PATH_JSON, LABEL_JSON, SHAPE_JSON
        ),
    );
}

#[test]
fn named_slices_json() {
    let value: NamedSlices = NamedSlices::default()
        .with_points(vec![point()])
        .with_paths(vec![path()])
        .with_labels(vec![label()])
        .with_colors(vec![Color::Red, Color::Blue])
        .with_shapes(vec![shape()]);
    assert_eq!(value.colors(), Some([Color::Red, Color::Blue].as_slice()));
    assert_json(
        &value,
        &format!(
            r#"{{"points":[{}],"paths":[{}],"labels":[{}],"colors":["Red","Blue"],"shapes":[{}]}}"#,
            POINT_JSON, PATH_JSON, LABEL_JSON, SHAPE_JSON
        ),
    );
}

#[test]
fn boxed_fields_hide_their_boxes() {
    let mut value: BoxedFields = BoxedFields::default();
    assert_eq!(value.point(), None);

    let result: Option<Point> = value.set_point(point());
    assert_eq!(result, None);
    let result: Option<Point> = value.point();
    assert_eq!(result, Some(point()));

    let value: BoxedFields = value
        .with_path(path())
        .with_label(label())
        .with_color(Color::Blue)
        .with_shape(shape());
    let result: Option<&Path> = value.path();
    assert_eq!(result, Some(&path()));
    let result: Option<&Label> = value.label();
    assert_eq!(result, Some(&label()));
    let result: Option<Color> = value.color();
    assert_eq!(result, Some(Color::Blue));
    let result: Option<&Shape> = value.shape();
    assert_eq!(result, Some(&shape()));
}

#[test]
fn boxed_fields_json() {
    assert_json(&BoxedFields::default(), "{}");
    assert_json(
        &BoxedFields::default()
            .with_point(point())
            .with_path(path())
            .with_label(label())
            .with_color(Color::Blue)
            .with_shape(shape()),
        &format!(
            r#"{{"point":{},"path":{},"label":{},"color":"Blue","shape":{}}}"#,
            POINT_JSON, PATH_JSON, LABEL_JSON, SHAPE_JSON
        ),
    );
}

#[test]
fn maps_default_to_none() {
    let value: MapFields = MapFields::default();
    assert_eq!(value.counts(), None);
    assert_eq!(value.points(), None);
    assert_eq!(value.names(), None);
}

#[test]
fn maps_getters() {
    let value: MapFields = MapFields::default()
        .with_counts(BTreeMap::from([("a".to_string(), 1)]))
        .with_points(BTreeMap::from([(Color::Green, point())]))
        .with_names(BTreeMap::from([(point(), "p".to_string())]));
    assert_eq!(
        value.counts(),
        Some(&BTreeMap::from([("a".to_string(), 1)]))
    );
    assert_eq!(
        value.points().and_then(|points| points.get(&Color::Green)),
        Some(&point())
    );
    assert_eq!(
        value
            .names()
            .and_then(|names| names.get(&point()))
            .map(String::as_str),
        Some("p")
    );
}

#[test]
fn maps_json() {
    assert_json(&MapFields::default(), "{}");
    assert_json(
        &MapFields::default().with_counts(BTreeMap::new()),
        r#"{"counts":{}}"#,
    );
    assert_json(
        &MapFields::default()
            .with_counts(BTreeMap::from([("b".to_string(), 2), ("a".to_string(), 1)]))
            .with_points(BTreeMap::from([(Color::Green, point())])),
        &format!(
            r#"{{"counts":{{"a":1,"b":2}},"points":{{"Green":{}}}}}"#,
            POINT_JSON
        ),
    );
}

#[test]
fn empty_messages_are_default() {
    assert_json(&Empty::default(), "{}");
}

#[test]
fn tags_do_not_change_the_json() {
    assert_json(
        &EdgeTags::default()
            .with_min(1_u32)
            .with_gap(2_u32)
            .with_max(3_u32),
        r#"{"min":1,"gap":2,"max":3}"#,
    );
}

#[test]
fn unordered_tags_keep_the_declaration_order() {
    assert_json(
        &UnorderedTags::default()
            .with_first(1_u32)
            .with_second(2_u32),
        r#"{"second":2,"first":1}"#,
    );
}
