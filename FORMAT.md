# Format

This document specifies the ProtoPacket binary format, & the JSON form of the same values.

The format is built to be fast: a field is read in place, from an offset known at compile time.

## Rules

- Everything is little-endian.
- A value never stores its own extent. Its extent comes from its container, & the extent of the
  root value is its buffer.
- Alignment is relative to the start of the root value, & is at most 8. Nothing inside a message
  is aligned.
- Readers never depend on alignment. They read values at any address.
- Writers zero all padding & unused bits, & readers ignore them.
- Slack is unused bytes at the end of an extent. It is legal, writers zero it, & readers ignore it.
  A `string` in a message or variant has no slack, since it is its whole extent.
- A canonical encoding has only the padding this document specifies, & no slack.

## Types

| Type            | Size     | Alignment | Encoding                                                 |
|-----------------|----------|-----------|----------------------------------------------------------|
| `bool`          | 1 bit    | none      | A bit in a struct's bit group. (see [Bits](#bits))       |
| `u8` to `u128`  | 1 to 16  | up to 8   | An unsigned integer. Its alignment is its size, up to 8. |
| `uuid`          | 16       | 8         | The 16 bytes in the order of its string form. (RFC 9562) |
| enum            | 2        | 2         | A `u16` of the case's tag number.                        |
| fixed struct    | its size | its own   | Its fixed block. (see [Structs](#structs))               |
| `string`        | variable | 1         | The UTF-8 bytes, without a terminator.                   |
| variable struct | variable | its own   | Its fixed block & tail. (see [Structs](#structs))        |
| message         | variable | 1         | Its index & values. (see [Messages](#messages))          |
| variant         | variable | 1         | Its tag & value. (see [Variants](#variants))             |

A `box` doesn't affect the encoding, so a boxed field is encoded like the type it boxes.

Tag numbers are from 1 to 8,191. A valid tag number that isn't one of the enum's cases is kept as
an unrecognized case. Tag number 0 is never a case, so it is the absent value of an optional enum.

## Structs

A struct's fields never change, so a struct has no tags & is not evolvable. Renaming a field is
safe. Reordering fields is not, since the declaration order decides the offsets.

A struct is fixed if all its fields are fixed-size, & variable otherwise. A struct that contains
itself, through any loop of types, is variable. (ex: `next: optional box ListNode;` in `ListNode`)

A struct is encoded as its fixed block followed by its tail. A fixed struct has no tail.

```text
[fixed block][tail]
```

A struct's alignment is the largest alignment of the fields of the struct & of every struct it
contains, including 4 if any of them has an end table. An empty struct has alignment 1 & size 0.

### Fixed Block

The fixed block has these units, in this order:

1. The fields of alignment 8.
2. The fields of alignment 4, followed by the end table.
3. The fields of alignment 2.
4. The fields of alignment 1.
5. The bit group.
6. Padding.

The fields of an alignment are in declaration order. A fixed-size field's size is a multiple of its
alignment, so nothing is padded before the final padding.

A fixed struct is padded to a multiple of its alignment, which is then its size. A variable struct
is padded to the alignment of its first tail entry.

### Tail

The tail holds the variable-size fields as entries: the variable structs, messages & variants by
descending alignment, followed by the strings. The fields of an alignment are in declaration order,
& so are the strings.

A growing string therefore moves only the strings after it.

The end table has a `u32` end offset for each entry, in tail order, from the start of the tail. The
`i`th entry is `tail[end[i-1]..end[i]]`, where `end[-1]` is 0. A tail is at most `u32::MAX` bytes.

Each entry is padded to the alignment of the next entry, whether or not that entry is present. The
padding is inside the entry's extent, & is not slack. The last entry isn't padded.

### Bits

The bit group has a bit for each `bool` field & for each presence bit, in as few bytes as needed.
The bits are in declaration order, & an `optional bool` has its presence bit followed by its value
bit.

The `n`th bit (from 0) is bit `n % 8` of byte `n / 8`, where bit 0 is the least significant.

### Optional Fields

An absent field is encoded by its kind:

- An optional enum is 0, with no presence bit.
- Any other fixed-size field has a presence bit, & keeps its place in the fixed block as zeros. A
  struct with only such optional fields is still fixed.
- A variable-size field has a presence bit, & its entry is empty.

Readers ignore the value of an absent field, so it isn't validated.

### Validation

Readers reject a struct when:

- Its extent is shorter than its fixed block.
- Its end table decreases, or its last end offset is past the end of its tail.
- A present `string` is not valid UTF-8.
- A present enum isn't a valid tag number.
- A present nested struct, message or variant is invalid.

### Examples

```text
struct Point { x: u32; y: u32; }

[x: u32][y: u32]

8 bytes, alignment 4

offset 0: x
offset 4: y
```

```text
struct Mixed { a: u8; b: u64; c: u16; d: bool; }

[b: u64][c: u16][a: u8][bits: d][padding: 4]

16 bytes, alignment 8

offset  0: b
offset  8: c
offset 10: a
offset 11: bits (d is bit 0)
```

```text
struct Line { start: Point; end: Point; width: u8; }

[start: Point][end: Point][width: u8][padding: 3]

20 bytes, alignment 4

offset  0: start
offset  8: end
offset 16: width
```

```text
struct Person { name: string; age: u8; }

[end: u32][age: u8][name bytes]

Person("Ada", 36) is 8 bytes, alignment 4

offset 0: end = 3
offset 4: age
offset 5: tail: "Ada"
```

```text
struct Entry { id: u64; text: string; }
struct Pair { first: Entry; second: Entry; note: string; }

[ends: u32 x 3][padding: 4][first][padding: 7][second][note]

Pair(Entry(1, "hello"), Entry(2, "hi"), "ok") is 56 bytes, alignment 8

offset  0: end[0] = 24
offset  4: end[1] = 38
offset  8: end[2] = 40
offset 16: tail: first, 17 bytes & 7 of padding
offset 40: tail: second, 14 bytes
offset 54: tail: "ok"
```

```text
enum Color { Red = 1; }
struct Opt { a: bool; b: optional u32; c: optional bool; d: optional string; e: optional Color; }

[b: u32][end: u32][e: Color][bits: a, b present, c present, c, d present][d bytes]

Opt(true, None, Some(false), Some("hi"), None) is 13 bytes, alignment 4

offset  0: b = 0
offset  4: end = 2
offset  8: e = 0
offset 10: bits = 0b0001_0101
offset 11: tail: "hi"
```

## Messages

A message can change: fields can be added & removed, but a field's tag number & type never change.
Only the present fields are encoded, so an absent field has no cost.

```text
[count: u16][tags: u16 x count][ends: u32 x count][values]
```

- The tags are strictly increasing, so a reader finds a field by binary search.
- The `i`th value is `values[end[i-1]..end[i]]`, where `end[-1]` is 0. `values` is at most `u32::MAX`
  bytes.
- Nothing is padded or aligned. A message has alignment 1, so adding a field never changes the
  layout of a struct that contains it.

### Fields

A field is encoded like its type, without a bit group: a `bool` is a byte, 1 for true & 0 for false.
Readers treat any non-zero byte as true. A field's extent can be longer than its value, & readers
ignore the rest, except for a `string`, which is its whole extent.

An extent never depends on the field's type, so a reader can skip a field it doesn't know. Unknown
fields are kept when a message is decoded & written back when it's encoded.

### Validation

Readers reject a message when:

- Its extent is shorter than its index.
- Its tags aren't strictly increasing, or one isn't a valid tag number.
- Its end table decreases, or its last end offset is past the end of its values.
- A known field is shorter than its type, or isn't valid for its type.

Unknown fields aren't validated.

### Examples

```text
message Item { text: string = 1; id: u32 = 5; }

[count][tags: 1, 5][ends: 2, 6]["hi"][id: u32]

Item("hi", 7) is 20 bytes, alignment 1

offset  0: count = 2
offset  2: tags = 1, 5
offset  6: ends = 2, 6
offset 14: "hi"
offset 16: id = 7
```

```text
message Item { text: string = 1; id: u32 = 5; }

Item(None, 7) is 12 bytes. The text isn't listed.

offset 0: count = 1
offset 2: tags = 5
offset 4: ends = 4
offset 8: id = 7
```

## Variants

A variant holds one of its cases. Cases can be added & removed, but a case's tag number & type never
change.

```text
[tag: u16][value]
```

- The value is encoded like a message field, & is the rest of the extent.
- A variant is always variable-size & has alignment 1, whatever its cases are, so adding a case
  never changes the layout of a struct that contains it.

A variant has no length of its own, so a `string` case can't have slack: extra bytes would be part
of the string. Any other case ignores the rest of its extent.

A case the reader doesn't know is kept with its tag & its bytes, & written back when it's encoded.

### Validation

Readers reject a variant when:

- Its extent is shorter than its tag.
- Its tag isn't a valid tag number.
- Its known case is shorter than its type, or isn't valid for its type.

Unknown cases aren't validated.

### Examples

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

## JSON

| Type            | JSON                                                          |
|-----------------|---------------------------------------------------------------|
| `bool`          | A boolean.                                                    |
| `u8` to `u128`  | A number.                                                     |
| `uuid`          | A string, in its string form.                                 |
| `string`        | A string.                                                     |
| enum            | A case name, or a tag number if the writer doesn't know it.   |
| struct, message | An object of the fields, by name. Absent fields aren't listed. |
| variant         | `{"var": <case name>, "val": <value>}`                        |

A renamed field or case keeps its schema name. Keys can be in any order, & other keys are ignored.

### Unknown Fields & Cases

A value that came from the binary format can hold fields & cases the writer doesn't know. They keep
their tag numbers, with their encoded values as base-64 text:

```text
message:  {"text": "hi", "7": "CQ=="}
variant:  {"var": 3, "val": "AQAAAAIAAAA="}
```

A key of only digits is a tag number, since a field name starts with a letter. A tag number can't
have leading zeros.

A reader that knows the tag number decodes the value as that field or case, with the validation of
the binary format. A reader that doesn't keeps it, so it round-trips.

### Different Schema Versions

JSON keeps only what has a tag number. A name doesn't, so a reader can't keep one it doesn't know:

- A field name the reader doesn't know is dropped.
- A case name the reader doesn't know is an error, in a variant or an enum.

Use the binary format to pass values through a reader with an older schema.

### Validation

Readers reject JSON when:

- A field, tag number, `var` or `val` is repeated.
- A field is given by name & by tag number.
- A key is an invalid tag number.
- A tag-numbered value isn't valid base-64, or isn't valid for a field or case the reader knows.
