# Contributing to OpenECraft

Thank you for helping. Bug reports, parts and symbols, translations, documentation and
code are all welcome. The rules below are the short version of [AGENTS.md](AGENTS.md),
which holds every convention of the code base (for people and AI agents alike).

## Reporting bugs and asking for features

- **Bugs:** open an issue with the *Bug report* form. Name the version
  (*Help › About OpenECraft*, or `openecraft-cli --version`), the platform and whether
  it happened in the desktop app or the web editor. A small project file (`.oec`,
  `.oecz`) that shows the problem helps most.
- **Features:** open an issue with the *Feature request* form and describe the task you
  want to get done, not only the button you are missing. How EPLAN, AutoCAD Electrical
  or another tool does it is a useful hint.
- **Parts and catalogs:** the *Parts or catalog request* form asks for the
  manufacturer, the part numbers and the public data sheets. The bundled catalogs
  (`crates/openecraft-parts/catalogs/`) are compiled from public data sheets only.
- For larger changes, open an issue first, so that the design can be agreed before the
  work starts.

## Building and testing

A recent stable Rust toolchain (`rustup update stable`):

```bash
cargo run --release -p openecraft          # the editor
cargo run -p openecraft-cli -- help        # command line, `-- mcp` for the MCP server
cd openecraft && trunk serve               # web version on http://127.0.0.1:8080
```

The web version needs `rustup target add wasm32-unknown-unknown` and
`cargo install trunk --locked`; on Linux the desktop build needs the window libraries
listed in [installation](docs/installation.md). Before a change is done, these must
pass **without warnings** (CI runs them with `-D warnings`):

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo clippy -p openecraft -p openecraft-parts-manager -p openecraft-website --target wasm32-unknown-unknown
```

## Conventions

- **Where code goes:** [architecture](docs/architecture.md) describes the crates and
  their layers. Project logic without a window belongs in `openecraft-core`, code used
  by more than one program in a library crate under `crates/`, never copied.
- **Extensions first:** a new check, report, export or command on the project is an
  entry of a built-in plugin (`crates/openecraft-api/src/builtin/`), so the editor, the
  command line and the MCP server get it at once ([plugins](docs/plugins.md)).
- **Four languages:** every text the user sees exists in **English, German, French and
  Spanish**. Write it in English, pass it through `tr("…")` / `tr_fmt!("… {}", x)`,
  add a line `English<TAB>Deutsch<TAB>Français<TAB>Español` to a
  `crates/openecraft-i18n/src/catalog/<area>/required.tsv` and run
  `cargo run -p openecraft-i18n --example tidy`. Translations into the 16 further
  languages (`<code>.tsv` in the same folders) are welcome too. Never build a sentence
  from translated pieces.
- **Tests:** new behaviour gets a test, a bug fix a regression test. Tests never live
  inline: a module `src/foo.rs` ends with `#[cfg(test)] mod tests;` and its tests are in
  `src/foo/tests.rs`, starting with `use super::*;`. `cargo test` enforces this.
- **Glyphs:** every character in a string literal must exist in the bundled fonts
  (native builds have no fallback); `cargo test` checks it.
- **Editor functions** are commands with a place in the ribbon
  (`crates/openecraft-shell/src/commands.rs`, `ribbon.rs`), icons are vector paths in
  `openecraft_ui::icons`.
- **Style:** match the surrounding code, keep comments as sparse as the existing ones.

## Pull requests

- One topic per pull request, with a short description of what changes for the user
  and how you tested it.
- User-visible changes get an entry in the `unreleased` section of
  [CHANGELOG.md](CHANGELOG.md); documentation in `docs/` is updated with the feature.
- Commit messages: an imperative summary line and a short body listing the changes.
- Do not commit build output (`target/`, `dist/`) or IDE folders.

## License

OpenECraft is licensed under MIT or Apache-2.0, at your option. Unless you state
otherwise, any contribution you submit is licensed the same way, without additional
terms or conditions.
