# Format

This document specifies the ProtoPacket binary format.

The format is built to be fast: fixed-size values are their little-endian bytes in memory, so they
are read & written by copying, & can be read in place without decoding.

## Rules

- Everything is little-endian.
- There is no padding.
- A value has exactly one encoding, so equal values have equal bytes.
- Writers always write the canonical encoding. Safe readers accept any float & normalize it.

## Fixed-Size Types

| Type                 | Bytes    | Encoding                                                  |
|----------------------|----------|-----------------------------------------------------------|
| `bool`               | 1 bit    | A bit in a struct's bit group. (see [Bits](#bits))        |
| `u8` to `u128`       | 1 to 16  | An unsigned integer.                                      |
| `i8` to `i128`       | 1 to 16  | A two's complement signed integer.                        |
| `f32`, `f64`         | 4, 8     | An IEEE 754 binary32 or binary64 float.                   |
| `uuid`               | 16       | The 16 bytes in the order of its string form. (RFC 9562)  |
| `timestamp`          | 8        | An `i64` of the milliseconds since 1970-01-01T00:00:00Z.  |
| `date`               | 4        | An `i32` of the days since 1970-01-01.                    |
| enum                 | 2        | A `u16` of the case's tag number.                         |
| fixed struct         | its head | The struct's head. (see [Structs](#structs))              |

### Floats

Floats are written normalized:

1. Every NaN is the canonical NaN. (`0x7FC00000` & `0x7FF8000000000000`)
2. Negative zero is positive zero.

Safe readers accept any float & normalize it the same way.

### Enums

