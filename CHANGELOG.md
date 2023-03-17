# Change Log

All notable changes will be documented in this file.

<!-- Check [Keep a Changelog](https://keepachangelog.com/) for recommendations on how to structure this file. -->

## v0.2.0

- feat(*): port to tower lsp
    - This will allow the usage of this LS by other IDEs.
    - The vscode extension will prompt you to download the server.
    - Other IDEs will have to have the LS installed via `cargo install`.
- feat(*): add configuration options
- feat(server, #6): diagnostic for unused rules
- feat(server, #7): show rule docs (`///`) on hover
- fix(server, #8): solve issue relating to 0 vs 1 indexing causing diagnostics to occur at the wrong locations
- feat(server): add a version checker
- feat(readme, #2): update readme and add demo gif
- feat(ci, #4): automatically populate changelog
- fix(ci): lint all rust code

## v0.1.2

- feat: upgrade pest v2.5.6, pest-fmt v0.2.3. See [Release Notes](https://github.com/pest-parser/pest/releases/tag/v2.5.6).
- fix(server): solve issue relating to 0 vs 1 indexing.
- feat(server): suggest user-defined rule names in intellisense.

## v0.1.1

- feat(server): add hover information for `SOI` and `EOI`
- fix(ci): allow the release workflow to create releases.
- fix(vscode): add a readme for the vscode extension.
- fix(vscode): add a changelog.

## v0.1.0

- Initial release
