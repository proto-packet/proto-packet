mod common;

use common::ID;
use proto_packet::io::DecodeError;
use proto_packet::types::Float64;
use proto_packet_test::format::examples::{Color, Entry, Point, PointView, PointViewMut};
use proto_packet_test::format::messages::Item;
use proto_packet_test::format::variants::{
    AllCases, AllCasesCase, AllCasesRef, NewerCases, OlderCases, OlderCasesCase, OlderCasesRef,
    Shape, ShapeBuf, ShapeCase, ShapeMut, ShapeRef, ShapeView, ShapeViewMut, WithCases,
    WithVariants, WithVariantsMut, WithVariantsRef,
};
use proto_packet_test::variants::boxed::{
    Expr, ExprBuf, ExprMut, ExprRef, ExprView, ExprViewMut, Sum, SumView, SumViewMut,
};
use proto_packet_test::variants::empty::{EmptyVariant, EmptyVariantRef};

/// Asserts the `value` of the `type` encodes to the `bytes`, & decodes back from them.
macro_rules! assert_encoding {
    ($type:ty, $value:expr, $bytes:expr) => {{
        let value: $type = $value;
        let bytes: &[u8] = $bytes;
        assert_eq!(value.encode(), bytes);
        assert_eq!(value.encoded_len(), bytes.len());
        assert_eq!(<$type>::decode(bytes), Ok(value.clone()));
        assert_eq!(unsafe { <$type>::decode_unchecked(bytes) }, value);
        let (len, encoded): (usize, Vec<u8>) = common::ViewEncode::view_encode(&value);
        assert_eq!(encoded, bytes);
        assert_eq!(len, bytes.len());
        common::assert_one_encoding(bytes, |changed| {
            let value: $type = <$type>::decode(changed).ok()?;
            assert_eq!(value.encoded_len(), changed.len());
            Some(value.encode())
        });
    }};
}

common::plain!(
    proto_packet_test::format::variants::AllCasesView,
    AllCases,
    {
        fn tag_number(&self) -> u16;
        fn as_flag(&self) -> Option<bool>;
        fn as_byte(&self) -> Option<u8>;
        fn as_large(&self) -> Option<u128>;
        fn as_id(&self) -> Option<proto_packet::types::Uuid>;
        fn as_color(&self) -> Option<proto_packet_test::format::examples::Color>;
        fn as_text(&self) -> Option<&str>;
        fn as_point(&self) -> Option<impl proto_packet_test::format::examples::PointView>;
        fn as_entry(&self) -> Option<impl proto_packet_test::format::examples::EntryView>;
        fn as_item(&self) -> Option<impl proto_packet_test::format::messages::ItemView>;
        fn as_shape(&self) -> Option<impl proto_packet_test::format::variants::ShapeView>;
        fn as_boxed_color(&self) -> Option<proto_packet_test::format::examples::Color>;
        fn as_signed(&self) -> Option<i64>;
        fn as_float(&self) -> Option<proto_packet::types::Float64>;
        fn as_at(&self) -> Option<proto_packet::types::Timestamp>;
        fn as_price(&self) -> Option<proto_packet::types::Decimal<2>>;
    }
);

common::plain!(proto_packet_test::variants::boxed::ExprView, Expr, {
    fn tag_number(&self) -> u16;
    fn as_number(&self) -> Option<u64>;
    fn as_negate(&self) -> Option<impl proto_packet_test::variants::boxed::ExprView>;
    fn as_add(&self) -> Option<impl proto_packet_test::variants::boxed::SumView>;
});

common::plain!(proto_packet_test::format::variants::ShapeView, Shape, {
    fn tag_number(&self) -> u16;
    fn as_circle(&self) -> Option<u32>;
    fn as_square(&self) -> Option<impl proto_packet_test::format::examples::PointView>;
});

common::plain!(
    proto_packet_test::format::variants::WithCasesView,
    WithCases,
    {
        fn shape(&self) -> Option<impl proto_packet_test::format::variants::ShapeView>;
        fn older(&self) -> Option<impl proto_packet_test::format::variants::OlderCasesView>;
        fn id(&self) -> Option<u32>;
    }
);

