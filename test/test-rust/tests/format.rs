mod common;

use common::ID;
use proto_packet::io::DecodeError;
use proto_packet_test::format::edges::{
    AbsentChild, FixedKinds, FixedKindsRef, LoopByte, LoopLong, NineBits, NineBitsRef,
};
use proto_packet_test::format::examples::{
    Color, Entry, Line, LineBuf, LineRef, Mixed, MixedRef, Opt, OptRef, Pair, PairRef, Person,
    PersonRef, Point, PointBuf, PointRef,
};
use proto_packet_test::structs::boxed::{ListNode, ListNodeRef};
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

#[test]
fn person_example() {
    let bytes: [u8; 8] = [3, 0, 0, 0, 36, b'A', b'd', b'a'];
    assert_encoding!(Person, Person::new("Ada".to_string(), 36), &bytes);

    let person: PersonRef = PersonRef::new(&bytes).unwrap();
    assert_eq!((person.name(), person.age()), ("Ada", 36));
}

/// Gets the bytes of the `Pair` example of `FORMAT.md`.
fn pair_bytes() -> Vec<u8> {
    [
        &[24, 0, 0, 0, 38, 0, 0, 0, 40, 0, 0, 0, 0, 0, 0, 0][..], // ends & padding
        &[1, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0],                    // first
        b"hello",
        &[0; 7],
        &[2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0], // second
        b"hi",
        b"ok",
    ]
    .concat()
}

/// Gets the value of the `Pair` example of `FORMAT.md`.
fn pair() -> Pair {
    Pair::new(
        Entry::new(1, "hello".to_string()),
        Entry::new(2, "hi".to_string()),
        "ok".to_string(),
    )
}

#[test]
fn pair_example() {
    let bytes: Vec<u8> = pair_bytes();
    assert_eq!(bytes.len(), 56);
    assert_encoding!(Pair, pair(), &bytes);

    let pair: PairRef = PairRef::new(&bytes).unwrap();
    assert_eq!((pair.first().id(), pair.first().text()), (1, "hello"));
    assert_eq!((pair.second().id(), pair.second().text()), (2, "hi"));
    assert_eq!(pair.note(), "ok");
}

#[test]
fn variable_children_are_aligned() {
    let bytes: Vec<u8> = pair_bytes();
    let pair: PairRef = PairRef::new(&bytes).unwrap();
    let start: usize = bytes.as_ptr() as usize;
    let second: usize = pair.second().as_bytes().as_ptr() as usize;
    assert_eq!((second - start) % 8, 0);
}

#[test]
fn opt_example() {
    let bytes: [u8; 13] = [0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0b0001_0101, b'h', b'i'];
    let value: Opt = Opt::new(true, None, Some(false), Some("hi".to_string()), None);
    assert_encoding!(Opt, value, &bytes);

    let opt: OptRef = OptRef::new(&bytes).unwrap();
    assert!(opt.a());
    assert_eq!(opt.b(), None);
    assert_eq!(opt.c(), Some(false));
    assert_eq!(opt.d(), Some("hi"));
    assert_eq!(opt.e(), None);
}

#[test]
fn absent_entries_are_empty() {
    let second: Entry = Entry::new(1, "hi".to_string());
    let bytes: Vec<u8> = [
        &[0, 0, 0, 0, 14, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0][..], // ends, bits & padding
        &[1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0],                   // second
        b"hi",
        b"ok",
    ]
    .concat();
    let value: AbsentChild = AbsentChild::new(None, second.clone(), "ok".to_string());
    assert_encoding!(AbsentChild, value, &bytes);

    let first: Entry = Entry::new(2, "hello".to_string());
    let present: AbsentChild = AbsentChild::new(Some(first), second, "ok".to_string());
    let bytes: Vec<u8> = present.encode();
    assert_eq!(bytes.len(), 56);
    assert_encoding!(AbsentChild, present, &bytes);
}

#[test]
fn loops_share_their_alignment() {
    let bytes: Vec<u8> = [
        &[16, 0, 0, 0, 1, 1, 0, 0][..],                        // LoopByte
        &[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0][..], // LoopLong
    ]
    .concat();
    let value: LoopByte = LoopByte::new(1, Some(LoopLong::new(2, None)));
    assert_encoding!(LoopByte, value, &bytes);

    let long: LoopLong = LoopLong::new(1, Some(LoopByte::new(2, None)));
    let bytes: Vec<u8> = long.encode();
    assert_eq!(bytes.len(), 16 + 8);
    assert_encoding!(LoopLong, long, &bytes);
}

#[test]
fn linked_lists_nest() {
    let tail: ListNode = ListNode::new(3, None);
    let list: ListNode = ListNode::new(1, Some(ListNode::new(2, Some(tail))));
    let bytes: Vec<u8> = list.encode();
    assert_encoding!(ListNode, list, &bytes);

    let node: ListNodeRef = ListNodeRef::new(&bytes).unwrap();
    assert_eq!(node.next().unwrap().next().unwrap().value(), 3);
    assert!(node.next().unwrap().next().unwrap().next().is_none());
}

#[test]
fn slack_is_dropped_by_decoding() {
    let bytes: [u8; 10] = [3, 0, 0, 0, 36, b'A', b'd', b'a', 0xFF, 0xFF];
    let person: Person = Person::decode(&bytes).unwrap();
    assert_eq!(person, Person::new("Ada".to_string(), 36));
    assert_eq!(person.encode(), bytes[..8]);
}

#[test]
fn rejects_end_tables() {
    let past_the_end: [u8; 6] = [9, 0, 0, 0, 36, b'A'];
    assert_eq!(
        Person::decode(&past_the_end),
        Err(DecodeError::InvalidEndTable)
    );

    let mut decreasing: Vec<u8> = pair_bytes();
    decreasing[4] = 10;
    assert_eq!(Pair::decode(&decreasing), Err(DecodeError::InvalidEndTable));

    let mut too_long: Vec<u8> = pair_bytes();
    too_long[8] = 41;
    assert_eq!(Pair::decode(&too_long), Err(DecodeError::InvalidEndTable));
}

#[test]
fn rejects_short_fixed_blocks() {
    assert_eq!(
        Person::decode(&[0; 4]),
        Err(DecodeError::TooShort {
            expected: 5,
            actual: 4,
        })
    );
}

#[test]
fn rejects_invalid_utf8() {
    assert_eq!(
        Person::decode(&[1, 0, 0, 0, 36, 0xFF]),
        Err(DecodeError::InvalidUtf8)
    );
}

#[test]
fn rejects_invalid_children() {
    let mut bytes: Vec<u8> = pair_bytes();
    bytes[16 + 8] = 200; // the end of `first`
    assert_eq!(Pair::decode(&bytes), Err(DecodeError::InvalidEndTable));

    let mut bytes: Vec<u8> = pair_bytes();
    bytes[16 + 12] = 0xFF; // the text of `first`
    assert_eq!(Pair::decode(&bytes), Err(DecodeError::InvalidUtf8));
}

#[test]
fn absent_fields_are_not_validated() {
    let bytes: [u8; 13] = [0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0b0000_0001, 0xFF, 0xFF];
    let value: Opt = Opt::new(true, None, None, None, None);
    assert_eq!(Opt::decode(&bytes), Ok(value));
}
