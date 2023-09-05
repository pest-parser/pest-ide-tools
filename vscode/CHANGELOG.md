# Change Log

All notable changes will be documented in this file.

<!-- Check [Keep a Changelog](https://keepachangelog.com/) for recommendations on how to structure this file. -->

## v0.3.4

- [[notpeter](https://github.com/notpeter)] fix(server): off-by-one error (#37)
- [[notpeter](https://github.com/notpeter)] fix(grammar): issues with highlighting rules that started with a built-in's name (#38)

## v0.3.3

- fix(server): hotfix for #28, ranges (along with other tokens like insensitive strings) no longer crash the server

A proper fix for the code causing the crash in #28 will be released, but I do not have the time at the moment.

## v0.3.2

- fix(vscode): update checker is now enabled by default, and some of its logic
  has been modified and fixed. It also supports cancellation of the install task
- fix(vscode): give defaults to all config options
- fix(server): fix crash with code actions

## v0.3.1

- revert(server): revert performance marks temporarily, while they are
  refactored into a more generic crate

## v0.3.0

- feat(server): add performance marks for debugging
- feat(server): simple rule extraction support
- fix(server): validate AST to catch errors like non-progressing expressions

## v0.2.2

- feat(vscode): allow relative paths in `pestIdeTools.serverPath`
- fix(vscode): allow `pestIdeTools.serverPath` to be `null` in the schema
- fix(server): CJK/non-ascii characters no longer crash the server
- fix(server): add a CJK test case to the manual testing recommendations

## v0.2.1

- fix(vscode): scan both stdout and stderr of Cargo commands, fixes some issues
  with installation flow
- feat(*): documentation, issue templates
- feat(sublime): begin publishing a sublime text package
- fix(server, vscode): server now hot-reloads config updates more reliably
- fix(server, vscode): bump problematic dependencies (love the JS ecosystem...a
  CVE a day keeps the doctor away)
- feat(server): add rule inlining code action
- feat(server): ignore unused rule name analysis if there is only one unused
  rule (hack fix)

## v0.2.0

- feat(*): port to tower lsp
  - This will allow the usage of this LS by other IDEs.
  - The vscode extension will prompt you to download the server.
  - Other IDEs will have to have the LS installed via `cargo install`.
- feat(*): add configuration options
- feat(server, #6): diagnostic for unused rules
- feat(server, #7): show rule docs (`///`) on hover
- fix(server, #8): solve issue relating to 0 vs 1 indexing causing diagnostics
  to occur at the wrong locations
- feat(server): add a version checker
- feat(readme, #2): update readme and add demo gif
- feat(ci, #4): automatically populate changelog
- fix(ci): lint all rust code

## v0.1.2

- feat: upgrade pest v2.5.6, pest-fmt v0.2.3. See
  [Release Notes](https://github.com/pest-parser/pest/releases/tag/v2.5.6).
- fix(server): solve issue relating to 0 vs 1 indexing.
- feat(server): suggest user-defined rule names in intellisense.

## v0.1.1

- feat(server): add hover information for `SOI` and `EOI`
- fix(ci): allow the release workflow to create releases.
- fix(vscode): add a readme for the vscode extension.
- fix(vscode): add a changelog.

## v0.1.0

- Initial release
