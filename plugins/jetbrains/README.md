# JetBrains Plugin

This plugin adds ProtoPacket schema files to the JetBrains IDEs, such as RustRover.

- It highlights the syntax of the `.pps` files, with the TextMate bundle in `../textmate`.
- It shows the errors of a schema file as it is edited, from the language server of the `packet`
  compiler.

## Requirements

- An IDE from version 2026.1.4 that has the language server support. The open source builds of
  IntelliJ IDEA & Android Studio don't.
- The `packet` compiler on the `PATH`, or in `~/.cargo/bin`. Install it from the root of this
  repository with `cargo install --path compile/proto-packet-cli`.

## Build

Run `build.sh` to build the plugin & verify it against an installed IDE. It takes the path of the
IDE, which is `/Applications/RustRover.app` by default, & it runs Gradle with the Java inside that
IDE. The plugin is written to `build/distributions`.

Without the `ide` property, Gradle downloads a RustRover to build against. (ex:
`./gradlew buildPlugin`)

## Install

Choose the zip file of the plugin in `Settings | Plugins | Install Plugin from Disk`.

## Files

- `src/main/resources/META-INF/plugin.xml`: the plugin's description & its extensions.
- `SchemaBundleProvider`: gives the IDE the TextMate bundle.
- `SchemaLspProvider` & `SchemaLspDescriptor`: start `packet lsp` when a schema file is opened.