common::plain!(
    proto_packet_test::format::variants::WithVariantsView,
    WithVariants,
    {
        fn before(&self) -> &str;
        fn shape(&self) -> impl proto_packet_test::format::variants::ShapeView;
        fn number(&self) -> u64;
        fn maybe(&self) -> Option<impl proto_packet_test::format::variants::ShapeView>;
    }
);

/// Gets a case of each kind.
fn all_cases() -> Vec<AllCases> {
    vec![
        AllCases::Flag(true),
        AllCases::Byte(1),
        AllCases::Large(2),
        AllCases::Id(ID),
        AllCases::Color(Color::Red),
        AllCases::Text("text".to_string()),
        AllCases::Point(Point::new(3, 4)),
        AllCases::Entry(Entry::new(5, "entry".to_string())),
        AllCases::Item(Item::default().with_text("hi".to_string()).with_id(7)),
        AllCases::Shape(Shape::Circle(8)),
        AllCases::BoxedColor(Box::new(Color::Red)),
        AllCases::Signed(-2),
        AllCases::Float(Float64::new(1.5)),
    ]
}

/// Gets the bytes of the newer `Point` case, which an older reader doesn't know.
fn unknown_bytes() -> Vec<u8> {
    NewerCases::Point(Point::new(1, 2)).encode()
}

#[test]
fn shape_examples() {
    assert_encoding!(Shape, Shape::Circle(5), &[1, 0, 5, 0, 0, 0]);
    assert_encoding!(
        Shape,
        Shape::Square(Point::new(1, 2)),
        &[2, 0, 1, 0, 0, 0, 2, 0, 0, 0]
    );
}

