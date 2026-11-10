mod common;

use common::assert_json;
use proto_packet::io::{DecodeError, ListView, MapView};
use proto_packet::types::Float32;
use proto_packet_test::format::arrays::{
    ArrayCases, ArrayCasesRef, ArrayKinds, ArrayKindsRef, Mesh, MeshBuf, MeshMut, MeshRef,
    MeshView, OptionKinds, OptionKindsRef, OptionalNumbers, OptionalScores, OptionalScoresRef,
    OptionalScoresView, OptionalTexts, OptionalTextsRef, Shade, Slots, SlotsBuf, SlotsMut,
    SlotsRef, SlotsView, WithArrays, WithArraysRef,
};
use proto_packet_test::format::examples::{Entry, Point};
use std::collections::BTreeMap;

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
    proto_packet_test::format::arrays::ArrayCasesView,
    ArrayCases,
    {
        fn tag_number(&self) -> u16;
        fn as_origin(
            &self,
        ) -> Option<impl proto_packet::io::ListView<Item = proto_packet::types::Float32>>;
        fn as_numbers(&self) -> Option<impl proto_packet::io::ListView<Item = Option<u32>>>;
        fn as_scores(
            &self,
        ) -> Option<impl proto_packet::io::MapView<String, Key = &str, Value = Option<u32>>>;
        fn as_number(&self) -> Option<u32>;
    }
);

common::plain!(
    proto_packet_test::format::arrays::ArrayKindsView,
    ArrayKinds,
    {
        fn bytes(&self) -> &[u8; 4];
        fn flags(&self) -> impl proto_packet::io::ListView<Item = bool>;
        fn longs(&self) -> impl proto_packet::io::ListView<Item = u64>;
        fn colors(
            &self,
        ) -> impl proto_packet::io::ListView<Item = proto_packet_test::format::arrays::Shade>;
        fn points(
            &self,
        ) -> impl proto_packet::io::ListView<Item = impl proto_packet_test::format::examples::PointView>;
        fn grid(&self) -> impl proto_packet::io::ListView<Item = &[u8; 3]>;
        fn maybe(&self) -> Option<impl proto_packet::io::ListView<Item = u16>>;
    }
);

common::plain!(proto_packet_test::format::arrays::MeshView, Mesh, {
    fn id(&self) -> u32;
    fn origin(&self) -> impl proto_packet::io::ListView<Item = proto_packet::types::Float32>;
    fn tag(&self) -> &[u8; 4];
});

common::plain!(
    proto_packet_test::format::arrays::OptionKindsView,
    OptionKinds,
    {
        fn colors(
            &self,
        ) -> impl proto_packet::io::ListView<Item = Option<proto_packet_test::format::arrays::Shade>>;
        fn points(
            &self,
        ) -> impl proto_packet::io::ListView<
            Item = Option<impl proto_packet_test::format::examples::PointView>,
        >;
        fn shades(
            &self,
        ) -> impl proto_packet::io::ListView<Item = Option<proto_packet_test::format::arrays::Shade>>;
        fn entries(
            &self,
        ) -> impl proto_packet::io::ListView<
            Item = Option<impl proto_packet_test::format::examples::EntryView>,
        >;
        fn flags(&self) -> impl proto_packet::io::ListView<Item = Option<bool>>;
        fn names(&self) -> impl proto_packet::io::MapView<u8, Key = u8, Value = Option<&str>>;
        fn by_name(
            &self,
        ) -> impl proto_packet::io::MapView<
            String,
            Key = &str,
            Value = Option<proto_packet_test::format::arrays::Shade>,
        >;
    }
);

common::plain!(
    proto_packet_test::format::arrays::OptionalNumbersView,
    OptionalNumbers,
    {
        fn numbers(&self) -> impl proto_packet::io::ListView<Item = Option<u32>>;
    }
);

common::plain!(
    proto_packet_test::format::arrays::OptionalScoresView,
    OptionalScores,
    {
        fn scores(&self)
        -> impl proto_packet::io::MapView<String, Key = &str, Value = Option<u32>>;
    }
);

common::plain!(
    proto_packet_test::format::arrays::OptionalTextsView,
    OptionalTexts,
    {
        fn texts(&self) -> impl proto_packet::io::ListView<Item = Option<&str>>;
    }
);

