mod common;

use common::ID;
use proto_packet::io::DecodeError;
use proto_packet::types::{Float32, Float64, Uuid};
use proto_packet_test::format::edges::{
    AbsentChild, FixedKinds, FixedKindsBuf, FixedKindsMut, FixedKindsRef, FixedKindsView,
    FixedKindsViewMut, FloatKinds, FloatKindsMut, FloatKindsRef, LoopByte, LoopLong, NineBits,
    NineBitsMut, NineBitsRef, SignedKinds, SignedKindsMut, SignedKindsRef,
};
use proto_packet_test::format::examples::{
    Color, Entry, EntryBuf, EntryRef, EntryView, EntryViewMut, Line, LineBuf, LineMut, LineRef,
    LineView, LineViewMut, Mixed, MixedMut, MixedRef, Opt, OptBuf, OptMut, OptRef, OptView,
    OptViewMut, Outer, OuterRef, Padded, PaddedRef, Pair, PairBuf, PairMut, PairRef, PairView,
    PairViewMut, Person, PersonRef, Point, PointBuf, PointMut, PointRef, PointView, PointViewMut,
};
use proto_packet_test::structs::boxed::{
    ListNode, ListNodeBuf, ListNodeMut, ListNodeRef, ListNodeView, ListNodeViewMut,
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
    proto_packet_test::format::edges::AbsentChildView,
    AbsentChild,
    {
        fn first(&self) -> Option<impl proto_packet_test::format::examples::EntryView>;
        fn second(&self) -> impl proto_packet_test::format::examples::EntryView;
        fn text(&self) -> &str;
    }
);

common::plain!(
    proto_packet_test::structs::empty::EmptyStructView,
    EmptyStruct,
    {}
);

common::plain!(
    proto_packet_test::format::edges::FixedKindsView,
    FixedKinds,
    {
        fn id(&self) -> proto_packet::types::Uuid;
        fn color(&self) -> proto_packet_test::format::examples::Color;
        fn point(&self) -> impl proto_packet_test::format::examples::PointView;
        fn maybe_point(&self) -> Option<impl proto_packet_test::format::examples::PointView>;
        fn maybe_color(&self) -> Option<proto_packet_test::format::examples::Color>;
    }
);

common::plain!(
    proto_packet_test::format::edges::FloatKindsView,
    FloatKinds,
    {
        fn a(&self) -> proto_packet::types::Float32;
        fn b(&self) -> proto_packet::types::Float64;
        fn c(&self) -> Option<proto_packet::types::Float32>;
    }
);

common::plain!(proto_packet_test::format::examples::LineView, Line, {
    fn start(&self) -> impl proto_packet_test::format::examples::PointView;
    fn end(&self) -> impl proto_packet_test::format::examples::PointView;
    fn width(&self) -> u8;
});

common::plain!(proto_packet_test::structs::boxed::ListNodeView, ListNode, {
    fn value(&self) -> u32;
    fn next(&self) -> Option<impl proto_packet_test::structs::boxed::ListNodeView>;
});

common::plain!(proto_packet_test::format::edges::LoopByteView, LoopByte, {
    fn value(&self) -> u8;
    fn next(&self) -> Option<impl proto_packet_test::format::edges::LoopLongView>;
});

common::plain!(proto_packet_test::format::edges::LoopLongView, LoopLong, {
    fn value(&self) -> u64;
    fn next(&self) -> Option<impl proto_packet_test::format::edges::LoopByteView>;
});

common::plain!(proto_packet_test::format::examples::MixedView, Mixed, {
    fn a(&self) -> u8;
    fn b(&self) -> u64;
    fn c(&self) -> u16;
    fn d(&self) -> bool;
});

common::plain!(proto_packet_test::format::edges::NineBitsView, NineBits, {
    fn a(&self) -> bool;
    fn b(&self) -> bool;
    fn c(&self) -> bool;
    fn d(&self) -> bool;
    fn e(&self) -> bool;
    fn f(&self) -> bool;
    fn g(&self) -> bool;
    fn h(&self) -> bool;
    fn i(&self) -> bool;
});

