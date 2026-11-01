# Contributing

This guide outlines how contributions should be made to the codebase.

## Commits

Each commit is one small, focused change. Its message is a tag with one sentence. Example:

```text
[compile/parse] Added support for parsing packet types.
```

There shouldn't be a description.

### Tags

The tag denotes the section of the code base the commit is relevant to.

The project:

- `[cargo]`: the project, typically anything focused outside the workspaces.

The workspaces:

- `[compile]`: the compiler, if working on the compiler as a whole.
- `[compile/{slice}]:` the compiler, if working on a specific slice.
