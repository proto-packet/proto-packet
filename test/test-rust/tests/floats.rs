mod common;

use common::assert_json;
use proto_packet::types::{Float32, Float64};
use proto_packet_test::messages::fields::FloatFields as FloatMessage;
use proto_packet_test::structs::fields::FloatFields;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;

/// Asserts the type `T` has a total equality, a total order & a hash.
fn assert_identity<T: Copy + Eq + Ord + Hash + Default>() {}

/// Asserts the type `T` has a total equality, a total order & a hash.
fn assert_message_identity<T: Clone + Eq + Ord + Hash + Default>() {}

#[test]
fn every_nan_is_the_one_nan() {
    for nan in [f32::NAN, -f32::NAN, f32::from_bits(0x7FA0_0001)] {
        assert_eq!(Float32::new(nan), Float32::NAN);
        assert_eq!(Float32::new(nan).to_bits(), 0x7FC0_0000);
    }
    for nan in [f64::NAN, -f64::NAN, f64::from_bits(0xFFF0_0000_0000_0001)] {
        assert_eq!(Float64::new(nan), Float64::NAN);
        assert_eq!(Float64::new(nan).to_bits(), 0x7FF8_0000_0000_0000);
    }
    assert!(Float32::NAN.get().is_nan());
    assert!(Float64::from(f64::NAN).get().is_nan());
}

#[test]
fn negative_zero_is_its_own_value() {
    assert_ne!(Float32::new(-0.0), Float32::new(0.0));
    assert_ne!(Float64::new(-0.0), Float64::new(0.0));
    assert!(Float64::new(-0.0) < Float64::new(0.0));
    assert_eq!(Float64::default(), Float64::new(0.0));
}

#[test]
fn floats_have_a_total_order() {
    let sorted: [f64; 7] = [
        f64::NEG_INFINITY,
        -1.0,
        -0.0,
        0.0,
        1.0,
        f64::INFINITY,
        f64::NAN,
    ];
    let mut floats: Vec<Float64> = sorted.iter().rev().map(|value| (*value).into()).collect();
    floats.sort();
    let bits = |floats: &[Float64]| -> Vec<u64> { floats.iter().map(|f| f.to_bits()).collect() };
    let expected: Vec<Float64> = sorted.iter().map(|value| Float64::new(*value)).collect();
    assert_eq!(bits(&floats), bits(&expected));
}

#[test]
fn floats_are_keys() {
    let hashed: HashSet<Float32> = [f32::NAN, -f32::NAN, 0.0, -0.0, 1.5, 1.5]
        .into_iter()
        .map(Float32::new)
        .collect();
    assert_eq!(hashed.len(), 4);

    let ordered: BTreeSet<Float64> = [f64::NAN, f64::NAN, 2.0]
        .into_iter()
        .map(Float64::new)
        .collect();
    assert_eq!(ordered.len(), 2);
}

#[test]
fn bits_are_checked() {
    assert_eq!(Float32::from_bits(0x3FC0_0000), Some(Float32::new(1.5)));
    assert_eq!(Float32::from_bits(0x7FC0_0000), Some(Float32::NAN));
    assert_eq!(Float32::from_bits(0xFFC0_0000), None);
    assert_eq!(Float32::from_bits(0x7FC0_0001), None);
    assert!(Float32::is_valid_bits(0xFF80_0000)); // negative infinity
    assert_eq!(Float64::from_bits(0xFFF8_0000_0000_0000), None);
    assert_eq!(f32::from(Float32::new(1.5)), 1.5);
    assert_eq!(Float64::new(-2.0).to_le_bytes(), (-2.0_f64).to_le_bytes());
}

#[test]
fn generated_types_have_an_identity() {
    assert_identity::<Float32>();
    assert_identity::<Float64>();
    assert_message_identity::<FloatMessage>();

    let nan: FloatFields = FloatFields::new(Float32::NAN, Float64::new(-f64::NAN));
    assert_eq!(nan, nan);
    let keys: HashSet<FloatFields> = [nan, nan].into_iter().collect();
    assert_eq!(keys.len(), 1);
    assert!(FloatFields::new(Float32::new(1.0), Float64::new(-0.0)) < nan);
}

