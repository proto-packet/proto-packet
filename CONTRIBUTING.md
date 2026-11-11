# Contributing

This guide outlines how contributions should be made to the codebase.

## Checks

The `build.sh` script formats & lints the runtime & the compiler, then runs the end-to-end tests
with `test/build.sh`, which are what tests them. It must pass before each commit.

## Commits

Each commit is one small, focused change. Its message is a tag with one sentence. Example:

```text
[compile/parse] Added support for parsing packet types.
```

There shouldn't be a description.

### Tags

The tag denotes the section of the code base the commit is relevant to.

The project:

- `[cargo]`: the project, typically anything focused outside the `compile`, `dep` & `test`
  folders.

The folders:

- `[compile]`: the compiler, if working on the compiler as a whole.
- `[compile/{slice}]`: the compiler, if working on a specific slice.
- `[dep]`: the runtime libraries, if working on the runtime libraries as a whole.
- `[dep/{language}]`: the runtime libraries, if working on a specific language.
- `[test]`: the end-to-end tests, if working on the tests as a whole.
- `[test/{language}]`: the end-to-end tests, if working on a specific language.
- `[plugins]`: the editor plugins, if working on the plugins as a whole.
- `[plugins/{plugin}]`: the editor plugins, if working on a specific plugin.

A `{slice}` is a compiler crate's name without `proto-packet-`. (ex: `[compile/parse]` for
`proto-packet-parse`) A `{language}` is a language folder's name without `dep-` or `test-`. (ex:
`[dep/rust]` for `dep/dep-rust`) A `{plugin}` is a plugin folder's name. (ex: `[plugins/textmate]`)