common::plain!(proto_packet_test::format::arrays::SlotsView, Slots, {
    fn slots(&self) -> impl proto_packet::io::ListView<Item = Option<u32>>;
});

common::plain!(
    proto_packet_test::format::arrays::WithArraysView,
    WithArrays,
    {
        fn origin(
            &self,
        ) -> Option<impl proto_packet::io::ListView<Item = proto_packet::types::Float32>>;
        fn slots(&self) -> Option<impl proto_packet::io::ListView<Item = Option<u32>>>;
        fn rows(
            &self,
        ) -> Option<
            impl proto_packet::io::ListView<Item = impl proto_packet::io::ListView<Item = u16>>,
        >;
        fn entries(
            &self,
        ) -> Option<
            impl proto_packet::io::ListView<
                Item = Option<impl proto_packet_test::format::examples::EntryView>,
            >,
        >;
        fn scores(
            &self,
        ) -> Option<impl proto_packet::io::MapView<String, Key = &str, Value = Option<u32>>>;
        fn by_id(
            &self,
        ) -> Option<impl proto_packet::io::MapView<[u8; 2], Key = &[u8; 2], Value = u8>>;
    }
);

/// Gets the `Mesh` example of `format/binary.md`.
fn mesh() -> Mesh {
    Mesh::new(
        1,
        [Float32::new(1.0), Float32::new(2.0), Float32::new(3.0)],
        [9, 8, 7, 6],
    )
}

/// Gets the bytes of the `Mesh` example.
fn mesh_bytes() -> Vec<u8> {
    [
        &[1, 0, 0, 0][..],   // id
        &[0, 0, 0x80, 0x3F], // origin[0] = 1.0
        &[0, 0, 0, 0x40],    // origin[1] = 2.0
        &[0, 0, 0x40, 0x40], // origin[2] = 3.0
        &[9, 8, 7, 6],       // tag
    ]
    .concat()
}

/// Gets the bytes of the `Slots` example, with the numbers 1 & 3 around an empty slot.
fn slots_bytes() -> Vec<u8> {
    [
        &[1, 0, 0, 0][..], // slots[0] = 1
        &[0, 0, 0, 0],     // slots[1] is absent
        &[3, 0, 0, 0],     // slots[2] = 3
        &[0b101],          // the presence bits
    ]
    .concat()
}

#[test]
fn mesh_example() {
    assert_encoding!(Mesh, mesh(), &mesh_bytes());
    let bytes: Vec<u8> = mesh_bytes();
    let mesh: MeshRef = MeshRef::new(&bytes).unwrap();
    assert_eq!(mesh.id(), 1);
    assert_eq!(mesh.origin().len(), 3);
    assert_eq!(mesh.origin().get(1), Some(Float32::new(2.0)));
    assert_eq!(mesh.origin().get(3), None);
    let tag: &[u8; 4] = mesh.tag();
    assert_eq!(tag, &[9, 8, 7, 6]);
}

#[test]
fn slots_example() {
    assert_encoding!(Slots, Slots::new([Some(1), None, Some(3)]), &slots_bytes());
    assert_encoding!(Slots, Slots::new([None, None, None]), &[0; 13]);
    let bytes: Vec<u8> = slots_bytes();
    let slots: SlotsRef = SlotsRef::new(&bytes).unwrap();
    assert_eq!(slots.slots().len(), 3);
    assert_eq!(slots.slots().get(0), Some(Some(1)));
    assert_eq!(slots.slots().get(1), Some(None));
    assert_eq!(slots.slots().get(3), None);
    assert_eq!(
        slots.slots().iter().collect::<Vec<Option<u32>>>(),
        [Some(1), None, Some(3)]
    );
}

