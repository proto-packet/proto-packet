//! The benchmarks of the generated code. Run them with `cargo bench`.
//!
//! Each line is the median time of one call, in nanoseconds.

use proto_packet::io::{
    ListView, MessageWriter, trim_padding, write_end, write_padding, write_zeros,
};
use proto_packet_test::format::examples::{Entry, EntryRef, EntryView, Point, PointRef, PointView};
use proto_packet_test::format::lists::{WithLists, WithListsRef, WithListsView};
use proto_packet_test::format::maps::{Scores, ScoresRef};
use proto_packet_test::format::messages::{Newer, NewerRef, NewerView};
use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

/// The count of the samples of a benchmark.
const SAMPLES: usize = 15;

/// The least time of a sample.
const SAMPLE_TIME: Duration = Duration::from_millis(10);

/// Measures the `run` function, & prints the median time of a call.
fn bench<T>(name: &str, mut run: impl FnMut() -> T) {
    let mut calls: u64 = 1;
    let mut time = |calls: u64| -> Duration {
        let start: Instant = Instant::now();
        for _ in 0..calls {
            black_box(run());
        }
        start.elapsed()
    };
    while time(calls) < SAMPLE_TIME && calls < 1 << 30 {
        calls *= 2;
    }
    let mut samples: Vec<f64> = (0..SAMPLES)
        .map(|_| time(calls).as_nanos() as f64 / calls as f64)
        .collect();
    samples.sort_by(f64::total_cmp);
    println!("{:<40} {:>12.1} ns", name, samples[SAMPLES / 2]);
}

/// Appends a list of the variable-size `items` to the `out` bytes, each written by `write`.
///
/// # Note
/// - This is what a builder would write, with no native list in between.
fn write_list<T>(
    out: &mut Vec<u8>,
    items: &[T],
    alignment: usize,
    mut write: impl FnMut(&T, &mut Vec<u8>),
) {
    if items.is_empty() {
        return;
    }
    let start: usize = out.len();
    out.extend_from_slice(&(items.len() as u32).to_le_bytes());
    write_zeros(out, (items.len() - 1) * 4);
    for (index, item) in items.iter().enumerate() {
        let before: usize = out.len();
        write_padding(out, start, alignment);
        let padded: usize = out.len();
        write(item, out);
        trim_padding(out, before, padded);
        if index + 1 < items.len() {
            write_end(out, start + 4 + index * 4, start);
        }
    }
}

/// An entry of borrowed data, written through its view.
#[derive(Copy, Clone)]
struct Row<'a> {
    id: u64,
    text: &'a str,
}

impl EntryView for Row<'_> {
    fn id(&self) -> u64 {
        self.id
    }

    fn text(&self) -> &str {
        self.text
    }
}

/// A message of borrowed data, written through its view.
struct Profile<'a> {
    name: &'a str,
    nickname: &'a str,
    text: &'a str,
}

impl NewerView for Profile<'_> {
    fn id(&self) -> Option<u64> {
        Some(1)
    }

    fn name(&self) -> Option<&str> {
        Some(self.name)
    }

    fn nickname(&self) -> Option<&str> {
        Some(self.nickname)
    }

    fn age(&self) -> Option<u32> {
        Some(36)
    }

    fn entry(&self) -> Option<impl EntryView> {
        Some(Row {
            id: 2,
            text: self.text,
        })
    }
}

/// A list of the rows of borrowed texts, with no list of rows in between.
#[derive(Copy, Clone)]
struct Rows<'a> {
    texts: &'a [String],
}

impl<'a> ListView for Rows<'a> {
    type Item = Row<'a>;

    fn len(&self) -> usize {
        self.texts.len()
    }

    fn get(&self, index: usize) -> Option<Row<'a>> {
        self.texts.get(index).map(|text| Row {
            id: index as u64,
            text,
        })
    }

    fn iter(&self) -> impl Iterator<Item = Row<'a>> {
        self.texts.iter().enumerate().map(|(id, text)| Row {
            id: id as u64,
            text,
        })
    }
}

/// A message of borrowed lists, written through its view.
struct Table<'a> {
    texts: &'a [String],
}

