# Pest IDE Tools

_IDE support for [Pest](https://pest.rs), via the LSP._

This repository contains an implementation of the _Language Server Protocol_ for the [Pest](https://pest.rs) parser generator.

## Features

- [x] Error reporting.
- [x] Syntax highlighting.
- [x] Rename support.
- [x] Go to declaration/definition.
- [x] Find references.
- [x] Highlighting a rule highlights it's references.
- [x] Hover information for built-in rules.
- [x] Intellisense/completion of rule names.
- [x] Formatting.
- [ ] Debugging window (as on the Pest website).

## Supported IDEs

- [Visual Studio Code](https://marketplace.visualstudio.com/items?itemName=pest.pest-ide-tools)

Due to the usage of the LSP by this project, adding support for new IDEs should
be far more achievable than a custom implementation for each editor.

### Planned Support

- Sublime.
- Neovim.
- Fleet, once it releases.

## Development

This repository uses a [Taskfile](https://taskfile.dev); I recommend installing the `task` command for a better experience developing in this repository.

The task `fmt-and-lint` can be used to format and lint your code to ensure it fits with the rest of the repository.

### Architecture

The server itself is implemented in Rust using `tower-lsp`. It communicates with editors via JSON-RPC through standard input/output, according to the language server protocol.

## Contributing

We appreciate contributions! I recommend reaching out on Discord (the invite to which can be found at [pest.rs](https://pest.rs)) before contributing, to check with us.

## Credits

- [OsoHQ](https://github.com/osohq), for their [blog post](https://www.osohq.com/post/building-vs-code-extension-with-rust-wasm-typescript), and open source code which was used to originally scaffold the build tools and boilerplate of getting a LS running.
- [Stef Gijsberts](https://github.com/Stef-Gijsberts) for their [Pest syntax highlighting TextMate bundle](https://github.com/Stef-Gijsberts/pest-Syntax-Highlighting-for-vscode) which is used in this extension under the MIT license.