#[test]
fn lists_of_optional_fixed_size_elements_have_a_count() {
    assert_encoding!(
        OptionalNumbers,
        OptionalNumbers::new(vec![Some(1), None, Some(3)]),
        &[
            &[3, 0, 0, 0][..], // the count
            &[1, 0, 0, 0],     // numbers[0] = 1
            &[0, 0, 0, 0],     // numbers[1] is absent
            &[3, 0, 0, 0],     // numbers[2] = 3
            &[0b101],          // the presence bits
        ]
        .concat()
    );
    assert_encoding!(OptionalNumbers, OptionalNumbers::new(Vec::default()), &[]);
    assert_encoding!(
        OptionalNumbers,
        OptionalNumbers::new(vec![None; 9]),
        &[&[9, 0, 0, 0][..], &[0; 36], &[0, 0]].concat()
    );
}

#[test]
fn lists_of_optional_variable_size_elements_keep_their_count() {
    assert_encoding!(
        OptionalTexts,
        OptionalTexts::new(vec![Some("hi".to_string()), None, Some(String::default())]),
        &[
            &[3, 0, 0, 0][..],           // the count
            &[14, 0, 0, 0, 14, 0, 0, 0], // the ends of the first two
            b"hi",                       // texts[0], then two with no bytes
            &[0b101],                    // the presence bits
        ]
        .concat()
    );
    let bytes: Vec<u8> = OptionalTexts::new(vec![None, Some("a".to_string())]).encode();
    let texts: OptionalTextsRef = OptionalTextsRef::new(&bytes).unwrap();
    assert_eq!(texts.texts().get(0), Some(None));
    assert_eq!(texts.texts().get(1), Some(Some("a")));
}

#[test]
fn maps_with_optional_values_have_their_bits_last() {
    let scores: BTreeMap<String, Option<u32>> =
        BTreeMap::from([("a".to_string(), Some(1)), ("bc".to_string(), None)]);
    let bytes: Vec<u8> = [
        &[15, 0, 0, 0][..],        // the end of the keys
        &[2, 0, 0, 0, 9, 0, 0, 0], // the keys: the count & the end of the first
        b"abc",                    // "a" & "bc"
        &[0],                      // padding
        &[1, 0, 0, 0],             // the value of "a"
        &[0, 0, 0, 0],             // the value of "bc" is absent
        &[0b01],                   // the presence bits
    ]
    .concat();
    assert_encoding!(OptionalScores, OptionalScores::new(scores), &bytes);
    assert_encoding!(
        OptionalScores,
        OptionalScores::new(BTreeMap::default()),
        &[]
    );

    let scores: OptionalScoresRef = OptionalScoresRef::new(&bytes).unwrap();
    assert_eq!(scores.scores().len(), 2);
    assert_eq!(scores.scores().get("a"), Some(Some(1)));
    assert_eq!(scores.scores().get("bc"), Some(None));
    assert_eq!(scores.scores().get("zz"), None);
}

/// Gets a struct with an array of each kind.
fn array_kinds() -> ArrayKinds {
    ArrayKinds::new(
        [1, 2, 3, 4],
        [true, false, true],
        [5, 6],
        [Shade::Light, Shade::Dark],
        [Point::new(1, 2), Point::new(3, 4)],
        [[1, 2, 3], [4, 5, 6]],
        Some([7, 8]),
    )
}

#[test]
fn arrays_are_in_the_fixed_header() {
    let bytes: Vec<u8> = [
        &[5, 0, 0, 0, 0, 0, 0, 0][..],                     // longs[0]
        &[6, 0, 0, 0, 0, 0, 0, 0],                         // longs[1]
        &[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0], // points
        &[1, 0, 2, 0],                                     // colors
        &[7, 0, 8, 0],                                     // maybe
        &[1, 2, 3, 4],                                     // bytes
        &[1, 0, 1],                                        // flags
        &[1, 2, 3, 4, 5, 6],                               // grid
        &[1],                                              // the presence bit of maybe
    ]
    .concat();
    assert_encoding!(ArrayKinds, array_kinds(), &bytes);

    let kinds: ArrayKindsRef = ArrayKindsRef::new(&bytes).unwrap();
    assert_eq!(kinds.bytes(), &[1, 2, 3, 4]);
    assert_eq!(
        kinds.flags().iter().collect::<Vec<bool>>(),
        [true, false, true]
    );
    assert_eq!(kinds.longs().get(1), Some(6));
    assert_eq!(kinds.colors().get(1), Some(Shade::Dark));
    assert_eq!(kinds.points().get(1).unwrap().y(), 4);
    let row: &[u8; 3] = kinds.grid().get(1).unwrap();
    assert_eq!(row, &[4, 5, 6]);
    assert_eq!(kinds.maybe().unwrap().get(0), Some(7));

    let absent: ArrayKinds = ArrayKinds::new(
        [0; 4],
        [false; 3],
        [0; 2],
        [Shade::Light; 2],
        [Point::new(0, 0); 2],
        [[0; 3]; 2],
        None,
    );
    let bytes: Vec<u8> = absent.encode();
    assert_encoding!(ArrayKinds, absent, &bytes);
    assert!(ArrayKindsRef::new(&bytes).unwrap().maybe().is_none());
}