impl WithListsView for Table<'_> {
    fn bytes(&self) -> Option<&[u8]> {
        None
    }

    fn numbers(&self) -> Option<impl ListView<Item = u32>> {
        None::<&[u32]>
    }

    fn texts(&self) -> Option<impl ListView<Item = &str>> {
        Some(self.texts)
    }

    fn points(&self) -> Option<impl ListView<Item = impl PointView>> {
        None::<&[Point]>
    }

    fn entries(&self) -> Option<impl ListView<Item = impl EntryView>> {
        Some(Rows { texts: self.texts })
    }

    fn children(&self) -> Option<impl ListView<Item = impl WithListsView>> {
        None::<&[WithLists]>
    }
}

/// Benchmarks a fixed-size struct.
fn bench_point() {
    let point: Point = Point::new(1, 2);
    let bytes: Vec<u8> = point.encode();
    bench("point: encode", || black_box(&point).encode());
    bench("point: validate", || {
        PointRef::new(black_box(&bytes)).is_ok()
    });
    bench("point: decode", || Point::decode(black_box(&bytes)));
    bench("point: validate & read in place", || {
        let point: PointRef = PointRef::new(black_box(&bytes)).unwrap();
        point.x() + point.y()
    });
}

/// Benchmarks a struct with a string.
fn bench_entry() {
    let text: &str = "the quick brown fox";
    let entry: Entry = Entry::new(7, text.to_string());
    let bytes: Vec<u8> = entry.encode();
    let write = |id: u64, text: &str| -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(8 + text.len());
        out.extend_from_slice(&id.to_le_bytes());
        out.extend_from_slice(text.as_bytes());
        out
    };
    assert_eq!(write(7, text), bytes);
    assert_eq!(Row { id: 7, text }.encode(), bytes);

    bench("entry: build & encode", || {
        Entry::new(black_box(7), black_box(text).to_string()).encode()
    });
    bench("entry: encode a view of borrowed", || {
        Row {
            id: black_box(7),
            text: black_box(text),
        }
        .encode()
    });
    bench("entry: write by hand from borrowed", || {
        write(black_box(7), black_box(text))
    });
    bench("entry: encode", || black_box(&entry).encode());
    bench("entry: validate", || {
        EntryRef::new(black_box(&bytes)).is_ok()
    });
    bench("entry: decode", || Entry::decode(black_box(&bytes)));
    bench("entry: validate & read in place", || {
        let entry: EntryRef = EntryRef::new(black_box(&bytes)).unwrap();
        entry.id() as usize + entry.text().len()
    });
}

/// Benchmarks a message with five fields.
fn bench_message() {
    let (name, nickname, text): (&str, &str, &str) = ("Ada Lovelace", "Ada", "the quick brown fox");
    let build = |name: &str, nickname: &str, text: &str| -> Newer {
        Newer::default()
            .with_id(1)
            .with_name(name.to_string())
            .with_nickname(nickname.to_string())
            .with_age(36)
            .with_entry(Entry::new(2, text.to_string()))
    };
    let write = |name: &str, nickname: &str, text: &str| -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(64 + name.len() + nickname.len() + text.len());
        let mut writer: MessageWriter = MessageWriter::new(&mut out, 5, &[]);
        writer.field(1).extend_from_slice(&1_u64.to_le_bytes());
        writer.end_field();
        writer.field(2).extend_from_slice(name.as_bytes());
        writer.end_field();
        writer.field(3).extend_from_slice(nickname.as_bytes());
        writer.end_field();
        writer.field(4).extend_from_slice(&36_u32.to_le_bytes());
        writer.end_field();
        let entry: &mut Vec<u8> = writer.field(5);
        entry.extend_from_slice(&2_u64.to_le_bytes());
        entry.extend_from_slice(text.as_bytes());
        writer.end_field();
        writer.finish();
        out
    };
    let message: Newer = build(name, nickname, text);
    let bytes: Vec<u8> = message.encode();
    assert_eq!(write(name, nickname, text), bytes);
    let view = |name: &str, nickname: &str, text: &str| -> Vec<u8> {
        Profile {
            name,
            nickname,
            text,
        }
        .encode()
    };
    assert_eq!(view(name, nickname, text), bytes);

    bench("message: build & encode", || {
        build(black_box(name), black_box(nickname), black_box(text)).encode()
    });
    bench("message: encode a view of borrowed", || {
        view(black_box(name), black_box(nickname), black_box(text))
    });
    bench("message: write by hand from borrowed", || {
        write(black_box(name), black_box(nickname), black_box(text))
    });
    bench("message: encode", || black_box(&message).encode());
    bench("message: validate", || {
        NewerRef::new(black_box(&bytes)).is_ok()
    });
    bench("message: decode", || Newer::decode(black_box(&bytes)));
    bench("message: validate & read in place", || {
        let message: NewerRef = NewerRef::new(black_box(&bytes)).unwrap();
        message.id().unwrap_or_default() as usize
            + message.name().map_or(0, str::len)
            + message.nickname().map_or(0, str::len)
            + message.age().unwrap_or_default() as usize
            + message.entry().map_or(0, |entry| entry.text().len())
    });
}

