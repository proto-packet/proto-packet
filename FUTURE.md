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

Some schema names generate Rust that doesn't compile, or that hides other items.

- A service call named like a method `Arc<S>` already has through a prelude trait (ex: `clone`,
  `into`, `to_owned`, `as_ref`) calls that method in the router, since the router calls
  `service.{call}(request)` on an `Arc<S>`. The generated code doesn't compile. (ex: call it fully
  qualified, `<S as Svc>::clone(&service, request)`) (`gen_service.rs`)
- A service named `S` is shadowed by the router's type parameter `S`, in `where S: S + 'static`. The
  generated code doesn't compile. (`gen_service.rs`)
- A top-level mod named `lib` generates both `src/lib.rs` & `src/lib/mod.rs`, which doesn't compile.
  The check compares child mods to `mod`, the parent's file, but the root's file is `lib.rs`, so a
  harmless root mod named `mod` is rejected instead. (`generate.rs`)
- A field named like a prelude trait method (ex: `clone`, `eq`, `cmp`, `into`, `max`) gets a getter
  that hides the trait method, since inherent methods win. (ex: `a.clone()` returns the field, &
  `a.eq(&b)` stops compiling) A non-`Copy` field named `deref` or `borrow` fails clippy's
  `should_implement_trait`. Only the names of generated methods are reserved. (`fields.rs`)
- The errors for a conflict with a child mod name the Rust folder, not the schema mod path: empty at
  the root, & `self_` instead of `self`. (`generate.rs`)

### Doc Comments

The comments of enums, variants, services & their members are copied into the rustdoc unchanged, so
rustdoc reads them as Markdown. (`docs.rs`)

- An indented line after a blank comment line becomes a code block, & so a doctest that fails
  `cargo test` on the generated crate. (ex: `// Line one.`, `//`, `//     foo bar baz`)
- Ordinary text can fail `cargo doc` with `-D warnings`. (ex: `Vec<u32>` is an unclosed HTML tag, &
  `[Thing]` a broken intra-doc link)

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

### Target Folder

Compiling replaces the target folder with the generated files.
(`proto-packet-compile/src/compile.rs`)

- Any existing target folder is emptied. Only a target containing the source or a schema file is
  refused, so a typo like `compile rust schemas src` instead of `src/generated` silently deletes a
  handwritten `src/`.
  (ex: refuse a non-empty target unless every file starts with the generated header, or use a marker
  file)
- Replacing the target isn't atomic: the old files are deleted first, then the new ones are written
  one at a time. A write error partway through leaves a partial target, & the old output is gone.
  (ex: write to a temporary sibling folder, then rename it into place)

### Schema Extension

The config's `schema_extension` isn't validated, & schema files are matched by the end of their
paths. An extension of `.yml` reads the project's own `config.yml` as a schema, & one without a dot
(ex: `pps`) turns `a.pps` into the mod name `a.`, giving a confusing name error.
(`proto-packet-compile/src/read.rs`)
