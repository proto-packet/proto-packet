# JSON Format

This document specifies the JSON form of ProtoPacket values.

| Type            | JSON                                                           |
|-----------------|----------------------------------------------------------------|
| `bool`          | A boolean.                                                     |
| `u8` to `u128`  | A number.                                                      |
| `i8` to `i128`  | A number.                                                      |
| `f32`, `f64`    | A string of the number, or of a name if it isn't finite.       |
| time types      | A number, the integer the time is stored as.                   |
| `uuid`          | A string, in its string form.                                  |
| `string`        | A string.                                                      |
| enum            | A case name, or a tag number if the writer doesn't know it.    |
| list            | An array of the elements.                                      |
| map             | An object of the entries, with each key as text.               |
| struct, message | An object of the fields, by name. Absent fields aren't listed. |
| variant         | `{"var": <case name>, "val": <value>}`                         |

A renamed field or case keeps its schema name. Keys can be in any order, & other keys are ignored.

A float is a string, so that it reads back as the exact float that was written. (ex: `"1.5"`) A
JSON parser can round a plain number to another float, so readers reject one. A float that isn't
finite is `"NaN"`, `"Infinity"` or `"-Infinity"`.

A map's key is the text of its JSON, so a number or a `bool` is quoted. (ex: `{"1": true}`) A key
that is a struct, message, variant, list or map can't be written yet.

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
- A tag-numbered value isn't valid base-64, or isn't valid for a field or case the reader knows.
- A float isn't a string of a decimal number or of one of its names, or is out of the range of its
  type.
