<!--
DRAFT – not published. Notes for the maintainer (delete before publishing):

- This text was drafted by an AI agent from the repository. This Week in Rust does
  not accept LLM-written articles ("We don't accept articles written by LLMs", README
  of rust-lang/this-week-in-rust), and Hacker News and r/rust readers are sensitive to
  it too. Rewrite it in your own words before you publish it anywhere, keep the facts.
- Every number below was measured on master at ab8ee925 (2026-11-17); measure again
  on the day you publish (commands in posting-plan.md).
- [name], [blog-url] and [your story] are placeholders.
- Length target: 1,200–1,800 words.
-->

# Building an EPLAN-style electrical CAE in Rust that runs in the browser

*[name], [date]*

## What this is, for people who don't wire control cabinets

Every machine on a factory floor has a control cabinet: a steel box with circuit
breakers, contactors, relays, a PLC, power supplies and hundreds of terminals, all
connected by wires. Somebody has to draw those connections before anybody builds the
cabinet, and that drawing is a legal document: it is delivered with the machine, the
service technician reads it twenty years later, the inspector checks it.

The tools for that are called electrical CAE (or ECAD). They are not drawing programs.
A contactor `-K1` is one *device* whose coil sits on page 4 and whose contacts sit on
pages 2, 5 and 7; the program keeps cross references between them, numbers the wires,
knows which terminal of which strip a wire lands on, and generates the bill of
materials, the terminal diagrams, the cable lists and the PLC tag tables from the
schematic. EPLAN Electric P8 is the de-facto standard in Europe; it is Windows-only and
commercially licensed.

