# Future

This document lists the planned work that is not started yet.

## Parse

### Byte-Order Mark

A UTF-8 byte-order mark at the start of a schema file isn't skipped: it lexes as an unrecognized
token, so a file saved by an editor that writes one fails to parse, & the error points at an
invisible character. (ex: strip a leading `\u{FEFF}` before lexing)
(`proto-packet-parse/src/lexer.rs`)

## Link

### Schema Linker

`SchemaLinker::new` takes the schema file to build its name resolver, & the public `link` takes the
schema file again. (`link_project` passes the same one twice) Linking a different file than the
linker was made for resolves its names against the wrong file's types & imports. (ex: keep the file
in the linker & make `link(self)` take no file) (`proto-packet-link/src/linker/schema_linker.rs`)

## Gen

### Naming Conflicts

A field named like a method of a trait keeps its name, so its getter & the method share it. (ex:
rename the getter, like `self_` for the field `self`) (`fields.rs`)

- A method that borrows is hidden by the getter. (ex: `a.clone()` returns the field `clone`, so the
  value is cloned with `Clone::clone(&a)`)
- A method that takes the value hides the getter. (ex: `a.max()` is `Ord::max`, so the field `max`
  is read with `A::max(&a)`)
- A field named `from` makes `ABuf::from(&a)` ambiguous, since the `View` trait has a getter of that
  name.

### Large Variant Cases

A variant with a case much larger than its others fails clippy's `large_enum_variant` in the
generated code, until the case is a `box` in the schema. (`gen_variant.rs`)

### Variant `u128` & `i128` Cases

A variant is `{"var": ..., "val": ...}` in JSON. When `val` comes before `var`, the value is buffered
until the case is known, & serde_json reports a number above `u64::MAX` or below `i64::MIN` as a
float, so a `u128` or `i128` value outside that range fails to read. (ex: sorted keys) The generated
code writes `var` first, so only reordered input fails. The full fix waits for a JSON encoding
without serde. (`dep/dep-rust/src/types/variant_json.rs` & `buffered.rs`)

### Lists & Maps

The lists & the maps are missing from some of the generated code.

- The `ViewMut` traits & the `Mut` types have no setter for a value that can change its length in
  place. That is a string, a list or a map of a struct, any field of a message & any case of a
  variant. (ex: a setter that fails unless the length is the same, or one that moves the bytes after
  it) (`gen_struct.rs`, `gen_struct_mut.rs`, `gen_message_mut.rs` & `gen_variant_mut.rs`)
- An array has no setter in place, though its length never changes. (ex: a setter of the whole
  array, or of one element) (`gen_struct_mut.rs`)
- A native array is inline, so a large one is a large value on the stack. (ex: box an array over a
  size) (`typing.rs`)
- The `MapView` trait looks up a value by the key's own type, so a `string` key is a `&String`. (ex:
  look up by the borrowed key, like `&str`) (`dep/dep-rust/src/io/list/map_view.rs`)
- A reader checks the order of the keys of a map by decoding them, unless the key is a number, a
  `bool`, a `uuid`, a `string` or ordered by its bytes. (`dep/dep-rust/src/io/list/element.rs`)
- A writer encodes each key that is ordered by its bytes twice, once to sort the keys & once to
  write them. (ex: write the keys from the sorted bytes) (`dep/dep-rust/src/io/list/map_ref.rs`)

### Views

A `View` trait encodes any type that has its getters, with these limits. (`gen_view_encoding.rs`)

- A map of a view is a `BTreeMap` or a `MapRef`, since a writer trusts the order of its keys. So a
  map can't be written from borrowed entries. (ex: a map view that checks the order of its keys)
- A view has no unknown fields or cases, so only the generated forms keep theirs.
- An encoded form inside a view is validated again when it is written, to check its depth. (ex: skip
  the check for a type that can't nest deeper than the levels left)
- The encoding of a view isn't a `Buf` until it is validated, since a view can give another value
  each time it is read. (ex: a `Buf` from a view that validates its encoding)

### Decimals

A `decimal<S>` is an `i64` of units, so it holds about 18 digits. (`dep/dep-rust/src/types/decimal.rs`)

- It can only be added & subtracted. Multiplying & dividing need a rounding rule.
- There is no larger decimal. (ex: an `i128` of units, for 38 digits)
- Two scales can't be converted to each other.

### Recursive Types

A native value that nests more than 64 levels panics when it is encoded, but getting its encoded
length & dropping it recurse once for each level, with no limit. (ex: 200,000 levels built by hand
overflow the stack) (`gen_struct.rs`, `gen_message.rs` & `gen_variant.rs`)

## Compile

### Target Mode

The config's `target_mode` only has `replace`, which clears the target folder & fails if the folder
holds a file that was not generated. (`proto-packet-compile/src/target_mode.rs`)

- A folder shared with handwritten files can't be a target. (ex: a mode that writes its files &
  deletes nothing, like `protoc`)
- A file that an editor or the system adds to the target fails the compile. (ex: `.DS_Store`)

### Schema Extension

The config's `schema_extension` isn't validated, & schema files are matched by the end of their
paths. An extension of `.yml` reads the project's own `config.yml` as a schema, & one without a dot
(ex: `pps`) turns `a.pps` into the mod name `a.`, giving a confusing name error.
(`proto-packet-compile/src/read.rs`)

## Editors

### Language Server

The language server only reports the errors of a schema file. (`proto-packet-lsp`)

- A file is checked alone, so an unknown type or a bad import isn't reported. The server doesn't
  know the source folder of the project, & a project without a config file doesn't mark it. (ex: a
  setting of the plugin, or the nearest folder with a config file)
- The checks stop at the first error of a file, as the compiler does. (ex: report every error of a
  file, & keep checking a file that is half typed)
- There is no go to definition, hover or completion. Only the parse tree has the positions they
  need.

### JetBrains Plugin

- The compiler is only looked for on the `PATH` & in `~/.cargo/bin`. (ex: a setting for its path)
- Only the `.pps` files are schema files, whatever the `schema_extension` of the config is.
- The plugin isn't published, so it is installed from its zip file.
