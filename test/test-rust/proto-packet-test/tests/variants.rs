mod common;

use common::*;
use proto_packet::types::{Float32, Float64};
use proto_packet_test::types::Color;
use proto_packet_test::variants::boxed::BoxedCases;
use proto_packet_test::variants::named::{NamedCases, NamedSlices};
use proto_packet_test::variants::{cases, slices};

#[test]
fn cases_json() {
    assert_json(
        &cases::BooleanCase::One(true),
        r#"{"var":"One","val":true}"#,
    );
    assert_json(
        &cases::UnsignedIntCases::One(u8::MAX),
        &format!(r#"{{"var":"One","val":{}}}"#, u8::MAX),
    );
    assert_json(
        &cases::UnsignedIntCases::Two(u16::MAX),
        &format!(r#"{{"var":"Two","val":{}}}"#, u16::MAX),
    );
    assert_json(
        &cases::UnsignedIntCases::Three(u32::MAX),
        &format!(r#"{{"var":"Three","val":{}}}"#, u32::MAX),
    );
    assert_json(
        &cases::UnsignedIntCases::Four(u64::MAX),
        &format!(r#"{{"var":"Four","val":{}}}"#, u64::MAX),
    );
    assert_json(
        &cases::UnsignedIntCases::Five(u128::MAX),
        &format!(r#"{{"var":"Five","val":{}}}"#, u128::MAX),
    );
    assert_json(
        &cases::SignedIntCases::One(i8::MIN),
        &format!(r#"{{"var":"One","val":{}}}"#, i8::MIN),
    );
    assert_json(
        &cases::SignedIntCases::Five(i128::MIN),
        &format!(r#"{{"var":"Five","val":{}}}"#, i128::MIN),
    );
    assert_json(
        &cases::FloatCases::One(Float32::new(1.5)),
        r#"{"var":"One","val":1.5}"#,
    );
    assert_json(
        &cases::FloatCases::Two(Float64::new(-2.5)),
        r#"{"var":"Two","val":-2.5}"#,
    );
    assert_json(
        &cases::SpecialCases::One(id()),
        &format!(r#"{{"var":"One","val":{}}}"#, ID_JSON),
    );
    assert_json(
        &cases::SpecialCases::Two("text".to_string()),
        r#"{"var":"Two","val":"text"}"#,
    );
    assert_json(&cases::TimeCases::One(-1), r#"{"var":"One","val":-1}"#);
    assert_json(
        &cases::TimeCases::Two(day()),
        &format!(r#"{{"var":"Two","val":{}}}"#, DAY_JSON),
    );
}

#[test]
fn slices_json() {
    assert_json(
        &slices::BooleanCase::One(vec![true, false]),
        r#"{"var":"One","val":[true,false]}"#,
    );
    assert_json(
        &slices::UnsignedIntCases::Five(vec![]),
        r#"{"var":"Five","val":[]}"#,
    );
    assert_json(
        &slices::SignedIntCases::Three(vec![-3, 4]),
        r#"{"var":"Three","val":[-3,4]}"#,
    );
    assert_json(
        &slices::FloatCases::Two(vec![Float64::new(-2.5)]),
        r#"{"var":"Two","val":[-2.5]}"#,
    );
    assert_json(
        &slices::SpecialCases::Two(vec!["a".to_string()]),
        r#"{"var":"Two","val":["a"]}"#,
    );
    assert_json(
        &slices::TimeCases::Two(vec![day()]),
        &format!(r#"{{"var":"Two","val":[{}]}}"#, DAY_JSON),
    );
    assert_json(
        &slices::NestedSlices::One(vec![vec![1, 2], vec![]]),
        r#"{"var":"One","val":[[1,2],[]]}"#,
    );
}

#[test]
fn named_cases_json() {
    assert_json(
        &NamedCases::Point(point()),
        &format!(r#"{{"var":"Point","val":{}}}"#, POINT_JSON),
    );
    assert_json(
        &NamedCases::Path(path()),
        &format!(r#"{{"var":"Path","val":{}}}"#, PATH_JSON),
    );
    assert_json(
        &NamedCases::Label(label()),
        &format!(r#"{{"var":"Label","val":{}}}"#, LABEL_JSON),
    );
    assert_json(
        &NamedCases::Color(Color::Green),
        r#"{"var":"Color","val":"Green"}"#,
    );
    assert_json(
        &NamedCases::Shape(shape()),
        &format!(r#"{{"var":"Shape","val":{}}}"#, SHAPE_JSON),
    );
}

#[test]
fn named_slices_json() {
    assert_json(
        &NamedSlices::Points(vec![point()]),
        &format!(r#"{{"var":"Points","val":[{}]}}"#, POINT_JSON),
    );
    assert_json(
        &NamedSlices::Colors(vec![Color::Red, Color::Blue]),
        r#"{"var":"Colors","val":["Red","Blue"]}"#,
    );
    assert_json(
        &NamedSlices::Shapes(vec![shape()]),
        &format!(r#"{{"var":"Shapes","val":[{}]}}"#, SHAPE_JSON),
    );
}

#[test]
fn boxed_cases_json() {
    assert_json(
        &BoxedCases::Point(Box::new(point())),
        &format!(r#"{{"var":"Point","val":{}}}"#, POINT_JSON),
    );
    assert_json(
        &BoxedCases::Path(Box::new(path())),
        &format!(r#"{{"var":"Path","val":{}}}"#, PATH_JSON),
    );
    assert_json(
        &BoxedCases::Label(Box::new(label())),
        &format!(r#"{{"var":"Label","val":{}}}"#, LABEL_JSON),
    );
    assert_json(
        &BoxedCases::Color(Box::new(Color::Red)),
        r#"{"var":"Color","val":"Red"}"#,
    );
    assert_json(
        &BoxedCases::Shape(Box::new(shape())),
        &format!(r#"{{"var":"Shape","val":{}}}"#, SHAPE_JSON),
    );
}