/// Gets a struct with optional elements of each kind.
fn option_kinds() -> OptionKinds {
    OptionKinds::new(
        [None, Some(Shade::Dark)],
        [Some(Point::new(1, 2)), None],
        vec![Some(Shade::Light), None],
        vec![None, Some(Entry::new(7, "seven".to_string())), None],
        vec![Some(true), None, Some(false)],
        BTreeMap::from([(1, Some("one".to_string())), (2, None)]),
        BTreeMap::from([
            ("a".to_string(), None),
            ("b".to_string(), Some(Shade::Light)),
        ]),
    )
}

#[test]
fn every_kind_of_optional_element_round_trips() {
    let value: OptionKinds = option_kinds();
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(OptionKinds, value, &bytes);

    let kinds: OptionKindsRef = OptionKindsRef::new(&bytes).unwrap();
    assert_eq!(kinds.colors().get(0), Some(None));
    assert_eq!(kinds.colors().get(1), Some(Some(Shade::Dark)));
    assert_eq!(kinds.points().get(0).unwrap().unwrap().y(), 2);
    assert!(kinds.points().get(1).unwrap().is_none());
    assert_eq!(kinds.shades().len(), 2);
    assert_eq!(kinds.shades().get(1), Some(None));
    assert_eq!(kinds.entries().get(1).unwrap().unwrap().text(), "seven");
    assert!(kinds.entries().get(2).unwrap().is_none());
    assert_eq!(
        kinds.flags().iter().collect::<Vec<Option<bool>>>(),
        [Some(true), None, Some(false)]
    );
    assert_eq!(kinds.names().get(1), Some(Some("one")));
    assert_eq!(kinds.names().get(2), Some(None));
    assert_eq!(kinds.by_name().get("b"), Some(Some(Shade::Light)));
    assert_eq!(kinds.by_name().get("a"), Some(None));
}

#[test]
fn optional_enums_have_no_presence_bits() {
    let value: OptionKinds = OptionKinds::new(
        [None, Some(Shade::Dark)],
        [None, None],
        vec![Some(Shade::Light), None],
        Vec::default(),
        Vec::default(),
        BTreeMap::default(),
        BTreeMap::default(),
    );
    let kinds_bytes: Vec<u8> = value.encode();
    let kinds: OptionKindsRef = OptionKindsRef::new(&kinds_bytes).unwrap();
    assert_eq!(kinds.colors().as_bytes(), [0, 0, 2, 0]);
    assert_eq!(kinds.shades().as_bytes(), [1, 0, 0, 0]);
}

/// Gets a message with arrays & optional elements.
fn with_arrays() -> WithArrays {
    WithArrays::default()
        .with_origin([Float32::new(1.0), Float32::new(2.0), Float32::new(3.0)])
        .with_slots([Some(1), None, Some(3)])
        .with_rows(vec![[1, 2], [3, 4]])
        .with_entries(vec![Some(Entry::new(1, "a".to_string())), None])
        .with_scores(BTreeMap::from([
            ("a".to_string(), Some(1)),
            ("bc".to_string(), None),
        ]))
        .with_by_id(BTreeMap::from([([1, 2], 3), ([0, 9], 4)]))
}

