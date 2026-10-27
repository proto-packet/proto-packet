mod common;

use common::*;
use proto_packet::io::DecodeError;
use proto_packet::types::{Float32, Float64};
use proto_packet_test::enums::cases::{EdgeTags, MultiCase};
use proto_packet_test::enums::empty::Empty as EmptyEnum;
use proto_packet_test::format::examples::{
    self, Bits, Lists, ListsBuf, ListsRef, MyStruct, Odd, Opt, OptRef, OptionalValues,
    OptionalValuesBuf, OptionalValuesRef, S, SBuf, SRef, T, TRef, ValidatedLists,
    ValidatedListsBuf, ValidatedListsRef,
};
use proto_packet_test::links::imports::ImportedFields;
use proto_packet_test::links::local::{After, Before};
use proto_packet_test::links::loops::{ListNode, ListNodeRef, TreeNode, TreeNodeRef};
use proto_packet_test::mods::parent::child::Child;
use proto_packet_test::names::casing::{HTTPServer, My_Struct, Vec3};
use proto_packet_test::names::keywords::{KeywordStruct, Self_, SelfBuf, SelfRef};
use proto_packet_test::names::methods::DefaultField;
use proto_packet_test::names::r#move::MoveStruct;
use proto_packet_test::names::prelude;
use proto_packet_test::names::r#type::self_::SelfStruct;
use proto_packet_test::structs::arguments::EightFields;
use proto_packet_test::structs::copy::{
    CopyFields, CopyFieldsBuf, CopyFieldsRef, NestedCopy, NestedCopyBuf, NestedCopyRef,
    OptionalCopy,
};
use proto_packet_test::structs::empty::{Empty, EmptyBuf, EmptyRef};
use proto_packet_test::structs::{fields, slices};
use proto_packet_test::types::{Color, Point, PointRef};

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

/// Asserts the `bytes` are rejected with the `error` by the `type`'s `validate` & `decode`, & by
/// its `Ref` & `Buf` types' `new`.
macro_rules! assert_rejected {
    ($type:ty, $ref:ty, $buf:ty, $bytes:expr, $error:expr) => {{
        let bytes: &[u8] = $bytes;
        let error: DecodeError = $error;
        assert_eq!(<$type>::validate(bytes), Err(error));
        assert_eq!(<$type>::decode(bytes).map(|_| ()), Err(error));
        assert_eq!(<$ref>::new(bytes).map(|_| ()), Err(error));
        assert_eq!(<$buf>::new(bytes.to_vec()).map(|_| ()), Err(error));
    }};
}

/// Asserts the `bytes` of `SpecialFields` are rejected with the `error`.
macro_rules! assert_special_fields_rejected {
    ($bytes:expr, $error:expr) => {
        assert_rejected!(
            fields::SpecialFields,
            fields::SpecialFieldsRef,
            fields::SpecialFieldsBuf,
            $bytes,
            $error
        )
    };
}

/// Gets the invalid length error of the `expected` & `actual` lengths.
fn invalid_length(expected: usize, actual: usize) -> DecodeError {
    DecodeError::InvalidLength { expected, actual }
}

