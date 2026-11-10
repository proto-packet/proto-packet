# Binary Format

This document specifies the ProtoPacket binary format.

TODO: some of the types below don't exist yet.

## Base Types

Primitives: 
- The `u8`, `u16`, `u32`, `u64`, & `u128` types are stored as little-endian.
- The `i8`, `i16`, `i32`, `i64`, & `i128` types are stored as two's complement little-endian.
- The `f16`, `f32`, `f64`, & `f128` types are stored as IEEE 754 & little-endian. A NaN is always
  stored with the sign bit clear & only the top fraction bit set, so every NaN has the same bytes.
  (ex: `0x7FC00000` for an `f32`) `-0.0` is stored as it is, so it is a different value than `0.0`.
- The `bool` type is a single bit in a struct & a single byte in a message or variant.

Special Types:
- The `uuid` type is stored as 16 bytes, in the order of its string form.
  (ex: `123e4567-e89b-12d3-a456-426614174000` starts with the byte `0x12`)
- The `string` type is stored as UTF-8 bytes, with no terminator. Its length comes from its
  container.

Time Types:
todo -- decide the range. Unrestricted is likely, with an optional conversion to each language's
time type.
- The epoch of these types is `1970-01-01 00:00:00 UTC`. Leap seconds aren't counted.
- The `timestamp` type is stored as an `i64`. It is the number of milliseconds since the epoch.
- The `microtime` type is stored as an `i64`. It is the number of microseconds since the epoch.
- The `nanotime` type is stored as an `i64`. It is the number of nanoseconds since the epoch, as
  one number & not as seconds with nanoseconds. It covers the years 1677 to 2262.
- The `date` type is stored as an `i32`. It is the number of days since the epoch. It is a calendar
  date with no time zone, in the proleptic Gregorian calendar.
- The `duration` type is stored as an `i64`. It is a number of nanoseconds, & it can be negative.

## Structs

A struct is a record that never changes so the format does not support schema evolution. Renaming a
field is safe. Adding, removing or reordering fields changes the layout.

```text
struct = [fixed-header][variable-tail]
```

The `fixed-header` holds the fixed-size fields. Every field in it is at an offset known at compile
time. The `variable-tail` holds the variable-size fields, one after another.

A struct with only fixed-size fields has no tail, so the struct itself is fixed-size. A struct with
any variable-size field is variable-size. A struct that contains itself is always variable-size.
(ex: `next: optional box ListNode;` in `ListNode`)

### Fixed Header

The order of the fields in the `fixed-header` depends first on the alignment. The largest alignment
goes first, which keeps padding rare. Fields with the same alignment stay in the order they are
declared, except that struct fields go after the others.

| Types                                                                 | Size     | Alignment |
|-----------------------------------------------------------------------|----------|-----------|
| `u128`, `i128`, `f128`, `uuid`                                        | 16       | 8         |
| `u64`, `i64`, `f64`, `timestamp`, `microtime`, `nanotime`, `duration` | 8        | 8         |
| `u32`, `i32`, `f32`, `date`                                           | 4        | 4         |
| `u16`, `i16`, `f16`, enums                                            | 2        | 2         |
| `u8`, `i8`                                                            | 1        | 1         |
| fixed-size structs                                                    | its size | its own   |

A `bool` isn't in the table because it is stored as a bit.

The header has these parts, in this order.