common::plain!(proto_packet_test::format::examples::OptView, Opt, {
    fn a(&self) -> bool;
    fn b(&self) -> Option<u32>;
    fn c(&self) -> Option<bool>;
    fn d(&self) -> Option<&str>;
    fn e(&self) -> Option<proto_packet_test::format::examples::Color>;
});

common::plain!(proto_packet_test::format::examples::OuterView, Outer, {
    fn m(&self) -> impl proto_packet_test::format::examples::MixedView;
    fn x(&self) -> u32;
    fn y(&self) -> u8;
});

common::plain!(proto_packet_test::format::examples::PaddedView, Padded, {
    fn a(&self) -> impl proto_packet_test::format::examples::LineView;
    fn x(&self) -> u32;
    fn b(&self) -> impl proto_packet_test::format::examples::LineView;
});

common::plain!(proto_packet_test::format::examples::PairView, Pair, {
    fn first(&self) -> impl proto_packet_test::format::examples::EntryView;
    fn second(&self) -> impl proto_packet_test::format::examples::EntryView;
    fn note(&self) -> &str;
});

common::plain!(proto_packet_test::format::examples::PersonView, Person, {
    fn name(&self) -> &str;
    fn age(&self) -> u8;
});

common::plain!(proto_packet_test::format::examples::PointView, Point, {
    fn x(&self) -> u32;
    fn y(&self) -> u32;
});

common::plain!(
    proto_packet_test::format::edges::SignedKindsView,
    SignedKinds,
    {
        fn a(&self) -> i8;
        fn b(&self) -> i16;
        fn c(&self) -> i32;
        fn d(&self) -> i64;
        fn e(&self) -> i128;
        fn f(&self) -> Option<i16>;
    }
);

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
    let bytes: [u8; 12] = [2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 1, 1];
    assert_encoding!(Mixed, Mixed::new(1, 2, 3, true), &bytes);

    let mixed: MixedRef = MixedRef::new(&bytes).unwrap();
    assert_eq!(
        (mixed.a(), mixed.b(), mixed.c(), mixed.d()),
        (1, 2, 3, true)
    );
}

#[test]
fn line_example() {
    let bytes: [u8; 17] = [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5];
    let line: Line = Line::new(Point::new(1, 2), Point::new(3, 4), 5);
    assert_encoding!(Line, line, &bytes);

    let line_ref: LineRef = LineRef::new(&bytes).unwrap();
    assert_eq!(line_ref.start().y(), 2);
    assert_eq!(line_ref.end().x(), 3);
    assert_eq!(line_ref.width(), 5);
    assert_eq!(line_ref.decode(), line);
}

#[test]
fn outer_example() {
    let bytes: Vec<u8> = [
        &[2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 1, 1][..], // m
        &[4, 0, 0, 0],                             // x
        &[5],                                      // y
    ]
    .concat();
    let outer: Outer = Outer::new(Mixed::new(1, 2, 3, true), 4, 5);
    assert_eq!(bytes.len(), 17);
    assert_encoding!(Outer, outer, &bytes);

    let outer_ref: OuterRef = OuterRef::new(&bytes).unwrap();
    assert_eq!(outer_ref.m().b(), 2);
    assert_eq!((outer_ref.x(), outer_ref.y()), (4, 5));
}

