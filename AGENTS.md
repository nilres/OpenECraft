# AGENTS.md

Guidelines for AI coding agents (and humans) working on OpenECraft – an EPLAN-style
electrical CAE editor in Rust (egui/eframe) that runs natively on Windows, Linux,
macOS and in the browser (WebAssembly).

## Build, test, lint

Run these before considering a change done. All of them must pass without warnings.

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo clippy -p openecraft -p openecraft-parts-manager --target wasm32-unknown-unknown
```

- Desktop: `cargo run --release -p openecraft`
- Parts server: `cargo run -p openecraft-parts-server -- --token <secret>` (http://127.0.0.1:8090)
- Project server: `cargo run -p openecraft-project-server -- --token <secret>` (http://127.0.0.1:8091)
- Both servers also take `--config <file.toml>` with the settings of their options
  (`parts-server/parts-server.example.toml`, `project-server/project-server.example.toml`);
  a new option gets a key in the server's `settings.rs` `File` too
- Parts manager: `cargo run -p openecraft-parts-manager`; web: `cd parts-manager && trunk serve` (http://127.0.0.1:8082)
- Command line / MCP server: `cargo run -p openecraft-cli -- help`, `cargo run -p openecraft-cli -- mcp`
- Web: `cd openecraft && trunk serve` (dev server on http://127.0.0.1:8080), `cd openecraft && trunk build --release`
- Windows installer: `powershell -ExecutionPolicy Bypass -File installer/windows/build.ps1`
  (Inno Setup 6, script `installer/windows/openecraft.iss`, output `target/installer/`);
  installer texts are in all four languages too (`[CustomMessages]`)
- Linux packages: `bash installer/linux/build.sh [tar.gz] [deb] [rpm] [appimage] [flatpak] [snap]`
  (desktop entries, MIME types and AppStream metainfo in `installer/linux/`, translated
  into all four languages; a new version needs a `<release>` in the metainfo)
- macOS disk image: `bash installer/macos/build.sh` (universal app bundles, output
  `target/installer/`)
- The installer scripts and `trunk build --release` need cargo-about for the third-party
  licenses (`cargo install cargo-about@0.9.3 --locked --features cli`, configuration
  `installer/about.toml`); the installer scripts take the version from
  `[workspace.package]` of the root `Cargo.toml`, which every crate inherits
  (`version.workspace = true`)
- Releases: a tag `v<version>` builds a draft GitHub release (`docs/releasing.md`);
  `CHANGELOG.md` gets an entry for user-visible changes
- The web target needs `rustup target add wasm32-unknown-unknown` and `cargo install trunk --locked`.

## Workspace layout

| Crate | Path | Purpose |
| --- | --- | --- |
| `openecraft` | `openecraft/` | Editor (desktop + web), built by trunk (own `index.html` / `Trunk.toml`) |
| `openecraft-core` | `crates/openecraft-core` | Headless project model: pages, symbols, devices, netlist, checks, reports, panel layout, SVG / PDF / DXF / DWG export |
| `openecraft-parts` | `crates/openecraft-parts` | Shared library: part model, function templates, CSV/JSON, catalog, EPLAN product groups, parts server API types, data portal rules (faceted search, lifecycle, update check), role based access control of both servers (`access`) |
| `openecraft-i18n` | `crates/openecraft-i18n` | Languages and UI translations (English, German, French, Spanish and 16 further languages); no dependencies |
| `openecraft-fonts` | `crates/openecraft-fonts` | Bundled Chinese, Japanese and Korean fonts (Noto Sans subsets, `subset.py`) and the order of the fallback fonts, used on screen (`openecraft-ui`) and in PDF exports (`openecraft-core`); no dependencies |
| `openecraft-step` | `crates/openecraft-step` | STEP, IGES and Parasolid import (B-rep → part mesh) with the `monstertruck` CAD kernel (Parasolid parsed by `parasolid-core`); kept out of `openecraft-parts` because of its size |
| `openecraft-edz` | `crates/openecraft-edz` | EDZ import (EPLAN Data Portal archives): unpacks the 7-Zip archive and reads its parts XML with `openecraft-parts`; kept out of `openecraft-parts` because of the 7-Zip dependency |
| `openecraft-jt` | `crates/openecraft-jt` | JT writer (panel layout export) and reader (part models) for triangle meshes |
| `openecraft-xt` | `crates/openecraft-xt` | Parasolid XT writer (panel layout export): meshes as faceted B-rep bodies; no dependencies |
| `openecraft-api` | `crates/openecraft-api` | Extension API: `Plugin` trait, registry of commands, checks, reports and exporters, external (process) plugins, built-in plugins, shared command parameters |
| `openecraft-auth` | `crates/openecraft-auth` | Access control of the servers: password hashes (Argon2), session tokens, throttled sign-in, JSON error answers, Active Directory / LDAP sign-in, access settings file (roles and grants: `openecraft_parts::access`), configuration files of the servers (`server_config`, TOML) |
| `openecraft-client` | `crates/openecraft-client` | Requests to the parts and project servers and to Autodesk Vault (`vault`) and decoding of their answers (ehttp), shared by the UI client and the command line; `eplan` wraps the EPLAN Data Portal API as a read-only parts server |
| `openecraft-ui` | `crates/openecraft-ui` | Shared egui library: part tree / editor / previews, parts server client, file dialogs, language menu |
| `openecraft-parts-server` | `parts-server/` | Data portal server (axum; SQLite or JSON storage, users and roles, catalogs, files, history, web page) |
| `openecraft-project-server` | `project-server/` | Project server: shared projects, page locks for several users (axum, project files) |
| `openecraft-cli` | `cli/` | Command line and MCP server (AI agents) on top of `openecraft-core` |
| `openecraft-parts-manager` | `parts-manager/` | Standalone parts server maintenance app (desktop + web, own `index.html` / `Trunk.toml`) |

- `docs/architecture.md` describes the crate graph, the layers inside `openecraft-core`
  and where a new feature goes; `openecraft/src/conventions/tests.rs` enforces the layer
  rules (`CORE_RULES`). Extend the table when you add or layer a module.
- Code needed by more than one executable belongs in a library crate under `crates/`,
  never duplicated.
  - `openecraft-parts` stays free of UI and platform dependencies (only `serde` /
    `serde_json`); pure logic such as 3D math (`vec3`) goes here.
  - `openecraft-core` holds everything about a project that needs no window; the editor
    only adds the UI (`app`, `editor`, `panel_ui`, `parts_ui`, `io`). It must not depend
    on egui or an application crate.
  - `openecraft-ui` holds egui widgets and platform code shared by the editor and the
    parts manager. It must not depend on an application crate.
  - `openecraft-api` depends only on `openecraft-core`, `openecraft-i18n`, `serde` and
    `serde_json`: no UI, no application crate, no `cfg(target_arch)` branches.
- API types and request paths of the parts server live in `openecraft_parts::api`, the
  requests in `openecraft-client`; change server and client together. The same holds
  for the project server with `openecraft_core::collab`, which also holds the merge and
  lock rules of shared projects.
- New crates are added to `[workspace] members` in the root `Cargo.toml`; the
  convention, glyph and catalog tests read that list, so their rules apply to every member.

## Extensibility – mandatory for every feature

OpenECraft is extended through `openecraft-api` (see `docs/plugins.md`). Plugins add
commands, checks, reports and exporters to a `Registry`; the editor, the command line
and the MCP server take them from `Plugins`, so an extension is available everywhere
at once. New features must use and grow this system instead of bypassing it:

- A new check, report, export or function on the project is registered as an extension
  of a built-in plugin (`crates/openecraft-api/src/builtin.rs`, one plugin per area),
  not hard-wired into one front end. The built-in checks, report tables and exports of
  core are registry entries too (`openecraft_core::{checks, reports, exporters}::builtin`,
  published by the plugins `project_checks`, `project_reports`, `project_exports`):
  a new built-in one is a new entry there. Front ends never special-case a built-in
  plugin; they only show what `Plugins` offers.
- When a feature needs something a plugin cannot do yet (a new kind of extension such
  as symbols, importers or project events, or more data for an existing one), add the
  extension point to `Registry` first, consume it in every front end (editor, command
  line / MCP) and in the external protocol (`external.rs`, `docs/plugins.md`), then
  build the feature on it. Built-in and third-party plugins have the same powers.
- Command parameters use `openecraft_api::params` (`param`, `optional`, `Kind`); the
  built-in tools of `cli/` share it. Names of plugins, commands, checks, reports and
  exporters are snake_case and unique (`Plugins::add` refuses duplicates).
- Changes to the external protocol stay backwards compatible; a breaking change raises
  `API_VERSION`. Update `docs/plugins.md` with every change of the API.
- Extensions are tested like other code: register them in a `Plugins` in the test and
  run them through it (see `crates/openecraft-api/src/host/tests.rs`).

## Languages – mandatory for every feature

The app is available in **English, German, French and Spanish**. All four are required:
a feature, menu entry, dialog, message, tooltip, report heading or placeholder text is
not done until it is available in all of them.

It is also available in 16 further languages (`Lang::ALL`): Portuguese (Brazil), Italian,
Dutch, Polish, Czech, Russian, Ukrainian, Turkish, Swedish, Danish, Finnish, Hungarian,
Romanian, Chinese (Simplified), Japanese and Korean. Each has a catalog file in every
area folder, `crates/openecraft-i18n/src/catalog/<area>/<code>.tsv`
(`English<TAB>translation`); a text without a line there is shown in English. Add the
new texts of a feature to them too when you can (same placeholders and leading /
trailing spaces; the tests check this). Chinese, Japanese and Korean use the bundled
fonts of `openecraft-fonts`: after a translation into one of them gained characters,
run `crates/openecraft-fonts/subset.py` (see its header) or
`openecraft/src/glyphs/tests.rs` fails. A new language is a `Lang` variant, a
`<code>.tsv` in every area folder listed in `catalog.rs` and, for a script the bundled
fonts lack, a font in `openecraft-fonts`. Scripts written right to left or needing
complex shaping (Arabic, Hebrew, Indic scripts, Thai) cannot be drawn by egui.

- Write texts in English in the code and pass them through `openecraft_i18n`:
  `tr("Open")` for plain texts, `tr_fmt!("Saved {}", name)` for texts with `{}`
  placeholders (the translation may reorder with `{0}`, `{1}`), `tr_const!("…")` in
  `const` / `static` tables (translate with `tr(…)` where the text is shown), and
  `openecraft_core::issue!(severity, target, "Cable {} defined {} times", tag, n)` for
  the issues of a check (the template and its arguments are kept, `Issue::message()`
  translates where the issue is shown; plugin and generated texts use `Issue::plain`).
  Never build sentences from translated fragments (`"Macro " + tr("saved")`); translate
  the whole sentence, one catalog entry per variant.
- Every English text needs a line in `crates/openecraft-i18n/src/catalog/<area>/required.tsv`
  (`English<TAB>Deutsch<TAB>Français<TAB>Español`; `\n` and `\"` as in Rust source;
  `{}` placeholders must appear in every translation). The catalog is split by area,
  the crate group that shows the text (`editor`, `core`, `plugins`, `cli`, `ui`, `parts`;
  `common` for texts shown by several, see `openecraft_i18n::scan`), and every file is
  sorted by its English text, so that branches seldom change the same lines. Do not
  place or sort by hand: add the lines to any catalog file and run
  `cargo run -p openecraft-i18n --example tidy`, which moves each text to the file of
  its area and sorts the files (also the way to resolve a merge conflict in them). The
  catalog is checked by `cargo test`: all four languages present, placeholders equal,
  files sorted and texts in the file of their area, every literal passed to `tr` /
  `tr_fmt!` / `tr_const!` / `issue!` and every built-in symbol name is in the catalog,
  and all translated characters exist in the bundled fonts.
