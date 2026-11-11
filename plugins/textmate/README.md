# TextMate Bundle

This bundle holds the syntax of ProtoPacket schema files, for highlighting.

It has the layout of a VS Code extension, which JetBrains IDEs & VS Code both read.

- `package.json`: the language & its file extension.
- `language-configuration.json`: the comments & the brackets.
- `syntaxes/pps.tmLanguage.json`: the grammar.

## JetBrains IDEs

Add this folder in `Settings | Editor | TextMate Bundles` to highlight the `.pps` files.
