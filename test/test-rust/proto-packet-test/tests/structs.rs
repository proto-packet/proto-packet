mod common;

use common::*;
use proto_packet::types::{Float32, Float64};
use proto_packet_test::structs::arguments::EightFields;
use proto_packet_test::structs::boxed::{BoxedFields, OptionalBoxedFields};
use proto_packet_test::structs::copy::{CopyFields, NestedCopy, OptionalCopy};
use proto_packet_test::structs::empty::Empty;
use proto_packet_test::structs::named::{NamedFields, NamedSlices};
use proto_packet_test::structs::optional::OptionalFields;
use proto_packet_test::structs::{fields, slices};
use proto_packet_test::types::{Color, Label, Path, Point, Shape};

/// Asserts the type `T` is `Copy`.
fn assert_copy<T: Copy>() {}

#[test]
fn fields_getters() {
    let boolean: fields::BooleanField = fields::BooleanField::new(true);
    assert!(boolean.one());

    let unsigned: fields::UnsignedIntFields =
        fields::UnsignedIntFields::new(u8::MAX, u16::MAX, u32::MAX, u64::MAX, u128::MAX);
    assert_eq!(unsigned.one(), u8::MAX);
    assert_eq!(unsigned.two(), u16::MAX);
    assert_eq!(unsigned.three(), u32::MAX);
    assert_eq!(unsigned.four(), u64::MAX);
    assert_eq!(unsigned.five(), u128::MAX);

    let signed: fields::SignedIntFields =
        fields::SignedIntFields::new(i8::MIN, i16::MIN, i32::MIN, i64::MIN, i128::MIN);
    assert_eq!(signed.one(), i8::MIN);
    assert_eq!(signed.two(), i16::MIN);
    assert_eq!(signed.three(), i32::MIN);
    assert_eq!(signed.four(), i64::MIN);
    assert_eq!(signed.five(), i128::MIN);

    let floats: fields::FloatFields = fields::FloatFields::new(1.5, -2.5);
    assert_eq!(floats.one(), 1.5);
    assert_eq!(floats.two(), -2.5);

    let special: fields::SpecialFields = fields::SpecialFields::new(id(), "text".to_string());
    assert_eq!(special.one(), id());
    assert_eq!(special.two(), "text");

    let time: fields::TimeFields = fields::TimeFields::new(-1, day());
    assert_eq!(time.one(), -1);
    assert_eq!(time.two(), day());
}

#[test]
fn fields_setters_return_the_previous_value() {
    let mut value: fields::SpecialFields = fields::SpecialFields::new(id(), "old".to_string());
    let result: String = value.set_two("new".to_string());
    assert_eq!(result, "old");
    assert_eq!(value.two(), "new");

    let mut value: fields::FloatFields = fields::FloatFields::new(1.0, 2.0);
    let result: f32 = value.set_one(3.0);
    assert_eq!(result, 1.0);
    assert_eq!(value.one(), 3.0);
}

#[test]
fn fields_builders() {
    let result: fields::UnsignedIntFields = fields::UnsignedIntFields::new(0, 0, 0, 0, 0)
        .with_one(1)
        .with_five(5);
    let expected: fields::UnsignedIntFields = fields::UnsignedIntFields::new(1, 0, 0, 0, 5);
    assert_eq!(result, expected);
}