- `openecraft/src/conventions/tests.rs` fails for plain string literals passed to widgets
  (`ui.button("…")`, `.label("…")`, `.on_hover_text("…")`, `Window::new("…")`, …).
- Translate at the point where the text is shown, so that switching the language in
  View › Language (or the parts manager's toolbar) takes effect immediately. Texts
  that are stored in the project (user input, names of built-in forms) stay as they are;
  generated content (reports, title blocks, check messages) is translated when it is
  generated, `App` regenerates it when the language changes.
- Do not translate file formats, CSV headers of machine readable exports, JSON keys,
  part data and log messages.
- Language choice: stored by the apps (`language` key), otherwise the OS / browser
  language if supported, otherwise English (`openecraft_i18n::init`).

## Unit tests – mandatory layout

**Unit tests never live inline in a module file.** Each module keeps its tests in a
separate file next to it:

```
openecraft/src/geom.rs        ← module code, ends with:
                            #[cfg(test)]
                            mod tests;
openecraft/src/geom/tests.rs  ← the tests, starting with `use super::*;`
```

Rules (for every crate of the workspace):

1. No `#[test]` functions and no `mod tests { … }` blocks in module files.
2. The module file declares its tests exactly as `#[cfg(test)]` followed by `mod tests;`
   (at the end of the file).
3. The test file is `src/<module>/tests.rs` and starts with `use super::*;` (it is a
   child module, so private items stay testable).
4. Test-only modules (e.g. `glyphs`, `conventions`) are declared in `main.rs` with
   `#[cfg(test)]` and follow the same layout: helpers in `src/<module>.rs`, tests in
   `src/<module>/tests.rs`.
5. When adding a new module with tests, create `src/<module>/tests.rs` right away – do
   not add an inline block "for now".
6. New behaviour gets a test. Bug fixes get a regression test.

These rules are enforced by `openecraft/src/conventions/tests.rs` – `cargo test` fails on
violations. Do not weaken or delete that test to make a change pass.

## Other conventions

- **Glyphs:** every character used in a string literal must exist in the bundled
  egui fonts. Native builds have no font fallback, so missing characters render as
  empty boxes (the browser hides the problem). `openecraft/src/glyphs/tests.rs` checks this for
  every workspace member; if
  it fails, pick a character the fonts contain instead of disabling the check. Keep
  the egui feature `monochrome_emoji_fonts` enabled.
- **eframe features:** default features are disabled in both app manifests; when changing
  the feature list keep `clipboard` (desktop copy/paste), `persistence`,
  `default_fonts` and `glow`.
- **Demo and example projects:** `crates/openecraft-core/src/sample.rs` (demo) and the
  example projects of `sample::EXAMPLES` (`sample/<example>.rs`, simplest first) are
  checked by `sample/tests.rs` (no check errors, no warnings). Keep them consistent when
  changing symbols, parts or checks. A new example gets the next level, an entry in
  `EXAMPLES` (title and description in the catalogs), a test of the features it shows
  and a row in the README table.
- **Rendering:** draw through the `render::Draw` trait so the canvas and SVG / print
  export stay identical.
- **Undo:** edits are captured by snapshot comparison (`history.rs`); just mutate the
  `Project`, no explicit undo bookkeeping is needed.
- **Platform code:** keep `#[cfg(target_arch = "wasm32")]` branches confined to
  `crates/openecraft-ui/src/io.rs` and the `main.rs` files.
- **Command line / MCP:** every operation on the project is a command of a built-in
  plugin in `crates/openecraft-api/src/builtin/<area>.rs` (an entry of the area's
  `COMMANDS` table: name, help text, parameters, `fn(&mut Project, &Args)`); the command
  line, the MCP server and the editor's Plugins tab take it from there, so add new
  functions there once. Only tools that need the session or the machine (project files,
  undo, exports to files, the parts server, plugins) are `Tool`s in
  `cli/src/tools/<area>.rs`. Help texts use `tr_const!` and need catalog entries; MCP
  clients get them in English.
- **Editor shell:** every function of the editor is a command of
  `openecraft/src/commands.rs` with a place in the ribbon, the backstage or View › Panes
  (`openecraft/src/ribbon.rs`); a tool is a pane (`ToolPane` in `openecraft/src/dock.rs`)
  shown by a command. The tests of both modules fail for a command without a place
  or a pane without a command. Icons are vector paths in `openecraft_ui::icons`
  (no new glyphs); state colours come from `openecraft/src/style.rs`.
- **Parts manager:** keep its editing rules (drafts, renames, unsaved changes) in
  `parts-manager/src/session.rs` (no UI code, unit tested); `app.rs` only draws.
- **Style:** match the surrounding code; keep comments as sparse as the existing ones.

## Commits

- Commit only when asked. Use an imperative summary line and a short body listing
  the changes.
- Never commit IDE folders (`.idea/` is ignored) or build output (`target/`, `dist/`).
