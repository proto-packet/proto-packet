mod common;

use common::*;
use proto_packet::io::DecodeError;
use proto_packet_test::enums::cases::{EdgeTags, MultiCase};
use proto_packet_test::enums::empty::Empty as EmptyEnum;
use proto_packet_test::format::examples::{self, Bits, MyStruct, Odd, S, T};
use proto_packet_test::links::imports::ImportedFields;
use proto_packet_test::links::local::{After, Before};
use proto_packet_test::mods::parent::child::Child;
use proto_packet_test::names::casing::{HTTPServer, My_Struct, Vec3};
use proto_packet_test::names::keywords::KeywordStruct;
use proto_packet_test::names::methods::DefaultField;
use proto_packet_test::names::r#move::MoveStruct;
use proto_packet_test::names::prelude;
use proto_packet_test::names::r#type::self_::SelfStruct;
use proto_packet_test::structs::arguments::EightFields;
use proto_packet_test::structs::copy::{CopyFields, NestedCopy};
use proto_packet_test::structs::empty::Empty;
use proto_packet_test::structs::fields;
use proto_packet_test::types::{Color, Point};

/// Asserts the `value` of the `type` encodes to bytes that decode back to it, with & without
/// validation.
macro_rules! assert_round_trip {
    ($type:ty, $value:expr) => {{
        let value: $type = $value;
        let bytes: Vec<u8> = value.encode();
        assert_eq!(bytes.len(), value.encoded_len());
        assert_eq!(<$type>::decode(&bytes), Ok(value.clone()));
        assert_eq!(unsafe { <$type>::decode_unchecked(&bytes) }, value);
    }};
}

#[test]
fn my_struct_example() {
    let value: MyStruct = MyStruct::new("hi".to_string(), 0x0403_0201, 5);
    let expected: Vec<u8> = [
        &5_u128.to_le_bytes()[..],
        &0x0403_0201_u32.to_le_bytes(),
        &2_u32.to_le_bytes(),
        b"hi",
    ]
    .concat();
    assert_eq!(value.encode(), expected);
    assert_round_trip!(MyStruct, value);
}

#[test]
fn s_example() {
    let value: S = S::new("ab".to_string(), 7, 1 << 100, "c".to_string(), true, 9);
    let expected: Vec<u8> = [
        &(1_u128 << 100).to_le_bytes()[..],
        &9_u64.to_le_bytes(),
        &7_u32.to_le_bytes(),
        &2_u32.to_le_bytes(),
        &1_u32.to_le_bytes(),
        &[1],
        b"ab",
        b"c",
    ]
    .concat();
    assert_eq!(value.encode(), expected);
    assert_round_trip!(S, value);
}

#[test]
fn t_example() {
    let value: T = T::new(Odd::new(1, 2), examples::Point::new(-1, 3), 4, true);
    let expected: Vec<u8> = [
        &4_u64.to_le_bytes()[..],
        &(-1_i32).to_le_bytes(),
        &3_i32.to_le_bytes(),
        &1_u64.to_le_bytes(),
        &[2],
        &[1],
    ]
    .concat();
    assert_eq!(value.encode(), expected);
    assert_round_trip!(T, value);
}

#[test]
fn bits_fill_bytes_from_the_least_significant() {
    let value: Bits = Bits::new(true, false, true, false, false, false, false, true, true);
    assert_eq!(value.encode(), [0b1000_0101, 0b0000_0001]);
    assert_round_trip!(Bits, value);
    assert_eq!(
        Bits::decode(&[0b1000_0101, 0b1111_1111]),
        Ok(value),
        "the unused bits are ignored"
    );
}

#[test]
fn structs_round_trip() {
    assert_round_trip!(fields::BooleanField, fields::BooleanField::new(true));
    assert_round_trip!(
        fields::UnsignedIntFields,
        fields::UnsignedIntFields::new(u8::MAX, u16::MAX, u32::MAX, u64::MAX, u128::MAX)
    );
    assert_round_trip!(
        fields::SignedIntFields,
        fields::SignedIntFields::new(i8::MIN, i16::MIN, i32::MIN, i64::MIN, i128::MIN)
    );
    assert_round_trip!(fields::FloatFields, fields::FloatFields::new(1.5, -2.5));
    assert_round_trip!(
        fields::SpecialFields,
        fields::SpecialFields::new(id(), "é".to_string())
    );
    assert_round_trip!(fields::TimeFields, fields::TimeFields::new(-1, day()));
    assert_round_trip!(CopyFields, copy_fields());
    assert_round_trip!(NestedCopy, NestedCopy::new(copy_fields()));
    assert_round_trip!(Empty, Empty::default());
    assert_round_trip!(EightFields, EightFields::new(1, 2, 3, 4, 5, 6, 7, 8));
    assert_round_trip!(Point, point());
    assert_round_trip!(Before, Before::new(After::new(1)));
    assert_round_trip!(
        ImportedFields,
        ImportedFields::new(
            point(),
            Child::new(1),
            MoveStruct::new(2),
            SelfStruct::new(3),
        )
    );
}