#[test]
fn signed_integers_are_twos_complement() {
    let bytes: [u8; 10] = [12, 0, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    assert_encoding!(AllCases, AllCases::Signed(-2), &bytes);
    assert_eq!(
        serde_json::to_string(&AllCases::Signed(-2)).unwrap(),
        r#"{"var":"Signed","val":-2}"#
    );
}

#[test]
fn floats_are_their_bits() {
    let bytes: [u8; 10] = [13, 0, 0, 0, 0, 0, 0, 0, 0, 0xC0];
    assert_encoding!(AllCases, AllCases::Float(Float64::new(-2.0)), &bytes);

    let mut other_nan: [u8; 10] = bytes;
    other_nan[8..].copy_from_slice(&[0xF8, 0xFF]);
    assert_eq!(AllCases::decode(&other_nan), Err(DecodeError::InvalidFloat));
    assert_eq!(
        serde_json::to_string(&AllCases::Float(Float64::NAN)).unwrap(),
        r#"{"var":"Float","val":"NaN"}"#
    );
    assert_eq!(
        serde_json::from_str::<AllCases>(r#"{"val":"NaN","var":"Float"}"#).unwrap(),
        AllCases::Float(Float64::NAN)
    );
}

#[test]
fn getters_find_only_the_present_case() {
    let shape: Shape = Shape::Circle(5);
    assert_eq!(shape.tag_number(), 1);
    assert_eq!(shape.as_circle(), Some(5));
    assert_eq!(shape.as_square(), None);

    let bytes: Vec<u8> = Shape::Square(Point::new(1, 2)).encode();
    let view: ShapeRef = ShapeRef::new(&bytes).unwrap();
    assert_eq!(view.tag_number(), 2);
    assert_eq!(view.as_circle(), None);
    assert_eq!(view.as_square().unwrap().y(), 2);
}

#[test]
fn every_case_round_trips() {
    for value in all_cases() {
        let bytes: Vec<u8> = value.encode();
        assert_encoding!(AllCases, value, &bytes);
    }
}

#[test]
fn cases_match_without_copying() {
    let value: AllCases = AllCases::Text("text".to_string());
    let bytes: Vec<u8> = value.encode();
    let view: AllCasesRef = AllCasesRef::new(&bytes).unwrap();
    match view.case() {
        AllCasesCase::Text(text) => {
            assert_eq!(text, "text");
            assert_eq!(text.as_ptr(), bytes[2..].as_ptr());
        }
        other => panic!("{:?}", other),
    }

    let bytes: Vec<u8> = Shape::Square(Point::new(1, 2)).encode();
    match ShapeRef::new(&bytes).unwrap().case() {
        ShapeCase::Square(point) => assert_eq!(point.x(), 1),
        other => panic!("{:?}", other),
    }
}

#[test]
fn unknown_cases_survive_decoding() {
    let bytes: Vec<u8> = unknown_bytes();
    let older: OlderCases = OlderCases::decode(&bytes).unwrap();
    assert_eq!(older.tag_number(), 3);
    assert_eq!(older.unknown_bytes(), Some(&bytes[2..]));
    assert_eq!(older.as_text(), None);

    assert_eq!(older.encode(), bytes);
    assert_eq!(older.encoded_len(), bytes.len());
    assert_eq!(
        NewerCases::decode(&older.encode()),
        Ok(NewerCases::Point(Point::new(1, 2)))
    );
}

#[test]
fn unknown_cases_match_as_unrecognized() {
    let bytes: Vec<u8> = unknown_bytes();
    match OlderCasesRef::new(&bytes).unwrap().case() {
        OlderCasesCase::Unrecognized { tag_number, bytes } => {
            assert_eq!(tag_number, 3);
            assert_eq!(bytes.len(), 8);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn known_cases_read_across_versions() {
    let bytes: Vec<u8> = NewerCases::Text("hi".to_string()).encode();
    assert_eq!(
        OlderCases::decode(&bytes),
        Ok(OlderCases::Text("hi".to_string()))
    );
}

#[test]
fn empty_variants_only_hold_unknown_cases() {
    let bytes: [u8; 3] = [7, 0, 1];
    let value: EmptyVariant = EmptyVariant::decode(&bytes).unwrap();
    assert_eq!(value.tag_number(), 7);
    assert_eq!(value.encode(), bytes);
    assert_eq!(EmptyVariantRef::new(&bytes).unwrap().tag_number(), 7);
}

#[test]
fn rejects_extra_bytes() {
    let mut bytes: Vec<u8> = Shape::Circle(5).encode();
    bytes.extend_from_slice(&[0xFF; 3]);
    assert_eq!(
        Shape::decode(&bytes),
        Err(DecodeError::TooLong {
            expected: 4,
            actual: 7,
        })
    );

    let mut bytes: Vec<u8> = Shape::Square(Point::new(1, 2)).encode();
    bytes.push(0xFF);
    assert_eq!(
        Shape::decode(&bytes),
        Err(DecodeError::TooLong {
            expected: 8,
            actual: 9,
        })
    );
}

#[test]
fn strings_are_the_rest_of_the_variant() {
    let mut bytes: Vec<u8> = OlderCases::Text("hi".to_string()).encode();
    bytes.push(b'!');
    assert_eq!(
        OlderCases::decode(&bytes),
        Ok(OlderCases::Text("hi!".to_string()))
    );
}

#[test]
fn rejects_short_bytes() {
    assert_eq!(
        Shape::decode(&[1]),
        Err(DecodeError::TooShort {
            expected: 2,
            actual: 1,
        })
    );
    assert_eq!(
        Shape::decode(&[1, 0, 5]),
        Err(DecodeError::TooShort {
            expected: 4,
            actual: 1,
        })
    );
}

#[test]
fn rejects_invalid_tags() {
    assert_eq!(
        Shape::decode(&[0, 0, 5, 0, 0, 0]),
        Err(DecodeError::InvalidTagNumber { value: 0 })
    );
}

#[test]
fn rejects_invalid_values() {
    assert_eq!(
        OlderCases::decode(&[2, 0, 0xFF]),
        Err(DecodeError::InvalidUtf8)
    );
    assert_eq!(
        AllCases::decode(&[5, 0, 0, 0]),
        Err(DecodeError::InvalidTagNumber { value: 0 })
    );
    assert_eq!(
        AllCases::decode(&[10, 0, 0, 0]),
        Err(DecodeError::InvalidTagNumber { value: 0 })
    );
}

#[test]
fn rejects_bool_bytes_above_1() {
    let mut bytes: Vec<u8> = AllCases::Flag(true).encode();
    *bytes.last_mut().unwrap() = 2;
    assert_eq!(
        AllCases::decode(&bytes),
        Err(DecodeError::InvalidBool { value: 2 })
    );
}

#[test]
fn unknown_cases_are_ordered_by_tag_number() {
    let before: OlderCases = OlderCases::decode(&NewerCases::Number(5).encode()).unwrap();
    let known: OlderCases = OlderCases::Text(String::default());
    let after: OlderCases = OlderCases::decode(&unknown_bytes()).unwrap();
    assert!(before.tag_number() < known.tag_number());
    assert!(before < known && known < after);
}

#[test]
fn unknown_cases_are_not_validated() {
    OlderCases::decode(&[9, 0, 0xFF, 0xFF]).unwrap();
}

#[test]
fn mut_reaches_the_present_nested_case() {
    let mut bytes: Vec<u8> = Shape::Square(Point::new(1, 2)).encode();
    let mut value: ShapeMut = ShapeMut::new(&mut bytes).unwrap();
    value.as_square_mut().unwrap().set_y(9);
    assert_eq!(Shape::decode(&bytes), Ok(Shape::Square(Point::new(1, 9))));

    let mut bytes: Vec<u8> = Shape::Circle(5).encode();
    let mut value: ShapeMut = ShapeMut::new(&mut bytes).unwrap();
    assert!(value.as_square_mut().is_none());
}

#[test]
fn bufs_own_the_encoding() {
    let value: Shape = Shape::Square(Point::new(1, 2));
    let mut buf: ShapeBuf = ShapeBuf::from(&value);
    assert_eq!(buf.as_bytes(), value.encode());
    assert_eq!(buf.as_ref().tag_number(), 2);
    buf.as_square_mut().unwrap().set_x(9);
    assert_eq!(buf.decode(), Shape::Square(Point::new(9, 2)));
    assert_eq!(
        ShapeRef::new(&value.encode()).unwrap().to_buf().decode(),
        value
    );
}

/// Gets the tag & a coordinate of any form of a shape.
fn shape_fields<S: ShapeView>(shape: &S) -> (u16, Option<u32>, Option<u32>) {
    (
        shape.tag_number(),
        shape.as_circle(),
        shape.as_square().map(|point| point.y()),
    )
}

#[test]
fn views_read_every_form() {
    let value: Shape = Shape::Square(Point::new(1, 2));
    let expected = (2, None, Some(2));
    let mut bytes: Vec<u8> = value.encode();
    assert_eq!(shape_fields(&value), expected);
    assert_eq!(shape_fields(&&value), expected);
    assert_eq!(shape_fields(&ShapeRef::new(&bytes).unwrap()), expected);
    assert_eq!(shape_fields(&ShapeBuf::from(&value)), expected);
    assert_eq!(shape_fields(&ShapeMut::new(&mut bytes).unwrap()), expected);
}

/// Edits any editable form of a shape.
fn edit_shape<S: ShapeViewMut>(shape: &mut S) {
    if let Some(mut point) = shape.as_square_mut() {
        point.set_x(9);
    }
}

#[test]
fn view_muts_edit_every_form() {
    let value: Shape = Shape::Square(Point::new(1, 2));
    let expected: Shape = Shape::Square(Point::new(9, 2));

    let mut native: Shape = value.clone();
    edit_shape(&mut native);
    assert_eq!(native, expected);

    let mut buf: ShapeBuf = ShapeBuf::from(&value);
    edit_shape(&mut buf);
    assert_eq!(buf.decode(), expected);

    let mut bytes: Vec<u8> = value.encode();
    let mut edit: ShapeMut = ShapeMut::new(&mut bytes).unwrap();
    edit_shape(&mut edit);
    assert_eq!(edit.as_ref().as_square().unwrap().x(), 9);

    let mut circle: Shape = Shape::Circle(5);
    edit_shape(&mut circle);
    assert_eq!(circle, Shape::Circle(5));
}

/// Gets the value of any form of an expression.
fn eval<E: ExprView>(expr: &E) -> i64 {
    if let Some(number) = expr.as_number() {
        return number as i64;
    }
    if let Some(inner) = expr.as_negate() {
        return -eval(&inner);
    }
    let sum = expr.as_add().unwrap();
    let (left, right) = (sum.left(), sum.right());
    eval(&left) + eval(&right)
}

/// Counts the numbers of any editable form of an expression, through its mutable values.
fn count<E: ExprViewMut>(expr: &mut E) -> usize {
    if expr.as_number().is_some() {
        return 1;
    }
    if let Some(mut inner) = expr.as_negate_mut() {
        return count(&mut inner);
    }
    let mut sum = expr.as_add_mut().unwrap();
    let left: usize = count(&mut sum.left_mut());
    left + count(&mut sum.right_mut())
}

/// Gets a recursive expression: `1 + -(2 + 3)`.
fn expr(scale: u64) -> Expr {
    let inner: Sum = Sum::new(Expr::Number(2 * scale), Expr::Number(3 * scale));
    let negated: Expr = Expr::Negate(Box::new(Expr::Add(inner)));
    Expr::Add(Sum::new(Expr::Number(scale), negated))
}

#[test]
fn recursive_variants() {
    let value: Expr = expr(1);
    let mut bytes: Vec<u8> = value.encode();
    assert_encoding!(Expr, value.clone(), &bytes);

    assert_eq!(eval(&value), -4);
    assert_eq!(eval(&ExprRef::new(&bytes).unwrap()), -4);
    assert_eq!(eval(&ExprBuf::from(&value)), -4);
    assert_eq!(eval(&ExprMut::new(&mut bytes).unwrap()), -4);

    let mut native: Expr = expr(1);
    assert_eq!(count(&mut native), 3);
    assert_eq!(count(&mut ExprMut::new(&mut bytes).unwrap()), 3);
    assert_eq!(count(&mut ExprBuf::from(&value)), 3);
}

/// Gets a struct with variants.
fn with_variants() -> WithVariants {
    WithVariants::new(
        "before".to_string(),
        Shape::Square(Point::new(1, 2)),
        8,
        Some(Shape::Circle(5)),
    )
}

#[test]
fn structs_hold_variants() {
    let value: WithVariants = with_variants();
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(WithVariants, value, &bytes);

    let view: WithVariantsRef = WithVariantsRef::new(&bytes).unwrap();
    assert_eq!(view.before(), "before");
    assert_eq!(view.shape().as_square().unwrap().y(), 2);
    assert_eq!(view.maybe().unwrap().as_circle(), Some(5));

    let absent: WithVariants = WithVariants::new("x".to_string(), Shape::Circle(1), 8, None);
    let bytes: Vec<u8> = absent.encode();
    assert_encoding!(WithVariants, absent, &bytes);
    assert!(WithVariantsRef::new(&bytes).unwrap().maybe().is_none());
}

#[test]
fn structs_edit_their_variants() {
    let mut bytes: Vec<u8> = with_variants().encode();
    let mut value: WithVariantsMut = WithVariantsMut::new(&mut bytes).unwrap();
    value.shape_mut().as_square_mut().unwrap().set_y(9);
    assert!(value.maybe_mut().unwrap().as_square_mut().is_none());
    assert_eq!(value.as_ref().shape().as_square().unwrap().y(), 9);
    assert_eq!(value.as_ref().before(), "before");
}

#[test]
fn messages_hold_variants() {
    let value: WithCases = WithCases::default().with_shape(Shape::Circle(5)).with_id(7);
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(WithCases, value, &bytes);
}

#[test]
fn known_cases_stay_plain_json() {
    let assert_json = |value: &Shape, json: &str| {
        assert_eq!(serde_json::to_string(value).unwrap(), json);
        assert_eq!(&serde_json::from_str::<Shape>(json).unwrap(), value);
    };
    assert_json(&Shape::Circle(5), r#"{"var":"Circle","val":5}"#);
    assert_json(
        &Shape::Square(Point::new(1, 2)),
        r#"{"var":"Square","val":{"x":1,"y":2}}"#,
    );
}

#[test]
fn json_keys_can_be_in_any_order() {
    assert_eq!(
        serde_json::from_str::<Shape>(r#"{"val":5,"var":"Circle"}"#).unwrap(),
        Shape::Circle(5)
    );
    assert_eq!(
        serde_json::from_str::<Shape>(r#"{"val":{"x":1,"y":2},"var":"Square"}"#).unwrap(),
        Shape::Square(Point::new(1, 2))
    );
    assert_eq!(
        serde_json::from_str::<Shape>(r#"{"other":[1,{"a":null}],"var":"Circle","val":5}"#)
            .unwrap(),
        Shape::Circle(5)
    );
}

#[test]
fn every_case_round_trips_through_json() {
    for value in all_cases() {
        let json: String = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<AllCases>(&json).unwrap(), value);

        // The value first, so it goes through the buffer.
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let reordered: String = format!(r#"{{"val":{},"var":{}}}"#, parsed["val"], parsed["var"]);
        assert_eq!(serde_json::from_str::<AllCases>(&reordered).unwrap(), value);
    }
}

#[test]
fn large_integers_read_from_json() {
    let json: &str = r#"{"var":"Large","val":340282366920938463463374607431768211455}"#;
    assert_eq!(
        serde_json::from_str::<AllCases>(json).unwrap(),
        AllCases::Large(u128::MAX)
    );
}

#[test]
fn unknown_cases_are_base_64_json() {
    let older: OlderCases = OlderCases::decode(&unknown_bytes()).unwrap();
    assert_eq!(
        serde_json::to_string(&older).unwrap(),
        r#"{"var":3,"val":"AQAAAAIAAAA="}"#
    );
}

#[test]
fn unknown_cases_round_trip_through_json() {
    let older: OlderCases = OlderCases::decode(&unknown_bytes()).unwrap();
    let json: String = serde_json::to_string(&older).unwrap();
    assert_eq!(serde_json::from_str::<OlderCases>(&json).unwrap(), older);
}

#[test]
fn known_cases_decode_from_base_64() {
    let older: OlderCases = OlderCases::decode(&unknown_bytes()).unwrap();
    let json: String = serde_json::to_string(&older).unwrap();
    assert_eq!(
        serde_json::from_str::<NewerCases>(&json).unwrap(),
        NewerCases::Point(Point::new(1, 2))
    );

    let text: NewerCases = serde_json::from_str(r#"{"val":"aGk=","var":2}"#).unwrap();
    assert_eq!(text, NewerCases::Text("hi".to_string()));
}

#[test]
fn nested_unknown_cases_round_trip_through_json() {
    let older: OlderCases = OlderCases::decode(&unknown_bytes()).unwrap();
    let value: WithCases = WithCases::default().with_older(older);
    let json: String = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"{"older":{"var":3,"val":"AQAAAAIAAAA="}}"#);
    assert_eq!(serde_json::from_str::<WithCases>(&json).unwrap(), value);
    assert_eq!(
        WithCases::decode(&value.encode())
            .unwrap()
            .older()
            .unwrap()
            .tag_number(),
        3
    );
}

#[test]
fn empty_unknown_cases_round_trip_through_json() {
    let value: OlderCases = serde_json::from_str(r#"{"var":9,"val":""}"#).unwrap();
    assert_eq!(value.tag_number(), 9);
    assert_eq!(value.unknown_bytes(), Some(&[][..]));
    assert_eq!(
        serde_json::to_string(&value).unwrap(),
        r#"{"var":9,"val":""}"#
    );
}

#[test]
fn json_rejects_invalid_variants() {
    for json in [
        r#"{"var":"Nope","val":1}"#,                // an unknown name
        r#"{"var":0,"val":""}"#,                    // an invalid tag number
        r#"{"var":65536,"val":""}"#,                // a tag number above 65,535
        r#"{"var":-1,"val":""}"#,                   // a negative tag number
        r#"{"var":3,"val":"!!!!"}"#,                // invalid base-64
        r#"{"var":3,"val":5}"#,                     // a value that isn't base-64 text
        r#"{"var":3,"val":"AQ=="}"#,                // a known case too short for its type
        r#"{"var":"Circle"}"#,                      // no value
        r#"{"val":5}"#,                             // no case
        r#"{"var":"Number","val":1,"var":"Text"}"#, // a duplicate case
        r#"{"var":"Number","val":1,"val":2}"#,      // a duplicate value
        r#"{"val":1,"val":2,"var":"Number"}"#,      // a duplicate buffered value
    ] {
        assert!(
            serde_json::from_str::<NewerCases>(json).is_err(),
            "{}",
            json
        );
    }
}

#[test]
fn unknown_fields_in_cases_round_trip_through_json() {
    let inner: Item = serde_json::from_str(r#"{"text":"hi","7":"CQ=="}"#).unwrap();
    let value: AllCases = AllCases::Item(inner);
    let json: String = serde_json::to_string(&value).unwrap();
    assert_eq!(serde_json::from_str::<AllCases>(&json).unwrap(), value);

    // The value first, so it goes through the buffer.
    let reordered: &str = r#"{"val":{"text":"hi","7":"CQ=="},"var":"Item"}"#;
    assert_eq!(serde_json::from_str::<AllCases>(reordered).unwrap(), value);
    assert_eq!(
        value.encode(),
        serde_json::from_str::<AllCases>(&json).unwrap().encode()
    );
}

#[test]
fn older_readers_have_one_encoding_of_newer_bytes() {
    for bytes in [unknown_bytes(), NewerCases::Number(5).encode()] {
        common::assert_one_encoding(&bytes, |changed| {
            let older: OlderCases = OlderCases::decode(changed).ok()?;
            assert_eq!(older.encoded_len(), changed.len());
            Some(older.encode())
        });
    }
}

#[test]
fn forms_are_equal_across_forms() {
    let shape: Shape = Shape::Square(Point::new(1, 2));
    let bytes: Vec<u8> = shape.encode();
    let shape_ref: ShapeRef = ShapeRef::new(&bytes).unwrap();
    let buf: ShapeBuf = ShapeBuf::from(&shape);
    assert!(shape == shape_ref && shape_ref == shape);
    assert!(shape == buf && buf == shape && buf == shape_ref && shape_ref == buf);
    assert!(Shape::Circle(1) != shape_ref && Shape::Circle(1) != buf);

    // an older reader keeps the case it doesn't know, so its forms are still equal
    let unknown: Vec<u8> = unknown_bytes();
    let older: OlderCases = OlderCases::decode(&unknown).unwrap();
    let older_ref: OlderCasesRef = OlderCasesRef::new(&unknown).unwrap();
    assert!(older == older_ref && older_ref == older);
    assert!(OlderCases::Text("a".to_string()) != older_ref);
}

/// Gets a number inside the `count` of negations.
fn negated(count: usize) -> Expr {
    (0..count).fold(Expr::Number(1), |expr, _| Expr::Negate(Box::new(expr)))
}

/// Encodes the `expr` with a writer that is given one level more than a value can nest.
fn encode_deep(expr: &Expr) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::default();
    expr.encode_to_with_depth(&mut bytes, 65);
    bytes
}

#[test]
fn variants_nest_at_most_64_levels() {
    assert_eq!(Expr::decode(&negated(63).encode()), Ok(negated(63)));
    assert_eq!(
        Expr::decode(&encode_deep(&negated(64))),
        Err(DecodeError::TooDeep)
    );

    // a sum is a struct with a variable tail, so it is a level between its expressions
    let sum = |inner: Expr| -> Expr { Expr::Add(Sum::new(Expr::Number(1), inner)) };
    assert_eq!(
        Expr::decode(&sum(negated(61)).encode()),
        Ok(sum(negated(61)))
    );
    assert_eq!(
        Expr::decode(&encode_deep(&sum(negated(62)))),
        Err(DecodeError::TooDeep)
    );
}

#[test]
#[should_panic(expected = "the value nests deeper than 64 levels")]
fn writers_refuse_variants_that_nest_too_deep() {
    let _ = negated(64).encode();
}
