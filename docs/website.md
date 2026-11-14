# Website

`website/` is the public page of the project: what OpenECraft does, shown with live
demos instead of screenshots. The demos are the editor's own code compiled to
WebAssembly, not copies of it:

| On the page | Runs |
| --- | --- |
| Schematic editor (select, move, wire, place symbols, undo) | `openecraft_canvas::editor`, the page editor of the editor |
| Panel in 3D / mounting plate, with the routed wiring; the enclosures, the rooms (floor plan) and the site (site plan) of an example to switch between | `openecraft_canvas::panel_ui`, the panel layout view of the editor |
| Project check, counts, undo and redo | `openecraft_session::session::Session` with the checks of the plugins |
| Reports | The reports of `openecraft_api::Plugins` |
| Console | The commands of the plugins, parameters parsed as on the command line (`openecraft_core::params::cli_words`, `from_cli`) |
| Exports (PDF, DXF, DWG, STEP, …) | The exporters of the plugins, run in the browser |
| Sheet of the first screen | `openecraft_core::svg`, the SVG export of the page being edited |
| Languages | `openecraft_i18n`: the canvases, the generated texts and the page itself |
| Examples | `openecraft_core::sample::EXAMPLES` |

All of it works on one project in the browser (`src/studio.rs`, the `Studio`): a wire
deleted in the schematic shows in the check, in the reports and in the next export.

## Parts

- `index.html`, `site.css`, `site.js`: the page. Its texts are written in English
  (`data-t`, `data-t-placeholder`, `t("…")` in the script) and listed in `src/texts.rs`,
  whose translations the page loads from the WebAssembly; a test compares the table
  with the page. They belong to the catalog area `website`
  (`crates/openecraft-i18n/src/catalog/website/`).
- `src/studio.rs`: the project and the functions the page calls by name
  (`Studio::call`, answers as JSON). No UI code, unit tested.
- `src/views.rs`: draws a canvas the way the editor does around its views (keys and
  gestures, the analysis before, the undo step after).
- `src/main.rs`: the exports for the page (`call`, `export`, `start`). The native build
  is a tool for working on the demos: `cargo run -p openecraft-website` shows both
  canvases in a window, `cargo run -p openecraft-website -- call state` prints an
  answer as the page gets it, `… -- export pdf` writes the file of an exporter.

Two settings are `<meta>` elements in `index.html`: `openecraft-editor`, where the
links to the web editor lead (`editor/` beside the page, as in the Docker image; any
other address; empty hides the links), and `openecraft-repository`, the address of
the source repository for the links to releases and documentation.

## Development

```bash
cd website && trunk serve --release     # http://127.0.0.1:8084
```

The debug build works too but takes long to load the first project; `--release` needs
cargo-about for the license files, like the other web builds
([installation](installation.md#web-webassembly)).

## Deployment by hand

```bash
cd website && trunk build --release     # static files in website/dist/
```

Copy `website/dist/` to any web server. The addresses in the page are relative, so it
works in a folder as well as at the root of a host. The server has to send
`application/wasm` for `.wasm`; the files named after a hash can be cached for long,
`index.html`, `site.js` and the pictures should be checked at every visit
(`installer/docker/website.conf` does this for nginx).

For the links to the web editor, either put a build of the editor into the folder
`editor/` beside the page

```bash
cd openecraft && trunk build --release --public-url ./ --dist ../website/dist/editor
```

or set the `openecraft-editor` element to the address of a web editor hosted
elsewhere.

## Docker image

`ghcr.io/nilres/openecraft-website` (same tags as the other images) serves the page
with nginx on port 8080, as an unprivileged user, with the web editor at `/editor/`:

```bash
docker run -d --name openecraft-website -p 127.0.0.1:8088:8080 ghcr.io/nilres/openecraft-website
docker build -f installer/docker/Dockerfile --target website -t openecraft-website .
```

`installer/docker/compose.yaml` has it as the service `website`. The nginx site is
`installer/docker/website.conf` (mount your own at `/etc/nginx/conf.d/default.conf`);
put a TLS reverse proxy in front of it for a public site. The page needs no server
side: nothing a visitor does leaves the browser.