#[test]
fn fields_json() {
    assert_json(&fields::BooleanField::new(true), r#"{"one":true}"#);
    assert_json(
        &fields::UnsignedIntFields::new(u8::MAX, u16::MAX, u32::MAX, u64::MAX, u128::MAX),
        &format!(
            r#"{{"one":{},"two":{},"three":{},"four":{},"five":{}}}"#,
            u8::MAX,
            u16::MAX,
            u32::MAX,
            u64::MAX,
            u128::MAX
        ),
    );
    assert_json(
        &fields::SignedIntFields::new(i8::MIN, i16::MIN, i32::MIN, i64::MIN, i128::MIN),
        &format!(
            r#"{{"one":{},"two":{},"three":{},"four":{},"five":{}}}"#,
            i8::MIN,
            i16::MIN,
            i32::MIN,
            i64::MIN,
            i128::MIN
        ),
    );
    assert_json(
        &fields::FloatFields::new(1.5, -2.5),
        r#"{"one":1.5,"two":-2.5}"#,
    );
    assert_json(
        &fields::SpecialFields::new(id(), "text".to_string()),
        &format!(r#"{{"one":{},"two":"text"}}"#, ID_JSON),
    );
    assert_json(
        &fields::TimeFields::new(-1, day()),
        &format!(r#"{{"one":-1,"two":{}}}"#, DAY_JSON),
    );
}

#[test]
fn float_fields_normalize_negative_zero() {
    let result: fields::FloatFields = fields::FloatFields::new(-0.0, -0.0);
    assert!(result.one().is_sign_positive());
    assert!(result.two().is_sign_positive());
    assert_eq!(result, fields::FloatFields::new(0.0, 0.0));
}

#[test]
fn float_fields_equal_with_nan() {
    let result: fields::FloatFields = fields::FloatFields::new(f32::NAN, -f64::NAN);
    assert!(result.one().is_nan());
    assert!(result.two().is_nan());
    assert_eq!(result, fields::FloatFields::new(-f32::NAN, f64::NAN));
}

#[test]
fn float_fields_order_totally() {
    let mut result: Vec<fields::FloatFields> = vec![
        fields::FloatFields::new(f32::NAN, 0.0),
        fields::FloatFields::new(1.0, 0.0),
        fields::FloatFields::new(f32::NEG_INFINITY, 0.0),
        fields::FloatFields::new(-1.0, 0.0),
    ];
    result.sort();
    let result: Vec<f32> = result.iter().map(fields::FloatFields::one).collect();
    assert_eq!(result[..3], [f32::NEG_INFINITY, -1.0, 1.0]);
    assert!(result[3].is_nan());
}

#[test]
fn slices_getters() {
    let boolean: slices::BooleanField = slices::BooleanField::new(vec![true, false]);
    assert_eq!(boolean.one(), [true, false]);

    let unsigned: slices::UnsignedIntFields =
        slices::UnsignedIntFields::new(vec![1], vec![2], vec![3], vec![4], vec![5, 6]);
    assert_eq!(unsigned.one(), [1]);
    assert_eq!(unsigned.five(), [5, 6]);

    let floats: slices::FloatFields =
        slices::FloatFields::new(vec![Float32::new(1.5)], vec![Float64::new(-2.5)]);
    assert_eq!(floats.one(), [Float32::new(1.5)]);
    assert_eq!(floats.two(), [Float64::new(-2.5)]);

    let special: slices::SpecialFields =
        slices::SpecialFields::new(vec![id()], vec!["a".to_string(), "b".to_string()]);
    assert_eq!(special.one(), [id()]);
    assert_eq!(special.two(), ["a", "b"]);

    let time: slices::TimeFields = slices::TimeFields::new(vec![-1, 1], vec![day()]);
    assert_eq!(time.one(), [-1, 1]);
    assert_eq!(time.two(), [day()]);

    let nested: slices::NestedSlices =
        slices::NestedSlices::new(vec![vec![1, 2], vec![]], vec![vec!["a".to_string()]]);
    assert_eq!(nested.one(), [vec![1, 2], vec![]]);
    assert_eq!(nested.two(), [vec!["a"]]);
}

#[test]
fn slices_json() {
    assert_json(
        &slices::BooleanField::new(vec![true, false]),
        r#"{"one":[true,false]}"#,
    );
    assert_json(
        &slices::UnsignedIntFields::new(vec![], vec![2], vec![3], vec![4], vec![5, 6]),
        r#"{"one":[],"two":[2],"three":[3],"four":[4],"five":[5,6]}"#,
    );
    assert_json(
        &slices::SignedIntFields::new(vec![-1], vec![-2], vec![], vec![-4], vec![-5, 6]),
        r#"{"one":[-1],"two":[-2],"three":[],"four":[-4],"five":[-5,6]}"#,
    );
    assert_json(
        &slices::FloatFields::new(vec![Float32::new(1.5)], vec![Float64::new(-2.5)]),
        r#"{"one":[1.5],"two":[-2.5]}"#,
    );
    assert_json(
        &slices::SpecialFields::new(vec![id()], vec!["a".to_string()]),
        &format!(r#"{{"one":[{}],"two":["a"]}}"#, ID_JSON),
    );
    assert_json(
        &slices::TimeFields::new(vec![-1], vec![day()]),
        &format!(r#"{{"one":[-1],"two":[{}]}}"#, DAY_JSON),
    );
    assert_json(
        &slices::NestedSlices::new(vec![vec![1, 2], vec![]], vec![vec!["a".to_string()]]),
        r#"{"one":[[1,2],[]],"two":[["a"]]}"#,
    );
}

#[test]
fn named_fields_getters() {
    let value: NamedFields = NamedFields::new(point(), path(), label(), Color::Green, shape());
    let result: Point = value.point();
    assert_eq!(result, point());
    let result: &Path = value.path();
    assert_eq!(result, &path());
    let result: &Label = value.label();
    assert_eq!(result, &label());
    let result: Color = value.color();
    assert_eq!(result, Color::Green);
    let result: &Shape = value.shape();
    assert_eq!(result, &shape());
}

#[test]
fn named_fields_json() {
    assert_json(
        &NamedFields::new(point(), path(), label(), Color::Green, shape()),
        &format!(
            r#"{{"point":{},"path":{},"label":{},"color":"Green","shape":{}}}"#,
            POINT_JSON, PATH_JSON, LABEL_JSON, SHAPE_JSON
        ),
    );
}

#[test]
fn named_slices_getters() {
    let value: NamedSlices = NamedSlices::new(
        vec![point()],
        vec![path()],
        vec![label()],
        vec![Color::Red, Color::Blue],
        vec![shape()],
    );
    assert_eq!(value.points(), [point()]);
    assert_eq!(value.paths(), [path()]);
    assert_eq!(value.labels(), [label()]);
    assert_eq!(value.colors(), [Color::Red, Color::Blue]);
    assert_eq!(value.shapes(), [shape()]);
}

#[test]
fn named_slices_json() {
    assert_json(
        &NamedSlices::new(
            vec![point()],
            vec![path()],
            vec![label()],
            vec![Color::Red, Color::Blue],
            vec![shape()],
        ),
        &format!(
            r#"{{"points":[{}],"paths":[{}],"labels":[{}],"colors":["Red","Blue"],"shapes":[{}]}}"#,
            POINT_JSON, PATH_JSON, LABEL_JSON, SHAPE_JSON
        ),
    );
}

/// Gets the optional fields with every field set.
fn optional_fields() -> OptionalFields {
    OptionalFields::new(
        Some(true),
        Some(u64::MAX),
        Some(-2.5),
        Some(id()),
        Some("text".to_string()),
        Some(-1),
        Some(day()),
        Some(vec![1, 2]),
        Some(point()),
        Some(path()),
        Some(label()),
        Some(Color::Blue),
        Some(shape()),
    )
}

/// Gets the optional fields with no field set.
fn optional_fields_none() -> OptionalFields {
    OptionalFields::new(
        None, None, None, None, None, None, None, None, None, None, None, None, None,
    )
}

#[test]
fn optional_fields_getters() {
    let value: OptionalFields = optional_fields();
    assert_eq!(value.boolean(), Some(true));
    assert_eq!(value.integer(), Some(u64::MAX));
    assert_eq!(value.float(), Some(-2.5));
    assert_eq!(value.id(), Some(id()));
    assert_eq!(value.text(), Some("text"));
    assert_eq!(value.time(), Some(-1));
    assert_eq!(value.day(), Some(day()));
    assert_eq!(value.bytes(), Some([1, 2].as_slice()));
    assert_eq!(value.point(), Some(point()));
    assert_eq!(value.path(), Some(&path()));
    assert_eq!(value.label(), Some(&label()));
    assert_eq!(value.color(), Some(Color::Blue));
    assert_eq!(value.shape(), Some(&shape()));

    let value: OptionalFields = optional_fields_none();
    assert_eq!(value.boolean(), None);
    assert_eq!(value.float(), None);
    assert_eq!(value.text(), None);
    assert_eq!(value.bytes(), None);
    assert_eq!(value.point(), None);
    assert_eq!(value.path(), None);
}

#[test]
fn optional_fields_setters_take_values_or_none() {
    let mut value: OptionalFields = optional_fields_none();
    let result: Option<u64> = value.set_integer(1_u64);
    assert_eq!(result, None);
    assert_eq!(value.integer(), Some(1));

    let result: Option<u64> = value.set_integer(None);
    assert_eq!(result, Some(1));
    assert_eq!(value.integer(), None);

    let result: Option<f64> = value.set_float(1.5);
    assert_eq!(result, None);
    let result: Option<f64> = value.set_float(Some(2.5));
    assert_eq!(result, Some(1.5));
    assert_eq!(value.float(), Some(2.5));

    let result: Option<String> = value.set_text("text".to_string());
    assert_eq!(result, None);
    assert_eq!(value.text(), Some("text"));

    let value: OptionalFields = optional_fields_none()
        .with_path(path())
        .with_shape(Some(shape()));
    assert_eq!(value.path(), Some(&path()));
    assert_eq!(value.shape(), Some(&shape()));
}

#[test]
fn optional_fields_json_skips_none() {
    assert_json(&optional_fields_none(), "{}");
    assert_json(
        &optional_fields_none().with_integer(1_u64),
        r#"{"integer":1}"#,
    );
    assert_json(
        &optional_fields(),
        &format!(
            concat!(
                r#"{{"boolean":true,"integer":{},"float":-2.5,"id":{},"text":"text","time":-1,"#,
                r#""day":{},"bytes":[1,2],"point":{},"path":{},"label":{},"color":"Blue","#,
                r#""shape":{}}}"#
            ),
            u64::MAX,
            ID_JSON,
            DAY_JSON,
            POINT_JSON,
            PATH_JSON,
            LABEL_JSON,
            SHAPE_JSON
        ),
    );
}

#[test]
fn boxed_fields_hide_their_boxes() {
    let mut value: BoxedFields = BoxedFields::new(point(), path(), label(), Color::Red, shape());
    let result: Point = value.point();
    assert_eq!(result, point());
    let result: &Path = value.path();
    assert_eq!(result, &path());
    let result: &Label = value.label();
    assert_eq!(result, &label());
    let result: Color = value.color();
    assert_eq!(result, Color::Red);
    let result: &Shape = value.shape();
    assert_eq!(result, &shape());

    let result: Point = value.set_point(Point::new(0, 0));
    assert_eq!(result, point());
    assert_eq!(value.point(), Point::new(0, 0));

    let value: BoxedFields = value.with_color(Color::Green);
    assert_eq!(value.color(), Color::Green);
}

#[test]
fn boxed_fields_json() {
    assert_json(
        &BoxedFields::new(point(), path(), label(), Color::Red, shape()),
        &format!(
            r#"{{"point":{},"path":{},"label":{},"color":"Red","shape":{}}}"#,
            POINT_JSON, PATH_JSON, LABEL_JSON, SHAPE_JSON
        ),
    );
}

#[test]
fn optional_boxed_fields_hide_their_boxes() {
    let mut value: OptionalBoxedFields = OptionalBoxedFields::new(None, None, None, None, None);
    assert_eq!(value.point(), None);
    assert_eq!(value.path(), None);

    let result: Option<Point> = value.set_point(point());
    assert_eq!(result, None);
    let result: Option<Point> = value.point();
    assert_eq!(result, Some(point()));

    let value: OptionalBoxedFields = value.with_path(path()).with_color(Color::Blue);
    let result: Option<&Path> = value.path();
    assert_eq!(result, Some(&path()));
    let result: Option<Color> = value.color();
    assert_eq!(result, Some(Color::Blue));
}

#[test]
fn optional_boxed_fields_json() {
    assert_json(
        &OptionalBoxedFields::new(None, None, None, None, None),
        "{}",
    );
    assert_json(
        &OptionalBoxedFields::new(
            Some(point()),
            Some(path()),
            Some(label()),
            Some(Color::Red),
            Some(shape()),
        ),
        &format!(
            r#"{{"point":{},"path":{},"label":{},"color":"Red","shape":{}}}"#,
            POINT_JSON, PATH_JSON, LABEL_JSON, SHAPE_JSON
        ),
    );
}

#[test]
fn copy_structs_are_copy() {
    assert_copy::<Point>();
    assert_copy::<CopyFields>();
    assert_copy::<NestedCopy>();
    assert_copy::<OptionalCopy>();
}

#[test]
fn copy_struct_fields_get_by_value() {
    let inner: CopyFields = CopyFields::new(true, 1, 1.5, id(), -1, day(), Color::Red, point());
    let value: NestedCopy = NestedCopy::new(inner);
    let result: CopyFields = value.inner();
    assert_eq!(result, inner);
    assert_eq!(result.float(), 1.5);
}

#[test]
fn empty_structs_are_default() {
    assert_json(&Empty::default(), "{}");
}

#[test]
fn eight_fields_construct() {
    let result: EightFields = EightFields::new(1, 2, 3, 4, 5, 6, 7, 8);
    assert_eq!(result.one(), 1);
    assert_eq!(result.eight(), 8);
}