#[test]
fn floats_are_text_in_json() {
    assert_json(
        &FloatFields::new(Float32::new(1.5), Float64::new(-2.0)),
        r#"{"float32":"1.5","float64":"-2.0"}"#,
    );
    assert_json(
        &FloatFields::new(Float32::new(0.1), Float64::new(-0.0)),
        r#"{"float32":"0.1","float64":"-0.0"}"#,
    );
    assert_json(
        &FloatFields::new(Float32::new(1e-7), Float64::new(1e300)),
        r#"{"float32":"1e-7","float64":"1e300"}"#,
    );
    assert_json(
        &FloatMessage::default().with_float64(Float64::new(0.25)),
        r#"{"float64":"0.25"}"#,
    );
}

#[test]
fn floats_that_are_not_finite_are_names_in_json() {
    assert_json(
        &FloatFields::new(Float32::NAN, Float64::new(f64::INFINITY)),
        r#"{"float32":"NaN","float64":"Infinity"}"#,
    );
    assert_json(
        &FloatFields::new(Float32::new(f32::NEG_INFINITY), Float64::NAN),
        r#"{"float32":"-Infinity","float64":"NaN"}"#,
    );
}

#[test]
fn json_reads_back_the_exact_float() {
    // Read as a plain number, each of these comes back as another float from some JSON parser.
    let single: Float32 = Float32::from_bits(0x15AE_43FD).unwrap();
    let double: Float64 = Float64::from_bits(0x0000_4EED_BBB9_694A).unwrap();
    let value: FloatFields = FloatFields::new(single, double);
    let json: String = serde_json::to_string(&value).unwrap();
    assert_eq!(
        json,
        r#"{"float32":"7.038531e-26","float64":"4.2876481011514e-310"}"#
    );
    let read: FloatFields = serde_json::from_str(&json).unwrap();
    assert_eq!(read.float32().to_bits(), 0x15AE_43FD);
    assert_eq!(read.float64().to_bits(), 0x0000_4EED_BBB9_694A);
}

#[test]
fn json_reads_a_plain_number() {
    for (json, float32, float64) in [
        (r#"{"float32":1.5,"float64":-2.25}"#, 1.5, -2.25),
        (r#"{"float32":2,"float64":-3}"#, 2.0, -3.0),
        (r#"{"float32":1e-7,"float64":1e300}"#, 1e-7, 1e300),
    ] {
        let read: FloatFields = serde_json::from_str(json).unwrap();
        assert_eq!(read.float32(), Float32::new(float32), "{}", json);
        assert_eq!(read.float64(), Float64::new(float64), "{}", json);
    }
}

#[test]
fn json_rejects_invalid_floats() {
    for json in [
        r#"{"float32":"1e100","float64":"0"}"#, // out of the range of an `f32`
        r#"{"float32":1e100,"float64":"0"}"#,   // a plain number out of the range of an `f32`
        r#"{"float32":"0","float64":"1e999"}"#, // out of the range of an `f64`
        r#"{"float32":"nan","float64":"0"}"#,   // not one of the names
        r#"{"float32":"inf","float64":"0"}"#,   // another name the standard parser takes
        r#"{"float32":"","float64":"0"}"#,      // empty text
        r#"{"float32":" 1.5","float64":"0"}"#,  // text around the number
        r#"{"float32":"0x10","float64":"0"}"#,  // not a decimal number
        r#"{"float32":true,"float64":"0"}"#,    // not text
    ] {
        assert!(
            serde_json::from_str::<FloatFields>(json).is_err(),
            "{}",
            json
        );
    }
}

#[test]
fn float_text_is_a_json_number() {
    let read = |text: &str| -> Option<f64> {
        serde_json::from_str::<Float64>(&format!("\"{}\"", text))
            .map(Float64::get)
            .ok()
    };
    assert_eq!(read("1.5"), Some(1.5));
    assert_eq!(read("-0.5"), Some(-0.5));
    assert_eq!(read("1e5"), Some(100_000.0));
    assert_eq!(read("1.5E-3"), Some(0.0015));
    assert_eq!(read("0"), Some(0.0));
    for text in [
        "+1.5", ".5", "5.", "01.5", "1.5e", "1e+", " 1.5", "1.5 ", "--1", "1.5.2", "", "-",
    ] {
        assert_eq!(read(text), None, "{}", text);
    }
}