#[test]
fn messages_and_variants_hold_arrays_and_optional_elements() {
    let value: WithArrays = with_arrays();
    let bytes: Vec<u8> = value.encode();
    assert_encoding!(WithArrays, value, &bytes);
    let message: WithArraysRef = WithArraysRef::new(&bytes).unwrap();
    assert_eq!(message.origin().unwrap().get(2), Some(Float32::new(3.0)));
    assert_eq!(message.slots().unwrap().get(1), Some(None));
    assert_eq!(message.rows().unwrap().get(1).unwrap().get(0), Some(3));
    assert!(message.entries().unwrap().get(1).unwrap().is_none());
    assert_eq!(message.scores().unwrap().get("a"), Some(Some(1)));
    assert_eq!(message.by_id().unwrap().get(&[1, 2]), Some(3));
    assert_eq!(message.by_id().unwrap().get(&[1, 3]), None);
    assert_encoding!(WithArrays, WithArrays::default(), &[0, 0]);

    for value in [
        ArrayCases::Origin([Float32::new(1.0), Float32::new(2.0), Float32::new(3.0)]),
        ArrayCases::Numbers(vec![Some(1), None]),
        ArrayCases::Scores(BTreeMap::from([("a".to_string(), None)])),
        ArrayCases::Number(7),
    ] {
        let bytes: Vec<u8> = value.encode();
        assert_encoding!(ArrayCases, value, &bytes);
    }
    let bytes: Vec<u8> = ArrayCases::Numbers(vec![Some(1), None]).encode();
    let numbers = ArrayCasesRef::new(&bytes).unwrap().as_numbers().unwrap();
    assert_eq!(numbers.get(0), Some(Some(1)));
}

#[test]
fn rejects_invalid_arrays_and_optional_elements() {
    assert!(matches!(
        Mesh::decode(&mesh_bytes()[..19]),
        Err(DecodeError::TooShort { .. })
    ));
    let mut longer: Vec<u8> = mesh_bytes();
    longer.push(0);
    assert!(matches!(
        Mesh::decode(&longer),
        Err(DecodeError::TooLong { .. })
    ));

    // a presence bit past the last element
    let mut bytes: Vec<u8> = slots_bytes();
    bytes[12] = 0b1101;
    assert_eq!(Slots::decode(&bytes), Err(DecodeError::InvalidPadding));

    // an absent element that isn't zeros
    let mut bytes: Vec<u8> = slots_bytes();
    bytes[4] = 1;
    assert_eq!(Slots::decode(&bytes), Err(DecodeError::InvalidAbsentField));

    // a count of 0, & a count that isn't the count of the elements
    assert_eq!(
        OptionalNumbers::decode(&[0, 0, 0, 0]),
        Err(DecodeError::InvalidList)
    );
    let bytes: Vec<u8> = [&[2, 0, 0, 0][..], &[0; 12], &[0]].concat();
    assert_eq!(
        OptionalNumbers::decode(&bytes),
        Err(DecodeError::InvalidList)
    );

    // an absent string with bytes
    let bytes: Vec<u8> = [&[2, 0, 0, 0][..], &[9, 0, 0, 0], b"a", b"b", &[0b10]].concat();
    assert_eq!(
        OptionalTexts::decode(&bytes),
        Err(DecodeError::InvalidAbsentField)
    );
}

/// Gets the count of the filled slots of any form.
fn filled<S: SlotsView>(slots: &S) -> usize {
    slots.slots().iter().flatten().count()
}

/// Gets the sum of the origin of any form of a mesh.
fn origin_sum<M: MeshView>(mesh: &M) -> f32 {
    mesh.origin().iter().map(Float32::get).sum()
}

/// Gets the scores of any form, with the names that have none.
fn scores_of<S: OptionalScoresView>(scores: &S) -> (usize, Option<Option<u32>>) {
    let map = scores.scores();
    (map.len(), map.get(&"bc".to_string()))
}

