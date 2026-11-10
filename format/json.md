# JSON Format

This document specifies the JSON form of ProtoPacket values.

| Type            | JSON                                                           |
|-----------------|----------------------------------------------------------------|
| `bool`          | A boolean.                                                     |
| `u8` to `u32`   | A number.                                                      |
| `i8` to `i32`   | A number.                                                      |
| `u64`, `u128`   | A string of the number.                                        |
| `i64`, `i128`   | A string of the number.                                        |
| `f32`, `f64`    | A string of the number, or of a name if it isn't finite.       |
| time types      | A string of the integer the time is stored as.                 |
| `decimal<S>`    | A string of the number, with all `S` of its fraction digits.   |
| `uuid`          | A string, in its string form.                                  |
| `string`        | A string.                                                      |
| enum            | A case name, or a tag number if the writer doesn't know it.    |
| `[]u8`          | A string of the bytes, as base-64.                             |
| list            | An array of the elements.                                      |
| map             | An object of the entries, or an array if a key isn't text.     |
| struct, message | An object of the fields, by name. Absent fields aren't listed. |
| variant         | `{"var": <case name>, "val": <value>}`                         |

A renamed field or case keeps its schema name. Keys can be in any order, & other keys are ignored.
A `null` is read as an absent field, & writers leave an absent field out.

## Numbers In Strings

Many JSON parsers read a plain number as a float, which can't hold every number. So these types are
strings, & readers reject a plain number for one.

- A `u64`, `i64`, `u128` or `i128` is its digits. (ex: `"18446744073709551615"`)
- A time is the integer it is stored as. (ex: `"1700000000000"` for a `timestamp`)
- A float is the shortest number that reads back as the same float. (ex: `"1.5"`) A float that
  isn't finite is `"NaN"`, `"Infinity"` or `"-Infinity"`.
- A decimal has all `S` of its fraction digits. (ex: `"1.50"`) A reader takes fewer fraction digits
  than its scale, but not more. (ex: `"1.5"`)

The number in the string is written as JSON writes a number. It has no `+` before it, no leading
zeros, no space & a digit on each side of a `.`. Only a float can have an exponent. (ex: `"1e-7"`)

## Bytes

Base-64 has the standard alphabet, with `+` & `/`, & its `=` padding. (ex: `"CQg="`)

## Maps

A map's key is the text of its JSON, so a number or a `bool` is quoted. (ex: `{"1": true}`) A
`[]u8` key is its base-64. A map can't repeat a key.

A key that is a struct, message, variant, list or map isn't text. Its map is an array of the
entries, each an array of the key & the value:

```text
map<Point, u8>:  [[{"x": 1, "y": 2}, 3], [{"x": 4, "y": 5}, 6]]
```

## Unknown Fields & Cases

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

## Different Schema Versions

JSON keeps only what has a tag number. A name doesn't, so a reader can't keep one it doesn't know:

- A field name the reader doesn't know is dropped.
- A case name the reader doesn't know is an error, in a variant or an enum.

Use the binary format to pass values through a reader with an older schema.

## Validation

Readers reject JSON when:

- A field, tag number, `var` or `val` is repeated.
- A field is given by name & by tag number.
- A key is an invalid tag number.
- A tag-numbered value isn't valid for a field or case the reader knows.
- Base-64 isn't valid, or doesn't have its padding.
- A number in a string isn't written as JSON writes a number, or is out of the range of its type.
- A type that is a number in a string is a plain number.
- A map is an object where its key isn't text, or an array where it is.
- A map repeats a key, or an entry of an array isn't a key & a value.
