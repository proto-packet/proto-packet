mod common;

use common::ID;
use proto_packet::io::{CapacityError, DecodeError};
use proto_packet::types::Uuid;
use proto_packet_test::format::edges::{
    AbsentChild, FixedKinds, FixedKindsMut, FixedKindsRef, FixedKindsView, LoopByte, LoopLong,
    NineBits, NineBitsMut, NineBitsRef, Strings, StringsMut,
};
use proto_packet_test::format::examples::{
    Color, Entry, EntryView, Line, LineBuf, LineMut, LineRef, LineView, Mixed, MixedMut, MixedRef,
    Opt, OptMut, OptRef, OptView, Pair, PairBuf, PairMut, PairRef, PairView, Person, PersonMut,
    PersonRef, Point, PointBuf, PointMut, PointRef, PointView,
};
use proto_packet_test::structs::boxed::{
    ListNode, ListNodeBuf, ListNodeMut, ListNodeRef, ListNodeView,
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
fn encodings_append() {
    let mut out: Vec<u8> = vec![0xAA];
    pair().encode_to(&mut out);
    assert_eq!(out[0], 0xAA);
    assert_eq!(out[1..], pair_bytes());
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

#[test]
fn mut_sets_integers_in_place() {
    let mut bytes: Vec<u8> = Mixed::new(1, 2, 3, true).encode();
    let mut mixed: MixedMut = MixedMut::new(&mut bytes).unwrap();
    mixed.set_b(9);
    mixed.set_c(8);
    assert_eq!(mixed.as_ref().b(), 9);
    assert_eq!(Mixed::decode(&bytes), Ok(Mixed::new(1, 9, 8, true)));
}

#[test]
fn mut_sets_bits_without_touching_the_others() {
    let mut bytes: Vec<u8> =
        NineBits::new(true, false, true, false, false, false, false, true, true).encode();
    let mut bits: NineBitsMut = NineBitsMut::new(&mut bytes).unwrap();
    bits.set_b(true);
    bits.set_a(false);
    bits.set_i(false);
    assert_eq!(bytes, [0b1000_0110, 0]);
}

#[test]
fn mut_sets_optional_fields() {
    let mut bytes: Vec<u8> = Opt::new(false, None, None, None, None).encode();
    let mut opt: OptMut = OptMut::new(&mut bytes).unwrap();
    opt.set_b(5);
    opt.set_c(true);
    opt.set_e(Color::Red);
    opt.set_a(true);
    assert_eq!(
        Opt::decode(&bytes),
        Ok(Opt::new(true, Some(5), Some(true), None, Some(Color::Red)))
    );

    let mut opt: OptMut = OptMut::new(&mut bytes).unwrap();
    opt.set_b(None);
    opt.set_c(None);
    opt.set_e(None);
    assert_eq!(bytes, Opt::new(true, None, None, None, None).encode());
}

#[test]
fn mut_sets_uuids_and_enums() {
    let mut bytes: Vec<u8> =
        FixedKinds::new(Uuid::nil(), Color::Red, Point::new(1, 2), None, None).encode();
    let mut kinds: FixedKindsMut = FixedKindsMut::new(&mut bytes).unwrap();
    kinds.set_id(ID);
    kinds.set_maybe_color(Color::Red);
    let decoded: FixedKinds = FixedKinds::decode(&bytes).unwrap();
    assert_eq!(decoded.id(), ID);
    assert_eq!(decoded.maybe_color(), Some(Color::Red));
}

#[test]
fn mut_edits_nested_structs_in_place() {
    let mut bytes: Vec<u8> = Line::new(Point::new(1, 2), Point::new(3, 4), 5).encode();
    let mut line: LineMut = LineMut::new(&mut bytes).unwrap();
    line.start_mut().set_y(9);
    line.end_mut().set_x(8);
    assert_eq!(
        Line::decode(&bytes),
        Ok(Line::new(Point::new(1, 9), Point::new(8, 4), 5))
    );
}

#[test]
fn mut_only_reaches_present_optional_structs() {
    let present: FixedKinds = FixedKinds::new(
        ID,
        Color::Red,
        Point::new(1, 2),
        Some(Point::new(3, 4)),
        None,
    );
    let mut bytes: Vec<u8> = present.encode();
    let mut kinds: FixedKindsMut = FixedKindsMut::new(&mut bytes).unwrap();
    kinds.maybe_point_mut().unwrap().set_y(9);
    assert_eq!(kinds.as_ref().maybe_point().unwrap().y(), 9);

    let mut bytes: Vec<u8> = FixedKinds::new(ID, Color::Red, Point::new(1, 2), None, None).encode();
    let mut kinds: FixedKindsMut = FixedKindsMut::new(&mut bytes).unwrap();
    assert!(kinds.maybe_point_mut().is_none());
}

#[test]
fn mut_shrinks_strings_and_zeroes_the_freed_bytes() {
    let mut bytes: Vec<u8> = Person::new("Ada".to_string(), 36).encode();
    let mut person: PersonMut = PersonMut::new(&mut bytes).unwrap();
    person.set_name("Al").unwrap();
    assert_eq!(bytes, [2, 0, 0, 0, 36, b'A', b'l', 0]);
    assert_eq!(
        Person::decode(&bytes),
        Ok(Person::new("Al".to_string(), 36))
    );
}

#[test]
fn mut_grows_strings_into_slack() {
    let mut bytes: Vec<u8> = Person::new("Ada".to_string(), 36).encode();
    bytes.extend_from_slice(&[0; 3]);
    let mut person: PersonMut = PersonMut::new(&mut bytes).unwrap();
    person.set_name("Grace!").unwrap();
    assert_eq!(person.as_ref().name(), "Grace!");
}

#[test]
fn mut_rejects_strings_that_do_not_fit() {
    let original: Vec<u8> = Person::new("Ada".to_string(), 36).encode();
    let mut bytes: Vec<u8> = original.clone();
    let mut person: PersonMut = PersonMut::new(&mut bytes).unwrap();
    assert_eq!(
        person.set_name("Grace"),
        Err(CapacityError {
            required: 5,
            capacity: 3,
        })
    );
    assert_eq!(bytes, original);
}

/// Gets the test struct with strings after a variable struct.
fn strings() -> Strings {
    Strings::new(
        7,
        "first".to_string(),
        Entry::new(1, "text".to_string()),
        Some("second".to_string()),
        "third".to_string(),
    )
}

#[test]
fn mut_moves_the_later_strings() {
    let mut bytes: Vec<u8> = strings().encode();
    let mut value: StringsMut = StringsMut::new(&mut bytes).unwrap();
    value.set_first("1st").unwrap();
    assert_eq!(value.as_ref().second(), Some("second"));
    assert_eq!(value.as_ref().third(), "third");
    value.set_first("first").unwrap();
    assert_eq!(bytes, strings().encode());
}

#[test]
fn mut_never_moves_nested_structs() {
    let mut bytes: Vec<u8> = strings().encode();
    let mut value: StringsMut = StringsMut::new(&mut bytes).unwrap();
    let entry: usize = value.as_ref().entry().as_bytes().as_ptr() as usize;
    value.set_third("3").unwrap();
    value.set_first("").unwrap();
    assert_eq!(value.as_ref().entry().as_bytes().as_ptr() as usize, entry);
    assert_eq!(value.as_ref().entry().text(), "text");
}

#[test]
fn mut_sets_optional_strings() {
    let mut bytes: Vec<u8> = strings().encode();
    let mut value: StringsMut = StringsMut::new(&mut bytes).unwrap();
    value.set_second(None).unwrap();
    assert_eq!(value.as_ref().second(), None);
    assert_eq!(value.as_ref().third(), "third");
    value.set_second("2nd").unwrap();
    assert_eq!(value.as_ref().second(), Some("2nd"));
    value.set_second("").unwrap();
    assert_eq!(value.as_ref().second(), Some(""));
}

#[test]
fn mut_failed_optional_strings_stay_absent() {
    let mut bytes: Vec<u8> = Opt::new(false, None, None, None, None).encode();
    let original: Vec<u8> = bytes.clone();
    let mut opt: OptMut = OptMut::new(&mut bytes).unwrap();
    assert!(opt.set_d("hi").is_err());
    assert_eq!(bytes, original);
}

#[test]
fn children_grow_into_their_padding() {
    let mut bytes: Vec<u8> = pair_bytes();
    let mut pair: PairMut = PairMut::new(&mut bytes).unwrap();
    assert_eq!(
        pair.first_mut().set_text("hello, world!"),
        Err(CapacityError {
            required: 13,
            capacity: 12,
        })
    );
    pair.first_mut().set_text("hello, world").unwrap();
    assert_eq!(pair.as_ref().first().text(), "hello, world");
    assert_eq!(pair.as_ref().second().text(), "hi");
    assert_eq!(pair.as_ref().note(), "ok");
    PairRef::new(pair.as_bytes()).unwrap();
}

#[test]
fn children_keep_their_extent_when_they_shrink() {
    let mut bytes: Vec<u8> = pair_bytes();
    let mut pair: PairMut = PairMut::new(&mut bytes).unwrap();
    pair.first_mut().set_text("").unwrap();
    assert_eq!(pair.as_ref().second().id(), 2);
    assert_eq!(bytes.len(), 56);
    assert_eq!(bytes[..4], [24, 0, 0, 0]);
    let decoded: Pair = Pair::decode(&bytes).unwrap();
    assert_eq!(decoded.first().text(), "");
}

#[test]
fn mut_validates() {
    let mut bytes: [u8; 7] = [0; 7];
    assert!(matches!(
        PointMut::new(&mut bytes),
        Err(DecodeError::TooShort { .. })
    ));
}

#[test]
fn bufs_edit_in_place() {
    let mut buf: LineBuf = LineBuf::from(&Line::new(Point::new(1, 2), Point::new(3, 4), 5));
    buf.as_mut().set_width(9);
    buf.as_mut().end_mut().set_y(8);
    assert_eq!(
        buf.decode(),
        Line::new(Point::new(1, 2), Point::new(3, 8), 9)
    );
}

/// Gets the sum of the coordinates of any form of a point.
fn point_sum<P: PointView>(point: &P) -> u32 {
    point.x() + point.y()
}

#[test]
fn views_read_every_form() {
    let point: Point = Point::new(1, 2);
    let mut bytes: Vec<u8> = point.encode();
    assert_eq!(point_sum(&point), 3);
    assert_eq!(point_sum(&&point), 3);
    assert_eq!(point_sum(&PointRef::new(&bytes).unwrap()), 3);
    assert_eq!(point_sum(&PointBuf::from(&point)), 3);
    assert_eq!(point_sum(&PointMut::new(&mut bytes).unwrap()), 3);
}

/// Gets the end of any form of a line, through its `Copy` nested struct.
fn line_end_y<L: LineView>(line: &L) -> u32 {
    line.end().y()
}

/// Gets the text of any form of a pair, through its non-`Copy` nested structs.
fn pair_text<P: PairView>(pair: &P) -> String {
    let first = pair.first();
    let second = pair.second();
    format!("{} {} {}", first.text(), second.text(), pair.note())
}

#[test]
fn views_read_nested_structs() {
    let line: Line = Line::new(Point::new(1, 2), Point::new(3, 4), 5);
    let mut bytes: Vec<u8> = line.encode();
    assert_eq!(line_end_y(&line), 4);
    assert_eq!(line_end_y(&LineRef::new(&bytes).unwrap()), 4);
    assert_eq!(line_end_y(&LineBuf::from(&line)), 4);
    assert_eq!(line_end_y(&LineMut::new(&mut bytes).unwrap()), 4);

    let mut bytes: Vec<u8> = pair_bytes();
    assert_eq!(pair_text(&pair()), "hello hi ok");
    assert_eq!(pair_text(&PairRef::new(&bytes).unwrap()), "hello hi ok");
    assert_eq!(pair_text(&PairBuf::from(&pair())), "hello hi ok");
    assert_eq!(pair_text(&PairMut::new(&mut bytes).unwrap()), "hello hi ok");
}

/// Gets the optional fields of any form of an `Opt`.
fn opt_fields<O: OptView>(opt: &O) -> (Option<u32>, Option<bool>, Option<String>, Option<Color>) {
    (opt.b(), opt.c(), opt.d().map(str::to_string), opt.e())
}

#[test]
fn views_read_optional_fields() {
    let value: Opt = Opt::new(
        true,
        Some(5),
        Some(false),
        Some("hi".to_string()),
        Some(Color::Red),
    );
    let expected = (
        Some(5),
        Some(false),
        Some("hi".to_string()),
        Some(Color::Red),
    );
    let bytes: Vec<u8> = value.encode();
    assert_eq!(opt_fields(&value), expected);
    assert_eq!(opt_fields(&OptRef::new(&bytes).unwrap()), expected);

    let absent: Opt = Opt::new(false, None, None, None, None);
    let bytes: Vec<u8> = absent.encode();
    assert_eq!(opt_fields(&absent), (None, None, None, None));
    assert_eq!(
        opt_fields(&OptRef::new(&bytes).unwrap()),
        (None, None, None, None)
    );
}

/// Gets the optional nested struct of any form of a `FixedKinds`.
fn maybe_point_y<K: FixedKindsView>(kinds: &K) -> Option<u32> {
    kinds.maybe_point().map(|point| point.y())
}

#[test]
fn views_read_optional_nested_structs() {
    let value: FixedKinds = FixedKinds::new(
        ID,
        Color::Red,
        Point::new(1, 2),
        Some(Point::new(3, 4)),
        None,
    );
    let bytes: Vec<u8> = value.encode();
    assert_eq!(maybe_point_y(&value), Some(4));
    assert_eq!(maybe_point_y(&FixedKindsRef::new(&bytes).unwrap()), Some(4));
}

/// Gets the sum of the values of any form of a linked list.
fn list_sum<N: ListNodeView>(node: &N) -> u32 {
    node.value() + node.next().map_or(0, |next| list_sum(&next))
}

#[test]
fn views_read_recursive_structs() {
    let tail: ListNode = ListNode::new(3, None);
    let list: ListNode = ListNode::new(1, Some(ListNode::new(2, Some(tail))));
    let mut bytes: Vec<u8> = list.encode();
    assert_eq!(list_sum(&list), 6);
    assert_eq!(list_sum(&ListNodeRef::new(&bytes).unwrap()), 6);
    assert_eq!(list_sum(&ListNodeBuf::from(&list)), 6);
    assert_eq!(list_sum(&ListNodeMut::new(&mut bytes).unwrap()), 6);
}