1. The fields with alignment 8, then the struct fields with alignment 8.
2. The fields with alignment 4, then the end table, then the struct fields with alignment 4.
3. The fields with alignment 2, then the struct fields with alignment 2.
4. The fields with alignment 1, then the struct fields with alignment 1.
5. The bits. (see [Bits](#bits))

A struct only has an end table when it has two or more variable-size fields.
(see [Variable Tail](#variable-tail))

Each part starts at the next multiple of its own alignment. If the part before it ends somewhere
else, zeros are added between them as padding. The end table has alignment 4 & the bits have
alignment 1. Padding can only follow a struct field, since every other part has a size that is a
multiple of its alignment. That is why struct fields go last.

Nothing is padded at the end of a struct, so a struct's size is exactly what it uses. Whatever holds
the struct adds padding after it if the next thing needs it.

A struct's own alignment is the largest alignment of its fields. An end table counts as 4. An empty
struct has size 0 & alignment 1.

Two fields with the same alignment stay in the order they are declared.

```text
struct Point { x: u32; y: u32; }

[x: u32][y: u32]

8 bytes, alignment 4

offset 0: x
offset 4: y
```

Fields with different alignments are reordered, & the `bool` becomes a bit. Nothing is padded at
the end, so the struct is 12 bytes.

```text
struct Mixed { a: u8; b: u64; c: u16; d: bool; }

[b: u64][c: u16][a: u8][bits: d]

12 bytes, alignment 8

offset  0: b
offset  8: c
offset 10: a
offset 11: bits (d is bit 0)
```

A fixed-size struct is placed by its alignment like any other field.

```text
struct Line { start: Point; end: Point; width: u8; }

[start: Point][end: Point][width: u8]

17 bytes, alignment 4

offset  0: start
offset  8: end
offset 16: width
```

A field after a struct starts at the next multiple of its own alignment. Here `m` ends at 12, which
is already a multiple of 4, so `x` needs no padding.

```text
struct Outer { m: Mixed; x: u32; y: u8; }

[m: Mixed][x: u32][y: u8]

17 bytes, alignment 8

offset  0: m
offset 12: x
offset 16: y
```

Struct fields go after the other fields with the same alignment, so `x` comes first here. Then `a`
ends at 21, so `b` gets 3 bytes of padding to start at 24.

```text
struct Padded { a: Line; x: u32; b: Line; }

[x: u32][a: Line][padding: 3][b: Line]

41 bytes, alignment 4

offset  0: x
offset  4: a
offset 24: b
```

### Bits

A `bool` field doesn't get a byte of its own. All the `bool` fields are packed into bits at the end
of the header, together with a presence bit for each optional field.
(see [Optional Fields](#optional-fields))

The bits are in the order the fields are declared, in as few bytes as needed. An `optional bool` has
its presence bit first, then its value bit. Unused bits are 0.

Bit `n`, counting from 0, is bit `n % 8` of byte `n / 8`. Bit 0 is the least significant bit.

### Variable Tail

Each variable-size field has an entry in the tail. The entries are in the order the fields are
declared.

A struct doesn't store where its last entry ends, since that is where the struct ends. So a struct
with one variable-size field needs nothing in its header for it.

```text
struct Person { name: string; age: u8; }

[age: u8][name bytes]

Person("Ada", 36) is 4 bytes, alignment 1

offset 0: age
offset 1: "Ada"
```

A struct with two or more variable-size fields has an end table in its header. The end table has a
`u32` for each entry except the last, which is the offset where the entry ends. Offsets count from
the start of the struct.

An entry starts at the next multiple of its own alignment after the entry before it ends. The first
entry does the same after the header ends. Zeros fill any gap as padding, the same as in the header.
An empty entry has no padding before it. Only a variable-size struct or a list can have an alignment
above 1, so only they can have padding before them.

Both `Entry` fields here have alignment 8. The header ends at 8, so `first` starts there. It ends at
21, so `second` starts at 24. The `note` is last, so it ends where the struct ends.

```text
struct Entry { id: u64; text: string; }
struct Pair { first: Entry; second: Entry; note: string; }

[ends: u32 x 2][first][padding: 3][second][note]

Pair(Entry(1, "hello"), Entry(2, "hi"), "ok") is 36 bytes, alignment 8

offset  0: end[0] = 21
offset  4: end[1] = 34
offset  8: first, 13 bytes
offset 24: second, 10 bytes
offset 34: "ok"
```

### Optional Fields

An optional field has a presence bit in the bits. It is 1 when the field is present.

- An absent fixed-size field keeps its place in the header, as zeros.
- An absent variable-size field has an empty entry in the tail.
- An optional enum has no presence bit. It is 0 when absent, since 0 is never a tag number.

A struct whose optional fields are all fixed-size is still fixed-size.

```text
enum Color { Red = 1; }
struct Opt { a: bool; b: optional u32; c: optional bool; d: optional string; e: optional Color; }

[b: u32][e: Color][bits: a, b present, c present, c, d present][d bytes]

Opt(true, None, Some(false), Some("hi"), None) is 9 bytes, alignment 4

offset 0: b = 0
offset 4: e = 0
offset 6: bits = 0b0001_0101
offset 7: "hi"
```

### Validation

A struct doesn't store its own length. The length comes from whatever holds the struct.

Readers reject a struct when any of these is true.

- A fixed-size struct isn't exactly its size, or a variable-size struct is shorter than its header.
- The end table starts before the end of the header, goes down, or ends past the end of the struct.
- The padding before an entry reaches or passes its end.
- Padding or an unused bit isn't 0.
- An absent field isn't all zeros in the header, or isn't empty in the tail.
- A required enum is 0.
- A float is a NaN with the wrong bits.
- A present `string` isn't valid UTF-8.
- A present struct, message or variant inside it is invalid.


------- end fixes ------------------------------------------------------------------------------------------
## Enums

An enum is stored as a `u16`. The value is the tag number of its case.

A tag number is a number from 1 to 65,535. It identifies a case of an enum, a field of a message or
a case of a variant. A tag number never changes once it is used, even when the name does, & that is
what lets these types change over time. 0 is never a tag number.

```text
enum Color { Red = 1; }

Red is 2 bytes

offset 0: 1
```

Cases can be added to an enum & removed from it. A reader that sees a tag number it doesn't know
keeps it as an unrecognized case, & writes it back unchanged.

Readers reject an enum that is 0. The one exception is an optional enum in a struct, which is 0 when
it is absent.

## Messages

A message is a record that can change. Fields can be added & removed over time, so a reader & a
writer with different versions of the schema can still share data. A field's tag number & type never
change.

Every field of a message is optional. Only the fields that are present are stored, so an absent
field costs nothing.

```text
message = [count][tags][ends][values]
```

- The `count` is a `u16`. It is the number of fields that are present.
- The `tags` are a `u16` for each present field, holding its tag number. They are sorted from
  smallest to largest, & a tag number can't repeat.
- The `ends` are a `u32` for each present field except the last, holding the offset where its value
  ends. Offsets count from the start of the `values`.
- The `values` are the values of the present fields, in the same order as the tags.

A value starts where the value before it ends, & the first value starts at 0. The last value ends
where the message ends. A reader finds a field by a binary search of the tags.

```text
message Item { text: string = 1; id: u32 = 5; }

[count][tags: 1, 5][ends: 2]["hi"][id: u32]

Item("hi", 7) is 16 bytes, alignment 1

offset  0: count = 2
offset  2: tags = 1, 5
offset  6: ends = 2
offset 10: "hi"
offset 12: id = 7
```

An absent field isn't in the message at all. A message with one field has no ends.

```text
message Item { text: string = 1; id: u32 = 5; }

Item(None, 7) is 8 bytes. The text isn't listed.

offset 0: count = 1
offset 2: tags = 5
offset 4: id = 7
```

### Values

A message has no bits & no padding, & nothing in it is aligned. So a few types are stored
differently than in a struct.

- A `bool` is one byte, 1 for true & 0 for false.
- A `string` is just its bytes. Its length is the length of the value.
- Every other type is stored the same as anywhere else. A struct keeps its own header & tail.

A value is exactly the bytes of its type, with nothing extra.

A message always has alignment 1 & is always variable-size, whatever its fields are. So changing a
message never changes the layout of a struct that holds it. A struct inside a message is only
aligned from its own start.

### Unknown Fields

The ends give the length of every value, so a reader can skip a field without knowing its type. A
reader that sees a tag number it doesn't know keeps the field's bytes, & writes them back unchanged.

### Validation

Readers reject a message when any of these is true.

- It is shorter than its `count`, `tags` & `ends`.
- The tags aren't sorted from smallest to largest, a tag repeats, or a tag is 0.
- The ends go down, or the last one is past the end of the message.
- A field the reader knows isn't valid for its type. (ex: a `u32` that isn't 4 bytes, or a `bool`
  byte of 2)

Fields the reader doesn't know aren't checked.

## Variants

A variant holds exactly one of its cases. Cases can be added & removed over time. A case's tag
number & type never change.

todo -- this layout is for now & may change.

```text
variant = [tag][value]
```

- The `tag` is a `u16`. It is the tag number of the case.
- The `value` is the rest of the variant. It is stored the same way as a value in a message.

```text
struct Point { x: u32; y: u32; }
variant Shape { Circle: u32 = 1; Square: Point = 2; }

Circle(5) is 6 bytes, alignment 1

offset 0: tag = 1
offset 2: value = 5

Square(Point(1, 2)) is 10 bytes, alignment 1

offset 0: tag = 2
offset 2: x = 1
offset 6: y = 2
```

A variant always has alignment 1 & is always variable-size, whatever its cases are. So changing a
variant never changes the layout of a struct that holds it.

### Unknown Cases

A reader that sees a tag number it doesn't know keeps the tag & the value's bytes, & writes them
back unchanged.

### Validation

Readers reject a variant when any of these is true.

- It is shorter than its `tag`.
- The tag is 0.
- A case the reader knows isn't valid for its type.

Cases the reader doesn't know aren't checked.

## Lists

A list is its elements in order. An empty list is no bytes.

A list is always variable-size, so in a struct it is an entry of the tail. Its alignment is the
alignment of its elements.

todo -- a fixed-length list is `[N]T`, for later.

### Fixed-Size Elements

The elements are back to back, with no count. A reader gets the count from the length of the list.

- A `bool` is one byte, 1 for true & 0 for false.
- A `[]u8` is just the bytes.
- Each element starts at the next multiple of its alignment. Only a struct can end off one, so only
  structs are padded apart. Nothing is padded after the last element.

```text
struct Numbers { count: u8; values: []u32; }

[count: u8][padding: 3][values]

Numbers(7, [1, 2]) is 12 bytes, alignment 4

offset 0: count
offset 4: values = 1, 2

Numbers(7, []) is 1 byte. The empty list has no bytes, so it has no padding.
```

A list can't hold elements of no size, such as a struct without fields. Its count couldn't be read
from its length.

### Variable-Size Elements

```text
list = [count][ends][elements]
```

- The `count` is a `u32`. It is the number of elements, & it is never 0.
- The `ends` are a `u32` for each element except the last, holding the offset where the element
  ends. Offsets count from the start of the list.
- The `elements` come next. Each one starts at the next multiple of its alignment after the one
  before it ends, & an empty element has no padding before it. The last element ends where the list
  ends.

The alignment of such a list is at least 4, for its count & ends.

```text
struct Texts { texts: []string; }

[count][ends: 14, 14]["hi"]["yo!"]

Texts(["hi", "", "yo!"]) is 17 bytes, alignment 4

offset  0: count = 3
offset  4: ends = 14, 14
offset 12: "hi"
offset 14: "yo!"
```

A list of lists is a list of variable-size elements. (ex: `[][]u8`)

### Validation

Readers reject a list when any of these is true.

- Its length doesn't fit a whole number of fixed-size elements.
- Its count is 0, or it is shorter than its count & ends.
- Its ends start before the elements, go down, or end past the end of the list.
- The padding before an element reaches or passes its end, or isn't zeros.
- An element is invalid.

## Maps

A map is its keys as a list, then its values as a list. The `n`th value is the value of the `n`th
key. An empty map is no bytes.

```text
map = [keys end][keys][values]
```

- The `keys end` is a `u32`. It is the offset where the keys end, from the start of the map.
- The `keys` are a list of the key type. (see [Lists](#lists))
- The `values` are a list of the value type, with one value for each key.

Each list starts at the next multiple of its own alignment, with zeros as padding before it. The
alignment of a map is the largest of the two lists, & at least 4.

```text
struct Scores { scores: map<string, u32>; }

[keys end][keys: "a", "bc"][padding: 1][values: 1, 2]

Scores({"a": 1, "bc": 2}) is 24 bytes, alignment 4

offset  0: keys end = 15
offset  4: keys: count = 2
offset  8: keys: ends = 9
offset 12: keys: "a", "bc"
offset 16: values = 1, 2
```

### Key Order

The keys are strictly increasing, so a map has no two equal keys & a reader finds a key by a binary
search. Any type can be a key.

| Type             | Order                                                     |
|------------------|-----------------------------------------------------------|
| `bool`           | False before true.                                        |
| integers         | By number.                                                |
| floats           | By number, with `-0.0` before `0.0` & the NaN last.       |
| `uuid`, `string` | By bytes.                                                 |
| enum             | By tag number.                                            |
| struct           | By the fields, in declaration order.                      |
| list             | By the elements. A list comes after the lists it extends. |
| map              | By the entries, each a key then its value.                |

todo -- a message or a variant as a key is ordered by its declaration order, which a schema change
can reorder. (see [Ordering](../FUTURE.md#ordering))

### Validation

Readers reject a map when any of these is true.

- It is shorter than its `keys end`.
- Its keys or its values are no bytes, or its keys end past the end of the map.
- Its keys or its values are an invalid list, or they aren't the same count.
- Its keys aren't strictly increasing.
- The padding before its keys or its values isn't zeros.

## Reading & Writing

These rules apply to every type.

- A value never stores its own length. The outermost value is exactly as long as the buffer, & every
  other value gets its length from whatever holds it.
- The format has no magic number, version or framing. Whatever stores or sends a value has to know
  its type & its length.
- Writers fill padding & unused bits with zeros.
- Readers work at any address. Alignment only helps when the buffer itself is aligned, & nothing
  depends on it.
- A `box` only changes how a language holds a value. It doesn't change the bytes.

Every value has exactly one encoding, & readers reject anything else. (ex: padding that isn't zeros,
or a `bool` byte of 2) So two values are equal exactly when their bytes are equal, & they can be
compared without decoding them. Unknown fields & cases are compared as their bytes.

## Limits

- A tag number is from 1 to 65,535, so a message has at most 65,535 fields.
- An alignment is 1, 2, 4 or 8.
- A struct is at most `u32::MAX` bytes, & so are the `values` of a message.
- A list of variable-size elements is at most `u32::MAX` elements & `u32::MAX` bytes.

todo -- nothing limits how deep values nest. (see [Recursive Types](../FUTURE.md#recursive-types))
