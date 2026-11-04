mod common;

use common::ID;
use proto_packet::io::DecodeError;
use proto_packet_test::format::edges::{FixedKinds, FixedKindsRef, NineBits, NineBitsRef};
use proto_packet_test::format::examples::{
    Color, Line, LineBuf, LineRef, Mixed, MixedRef, Point, PointBuf, PointRef,
};
use proto_packet_test::structs::empty::{EmptyStruct, EmptyStructRef};

/// Asserts the `value` of the `type` encodes to the `bytes`, & decodes back from them.
macro_rules! assert_encoding {
    ($type:ty, $value:expr, $bytes:expr) => {{
        let value: $type = $value;
        let bytes: &[u8] = $bytes;
        assert_eq!(value.encode(), bytes);
        assert_eq!(value.encoded_len(), bytes.len());
        assert_eq!(<$type>::decode(bytes), Ok(value.clone()));
        assert_eq!(unsafe { <$type>::decode_unchecked(bytes) }, value);
    }};
}

#[test]
fn point_example() {
    let bytes: [u8; 8] = [1, 0, 0, 0, 2, 0, 0, 0];
    assert_encoding!(Point, Point::new(1, 2), &bytes);

    let point: PointRef = PointRef::new(&bytes).unwrap();
    assert_eq!((point.x(), point.y()), (1, 2));
    assert_eq!(point.as_bytes(), bytes);
}

#[test]
fn mixed_example() {
    let bytes: [u8; 16] = [2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 1, 1, 0, 0, 0, 0];
    assert_encoding!(Mixed, Mixed::new(1, 2, 3, true), &bytes);

    let mixed: MixedRef = MixedRef::new(&bytes).unwrap();
    assert_eq!(
        (mixed.a(), mixed.b(), mixed.c(), mixed.d()),
        (1, 2, 3, true)
    );
}

#[test]
fn line_example() {
    let bytes: [u8; 20] = [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0];
    let line: Line = Line::new(Point::new(1, 2), Point::new(3, 4), 5);
    assert_encoding!(Line, line, &bytes);

    let line_ref: LineRef = LineRef::new(&bytes).unwrap();
    assert_eq!(line_ref.start().y(), 2);
    assert_eq!(line_ref.end().x(), 3);
    assert_eq!(line_ref.width(), 5);
    assert_eq!(line_ref.decode(), line);
}

#[test]
fn bits_cross_bytes() {
    let value: NineBits = NineBits::new(true, false, true, false, false, false, false, true, true);
    assert_encoding!(NineBits, value, &[0b1000_0101, 0b0000_0001]);

    let bits: NineBitsRef = NineBitsRef::new(&[0b1000_0101, 0b0000_0001]).unwrap();
    assert!(bits.a() && bits.c() && bits.h() && bits.i());
    assert!(!bits.b() && !bits.g());
}

#[test]
fn fixed_kinds() {
    let bytes: Vec<u8> = [
        ID.as_bytes().as_slice(),
        &[1, 0, 0, 0, 2, 0, 0, 0], // point
        &[0; 8],                   // maybe_point, absent
        &[1, 0],                   // color
        &[0, 0],                   // maybe_color, absent
        &[0, 0, 0, 0],             // bits & padding
    ]
    .concat();
    let value: FixedKinds = FixedKinds::new(ID, Color::Red, Point::new(1, 2), None, None);
    assert_encoding!(FixedKinds, value, &bytes);

    let present: FixedKinds = FixedKinds::new(
        ID,
        Color::Red,
        Point::new(1, 2),
        Some(Point::new(3, 4)),
        Some(Color::Red),
    );
    let encoded: Vec<u8> = present.encode();
    assert_encoding!(FixedKinds, present.clone(), &encoded);

    let kinds: FixedKindsRef = FixedKindsRef::new(&encoded).unwrap();
    assert_eq!(kinds.id(), ID);
    assert_eq!(kinds.maybe_point().unwrap().y(), 4);
    assert_eq!(kinds.maybe_color(), Some(Color::Red));
}

#[test]
fn empty_struct() {
    assert_encoding!(EmptyStruct, EmptyStruct::default(), &[]);
    assert_eq!(EmptyStructRef::new(&[]).unwrap().as_bytes(), [0_u8; 0]);
}

#[test]
fn bufs_own_the_encoding() {
    let line: Line = Line::new(Point::new(1, 2), Point::new(3, 4), 5);
    let buf: LineBuf = LineBuf::from(&line);
    assert_eq!(buf.as_bytes(), line.encode());
    assert_eq!(buf.as_ref().end().y(), 4);
    assert_eq!(buf.decode(), line);
    assert_eq!(
        LineBuf::new(line.encode()).unwrap().into_bytes(),
        line.encode()
    );

    let point: PointBuf = PointRef::new(&[1, 0, 0, 0, 2, 0, 0, 0]).unwrap().to_buf();
    assert_eq!(point.decode(), Point::new(1, 2));
}

#[test]
fn slack_is_ignored() {
    let bytes: [u8; 10] = [1, 0, 0, 0, 2, 0, 0, 0, 0xFF, 0xFF];
    assert_eq!(Point::decode(&bytes), Ok(Point::new(1, 2)));
}

#[test]
fn padding_is_ignored() {
    let mut bytes: [u8; 16] = [2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 1, 1, 0, 0, 0, 0];
    bytes[12] = 0xFF;
    assert_eq!(Mixed::decode(&bytes), Ok(Mixed::new(1, 2, 3, true)));
}

#[test]
fn rejects_short_bytes() {
    assert_eq!(
        Point::decode(&[0; 7]),
        Err(DecodeError::TooShort {
            expected: 8,
            actual: 7,
        })
    );
}

#[test]
fn rejects_invalid_tag_numbers() {
    let mut bytes: Vec<u8> = FixedKinds::new(ID, Color::Red, Point::new(1, 2), None, None).encode();
    bytes[32] = 0;
    bytes[33] = 0;
    assert_eq!(
        FixedKinds::decode(&bytes),
        Err(DecodeError::InvalidTagNumber { value: 0 })
    );
    bytes[32] = 1;
    bytes[34] = 0xFF;
    bytes[35] = 0xFF;
    assert_eq!(
        FixedKinds::decode(&bytes),
        Err(DecodeError::InvalidTagNumber { value: 0xFFFF })
    );
}

#[test]
fn keeps_unrecognized_tag_numbers() {
    let mut bytes: Vec<u8> = FixedKinds::new(ID, Color::Red, Point::new(1, 2), None, None).encode();
    bytes[32] = 9;
    let decoded: FixedKinds = FixedKinds::decode(&bytes).unwrap();
    assert_eq!(decoded.color().tag_number(), 9);
    assert_eq!(decoded.encode(), bytes);
}