#[test]
fn readers_reject_invalid_lengths() {
    let bytes: Vec<u8> = fields::SpecialFields::new(id(), "ab".to_string()).encode();
    assert_special_fields_rejected!(&bytes[..19], invalid_length(20, 19));
    assert_special_fields_rejected!(&bytes[..21], invalid_length(22, 21));
    assert_special_fields_rejected!(&[&bytes[..], &[0]].concat(), invalid_length(22, 23));
    let mut huge: Vec<u8> = bytes.clone();
    huge[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_special_fields_rejected!(&huge, invalid_length(20 + u32::MAX as usize, 22));

    assert_rejected!(
        CopyFields,
        CopyFieldsRef,
        CopyFieldsBuf,
        &[0; 50],
        invalid_length(51, 50)
    );
    assert_rejected!(Empty, EmptyRef, EmptyBuf, &[0], invalid_length(0, 1));
}

#[test]
fn readers_reject_invalid_utf8() {
    let mut bytes: Vec<u8> = fields::SpecialFields::new(id(), "ab".to_string()).encode();
    bytes[20] = 0xFF;
    assert_special_fields_rejected!(&bytes, DecodeError::InvalidUtf8);
}

#[test]
fn readers_reject_unknown_enum_tag_numbers() {
    let mut bytes: Vec<u8> = copy_fields().encode();
    bytes[48..50].copy_from_slice(&4_u16.to_le_bytes());
    let error: DecodeError = DecodeError::InvalidEnumTag { tag_number: 4 };
    assert_rejected!(CopyFields, CopyFieldsRef, CopyFieldsBuf, &bytes, error);
}

#[test]
fn readers_reject_dates_out_of_range() {
    let mut bytes: Vec<u8> = copy_fields().encode();
    bytes[36..40].copy_from_slice(&i32::MAX.to_le_bytes());
    let error: DecodeError = DecodeError::InvalidDate { days: i32::MAX };
    assert_rejected!(CopyFields, CopyFieldsRef, CopyFieldsBuf, &bytes, error);
}

#[test]
fn readers_reject_nested_errors() {
    let mut bytes: Vec<u8> = NestedCopy::new(copy_fields()).encode();
    bytes[48..50].copy_from_slice(&0_u16.to_le_bytes());
    let error: DecodeError = DecodeError::InvalidEnumTag { tag_number: 0 };
    assert_rejected!(NestedCopy, NestedCopyRef, NestedCopyBuf, &bytes, error);
}

#[test]
fn refs_read_the_fields() {
    let value: S = S::new("ab".to_string(), 7, 1 << 100, "c".to_string(), true, 9);
    let bytes: Vec<u8> = value.encode();
    let result: SRef<'_> = SRef::new(&bytes).unwrap();
    assert_eq!(result.a(), "ab");
    assert_eq!(result.b(), 7);
    assert_eq!(result.c(), 1 << 100);
    assert_eq!(result.d(), "c");
    assert!(result.e());
    assert_eq!(result.f(), 9);

    let value: CopyFields = copy_fields();
    let bytes: Vec<u8> = value.encode();
    let result: CopyFieldsRef<'_> = CopyFieldsRef::new(&bytes).unwrap();
    assert_eq!(result.boolean(), value.boolean());
    assert_eq!(result.integer(), value.integer());
    assert_eq!(result.float().to_bits(), value.float().to_bits());
    assert_eq!(result.id(), value.id());
    assert_eq!(result.time(), value.time());
    assert_eq!(result.day(), value.day());
    assert_eq!(result.color(), value.color());
    let point: PointRef<'_> = result.point();
    assert_eq!(
        (point.x(), point.y()),
        (value.point().x(), value.point().y())
    );
}

#[test]
fn refs_read_nested_structs() {
    let bytes: Vec<u8> = T::new(Odd::new(1, 2), examples::Point::new(-1, 3), 4, true).encode();
    let result: TRef<'_> = TRef::new(&bytes).unwrap();
    assert_eq!((result.o().a(), result.o().b()), (1, 2));
    assert_eq!((result.p().x(), result.p().y()), (-1, 3));
    assert_eq!(result.n(), 4);
    assert!(result.g());

    let value: prelude::Box = prelude::Box::new(point());
    let bytes: Vec<u8> = value.encode();
    let result: prelude::BoxRef<'_> = prelude::BoxRef::new(&bytes).unwrap();
    assert_eq!(result.point().decode(), point());
}

#[test]
fn ref_fields_outlive_their_refs() {
    let bytes: Vec<u8> = S::new("ab".to_string(), 7, 1, "c".to_string(), true, 9).encode();
    let a: &str = SRef::new(&bytes).unwrap().a();
    assert_eq!(a, "ab");
}

#[test]
fn ref_floats_are_normalized() {
    let mut bytes: Vec<u8> = copy_fields().encode();
    bytes[32..36].copy_from_slice(&0xFFC0_0001_u32.to_le_bytes());
    let result: CopyFieldsRef<'_> = CopyFieldsRef::new(&bytes).unwrap();
    assert_eq!(result.float().to_bits(), f32::NAN.to_bits());
}

#[test]
fn refs_convert() {
    let value: S = S::new("ab".to_string(), 7, 1, "c".to_string(), true, 9);
    let bytes: Vec<u8> = value.encode();
    let result: SRef<'_> = SRef::new(&bytes).unwrap();
    assert_eq!(result.as_bytes(), bytes);
    assert_eq!(result.decode(), value);
    assert_eq!(result.to_buf().as_bytes(), bytes);

    let result: SRef<'_> = unsafe { SRef::new_unchecked(&bytes) };
    assert_eq!(result.decode(), value);
}

#[test]
fn bufs_convert() {
    let value: S = S::new("ab".to_string(), 7, 1, "c".to_string(), true, 9);
    let bytes: Vec<u8> = value.encode();
    let result: SBuf = SBuf::from(&value);
    assert_eq!(result.as_bytes(), bytes);
    assert_eq!(result.as_ref().a(), "ab");
    assert_eq!(result.decode(), value);
    assert_eq!(result.into_bytes(), bytes);

    let result: SBuf = SBuf::new(bytes.clone()).unwrap();
    assert_eq!(result.decode(), value);
    let result: SBuf = unsafe { SBuf::new_unchecked(bytes) };
    assert_eq!(result.as_ref().d(), "c");
}

#[test]
fn encoding_types_of_renamed_structs() {
    let bytes: Vec<u8> = Self_::new(1).encode();
    assert_eq!(SelfRef::new(&bytes).unwrap().value(), 1);
    assert_eq!(SelfBuf::new(bytes).unwrap().decode(), Self_::new(1));
}

#[test]
fn opt_example() {
    let value: Opt = Opt::new(
        Some(7),
        true,
        Some("hi".to_string()),
        Some(true),
        Some(Color::Red),
    );
    let expected: Vec<u8> = [
        &7_u32.to_le_bytes()[..],
        &2_u32.to_le_bytes(),
        &1_u16.to_le_bytes(),
        &[0b0000_1111],
        b"hi",
    ]
    .concat();
    assert_eq!(value.encode(), expected);
    assert_round_trip!(Opt, value);
    let result: OptRef<'_> = OptRef::new(&expected).unwrap();
    assert_eq!(result.a(), Some(7));
    assert!(result.b());
    assert_eq!(result.c(), Some("hi"));
    assert_eq!(result.d(), Some(true));
    assert_eq!(result.e(), Some(Color::Red));
}

#[test]
fn opt_example_absent() {
    let value: Opt = Opt::new(None, false, None, None, None);
    let expected: Vec<u8> = [&[0; 4][..], &u32::MAX.to_le_bytes(), &[0; 2], &[0]].concat();
    assert_eq!(value.encode(), expected);
    assert_round_trip!(Opt, value);
    let result: OptRef<'_> = OptRef::new(&expected).unwrap();
    assert_eq!(result.a(), None);
    assert!(!result.b());
    assert_eq!(result.c(), None);
    assert_eq!(result.d(), None);
    assert_eq!(result.e(), None);
}

#[test]
fn empty_values_are_present() {
    let value: Opt = Opt::new(None, false, Some(String::new()), Some(false), None);
    let bytes: Vec<u8> = value.encode();
    assert_round_trip!(Opt, value);
    let result: OptRef<'_> = OptRef::new(&bytes).unwrap();
    assert_eq!(result.c(), Some(""));
    assert_eq!(result.d(), Some(false));
}

/// Gets the test `OptionalValues`, with its values present.
///
/// Its head is the `day` at 0, `point` at 4, `color` at 12 & the presence bits of `day` & `point`
/// at 14.
fn optional_values() -> OptionalValues {
    OptionalValues::new(Some(day()), Some(examples::Point::new(1, 2)), Color::Red)
}

#[test]
fn absent_values_are_not_validated() {
    let value: OptionalValues = OptionalValues::new(None, None, Color::Red);
    let mut bytes: Vec<u8> = value.encode();
    bytes[0..12].fill(0xFF);
    assert_round_trip!(OptionalValues, value);
    assert_eq!(OptionalValues::decode(&bytes), Ok(value));
    let result: OptionalValuesRef<'_> = OptionalValuesRef::new(&bytes).unwrap();
    assert_eq!(result.day(), None);
    assert!(result.point().is_none());

    let mut bytes: Vec<u8> = optional_values().encode();
    assert_round_trip!(OptionalValues, optional_values());
    bytes[0..4].copy_from_slice(&i32::MAX.to_le_bytes());
    let error: DecodeError = DecodeError::InvalidDate { days: i32::MAX };
    assert_rejected!(
        OptionalValues,
        OptionalValuesRef,
        OptionalValuesBuf,
        &bytes,
        error
    );
}

#[test]
fn optional_enums_are_absent_as_0() {
    let bytes: Vec<u8> = Opt::new(None, false, None, None, None).encode();
    assert_eq!(OptRef::new(&bytes).unwrap().e(), None);

    let mut bytes: Vec<u8> = optional_values().encode();
    bytes[12..14].copy_from_slice(&0_u16.to_le_bytes());
    let error: DecodeError = DecodeError::InvalidEnumTag { tag_number: 0 };
    assert_rejected!(
        OptionalValues,
        OptionalValuesRef,
        OptionalValuesBuf,
        &bytes,
        error
    );
}

#[test]
fn optional_structs_round_trip() {
    assert_round_trip!(OptionalCopy, OptionalCopy::new(Some(1)));
    assert_round_trip!(OptionalCopy, OptionalCopy::new(None));
    assert_round_trip!(prelude::Option, prelude::Option::new(Some(1)));
    assert_round_trip!(prelude::Some, prelude::Some::new(Some("text".to_string())));
    assert_round_trip!(prelude::None, prelude::None::new(None));
}

#[test]
fn structs_containing_themselves_round_trip() {
    let value: ListNode = ListNode::new(1, Some(ListNode::new(2, Some(ListNode::new(3, None)))));
    let bytes: Vec<u8> = value.encode();
    assert_eq!(bytes.len(), 24);
    assert_round_trip!(ListNode, value);
    let result: ListNodeRef<'_> = ListNodeRef::new(&bytes).unwrap();
    let values: Vec<u32> = std::iter::successors(Some(result), ListNodeRef::next)
        .map(|node| node.value())
        .collect();
    assert_eq!(values, [1, 2, 3]);
}

#[test]
fn list_examples() {
    let value: Lists = Lists::new(
        vec![1, 2, 3],
        vec![true, false, true],
        vec!["ab".to_string(), String::new(), "c".to_string()],
    );
    let expected: Vec<u8> = [
        &12_u32.to_le_bytes()[..],
        &5_u32.to_le_bytes(),
        &19_u32.to_le_bytes(),
        &1_u32.to_le_bytes(),
        &2_u32.to_le_bytes(),
        &3_u32.to_le_bytes(),
        &3_u32.to_le_bytes(),
        &[0b0000_0101],
        &3_u32.to_le_bytes(),
        &2_u32.to_le_bytes(),
        &2_u32.to_le_bytes(),
        &3_u32.to_le_bytes(),
        b"abc",
    ]
    .concat();
    assert_eq!(value.encode(), expected);
    assert_round_trip!(Lists, value);
    let result: ListsRef<'_> = ListsRef::new(&expected).unwrap();
    assert_eq!(result.numbers().to_vec(), [1, 2, 3]);
    assert_eq!(result.bits().to_vec(), [true, false, true]);
    assert_eq!(result.strings().to_vec(), ["ab", "", "c"]);
}

#[test]
fn lists_round_trip() {
    assert_round_trip!(
        slices::BooleanField,
        slices::BooleanField::new(vec![true; 9])
    );
    assert_round_trip!(
        slices::UnsignedIntFields,
        slices::UnsignedIntFields::new(
            vec![u8::MAX],
            vec![u16::MAX],
            vec![u32::MAX],
            vec![u64::MAX],
            vec![u128::MAX]
        )
    );
    assert_round_trip!(
        slices::SignedIntFields,
        slices::SignedIntFields::new(
            vec![i8::MIN],
            vec![i16::MIN],
            vec![i32::MIN],
            vec![i64::MIN],
            vec![i128::MIN]
        )
    );
    assert_round_trip!(
        slices::FloatFields,
        slices::FloatFields::new(vec![Float32::new(1.5)], vec![Float64::new(-2.5)])
    );
    assert_round_trip!(
        slices::SpecialFields,
        slices::SpecialFields::new(vec![id()], vec!["é".to_string(), String::new()])
    );
    assert_round_trip!(
        slices::TimeFields,
        slices::TimeFields::new(vec![-1, 1], vec![day()])
    );
    assert_round_trip!(
        slices::NestedSlices,
        slices::NestedSlices::new(
            vec![vec![1, 2], vec![]],
            vec![vec!["a".to_string()], vec![]]
        )
    );
    assert_round_trip!(
        slices::NestedSlices,
        slices::NestedSlices::new(vec![], vec![])
    );
}

#[test]
fn list_views() {
    let value: slices::UnsignedIntFields =
        slices::UnsignedIntFields::new(vec![1, 2], vec![3], vec![4, 5, 6], vec![], vec![7]);
    let bytes: Vec<u8> = value.encode();
    let result: slices::UnsignedIntFieldsRef<'_> =
        slices::UnsignedIntFieldsRef::new(&bytes).unwrap();
    let one: &[u8] = result.one();
    assert_eq!(one, [1, 2]);
    assert_eq!(result.three().len(), 3);
    assert_eq!(result.three().get(2), Some(6));
    assert_eq!(result.three().get(3), None);
    assert!(result.four().is_empty());
    assert_eq!(result.five().iter().rev().collect::<Vec<u128>>(), [7]);

    let value: slices::NestedSlices = slices::NestedSlices::new(
        vec![vec![1, 2], vec![]],
        vec![vec!["a".to_string()], vec![]],
    );
    let bytes: Vec<u8> = value.encode();
    let result: slices::NestedSlicesRef<'_> = slices::NestedSlicesRef::new(&bytes).unwrap();
    assert_eq!(result.one().to_vec(), [&[1, 2][..], &[]]);
    let two: Vec<Vec<&str>> = result.two().iter().map(|inner| inner.to_vec()).collect();
    assert_eq!(two, [vec!["a"], vec![]]);

    let mut bytes: Vec<u8> = slices::FloatFields::new(vec![Float32::new(1.5)], vec![]).encode();
    bytes[8..12].copy_from_slice(&0xFFC0_0001_u32.to_le_bytes());
    let result: slices::FloatFieldsRef<'_> = slices::FloatFieldsRef::new(&bytes).unwrap();
    assert_eq!(result.one().to_vec()[0].to_bits(), f32::NAN.to_bits());
}

#[test]
fn structs_containing_themselves_through_lists_round_trip() {
    let value: TreeNode = TreeNode::new(
        1,
        vec![
            TreeNode::new(2, vec![]),
            TreeNode::new(3, vec![TreeNode::new(4, vec![])]),
        ],
    );
    let bytes: Vec<u8> = value.encode();
    assert_round_trip!(TreeNode, value);
    let result: TreeNodeRef<'_> = TreeNodeRef::new(&bytes).unwrap();
    let leaf: Option<u32> = result
        .children()
        .get(1)
        .and_then(|node| node.children().get(0))
        .map(|node| node.value());
    assert_eq!(leaf, Some(4));
}

/// Gets the test `ValidatedLists`.
///
/// Its head is the lengths of `colors` at 0, `days` at 4 & `names` at 8. Its data is `colors` at 12,
/// `days` at 14 & `names` at 18. (its count at 18, its end at 22 & its `ab` at 26)
fn validated_lists() -> ValidatedLists {
    ValidatedLists::new(vec![Color::Red], vec![day()], Some(vec!["ab".to_string()]))
}

/// Asserts the `bytes` of `ValidatedLists` are rejected with the `error`.
macro_rules! assert_validated_lists_rejected {
    ($bytes:expr, $error:expr) => {
        assert_rejected!(
            ValidatedLists,
            ValidatedListsRef,
            ValidatedListsBuf,
            $bytes,
            $error
        )
    };
}

#[test]
fn readers_reject_invalid_list_elements() {
    let bytes: Vec<u8> = validated_lists().encode();
    assert_round_trip!(ValidatedLists, validated_lists());

    let mut invalid: Vec<u8> = bytes.clone();
    invalid[12..14].copy_from_slice(&9_u16.to_le_bytes());
    assert_validated_lists_rejected!(&invalid, DecodeError::InvalidEnumTag { tag_number: 9 });

    let mut invalid: Vec<u8> = bytes.clone();
    invalid[14..18].copy_from_slice(&i32::MAX.to_le_bytes());
    assert_validated_lists_rejected!(&invalid, DecodeError::InvalidDate { days: i32::MAX });

    let mut invalid: Vec<u8> = bytes.clone();
    invalid[26] = 0xFF;
    assert_validated_lists_rejected!(&invalid, DecodeError::InvalidUtf8);
}

#[test]
fn readers_reject_invalid_lists() {
    let mut invalid: Vec<u8> = validated_lists().encode();
    invalid[22..26].copy_from_slice(&1_u32.to_le_bytes());
    assert_validated_lists_rejected!(&invalid, DecodeError::InvalidList);

    let bytes: Vec<u8> = Lists::new(vec![1], vec![true], vec![]).encode();
    let mut invalid: Vec<u8> = bytes.clone();
    invalid[16..20].copy_from_slice(&9_u32.to_le_bytes());
    assert_rejected!(
        Lists,
        ListsRef,
        ListsBuf,
        &invalid,
        DecodeError::InvalidList
    );

    let mut invalid: Vec<u8> = bytes.clone();
    invalid[0..4].copy_from_slice(&3_u32.to_le_bytes());
    invalid.remove(12);
    assert_rejected!(
        Lists,
        ListsRef,
        ListsBuf,
        &invalid,
        DecodeError::InvalidList
    );
}

#[test]
fn absent_lists() {
    let value: ValidatedLists = ValidatedLists::new(vec![], vec![], None);
    let bytes: Vec<u8> = value.encode();
    assert_eq!(bytes[8..12], u32::MAX.to_le_bytes());
    assert_round_trip!(ValidatedLists, value);
    assert!(ValidatedListsRef::new(&bytes).unwrap().names().is_none());
}