#[test]
fn named_structs_round_trip() {
    assert_round_trip!(KeywordStruct, KeywordStruct::new(1, "match".to_string(), 2));
    assert_round_trip!(My_Struct, My_Struct::new(1));
    assert_round_trip!(
        HTTPServer,
        HTTPServer::new("https://example.com".to_string())
    );
    assert_round_trip!(Vec3, Vec3::new(1.0, 2.0, 3.0));
    assert_round_trip!(DefaultField, DefaultField::new(1));
    assert_round_trip!(prelude::String, prelude::String::new("text".to_string()));
    assert_round_trip!(prelude::Box, prelude::Box::new(point()));
}

#[test]
fn enums_are_their_tag_numbers() {
    assert_eq!(Color::Green.tag_number(), 2);
    assert_eq!(Color::from_tag_number(2), Some(Color::Green));
    assert_eq!(Color::from_tag_number(0), None);
    assert_eq!(Color::from_tag_number(4), None);
    for (case, tag_number) in [
        (EdgeTags::Min, 1),
        (EdgeTags::Gap, 100),
        (EdgeTags::Max, 8191),
    ] {
        assert_eq!(case.tag_number(), tag_number);
        assert_eq!(EdgeTags::from_tag_number(tag_number), Some(case));
    }
    assert_eq!(MultiCase::from_tag_number(3), Some(MultiCase::Three));
    assert_eq!(EmptyEnum::from_tag_number(1), None);
}

/// Gets the test `CopyFields`.
///
/// Its head is the `integer` at 0, `id` at 8, `time` at 24, `float` at 32, `day` at 36, `point` at
/// 40, `color` at 48 & the bit of `boolean` at 50.
fn copy_fields() -> CopyFields {
    CopyFields::new(true, 1, -0.0, id(), -1, day(), Color::Blue, point())
}

#[test]
fn encode_writes_normalized_floats() {
    let bytes: Vec<u8> = copy_fields().encode();
    assert_eq!(bytes[32..36], 0_f32.to_le_bytes(), "negative zero is zero");

    let bytes: Vec<u8> = fields::FloatFields::new(f32::NAN, -f64::NAN).encode();
    assert_eq!(bytes[8..12], f32::NAN.to_le_bytes());
    assert_eq!(bytes[0..8], f64::NAN.to_le_bytes());
}

#[test]
fn decode_normalizes_floats() {
    let mut bytes: Vec<u8> = copy_fields().encode();
    bytes[32..36].copy_from_slice(&0xFFC0_0001_u32.to_le_bytes());
    let value: CopyFields = CopyFields::decode(&bytes).unwrap();
    assert_eq!(value.float().to_bits(), f32::NAN.to_bits());

    bytes[32..36].copy_from_slice(&(-0.0_f32).to_le_bytes());
    let value: CopyFields = CopyFields::decode(&bytes).unwrap();
    assert_eq!(value.float().to_bits(), 0);
}

/// Gets the invalid length error of the `expected` & `actual` lengths.
fn invalid_length<T>(expected: usize, actual: usize) -> Result<T, DecodeError> {
    Err(DecodeError::InvalidLength { expected, actual })
}

#[test]
fn decode_rejects_invalid_lengths() {
    let bytes: Vec<u8> = fields::SpecialFields::new(id(), "ab".to_string()).encode();
    assert_eq!(
        fields::SpecialFields::decode(&bytes[..19]),
        invalid_length(20, 19)
    );
    assert_eq!(
        fields::SpecialFields::decode(&bytes[..21]),
        invalid_length(22, 21)
    );
    assert_eq!(
        fields::SpecialFields::decode(&[&bytes[..], &[0]].concat()),
        invalid_length(22, 23)
    );
    let mut huge: Vec<u8> = bytes.clone();
    huge[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        fields::SpecialFields::decode(&huge),
        invalid_length(20 + u32::MAX as usize, 22)
    );

    assert_eq!(CopyFields::decode(&[0; 50]), invalid_length(51, 50));
    assert_eq!(Empty::decode(&[0]), invalid_length(0, 1));
}

#[test]
fn decode_rejects_invalid_utf8() {
    let mut bytes: Vec<u8> = fields::SpecialFields::new(id(), "ab".to_string()).encode();
    bytes[20] = 0xFF;
    assert_eq!(
        fields::SpecialFields::decode(&bytes),
        Err(DecodeError::InvalidUtf8)
    );
}

#[test]
fn decode_rejects_unknown_enum_tag_numbers() {
    let mut bytes: Vec<u8> = copy_fields().encode();
    bytes[48..50].copy_from_slice(&4_u16.to_le_bytes());
    assert_eq!(
        CopyFields::decode(&bytes),
        Err(DecodeError::InvalidEnumTag { tag_number: 4 })
    );
}

#[test]
fn decode_rejects_dates_out_of_range() {
    let mut bytes: Vec<u8> = copy_fields().encode();
    bytes[36..40].copy_from_slice(&i32::MAX.to_le_bytes());
    assert_eq!(
        CopyFields::decode(&bytes),
        Err(DecodeError::InvalidDate { days: i32::MAX })
    );
}

#[test]
fn nested_decode_errors_are_returned() {
    let mut bytes: Vec<u8> = NestedCopy::new(copy_fields()).encode();
    bytes[48..50].copy_from_slice(&0_u16.to_le_bytes());
    assert_eq!(
        NestedCopy::decode(&bytes),
        Err(DecodeError::InvalidEnumTag { tag_number: 0 })
    );
}