#[test]
fn padded_example() {
    let line = |base: u32| Line::new(Point::new(base, base + 1), Point::new(base + 2, 9), 7);
    let bytes: Vec<u8> = [
        &[8, 0, 0, 0][..],                                    // x
        &[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 9, 0, 0, 0, 7], // a
        &[0; 3],                                              // padding
        &[4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 9, 0, 0, 0, 7], // b
    ]
    .concat();
    let padded: Padded = Padded::new(line(1), 8, line(4));
    assert_eq!(bytes.len(), 41);
    assert_encoding!(Padded, padded, &bytes);

    let padded_ref: PaddedRef = PaddedRef::new(&bytes).unwrap();
    assert_eq!(padded_ref.a().width(), 7);
    assert_eq!(padded_ref.x(), 8);
    assert_eq!(padded_ref.b().start().x(), 4);
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
        &[0],                      // bits
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
fn signed_integers_are_twos_complement() {
    let bytes: Vec<u8> = [
        &[0xFC, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF][..], // d
        &[0xFB],                                               // e
        &[0xFF; 15],
        &[0xFD, 0xFF, 0xFF, 0xFF], // c
        &[0xFE, 0xFF],             // b
        &[0, 0],                   // f, absent
        &[0xFF],                   // a
        &[0],                      // bits
    ]
    .concat();
    assert_eq!(bytes.len(), 34);
    assert_encoding!(
        SignedKinds,
        SignedKinds::new(-1, -2, -3, -4, -5, None),
        &bytes
    );

    let kinds: SignedKindsRef = SignedKindsRef::new(&bytes).unwrap();
    assert_eq!(
        (kinds.a(), kinds.b(), kinds.c(), kinds.d(), kinds.e()),
        (-1, -2, -3, -4, -5)
    );
    assert_eq!(kinds.f(), None);
}

#[test]
fn signed_integers_keep_their_range() {
    for value in [
        SignedKinds::new(
            i8::MIN,
            i16::MIN,
            i32::MIN,
            i64::MIN,
            i128::MIN,
            Some(i16::MIN),
        ),
        SignedKinds::new(
            i8::MAX,
            i16::MAX,
            i32::MAX,
            i64::MAX,
            i128::MAX,
            Some(i16::MAX),
        ),
        SignedKinds::new(0, 0, 0, 0, 0, Some(0)),
    ] {
        let bytes: Vec<u8> = value.encode();
        assert_encoding!(SignedKinds, value, &bytes);
    }
}

#[test]
fn mut_sets_signed_integers_in_place() {
    let mut bytes: Vec<u8> = SignedKinds::new(1, 2, 3, 4, 5, None).encode();
    let mut kinds: SignedKindsMut = SignedKindsMut::new(&mut bytes).unwrap();
    kinds.set_a(-1);
    kinds.set_d(i64::MIN);
    kinds.set_e(-5);
    kinds.set_f(-6);
    assert_eq!(
        SignedKinds::decode(&bytes),
        Ok(SignedKinds::new(-1, 2, 3, i64::MIN, -5, Some(-6)))
    );
}

/// Gets the bytes of the struct with a float of each size, with the `a` float.
fn float_kinds_bytes(a: [u8; 4]) -> Vec<u8> {
    [
        &[0, 0, 0, 0, 0, 0, 0, 0xC0][..], // b
        &a,                               // a
        &[0, 0, 0, 0],                    // c, absent
        &[0],                             // bits
    ]
    .concat()
}

#[test]
fn floats_are_their_bits() {
    let bytes: Vec<u8> = float_kinds_bytes([0, 0, 0xC0, 0x3F]);
    assert_eq!(bytes.len(), 17);
    let value: FloatKinds = FloatKinds::new(Float32::new(1.5), Float64::new(-2.0), None);
    assert_encoding!(FloatKinds, value, &bytes);

    let kinds: FloatKindsRef = FloatKindsRef::new(&bytes).unwrap();
    assert_eq!((kinds.a().get(), kinds.b().get()), (1.5, -2.0));
    assert_eq!(kinds.c(), None);
}

#[test]
fn nans_are_the_one_nan() {
    let bytes: Vec<u8> = float_kinds_bytes([0, 0, 0xC0, 0x7F]);
    let value: FloatKinds = FloatKinds::new(Float32::new(-f32::NAN), Float64::new(-2.0), None);
    assert_encoding!(FloatKinds, value, &bytes);

    for other_nan in [[0, 0, 0xC0, 0xFF], [1, 0, 0xC0, 0x7F], [1, 0, 0x80, 0x7F]] {
        assert_eq!(
            FloatKinds::decode(&float_kinds_bytes(other_nan)),
            Err(DecodeError::InvalidFloat)
        );
    }
}

#[test]
fn mut_sets_floats_in_place() {
    let mut bytes: Vec<u8> = float_kinds_bytes([0, 0, 0xC0, 0x3F]);
    let mut kinds: FloatKindsMut = FloatKindsMut::new(&mut bytes).unwrap();
    kinds.set_a(Float32::new(f32::NAN));
    kinds.set_b(Float64::new(0.5));
    kinds.set_c(Float32::new(-0.0));
    assert_eq!(
        FloatKinds::decode(&bytes),
        Ok(FloatKinds::new(
            Float32::NAN,
            Float64::new(0.5),
            Some(Float32::new(-0.0))
        ))
    );
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
fn rejects_extra_bytes() {
    let bytes: [u8; 10] = [1, 0, 0, 0, 2, 0, 0, 0, 0xFF, 0xFF];
    assert_eq!(
        Point::decode(&bytes),
        Err(DecodeError::TooLong {
            expected: 8,
            actual: 10,
        })
    );
}

#[test]
fn rejects_padding_that_is_not_zeros() {
    let line: Line = Line::new(Point::new(1, 2), Point::new(3, 4), 5);
    let mut in_the_header: Vec<u8> = Padded::new(line, 8, line).encode();
    in_the_header[21] = 0xFF;
    assert_eq!(
        Padded::decode(&in_the_header),
        Err(DecodeError::InvalidPadding)
    );

    let mut in_the_tail: Vec<u8> = pair_bytes();
    in_the_tail[21] = 0xFF;
    assert_eq!(Pair::decode(&in_the_tail), Err(DecodeError::InvalidPadding));
}

#[test]
fn rejects_unused_bits_that_are_not_zeros() {
    assert_eq!(
        NineBits::decode(&[0b1000_0101, 0b0000_0011]),
        Err(DecodeError::InvalidPadding)
    );
    let mut bytes: Vec<u8> = Mixed::new(1, 2, 3, true).encode();
    bytes[11] = 0b0000_0011;
    assert_eq!(Mixed::decode(&bytes), Err(DecodeError::InvalidPadding));
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
    let bytes: [u8; 4] = [36, b'A', b'd', b'a'];
    assert_encoding!(Person, Person::new("Ada".to_string(), 36), &bytes);

    let person: PersonRef = PersonRef::new(&bytes).unwrap();
    assert_eq!((person.name(), person.age()), ("Ada", 36));
}

/// Gets the bytes of the `Pair` example of `format/binary.md`.
fn pair_bytes() -> Vec<u8> {
    [
        &[21, 0, 0, 0, 34, 0, 0, 0][..], // ends
        &[1, 0, 0, 0, 0, 0, 0, 0],       // first
        b"hello",
        &[0; 3],                   // padding
        &[2, 0, 0, 0, 0, 0, 0, 0], // second
        b"hi",
        b"ok",
    ]
    .concat()
}

/// Gets the value of the `Pair` example of `format/binary.md`.
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
    assert_eq!(bytes.len(), 36);
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
    let bytes: [u8; 9] = [0, 0, 0, 0, 0, 0, 0b0001_0101, b'h', b'i'];
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
        &[9, 0, 0, 0, 26, 0, 0, 0][..], // ends
        &[0],                           // bits
        &[0; 7],                        // padding
        &[1, 0, 0, 0, 0, 0, 0, 0],      // second
        b"hi",
        b"ok",
    ]
    .concat();
    let value: AbsentChild = AbsentChild::new(None, second.clone(), "ok".to_string());
    assert_encoding!(AbsentChild, value, &bytes);

    let first: Entry = Entry::new(2, "hello".to_string());
    let present: AbsentChild = AbsentChild::new(Some(first), second, "ok".to_string());
    let bytes: Vec<u8> = present.encode();
    assert_eq!(bytes.len(), 44);
    assert_eq!(bytes[..8], [29, 0, 0, 0, 42, 0, 0, 0]);
    assert_encoding!(AbsentChild, present, &bytes);
}