[OpenECraft](https://github.com/nilres/OpenECraft) is my attempt at an open source tool
in that style (MIT / Apache-2.0). One Rust code base runs natively on Windows, Linux
and macOS and as WebAssembly in the browser. You can open the
[web editor](https://editor.openecraft.org) without installing anything, or look at the
[website](https://openecraft.org), whose live demos are the editor's own code.

## The numbers, and how it was built

The first commit is from 2026-10-19, a month ago. Since then: 468 commits, three
releases (0.1.0 on 5 November, 0.1.2 on 15 November), 24 crates in one workspace,
about 254,000 lines of Rust in 924 files, of which about 61,000 lines are tests
(around 2,800 `#[test]` functions).

That is not a pace one person types by hand, and I won't pretend it is: most of the
code was written by AI coding agents (Claude Code), with me [your story: what you
decided, what you reviewed, where you come from in electrical engineering]. 338 of the
468 commits carry a `Co-authored-by: Claude` trailer. What I find more interesting than
the speed is what you have to build so that a code base written that fast stays
coherent. Most of this post is about that.

## A headless core and thin front ends

The most important decision was that everything about a project lives in a crate that
knows nothing about windows: `openecraft-core` (about 120,000 lines). Pages, symbols,
devices, the netlist, the checks, the reports, the panel layout in 3D, the routing, and
the exports to SVG, PDF, DXF and DWG are all there. Its dependency list is short:
`serde`, `serde_json`, `miniz_oxide`, `sha2`, `png`, the default egui font files and
`acadrust` for DWG. The PDF writer, the TrueType subsetter for its fonts, the XML, ZIP
and XLSX writers and a QR code generator are small modules of their own.

Around it:

- `openecraft-api` – the plugin registry (more below), no UI, no `cfg(target_arch)`.
- `openecraft-shell`, `openecraft-session`, `openecraft-canvas` – the editor's command
  table and ribbon, the open document with its undo history, and the drawing views.
  They moved out of the editor binary so that the website can run them too.
- `openecraft` (the editor), `cli` (command line and MCP server), `parts-server`,
  `project-server`, `parts-manager`, `website` – the executables.

Inside the core crate the compiler would let any module use any other. A test does not:
`CORE_RULES` lists every module of the core with the modules it may refer to, and
`conventions/tests.rs` parses the `crate::…` paths of each file against it. A new edge
fails the test, so the layering (foundations, symbols, model, evaluation, features,
analysis, output formats, project files) is a reviewed decision, not an accident.

## One `Draw` trait, four back ends

A schematic page must look the same on screen, in the PDF that goes to the customer,
in SVG and in DXF for the people who still live in AutoCAD. So nothing draws directly.
Pages, title blocks, report forms and symbols draw through one trait in
`openecraft_core::render`:

```rust
pub trait Draw {
    fn line(&mut self, a: P, b: P, width: f32, ink: Ink, dashed: bool);
    fn poly(&mut self, pts: &[P], closed: bool, width: f32, ink: Ink, fill: bool);
    fn circle(&mut self, c: P, r: f32, width: f32, ink: Ink, fill: bool);
    fn text(&mut self, pos: P, text: &str, size: f32, h: Anchor, v: VAnchor, ink: Ink);
    /// Marks the area `min`..`max` as a link to another page (PDF export).
    fn link(&mut self, _min: P, _max: P, _page: usize) {}
    /// Layer of the following elements (DXF layers); None = by ink.
    fn layer(&mut self, _name: Option<&str>) {}
    // … styled text, rectangles, images, URLs
}
```

The egui canvas, `SvgDraw`, `PdfDraw` and `DxfDraw` implement it. The defaults matter:
a back end that has no notion of page links or layers simply ignores them, and one that
cannot show a bitmap draws a frame with the file name. Colours are semantic (`Ink`), so
the dark theme on screen and the white paper of the PDF come from the same call.
Tests implement the trait as well, to record what a page draws and assert on it.

## Everything is a registry entry

An EPLAN-style tool has hundreds of functions, and I wanted every one of them to be
reachable from the editor, from a shell script and from an AI agent, without writing
it three times. So functions on the project are entries of a `Registry` in
`openecraft-api`: commands with typed parameters that return JSON, checks, reports
and exporters. The built-in ones are plugins like any other; an area of the project
(pages, devices, PLC, panel layout, wiring, buildings …) is a table:

```rust
pub static COMMANDS: &[Spec] = &[
    Spec {
        name: "list_pages",
        help: tr_const!("Lists the pages of the project"),
        params: &[optional("work_area", Kind::Str, tr_const!("Only the pages of this work area (list_structure)"))],
        run: list_pages,
    },
    // …
];
```

There are about 300 such commands today. The command line exposes each as a
subcommand, `openecraft-cli mcp` offers the same set as tools over the Model Context
Protocol, and the editor lists them in its *Plugins* tab with a generated parameter
form. External plugins are programs in any language that exchange one JSON object per
call on stdin / stdout (desktop and command line only; the browser has no processes).
The website's console is the same registry compiled to WebAssembly.

## egui on the desktop and in the browser

The UI is [egui](https://github.com/emilk/egui) through eframe, with `egui_tiles` for
the docking panes. [one or two sentences of your own on immediate mode for a CAD
tool: the state of the page is the project, no widget tree to keep in sync]

Platform code is confined by convention (and by a test) to one file,
`openecraft-ui/src/io.rs`, plus the `main.rs` files: file dialogs, storage (a file per
key on the desktop, IndexedDB in the browser), downloads. Three things needed more
than a `cfg`:

- **Threads.** The analysis rebuilds incrementally (each page's netlist scan is cached
  by a fingerprint, so an edit of the schematic of the building services example takes
  25 ms instead of 3.8 s), and the checks run on several threads on the desktop. The 3D
  routing of cables is the expensive part: seconds for a building. In the browser it
  runs in a web worker that hosts a second instance of the same WebAssembly module;
  the page hands it the project and adopts the routes it finds. The routings of the
  example projects are computed at build time and shipped next to the page, so an
  example opens in about a second.
- **3D.** The panel view is drawn through `egui_glow` with OpenGL 3 / WebGL 2 (depth
  buffer, multisampling, a stencil pass so that wires show through cable ducts). Without
  either, it falls back to projecting the scene on the CPU.
- **Size.** See the next section.

## Getting the WebAssembly from 61 MB to 32 MB

A CAE tool with 3D import, a 3D router and twenty languages is a big binary, and the
first web build was 61 MB of WebAssembly. Commit #219 took it to 32 MB (8 MB with
Brotli, 12 MB with gzip):

| Step | Effect |
| --- | --- |
| The web build leaves out the catalogs of 16 of the 20 languages (feature `lazy` of `openecraft-i18n`) and fetches `i18n/<code>.tsv` when a language is chosen | part of 61 → 51 MB |
| Editor settings stored as JSON instead of RON, so the web build carries one serialisation format; project and parts files read by their libraries instead of a second copy in the editor | part of 61 → 51 MB |
| A cargo profile `web` for the page only: `opt-level = "s"`, fat LTO, one codegen unit | 51 → 32 MB |
| Compress once at image build time (gzip and Brotli), let nginx send the copies | 24 MB sent before, 8 MB now |

The price: the routing search of the biggest example went from 17.6 s to 18.9 s, which
is why the desktop builds keep the speed-optimised `release` profile. The page also
shows a progress bar now instead of a blank screen. 8 MB is still a lot for a web
page; I'd like to hear from people who got further with `wasm-opt` or by splitting
modules.

## Twenty languages, and no font fallback

The editor is available in 20 languages; English, German, French and Spanish are
mandatory for every text, the other 16 are best effort. There are about 8,900 English
texts in the catalogs. Code writes English and passes it through `tr("…")`,
`tr_fmt!("Saved {}", name)` or `tr_const!` for static tables; check messages are kept as
an English template with arguments (`issue!(…, "Cable {} defined {} times", tag, n)`)
and translated where they are shown, so switching the language does not rerun the
check and MCP clients get a stable key.

Tests enforce it: every literal passed to `tr` or `issue!` must be in the catalog
with all four required translations and equal placeholders; a plain string literal in
`ui.button("…")` fails `cargo test`. A small `tidy` example sorts the catalog files and
moves each text to the file of the crate group that shows it.

One detail that is easy to miss: native egui has no system font fallback. A character the
bundled fonts lack is drawn as an empty box. So a test scans every string and char
literal of every workspace crate against the fonts, and Chinese, Japanese and Korean
use subsets of Noto Sans built from exactly the characters of their translations
(`subset.py`; 299 KB, 249 KB and 111 KB). A new translation that needs a new glyph
fails the test until the subset is rebuilt.

## Tests as guardrails

[If you agree:] the conventions tests are what kept AI-written code workable at this
size. Besides the layer table, the catalog and the glyphs, they check that unit tests
live in `<module>/tests.rs` and never inline, that every command of the editor has a
place in the ribbon and every pane a command (so a feature cannot be unreachable), that
the 16 example projects pass the project check without a warning, and that clippy and
the tests pass on the desktop and the wasm target with `-D warnings`. `AGENTS.md`
describes the rules for humans and agents alike; the tests make sure they are kept.

## What works and what doesn't

Solid enough to try on real work: device-oriented schematics with cross references,
wire and device numbering, terminals, cables, plugs, the reports (bill of materials,
terminal and cable diagrams, connection lists), the project check, PDF / DXF export,
PLC tag tables for TIA Portal, Studio 5000, TwinCAT, CODESYS and others, and the
panel layout with virtual wiring and duct fill.

Young or experimental: the 3D routing through rooms and buildings, floor plans
scanned from paper (walls at right angles only), the STEP / IGES / Parasolid import
(through the `monstertruck` CAD kernel and `parasolid-core`), the collaboration
server, and the national symbol sets (GB, GOST, JIS are partial). There is no import
of EPLAN projects; EPLAN parts, macros and symbols come in through EDZ files. The
bundled manufacturer catalogs are compiled from public data sheets – check them before
you order. The file format is versioned and converted when opened, but the project is a
month old.

If you try it and something breaks, the [issues](https://github.com/nilres/OpenECraft/issues)
are open; [what kind of feedback you want most].

- Web editor: https://editor.openecraft.org
- Website with live demos: https://openecraft.org
- Code: https://github.com/nilres/OpenECraft
