# r/rust, egui "Show and tell", users.rust-lang.org

DRAFT – not posted.

## Before you post

- r/rust: read the sidebar rules and the moderators' post on project announcements
  ("No more code dumps", linked from rust-lang/this-week-in-rust issue #8575) on the
  day. I could not open Reddit from here, so the rules are not checked in this draft.
  What the TWiR issue says for sure: since issue 664, **TWiR takes its
  Project/Tooling links only from r/rust posts that follow the subreddit's
  guidelines** – this post is also the TWiR submission.
- The Rust community is sceptical of LLM-generated projects (see the TWiR README and
  the "Crate of the week quality rules" thread on users.rust-lang.org). Disclose the AI
  use in the post itself, not only when asked, and write the post in your own words
  – this draft is a starting point.
- Make it about Rust: what you learned, numbers, code. Not a feature list.
- Flair: the project flair of the subreddit.
- Link the blog post if it is out; otherwise the repository.
- Stay in the thread for the first hours; answer criticism of the AI use plainly.

## r/rust

**Title**

> OpenECraft: an EPLAN-style electrical CAE in Rust + egui, native and in the browser – notes on a 24-crate workspace, one Draw trait for screen/PDF/DXF, and getting wasm from 61 to 32 MB

Shorter alternative:

> I built an electrical CAE (control cabinet design) in Rust that runs natively and as wasm – what worked

**Body**

OpenECraft is an open source (MIT/Apache-2.0) electrical CAE: the kind of tool used to
design the control cabinets of machines – schematics where a contactor is one device
across pages, generated reports (bill of materials, terminal diagrams, PLC tag
tables), and the cabinet laid out in 3D with the wires routed through the ducts.

- Web editor (no install): https://editor.openecraft.org
- Website with live demos built from the editor's crates: https://openecraft.org
- Code: https://github.com/nilres/OpenECraft
- Write-up: [blog-url]

**Disclosure first:** the project is a month old and most of the code was written with
AI coding agents (Claude Code); 338 of 468 commits carry the co-author trailer. [Your
own words on your role: domain, specification, review.] I'm posting because some of
the engineering around that may be interesting here, not to pretend I typed 250k
lines.

Rust bits I'd like feedback on:

1. **Headless core, thin front ends.** `openecraft-core` (~120k lines) has no UI
   dependency: project model, netlist, checks, reports, 3D layout and routing, SVG / PDF
   / DXF / DWG export. Its dependencies are serde, serde_json, miniz_oxide, sha2, png,
   acadrust (DWG) and the egui font files; PDF writer, TTF subsetter, XML / ZIP / XLSX
   are small in-house modules. The editor, CLI, MCP server, two servers, a parts
   manager and the website sit on top – 24 crates.
2. **Layering inside one crate, enforced by a test.** A table (`CORE_RULES`) lists for
   every core module which modules it may use; a test parses the `crate::` paths. New
   edges fail `cargo test`. Cheaper than splitting the crate (a clean
   `cargo check -p openecraft-core` takes 16 s on my machine [check this number]).
3. **One `Draw` trait** (`line`, `poly`, `circle`, `text`, plus defaulted `link`,
   `layer`, `image`) implemented by the egui canvas, SVG, PDF and DXF. Defaults let a
   back end ignore what it can't express.
4. **A plugin registry as the single source of commands.** ~300 commands are
   `Spec { name, help, params, run: fn(&mut Project, &Args) -> Result<Value, String> }`
   tables; the CLI, the MCP server (`openecraft-cli mcp`) and the editor's command
   pane are generated from them. External plugins are any process speaking JSON on
   stdin/stdout.
5. **wasm size.** 61 MB → 32 MB (8 MB Brotli): 16 of 20 language catalogs fetched on
   demand behind a cargo feature, one serialisation format instead of two, a `web`
   profile with `opt-level = "s"`, fat LTO, `codegen-units = 1` (7 % slower routing,
   so the desktop keeps `release`), pre-compressed files served by nginx. What else
   would you try?
6. **Threads in the browser.** The 3D routing search takes seconds; on the web it runs
   in a worker hosting a second instance of the same module; the routings of the
   example projects are computed at build time.
7. **No font fallback in native egui.** A test scans every string literal of the
   workspace against the bundled fonts; CJK uses Noto Sans subsets cut to the
   characters of the translations (299 / 249 / 111 KB).
8. **i18n:** ~8,900 texts, 20 languages, `tr()` / `tr_fmt!` / `tr_const!` checked against TSV
   catalogs by tests; plain literals in widgets fail a test.

Honest state: schematics, reports, checks and exports are the solid part; 3D routing
in buildings, floor plan scans and STEP/IGES import (monstertruck kernel) are young.
Bug reports welcome.

## egui discussions – "Show and tell" (github.com/emilk/egui/discussions)

**Title:** OpenECraft – electrical CAE (schematics, 3D panel layout) in egui, desktop + web

OpenECraft is an open source electrical CAE editor built on eframe 0.37 and
`egui_tiles` – schematic pages, a ribbon, docking panes and a 3D view of the control
cabinet, natively on Windows / Linux / macOS and as WebAssembly.

- Try it: https://editor.openecraft.org
- Code: https://github.com/nilres/OpenECraft

egui-specific things in it that might be useful to others:

- The 3D view draws through `egui_glow` (OpenGL 3 / WebGL 2, its own framebuffer with
  multisampling and depth, a stencil pass for wires seen through cable ducts) and falls
  back to a CPU projection painted with egui shapes.
- Ribbon, quick access row and backstage are built from tables of commands
  (`openecraft-shell`), and a test checks every command has a place in the shell.
- CJK: subsets of Noto Sans added as fallback fonts, and a test that checks every string
  literal against the bundled fonts (native builds draw missing glyphs as boxes).
- Trackpad vs. mouse wheel handling with user settings (pinch, two-finger pan, a
  "Windows trackpad" heuristic).
- Web: a loading page with download progress, a worker with a second instance of the
  module for heavy work.

Thanks for egui – [one personal line]. Most of the code was written with AI coding
agents; feedback on the egui usage is welcome.

## users.rust-lang.org – category "community" / announcements

Check the category on the day (announcements may need a trust level). Keep it short:

**Title:** OpenECraft – an open source electrical CAE in Rust (desktop and wasm)

OpenECraft is an EPLAN-style electrical CAE for designing control cabinets: device-
oriented schematics, generated reports, PLC tag tables, a 3D panel layout with wire
routing, a command line and an MCP server. 24 crates, egui/eframe on the desktop and
in the browser, MIT or Apache-2.0.

Web editor: https://editor.openecraft.org · Code: https://github.com/nilres/OpenECraft
· Write-up: [blog-url]

It is a month old and most of the code was written with AI coding agents under my
direction [adjust]; the write-up describes the tests that keep it in shape. Issues and
criticism welcome.