#[test]
fn loops_share_their_alignment() {
    let bytes: Vec<u8> = [
        &[1, 1, 0, 0, 0, 0, 0, 0][..],    // LoopByte, with its padding
        &[2, 0, 0, 0, 0, 0, 0, 0, 0][..], // LoopLong
    ]
    .concat();
    let value: LoopByte = LoopByte::new(1, Some(LoopLong::new(2, None)));
    assert_encoding!(LoopByte, value, &bytes);

    let long: LoopLong = LoopLong::new(1, Some(LoopByte::new(2, None)));
    let bytes: Vec<u8> = long.encode();
    assert_eq!(bytes.len(), 16 + 2);
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
fn the_last_entry_ends_with_the_struct() {
    let bytes: [u8; 5] = [36, b'A', b'd', b'a', b'!'];
    let person: Person = Person::decode(&bytes).unwrap();
    assert_eq!(person, Person::new("Ada!".to_string(), 36));
}

#[test]
fn rejects_end_tables() {
    let mut before_the_header_ends: Vec<u8> = pair_bytes();
    before_the_header_ends[0] = 7;
    assert_eq!(
        Pair::decode(&before_the_header_ends),
        Err(DecodeError::InvalidEndTable)
    );

    let mut decreasing: Vec<u8> = pair_bytes();
    decreasing[4] = 20;
    assert_eq!(Pair::decode(&decreasing), Err(DecodeError::InvalidEndTable));

    let mut past_the_end: Vec<u8> = pair_bytes();
    past_the_end[4] = 37;
    assert_eq!(
        Pair::decode(&past_the_end),
        Err(DecodeError::InvalidEndTable)
    );
}

#[test]
fn rejects_entries_that_start_after_they_end() {
    let mut bytes: Vec<u8> = pair_bytes();
    bytes[4] = 22; // `second` starts at 24, after its padding
    assert_eq!(Pair::decode(&bytes), Err(DecodeError::InvalidEndTable));
}

#[test]
fn rejects_short_headers() {
    assert_eq!(
        Person::decode(&[]),
        Err(DecodeError::TooShort {
            expected: 1,
            actual: 0,
        })
    );
}

#[test]
fn rejects_invalid_utf8() {
    assert_eq!(Person::decode(&[36, 0xFF]), Err(DecodeError::InvalidUtf8));
}

#[test]
fn rejects_invalid_children() {
    let mut bytes: Vec<u8> = pair_bytes();
    bytes[0] = 12; // the end of `first`, which leaves it 4 bytes
    assert_eq!(
        Pair::decode(&bytes),
        Err(DecodeError::TooShort {
            expected: 8,
            actual: 4,
        })
    );

    let mut bytes: Vec<u8> = pair_bytes();
    bytes[8 + 8] = 0xFF; // the text of `first`
    assert_eq!(Pair::decode(&bytes), Err(DecodeError::InvalidUtf8));
}

#[test]
fn rejects_absent_fields_that_are_not_empty() {
    let absent: Vec<u8> = Opt::new(true, None, None, None, None).encode();
    assert_eq!(absent, [0, 0, 0, 0, 0, 0, 0b0000_0001]);

    let mut number: Vec<u8> = absent.clone();
    number[0] = 5;
    assert_eq!(Opt::decode(&number), Err(DecodeError::InvalidAbsentField));

    let mut bit: Vec<u8> = absent.clone();
    bit[6] = 0b0000_1001;
    assert_eq!(Opt::decode(&bit), Err(DecodeError::InvalidAbsentField));

    let mut string: Vec<u8> = absent.clone();
    string.extend_from_slice(b"hi");
    assert_eq!(Opt::decode(&string), Err(DecodeError::InvalidAbsentField));

    let mut point: Vec<u8> = FixedKinds::new(ID, Color::Red, Point::new(1, 2), None, None).encode();
    point[24] = 3;
    assert_eq!(
        FixedKinds::decode(&point),
        Err(DecodeError::InvalidAbsentField)
    );
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
fn mut_edits_nested_variable_structs() {
    let mut bytes: Vec<u8> = pair_bytes();
    let mut pair: PairMut = PairMut::new(&mut bytes).unwrap();
    pair.first_mut().set_id(7);
    pair.second_mut().set_id(8);
    assert_eq!(pair.as_ref().first().id(), 7);
    assert_eq!(pair.as_ref().first().text(), "hello");
    assert_eq!(pair.as_ref().second().id(), 8);
    assert_eq!(pair.as_ref().note(), "ok");
    PairRef::new(pair.as_bytes()).unwrap();
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

/// Adds 10 to the `x` of any editable form of a point.
fn shift<P: PointViewMut>(point: &mut P) {
    point.set_x(point.x() + 10);
}

#[test]
fn view_muts_edit_every_form() {
    let mut point: Point = Point::new(1, 2);
    shift(&mut point);
    assert_eq!(point, Point::new(11, 2));
    shift(&mut &mut point);
    assert_eq!(point, Point::new(21, 2));

    let mut buf: PointBuf = PointBuf::from(&Point::new(1, 2));
    shift(&mut buf);
    assert_eq!(buf.decode(), Point::new(11, 2));

    let mut bytes: Vec<u8> = Point::new(1, 2).encode();
    shift(&mut PointMut::new(&mut bytes).unwrap());
    assert_eq!(Point::decode(&bytes), Ok(Point::new(11, 2)));
}

/// Edits any editable form of a line, through its `Copy` nested struct.
fn raise_line<L: LineViewMut>(line: &mut L) {
    line.start_mut().set_y(9);
    line.set_width(1);
}

/// Edits any editable form of a pair, through its non-`Copy` nested structs.
fn retag<P: PairViewMut>(pair: &mut P) {
    pair.first_mut().set_id(7);
    pair.second_mut().set_id(8);
}

#[test]
fn view_muts_edit_nested_structs() {
    let line: Line = Line::new(Point::new(1, 2), Point::new(3, 4), 5);
    let expected: Line = Line::new(Point::new(1, 9), Point::new(3, 4), 1);

    let mut value: Line = line;
    raise_line(&mut value);
    assert_eq!(value, expected);

    let mut buf: LineBuf = LineBuf::from(&line);
    raise_line(&mut buf);
    assert_eq!(buf.decode(), expected);

    let mut bytes: Vec<u8> = line.encode();
    raise_line(&mut LineMut::new(&mut bytes).unwrap());
    assert_eq!(Line::decode(&bytes), Ok(expected));

    let expected: Pair = Pair::new(
        Entry::new(7, "hello".to_string()),
        Entry::new(8, "hi".to_string()),
        "ok".to_string(),
    );

    let mut value: Pair = pair();
    retag(&mut value);
    assert_eq!(value, expected);

    let mut buf: PairBuf = PairBuf::from(&pair());
    retag(&mut buf);
    assert_eq!(buf.decode(), expected);

    let mut bytes: Vec<u8> = pair_bytes();
    retag(&mut PairMut::new(&mut bytes).unwrap());
    assert_eq!(Pair::decode(&bytes), Ok(expected));
}

/// Sets the optional fixed-size fields of any editable form of an `Opt`.
fn fill_opt<O: OptViewMut>(opt: &mut O) {
    opt.set_b(5);
    opt.set_c(true);
    opt.set_e(Color::Red);
}

/// Clears the optional fixed-size fields of any editable form of an `Opt`.
fn clear_opt<O: OptViewMut>(opt: &mut O) {
    opt.set_b(None);
    opt.set_c(None);
    opt.set_e(None);
}

#[test]
fn view_muts_set_optional_fields() {
    let text: Option<String> = Some("hi".to_string());
    let filled: Opt = Opt::new(false, Some(5), Some(true), text.clone(), Some(Color::Red));
    let absent: Opt = Opt::new(false, None, None, text, None);

    let mut value: Opt = absent.clone();
    fill_opt(&mut value);
    assert_eq!(value, filled);
    clear_opt(&mut value);
    assert_eq!(value, absent);

    let mut bytes: Vec<u8> = absent.encode();
    let mut edit: OptMut = OptMut::new(&mut bytes).unwrap();
    fill_opt(&mut edit);
    assert_eq!(Opt::decode(edit.as_bytes()), Ok(filled.clone()));
    clear_opt(&mut edit);
    assert_eq!(Opt::decode(edit.as_bytes()), Ok(absent.clone()));

    let mut buf: OptBuf = OptBuf::from(&absent);
    fill_opt(&mut buf);
    assert_eq!(buf.decode(), filled);
    clear_opt(&mut buf);
    assert_eq!(buf.decode(), absent);
}

/// Edits the optional nested struct of any editable form of a `FixedKinds`, if it's present.
fn nudge<K: FixedKindsViewMut>(kinds: &mut K) -> bool {
    match kinds.maybe_point_mut() {
        Some(mut point) => {
            point.set_x(7);
            true
        }
        None => false,
    }
}

#[test]
fn view_muts_reach_present_optional_structs() {
    let some: FixedKinds = FixedKinds::new(
        ID,
        Color::Red,
        Point::new(1, 2),
        Some(Point::new(3, 4)),
        None,
    );
    let none: FixedKinds = FixedKinds::new(ID, Color::Red, Point::new(1, 2), None, None);

    let mut value: FixedKinds = some.clone();
    assert!(nudge(&mut value));
    assert_eq!(value.maybe_point(), Some(Point::new(7, 4)));
    assert!(!nudge(&mut none.clone()));

    let mut buf: FixedKindsBuf = FixedKindsBuf::from(&some);
    assert!(nudge(&mut buf));
    assert_eq!(buf.as_ref().maybe_point().unwrap().x(), 7);

    let mut bytes: Vec<u8> = none.encode();
    assert!(!nudge(&mut FixedKindsMut::new(&mut bytes).unwrap()));
}

/// Adds 1 to each value of any editable form of a linked list.
fn increment<N: ListNodeViewMut>(node: &mut N) {
    node.set_value(node.value() + 1);
    if let Some(mut next) = node.next_mut() {
        increment(&mut next);
    }
}

#[test]
fn view_muts_edit_recursive_structs() {
    let list = |base: u32| {
        let tail: ListNode = ListNode::new(base + 2, None);
        ListNode::new(base, Some(ListNode::new(base + 1, Some(tail))))
    };

    let mut value: ListNode = list(1);
    increment(&mut value);
    assert_eq!(value, list(2));

    let mut buf: ListNodeBuf = ListNodeBuf::from(&list(1));
    increment(&mut buf);
    assert_eq!(buf.decode(), list(2));

    let mut bytes: Vec<u8> = list(1).encode();
    increment(&mut ListNodeMut::new(&mut bytes).unwrap());
    assert_eq!(ListNode::decode(&bytes), Ok(list(2)));
}

#[test]
#[should_panic(expected = "the changed bytes decode")]
fn the_encoding_check_finds_a_second_encoding() {
    // a reader that takes any last byte, like padding it doesn't check
    common::assert_one_encoding(&[7, 0], |bytes| match bytes {
        [value, _] => Some(vec![*value, 0]),
        _ => None,
    });
}

#[test]
fn encoded_forms_are_equal_by_their_bytes() {
    use std::collections::HashSet;

    let entry: Entry = Entry::new(7, "hi".to_string());
    let other: Entry = Entry::new(7, "ho".to_string());
    let (bytes, other_bytes): (Vec<u8>, Vec<u8>) = (entry.encode(), other.encode());
    let entry_ref: EntryRef = EntryRef::new(&bytes).unwrap();
    let other_ref: EntryRef = EntryRef::new(&other_bytes).unwrap();
    let (buf, other_buf): (EntryBuf, EntryBuf) = (EntryBuf::from(&entry), EntryBuf::from(&other));

    assert_eq!(entry_ref, EntryRef::new(&bytes.clone()).unwrap());
    assert_ne!(entry_ref, other_ref);
    assert_eq!(buf, EntryBuf::from(&entry));
    assert_ne!(buf, other_buf);

    let refs: HashSet<EntryRef> = [entry_ref, other_ref, entry_ref].into_iter().collect();
    assert_eq!(refs.len(), 2);
    let bufs: HashSet<EntryBuf> = [buf.clone(), other_buf.clone(), buf.clone()]
        .into_iter()
        .collect();
    assert_eq!(bufs.len(), 2);
}

#[test]
fn forms_are_equal_across_forms() {
    let entry: Entry = Entry::new(7, "hi".to_string());
    let other: Entry = Entry::new(7, "hi!".to_string());
    let (bytes, other_bytes): (Vec<u8>, Vec<u8>) = (entry.encode(), other.encode());
    let entry_ref: EntryRef = EntryRef::new(&bytes).unwrap();
    let other_ref: EntryRef = EntryRef::new(&other_bytes).unwrap();
    let (buf, other_buf): (EntryBuf, EntryBuf) = (EntryBuf::from(&entry), EntryBuf::from(&other));

    assert!(entry == entry_ref && entry_ref == entry);
    assert!(entry == buf && buf == entry);
    assert!(entry_ref == buf && buf == entry_ref);
    assert!(entry != other_ref && other_ref != entry);
    assert!(entry != other_buf && other_buf != entry);
    assert!(entry_ref != other_buf && other_buf != entry_ref);

    let point: Point = Point::new(1, 2);
    let bytes: Vec<u8> = point.encode();
    let buf: PointBuf = PointBuf::from(&point);
    assert!(point == PointRef::new(&bytes).unwrap() && point == buf);
    assert!(Point::new(1, 3) != PointRef::new(&bytes).unwrap());
}

/// Gets the bytes of the `count` of list nodes, each the next node of the one before it.
fn nodes_bytes(count: usize) -> Vec<u8> {
    let mut bytes: Vec<u8> = [7, 0, 0, 0, 1, 0, 0, 0].repeat(count - 1);
    bytes.extend_from_slice(&[7, 0, 0, 0, 0]);
    bytes
}

#[test]
fn values_nest_at_most_64_levels() {
    let nodes: ListNode = ListNode::new(7, Some(ListNode::new(7, Some(ListNode::new(7, None)))));
    assert_eq!(nodes.encode(), nodes_bytes(3));

    assert!(ListNode::decode(&nodes_bytes(64)).is_ok());
    assert_eq!(
        ListNode::decode(&nodes_bytes(65)),
        Err(DecodeError::TooDeep)
    );
    assert_eq!(
        ListNodeRef::new(&nodes_bytes(65)),
        Err(DecodeError::TooDeep)
    );
    assert_eq!(
        ListNodeBuf::new(nodes_bytes(65)).map(|_| ()),
        Err(DecodeError::TooDeep)
    );
    assert_eq!(
        ListNodeMut::new(&mut nodes_bytes(65)).map(|_| ()),
        Err(DecodeError::TooDeep)
    );

    assert!(ListNodeRef::new_with_depth(&nodes_bytes(3), 3).is_ok());
    assert_eq!(
        ListNodeRef::new_with_depth(&nodes_bytes(3), 2),
        Err(DecodeError::TooDeep)
    );
}

#[test]
fn fixed_size_structs_are_not_levels() {
    use proto_packet::io::Element;

    // a point has no variable tail, so it needs no level
    let point: Vec<u8> = Point::new(1, 2).encode();
    assert_eq!(<Point as Element>::check(&point, 0), Ok(()));

    // an entry has a variable tail, so it is a level
    let entry: Vec<u8> = Entry::new(1, "a".to_string()).encode();
    assert_eq!(<Entry as Element>::check(&entry, 1), Ok(()));
    assert_eq!(
        <Entry as Element>::check(&entry, 0),
        Err(DecodeError::TooDeep)
    );
}

#[test]
fn deep_bytes_do_not_overflow_the_stack() {
    assert_eq!(
        ListNodeRef::new(&nodes_bytes(200_000)),
        Err(DecodeError::TooDeep)
    );
}

#[test]
fn the_deepest_value_fits_a_small_stack() {
    let bytes: Vec<u8> = nodes_bytes(64);
    let thread = std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(move || ListNode::decode(&bytes).map(|nodes| nodes.encode().len()));
    assert_eq!(thread.unwrap().join().unwrap(), Ok(64 * 8 - 3));
}

/// Gets the `count` of list nodes, each the next node of the one before it.
fn nodes(count: usize) -> ListNode {
    (1..count).fold(ListNode::new(7, None), |next, _| {
        ListNode::new(7, Some(next))
    })
}

#[test]
fn writers_take_the_deepest_value() {
    assert_eq!(nodes(64).encode(), nodes_bytes(64));
    assert_eq!(ListNodeBuf::from(&nodes(64)).as_bytes(), nodes_bytes(64));
}

#[test]
#[should_panic(expected = "the value nests deeper than 64 levels")]
fn writers_refuse_structs_that_nest_too_deep() {
    let _ = nodes(65).encode();
}
