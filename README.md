# Pest IDE Tools

_IDE support for [Pest](https://pest.rs), via the LSP._

This repository contains an implementation of the _Language Server Protocol_ in
Rust, for the Pest parser generator.

<p align="center">
  <img src="demo.gif" alt="A demo of the Pest VSCode extension." />
</p>

## Features

- Error reporting.
- Warnings for unused rules.
- Syntax highlighting definitions available.
- Rename rules.
- Go to rule declaration, definition, or references.
- Hover information for built-in rules and documented rules.
- Autocompletion of rule names.
- Inline and extract rules.
- Full-unicode support.
- Formatting.
- Update checking.

Please see the
[issues page](https://github.com/pest-parser/pest-ide-tools/issues) to suggest
features or view previous suggestions.

## Usage

You can find documentation on how to set up the server for in the `DOCS.md`
file.

### Supported IDEs

- [Visual Studio Code](https://marketplace.visualstudio.com/items?itemName=pest.pest-ide-tools)
  - VSCode has a pre-built extension that can compile, update, and start up the
    language server. It also includes syntax highlighting definitions.
  - The extension is also available on [OpenVSX](https://open-vsx.org/extension/pest/pest-ide-tools)
- Sublime Text
  - Sublime Text packages can be obtained from [the latest release](https://github.com/pest-parser/pest-ide-tools/releases/latest)
- Vim
  - Vim support is provided via the official [pest.vim](https://github.com/pest-parser/pest.vim) package.

Due to the usage of the LSP by this project, adding support for new IDEs should
be far more achievable than a custom implementation for each editor. Please see
the [tracking issue](https://github.com/pest-parser/pest-ide-tools/issues/10) to
request support for another IDE or view the current status of IDE support.

## Development

This repository uses a [Taskfile](https://taskfile.dev); install the `task`
command for a better experience developing in this repository.

The task `fmt-and-lint` can be used to check the formatting and lint your code
to ensure it fits with the rest of the repository.

In VSCode, press `F5` to build and debug the VSCode extension. This is the only
method of debugging that we have pre set-up.

### Architecture

The server itself is implemented in Rust using `tower-lsp`. It communicates with
editors via JSON-RPC through standard input/output, according to the language
server protocol.

### Contributing

We appreciate contributions! I recommend reaching out on Discord (the invite to
which can be found at [pest.rs](https://pest.rs)) before contributing, to check
with us.

## Credits

- [OsoHQ](https://github.com/osohq), for their
  [blog post](https://www.osohq.com/post/building-vs-code-extension-with-rust-wasm-typescript),
  and open source code which was used as inspiration.
- [Stef Gijsberts](https://github.com/Stef-Gijsberts) for their
  [Pest syntax highlighting TextMate bundle](https://github.com/Stef-Gijsberts/pest-Syntax-Highlighting-for-vscode)
  which is used in this extension under the MIT license.