/// Benchmarks a message with a list of 100 structs & a list of 100 strings.
fn bench_lists() {
    let texts: Vec<String> = (0..100)
        .map(|index| format!("text number {}", index))
        .collect();
    let build = |texts: &[String]| -> WithLists {
        WithLists::default()
            .with_texts(texts.to_vec())
            .with_entries(
                texts
                    .iter()
                    .enumerate()
                    .map(|(id, text)| Entry::new(id as u64, text.clone()))
                    .collect::<Vec<Entry>>(),
            )
    };
    let write = |texts: &[String]| -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(4096);
        let mut writer: MessageWriter = MessageWriter::new(&mut out, 2, &[]);
        write_list(writer.field(3), texts, 1, |text, out| {
            out.extend_from_slice(text.as_bytes());
        });
        writer.end_field();
        let mut id: u64 = 0;
        write_list(writer.field(5), texts, 8, |text, out| {
            out.extend_from_slice(&id.to_le_bytes());
            out.extend_from_slice(text.as_bytes());
            id += 1;
        });
        writer.end_field();
        writer.finish();
        out
    };
    let message: WithLists = build(&texts);
    let bytes: Vec<u8> = message.encode();
    assert_eq!(write(&texts), bytes);
    assert_eq!(Table { texts: &texts }.encode(), bytes);

    bench("lists: build & encode", || {
        build(black_box(&texts)).encode()
    });
    bench("lists: encode a view of borrowed", || {
        Table {
            texts: black_box(&texts),
        }
        .encode()
    });
    bench("lists: write by hand from borrowed", || {
        write(black_box(&texts))
    });
    bench("lists: encode", || black_box(&message).encode());
    bench("lists: validate", || {
        WithListsRef::new(black_box(&bytes)).is_ok()
    });
    bench("lists: decode", || WithLists::decode(black_box(&bytes)));
    bench("lists: validate & read in place", || {
        let message: WithListsRef = WithListsRef::new(black_box(&bytes)).unwrap();
        let texts: usize = message
            .texts()
            .map_or(0, |texts| texts.iter().map(str::len).sum());
        let entries: usize = message.entries().map_or(0, |entries| {
            entries
                .iter()
                .map(|entry| entry.id() as usize + entry.text().len())
                .sum()
        });
        texts + entries
    });
}

/// Benchmarks a struct with a map of 100 strings to numbers.
fn bench_map() {
    let names: Vec<String> = (0..100)
        .map(|index| format!("name number {}", index))
        .collect();
    let build = |names: &[String]| -> Scores {
        Scores::new(
            names
                .iter()
                .enumerate()
                .map(|(score, name)| (name.clone(), score as u32))
                .collect::<BTreeMap<String, u32>>(),
        )
    };
    let scores: Scores = build(&names);
    let bytes: Vec<u8> = scores.encode();
    let key: &str = "name number 57";

    bench("map: build & encode", || build(black_box(&names)).encode());
    bench("map: encode", || black_box(&scores).encode());
    bench("map: validate", || {
        ScoresRef::new(black_box(&bytes)).is_ok()
    });
    bench("map: decode", || Scores::decode(black_box(&bytes)));
    bench("map: validate & look up in place", || {
        let scores: ScoresRef = ScoresRef::new(black_box(&bytes)).unwrap();
        scores.scores().get(black_box(key))
    });
    bench("map: look up in place", || {
        let scores: ScoresRef = unsafe { ScoresRef::new_unchecked(black_box(&bytes)) };
        scores.scores().get(black_box(key))
    });
    bench("map: look up in the native map", || {
        black_box(&scores).scores().get(black_box(key)).copied()
    });
}

fn main() {
    bench_point();
    bench_entry();
    bench_message();
    bench_lists();
    bench_map();
}