Tag numbers are at most 8,191, so a `u16` always holds them. Safe readers reject a tag number that
is not one of the enum's cases. Tag number 0 is never a case, so it is the absent value of an
optional enum. (see [Optional Fields](#optional-fields))

### Dates

Dates are in the proleptic Gregorian calendar. Safe readers reject a date outside the range their
language can represent.

## Variable-Size Types

| Type                 | Encoding                                                  |
|----------------------|-----------------------------------------------------------|
| `string`             | The UTF-8 bytes, without a terminator.                    |
| non-fixed struct     | The struct's head & data. (see [Structs](#structs))       |
| list (`[]T`)         | The list's elements. (see [Lists](#lists))                |

The length of a variable-size value is not part of its encoding. It is stored by its container.
(ex: in the head of a struct) A length is at most `u32::MAX - 1`, since `u32::MAX` is the length of
an absent optional value. (see [Optional Fields](#optional-fields))

Safe readers reject a `string` that is not valid UTF-8.

## Structs

A struct is fixed if all its fields are fixed-size types, & non-fixed otherwise. A struct's fields
never change, so a struct's encoding has no tags & is not evolvable. A boxed field is encoded like
the type it boxes. An optional field is fixed-size if its type is.

A field of a struct that contains the field's own struct, through a loop of types, is variable-size,
since a loop has no fixed size. (ex: `next: optional box ListNode;` in `ListNode`)

A struct is encoded as its head followed by its data. A fixed struct has no data.

```text
[head][data]
```

### Head

The head has an item for each field:

- A fixed-size field is its encoding.
- A variable-size field is the `u32` length of its encoding, in bytes.
- A `bool` field is a bit.
- An optional field with a presence bit also has a bit. (see [Optional Fields](#optional-fields))

The items are placed in alignment groups of 8, 4, 2 & 1 bytes, followed by the bit group. Within an
alignment group, the fields are in declaration order, followed by the lengths in the declaration
order of their fields.

The alignment of an item is:

- The size of a fixed-size type, at most 8. (ex: 8 for a `u128`)
- 4 for a length.
- For a fixed struct, the largest alignment of its head's items, if its head's size is a multiple
  of it, & 1 otherwise. (ex: 4 for `{ x: i32; y: i32; }`, but 1 for `{ a: u64; b: u8; }`)

Each item in the 8, 4 & 2 alignment groups has a size that is a multiple of its group, so when the
head starts at an 8-byte aligned address, every item in those groups is aligned. Readers never
depend on the alignment, it only makes reading faster.

### Bits

The bit group is the last item of the head. It has the bits of the `bool` fields & the presence
bits of the optional fields in declaration order, in as few bytes as needed. (ex: 2 bytes for 9 to
16 bits) An `optional bool` has its presence bit followed by its value bit.

The `n`th bit (from 0) is bit `n % 8` of byte `n / 8`, where bit 0 is the least significant. Unused
bits are written as 0 & ignored by readers.

### Data

The data is the encoding of each present variable-size field, in declaration order. Each field's
encoding has the length in its head item, so a field's data starts at the head's size plus the
lengths of the present variable-size fields before it.

The data is not aligned: each variable-size value starts right after the one before it, & readers
read values at any address.

Safe readers reject a struct whose bytes are not exactly its head's size plus the lengths of its
present variable-size fields.

### Optional Fields

An optional field can be absent. Its absence is encoded with a value its type never has, if there
is one:

- An absent enum is the tag number 0.
- An absent variable-size value is the length `u32::MAX`, with no data.

Otherwise, the field has a presence bit in the bit group: 1 if present & 0 if absent. The field's
value keeps its place in the head, & is written as zeros when absent. (ex: an `optional u32`)

Safe readers ignore the value of an absent field, so it is not validated.

### Examples

```text
struct MyStruct { x: string; y: u32; z: u128; }

[z: u128][y: u32][x length: u32][x bytes]

offset  0: z
offset 16: y
offset 20: x length
offset 24: x bytes
```

```text
struct S { a: string; b: u32; c: u128; d: string; e: bool; f: u64; }

[c: u128][f: u64][b: u32][a length: u32][d length: u32][bits: e][a bytes][d bytes]

offset  0: c
offset 16: f
offset 24: b
offset 28: a length
offset 32: d length
offset 36: bits (e is bit 0)
offset 37: a bytes, then d bytes
```

```text
struct Point { x: i32; y: i32; }         // 8 bytes, alignment 4
struct Odd { a: u64; b: u8; }            // 9 bytes, alignment 1
struct T { o: Odd; p: Point; n: u64; g: bool; }

[n: u64][p: Point][o: Odd][bits: g]

offset  0: n
offset  8: p
offset 16: o
offset 25: bits (g is bit 0)
```

```text
enum Color { Red = 1; }
struct Opt { a: optional u32; b: bool; c: optional string; d: optional bool; e: optional Color; }

[a: u32][c length: u32][e: Color][bits: a present, b, d present, d][c bytes]

offset  0: a (zeros when absent)
offset  4: c length (u32::MAX when absent, with no bytes)
offset  8: e (0 when absent)
offset 10: bits (a present is bit 0, b is bit 1, d present is bit 2 & d is bit 3)
offset 11: c bytes
```

## Lists

A list (`[]T`) is a variable-size value. Its encoding depends on the type of its elements.

### Fixed-Size Elements

The elements are encoded one after another, with no count, since the count is the list's length
divided by the size of an element. (ex: a `[]u8` is its bytes) The `bool` elements are bits
instead. (see [Bit Elements](#bit-elements))

```text
[]u32 [1, 2, 3]

[1: u32][2: u32][3: u32]                                12 bytes
```

### Bit Elements

A `[]bool` is the `u32` count of its elements, followed by a bit for each element, in as few bytes
as needed. The `n`th element (from 0) is bit `n % 8` of byte `n / 8` of the bits, where bit 0 is
the least significant. Unused bits are written as 0 & ignored by readers.

```text
[]bool [true, false, true]

[count: u32 = 3][bits: 0b0000_0101]                     5 bytes
```

### Variable-Size Elements

A list of variable-size elements is the `u32` count of its elements, followed by the `u32` end
offset of each element, followed by the encodings of the elements one after another. The end
offsets are from the start of the elements' encodings, so the `n`th element (from 0) is from the end
offset of the element before it (0 for the first element) to its own end offset.

```text
[]string ["ab", "", "c"]

[count: u32 = 3][ends: u32 = 2, 2, 3][ab][][c]          19 bytes
```

### Validation

Safe readers reject a list:

- Of fixed-size elements, whose length is not a multiple of the size of an element.
- Of `bool` elements, whose length is not the size of its count & its bits.
- Of variable-size elements, whose end offsets decrease or whose last end offset is not the size of
  its elements' encodings.

They validate each element like any value of its type. (ex: the UTF-8 of each `string`)

## Reading

Safe readers validate the bytes: the lengths, the UTF-8 of strings, the enum tag numbers & the
dates of the present values. They normalize the floats.

Unsafe readers skip the validation & normalization, for speed when the bytes are known to be valid.
Only Rust has unsafe readers, as `unsafe` functions, since invalid bytes break its memory safety.

## Rust

Each struct has three Rust types:

| Type             | Description                                                              |
|------------------|--------------------------------------------------------------------------|
| `MyType`         | The Rust type, owned & editable, without the layout of the format.       |
| `MyTypeBuf`      | The owned encoding, read in place.                                       |
| `MyTypeRef<'a>`  | The borrowed encoding, read in place.                                    |

All three read & write the same format. `MyType` is encoded & decoded field by field, so it can stay
idiomatic Rust. `MyTypeBuf` & `MyTypeRef` read each field from its offset in the bytes when it is
accessed.

Creating a `MyTypeBuf` or `MyTypeRef` from bytes validates them, or skips the validation in an
`unsafe` function. Since a `MyTypeRef` borrows its bytes, its floats are normalized when they are
read.

A `MyTypeRef` reads a list field's elements through a view of them, reading each element from its
offset. A `[]u8` is borrowed as the bytes instead.

## Not Yet Specified

- Maps. (a list of structs with a key & a value)
- Messages. (a `u16` header of a 13-bit tag number & a 3-bit wire type)
- Variants.
- Values outside a struct. (ex: a `bool` or a `u32` as a service request)
- Enum tag numbers that are not one of the enum's cases, for schema evolution.
