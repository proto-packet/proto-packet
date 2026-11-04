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
  `should_implement_trait`. Only the constructor & the setters & builders are reserved.
  (`fields.rs`)
- The errors for a conflict with a child mod name the Rust folder, not the schema mod path: empty at
  the root, & `self_` instead of `self`. (`generate.rs`)

### Ordering

Generated enums order their cases by tag number, but variants derive `Ord`, ordering their cases by
declaration order, & messages derive it over their fields in declaration order. Reordering a
variant's cases or a message's fields in the schema doesn't change their tag numbers, but changes
their order, & so the order of the maps keyed by them. (`gen_variant.rs` & `gen_message.rs`)

### Doc Comments

The comments of enums, variants, services & their members are copied into the rustdoc unchanged, so
rustdoc reads them as Markdown. (`docs.rs`)

- An indented line after a blank comment line becomes a code block, & so a doctest that fails
  `cargo test` on the generated crate. (ex: `// Line one.`, `//`, `//     foo bar baz`)
- Ordinary text can fail `cargo doc` with `-D warnings`. (ex: `Vec<u32>` is an unclosed HTML tag, &
  `[Thing]` a broken intra-doc link)

### Variant `u128` Cases

A variant is `{"var": ..., "val": ...}` in JSON. When `val` comes before `var`, the value is buffered
until the case is known, & serde_json reports a number above `u64::MAX` as a float, so a larger
`u128` value fails to read. (ex: sorted keys) The generated code writes `var` first, so only
reordered input fails. The full fix waits for a JSON encoding without serde.
(`proto-packet/src/types/variant_json.rs`)

### Message Unknown Fields In JSON

A message's unknown fields survive encoding, but are dropped by JSON, unlike a variant's unknown
case, which is base-64 text. (`gen_message.rs`)

### Recursive Types

A `Ref` type validates its nested values by recursion, so a struct, message or variant that contains
itself recurses once for each level in its bytes. Nothing limits the depth, so crafted bytes
overflow the stack & abort the process. (ex: 200,000 levels of `ListNode`, 2.4 MB, overflow an 8 MB
stack) Decoding & dropping the decoded value recurse the same way. (ex: a depth limit with a
`DecodeError`) (`gen_struct_ref.rs`, `gen_message_ref.rs` & `gen_variant_ref.rs`)

## Compile

### Target Folder

Compiling replaces the target folder with the generated files.
(`proto-packet-compile/src/compile.rs`)

- Any existing target folder is emptied. Only a target containing the source is refused, so a typo
  like `compile rust schemas src` instead of `src/generated` silently deletes a handwritten `src/`.
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
