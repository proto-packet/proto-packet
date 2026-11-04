# Format

This document specifies the ProtoPacket binary format. Only structs are specified so far, & only
structs without message or variant fields can be encoded.

The format is built to be fast: a field is read in place, from an offset known at compile time.

## Rules

- Everything is little-endian.
- A value never stores its own extent. Its extent comes from its container, & the extent of the
  root value is its buffer.
- Alignment is relative to the start of the root value, & is at most 8.
- Readers never depend on alignment. They read values at any address.
- Writers zero all padding & unused bits, & readers ignore them.
- Slack is unused bytes at the end of an extent. It is legal, & readers ignore it.
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

The tail holds the variable-size fields as entries: the variable structs by descending alignment,
followed by the strings. The fields of an alignment are in declaration order, & so are the strings.

A growing string therefore never moves a variable struct.

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
- A present nested struct is invalid.

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