#[test]
fn views_read_arrays_and_optional_elements_of_every_form() {
    let slots: Slots = Slots::new([Some(1), None, Some(3)]);
    let mut bytes: Vec<u8> = slots_bytes();
    assert_eq!(filled(&slots), 2);
    assert_eq!(filled(&SlotsRef::new(&bytes).unwrap()), 2);
    assert_eq!(filled(&SlotsBuf::from(&slots)), 2);
    assert_eq!(filled(&SlotsMut::new(&mut bytes).unwrap()), 2);

    let mut bytes: Vec<u8> = mesh_bytes();
    assert_eq!(origin_sum(&mesh()), 6.0);
    assert_eq!(origin_sum(&MeshRef::new(&bytes).unwrap()), 6.0);
    assert_eq!(origin_sum(&MeshBuf::from(&mesh())), 6.0);
    assert_eq!(origin_sum(&MeshMut::new(&mut bytes).unwrap()), 6.0);

    let scores: OptionalScores = OptionalScores::new(BTreeMap::from([
        ("a".to_string(), Some(1)),
        ("bc".to_string(), None),
    ]));
    let bytes: Vec<u8> = scores.encode();
    assert_eq!(scores_of(&scores), (2, Some(None)));
    assert_eq!(
        scores_of(&OptionalScoresRef::new(&bytes).unwrap()),
        (2, Some(None))
    );
}

/// The slots of borrowed data, with no native slots.
struct Board<'a> {
    slots: &'a [Option<u32>],
}

impl SlotsView for Board<'_> {
    fn slots(&self) -> impl ListView<Item = Option<u32>> {
        self.slots
    }
}

#[test]
fn borrowed_slots_are_encoded_with_no_native_array() {
    let board: Board = Board {
        slots: &[Some(1), None, Some(3)],
    };
    assert_eq!(board.encode(), slots_bytes());
    assert_eq!(board.encoded_len(), 13);
}

#[test]
#[should_panic(expected = "the array has another length")]
fn views_refuse_arrays_of_another_length() {
    let _ = Board {
        slots: &[Some(1), None],
    }
    .encode();
}

#[test]
fn arrays_and_optional_elements_in_json() {
    assert_json(
        &mesh(),
        r#"{"id":1,"origin":["1.0","2.0","3.0"],"tag":"CQgHBg=="}"#,
    );
    assert_json(
        &Slots::new([Some(1), None, Some(3)]),
        r#"{"slots":[1,null,3]}"#,
    );
    assert_json(
        &OptionalTexts::new(vec![Some("hi".to_string()), None, Some(String::default())]),
        r#"{"texts":["hi",null,""]}"#,
    );
    assert_json(
        &OptionalScores::new(BTreeMap::from([
            ("a".to_string(), Some(1)),
            ("bc".to_string(), None),
        ])),
        r#"{"scores":{"a":1,"bc":null}}"#,
    );
    let json: String = serde_json::to_string(&array_kinds()).unwrap();
    assert_eq!(
        json,
        concat!(
            r#"{"bytes":"AQIDBA==","flags":[true,false,true],"longs":["5","6"],"#,
            r#""colors":["Light","Dark"],"points":[{"x":1,"y":2},{"x":3,"y":4}],"#,
            r#""grid":["AQID","BAUG"],"maybe":[7,8]}"#
        )
    );
    assert_eq!(
        serde_json::from_str::<ArrayKinds>(&json).unwrap(),
        array_kinds()
    );
    let json: String = serde_json::to_string(&option_kinds()).unwrap();
    assert_eq!(
        serde_json::from_str::<OptionKinds>(&json).unwrap(),
        option_kinds()
    );
    let json: String = serde_json::to_string(&with_arrays()).unwrap();
    assert_eq!(
        serde_json::from_str::<WithArrays>(&json).unwrap(),
        with_arrays()
    );
    assert!(json.contains(r#""by_id":{"AAk=":4,"AQI=":3}"#), "{}", json);
}

#[test]
fn json_rejects_arrays_of_another_length() {
    for json in [
        r#"{"slots":[1,null]}"#,
        r#"{"slots":[1,null,3,4]}"#,
        r#"{"slots":{}}"#,
    ] {
        assert!(serde_json::from_str::<Slots>(json).is_err(), "{}", json);
    }
    for json in [
        r#"{"id":1,"origin":["1.0","2.0"],"tag":"CQgHBg=="}"#,
        r#"{"id":1,"origin":["1.0","2.0","3.0"],"tag":"CQg="}"#,
        r#"{"id":1,"origin":["1.0","2.0","3.0"],"tag":[9,8,7,6]}"#,
    ] {
        assert!(serde_json::from_str::<Mesh>(json).is_err(), "{}", json);
    }
}
