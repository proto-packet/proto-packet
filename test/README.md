# Test

This folder holds the ProtoPacket end-to-end tests.

The `schema` project is compiled into a full project in each language, & the generated code is
checked & tested.

- `schema/`: the test schema project, with a folder per feature.
- `test-rust/`: the Rust project.

Run `build.sh` to compile the schema, then format, lint & test each project.
