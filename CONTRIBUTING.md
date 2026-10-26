# Contributing

This guide outlines how contributions should be made to the codebase.

## Commits

Each commit is one small, focused change. Its message is a tag & one sentence:

```text
[compile] Added the proto-packet-cli crate.
```

There shouldn't be a description.

### Tags

A tag is an area, narrowed when the change stays in one part of it. (ex: `[compile/parse]`)

The project:

- `[cargo]`: the project, typically anything focused outside the workspaces.

The workspaces:

- `[compile]`: the compiler in `compile/`, typically anything focused outside of `src/` folders.
- `[compile/cli]`: the command-line interface in `proto-packet-cli`.
- `[compile/compile]`: compiling projects in `proto-packet-compile`.
- `[compile/gen]`: generating code from schemas in `proto-packet-gen`.
- `[compile/link]`: linking schema files in `proto-packet-link`.
- `[compile/parse]`: parsing schema files in `proto-packet-parse`.
- `[compile/tree]`: the semantic schema tree in `proto-packet-tree`.
- `[compile/validate]`: validating schema files in `proto-packet-validate`.
- `[dep]`: the runtime libraries in `dep/`, typically anything focused outside a single language.
- `[dep/rust]`: the Rust runtime library in `dep/dep-rust`.
