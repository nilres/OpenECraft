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
- `worker.js`: a worker of the page with a second instance of the module. It
  searches the 3D routing of a project that is opened or shown in another language,
  which takes seconds for a big one and would stop the page (a page has one thread):
  the studio hands it the project (`project`), waits (`Session::await_routing`) and
  takes the routing it found (`routed`, `Cache::adopt_routing`). The page keeps the
  routings by language and project, for the visit and in a cache of the browser
  named after the build of the module (Cache Storage, `openecraft-routing-…`): an
  example opened again is there at once, a changed project is searched. In a browser without
  module workers the page searches the routing itself.
- `routing/` in the built page: the routings of the examples, searched by the build
  (a hook of `Trunk.toml` runs `cargo run --release -p openecraft-core --example
  routings`, release builds only), so that an example is there at once on the first
  visit too. An example whose routing has no translated text has one file for every
  language, the others one per language; `index.json` lists them. The page hands
  the file of an unchanged example to the worker, which takes it only if it is of
  the wiring of the project and searches otherwise.
- `images/`, `social.jpg`, `robots.txt`: the pictures of the gallery (WebP, made
  smaller from the screenshots of `docs/images/` of the same name: make them again
  when a screenshot changes), the preview image of links in chats and social media
  (1200 × 630) and the robots.txt of a page without a public address.
- `social.jpg` is drawn by `social.py`: icon, name and what OpenECraft is beside the
  screenshot `docs/images/panel-3d.png`.
- `sheet.svg`: the sheet of the first screen as a picture, shown until the module is
  loaded and draws it (`cargo run -p openecraft-website -- sheet > website/sheet.svg`;
  a test compares it with the page the demo opens with).
- `apple-touch-icon.png`: the icon of the page on the home screen of an iPhone or
  iPad, which takes no SVG: the drawing of `assets/icon.svg` (the icon of the page
  everywhere else) as an opaque square, drawn by `apple-touch-icon.py`.
- `src/main.rs`: the exports for the page (`call`, `export_file`, `start`, and
  `search_routing` for the worker). The native build
  is a tool for working on the demos: `cargo run -p openecraft-website` shows both
  canvases in a window, `cargo run -p openecraft-website -- call state` prints an
  answer as the page gets it, `… -- export pdf` writes the file of an exporter.

The module is 40 MB (15 MB compressed), so the page is shown first: trunk only names
the module in the page (`pattern_script` of `Trunk.toml`) and `site.js` loads it when
the visitor scrolls, clicks or types, and otherwise once the page is there and the
browser has time. A visitor who asked the browser to save data gets it only on such
a use. Until then the first screen shows `sheet.svg`.

Where the links of the page lead is set in `config.js` beside it, which the page
loads first; the Docker image takes the same settings from environment variables:

| `config.js` | Variable of the image | Leads to | Empty |
| --- | --- | --- | --- |
| `editor` | `OPENECRAFT_EDITOR_URL` | The web editor: a folder beside the page (`editor/`, the default) or any address | Hides the links |
| `repository` | `OPENECRAFT_REPOSITORY_URL` | The source repository | Hides the links |
| `downloads` | `OPENECRAFT_DOWNLOADS_URL` | The downloads | The latest release of the repository |
| `documentation` | `OPENECRAFT_DOCUMENTATION_URL` | The folder with `tutorial.md`, `plugins.md`, `command-line.md` and `installation.md` | The `docs` folder of the repository |
| `issues` | `OPENECRAFT_ISSUES_URL` | Where problems are reported | The issues of the repository |

### Legal notice

Who runs the site, as many countries require of a public one (in Germany the
*Impressum* of § 5 DDG), is set in `legal` of `config.js` or with these variables.
With a name the footer has a link *Legal notice* (*Impressum* in German) that opens
it in a dialog, and `#legal` at the end of the address opens it at once; without a
name there is neither. A line is shown only when its field is set. The address and
the text take several lines, separated by `\n` in `config.js` or by `|`:

| `config.js` | Variable of the image | Shows |
| --- | --- | --- |
| `legal.name` | `OPENECRAFT_LEGAL_NAME` | The person or company running the site; empty hides the notice |
| `legal.address` | `OPENECRAFT_LEGAL_ADDRESS` | Its postal address |
| `legal.email` | `OPENECRAFT_LEGAL_EMAIL` | Email address, as a link |
| `legal.phone` | `OPENECRAFT_LEGAL_PHONE` | Phone number |
| `legal.representative` | `OPENECRAFT_LEGAL_REPRESENTATIVE` | Who represents a company |
| `legal.register` | `OPENECRAFT_LEGAL_REGISTER` | Register court and number |
| `legal.vat` | `OPENECRAFT_LEGAL_VAT_ID` | VAT identification number |
| `legal.responsible` | `OPENECRAFT_LEGAL_RESPONSIBLE` | Who is responsible for the content, with address |
| `legal.text` | `OPENECRAFT_LEGAL_TEXT` | Further text: disclaimers, dispute resolution |
| `legal.privacy` | `OPENECRAFT_LEGAL_PRIVACY` | What the operator adds to the privacy notice (below), lines as in the text |

The labels follow the language of the page; the values are shown as they are set.
Whether a site needs a legal notice, and what it has to say, depends on the operator
and the country: OpenECraft only shows what is set.

### Privacy notice, licenses and disclaimers

The footer always has two more links, each opening a dialog (`#privacy` and
`#licenses` at the end of the address open them at once):

- *Privacy* says what the page itself does, in the language of the page: no cookies,
  no tracking, nothing loaded from other servers, the demos and exports run in the
  browser, the routings kept in the cache of the browser, the IP address every web
  server receives, and that the links lead to other sites. The Docker image logs
  the visits without whole addresses (below). What only the operator
  knows (who hosts the site, what the server logs and for how long, whom to ask, as
  Article 13 GDPR asks for) is set in `legal.privacy` and shown below it. Keep the
  texts of the page true: a change that stores or sends something changes them too.
- *Licenses* links the files the build puts beside the page (`installer/web/licenses.sh`):
  `LICENSE-MIT`, `LICENSE-APACHE`, `OFL-NotoSans.txt` of the bundled fonts and
  `THIRD-PARTY-LICENSES.html` of the libraries, which only a release build writes.

Below the links the footer says that the product names on the page are trademarks of
their owners, with whom OpenECraft is not affiliated, and that it comes without
warranty: its calculations and checks do not replace the verification by a qualified
person.

### What the page may load

The head of `index.html` has a Content Security Policy: scripts, styles, pictures and
requests of the page's own address only (`'unsafe-inline'` for the script trunk
writes into the page, `'wasm-unsafe-eval'` for the WebAssembly), so it holds on any
web server. A page that is to load something from another host needs it named there.
The nginx site of the Docker image adds `Referrer-Policy`, `X-Frame-Options` and
`X-Content-Type-Options`; `Strict-Transport-Security` belongs to the TLS reverse
proxy in front of it.

### Search engines and link previews

The page is plain HTML: search engines read its texts without running the demos.
Its head names the title and description for search results and link previews
(Open Graph, Twitter cards) and describes OpenECraft as structured data
(`SoftwareApplication`, JSON-LD); the pictures of the gallery have texts of their
own. The tags that need the public address of the site (the canonical address, the
address and preview image of a link, `robots.txt` with the sitemap and `sitemap.xml`)
are written by `installer/docker/website-site.sh` from the variable
`OPENECRAFT_SITE_URL` (`https://…/`, the folder of the page), between the markers
`site address` of `index.html`; without it they are left out, as an address in the
wrong place does more harm than none. The Docker image runs the script at every
start. For a page deployed by hand, run it once on the build:

```bash
OPENECRAFT_SITE_URL=https://openecraft.example.com/ sh installer/docker/website-site.sh website/dist website/dist
```

Running it again replaces the tags.

The page has an address in every language: the build writes `de.html`, `fr.html`, …
beside the English `index.html` (a hook of `Trunk.toml` runs `cargo run --release -p
openecraft-i18n --example pages`, `openecraft_i18n::page`), the same page with its
texts, its title and its description from the catalog, so that a search engine reads
them without running the demos. Such a page opens in its language; the English one
follows the language of the browser, and the visitor switches on either. With the
public address every page names itself as canonical and all of them as its
alternates (`hreflang`), and the sitemap lists them.

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

or set `editor` in `dist/config.js` to the address of a web editor hosted elsewhere.
`config.js` is a plain file: it can be edited on the server without a new build.

## Docker image

`ghcr.io/nilres/openecraft-website` (same tags as the other images) serves the page
with nginx on port 8080, as an unprivileged user, with the web editor at `/editor/`:

```bash
docker run -d --name openecraft-website -p 127.0.0.1:8088:8080 ghcr.io/nilres/openecraft-website
docker build -f installer/docker/Dockerfile --target website -t openecraft-website .
```

CI pushes the image with the others on master and for a release. To get only a new
website image, without the rest of CI, start the workflow *Website image*
(`.github/workflows/website-image.yml`) by hand under *Actions*, on the branch to
build: it builds the image, starts it once and pushes it with the tags `edge` (on
master; the name of any other branch), `sha-<commit>` and a tag of your choice, or
only builds and checks it with *push* off. From a terminal:

```bash
gh workflow run website-image.yml --ref master -f tag=latest
```

The public address of the site, the links of the page and the legal notice are set
with the variables above, for example the address, a web editor and a repository of
your own and the operator of the site:

```bash
docker run -d -p 127.0.0.1:8088:8080 \
  -e OPENECRAFT_SITE_URL=https://openecraft.example.com/ \
  -e OPENECRAFT_EDITOR_URL=https://editor.example.com/ \
  -e OPENECRAFT_REPOSITORY_URL=https://git.example.com/acme/openecraft \
  -e OPENECRAFT_LEGAL_NAME="ACME Automation GmbH" \
  -e OPENECRAFT_LEGAL_ADDRESS="Example Street 1 | 12345 Example City | Germany" \
  -e OPENECRAFT_LEGAL_EMAIL=info@example.com \
  ghcr.io/nilres/openecraft-website
```

`installer/docker/compose.yaml` has it as the service `website` and passes the same
variables on. At every start the image writes `config.js` from the variables
(`installer/docker/website-config.sh`, any character allowed in a value), the tags
of the public address, `robots.txt` and `sitemap.xml` (`website-site.sh`) and its nginx
site from `installer/docker/website.conf` (mount your own at
`/etc/nginx/templates/default.conf.template`; it answers `/config.js`, `/index.html`,
`/robots.txt` and `/sitemap.xml` with the files of `/tmp/openecraft/`);
put a TLS reverse proxy in front of it for a public site. The WebAssembly (40 MB,
15 MB compressed), the scripts and the routings have a compressed copy from the
build of the image, which nginx sends as it is (`gzip_static`), also to a reverse
proxy speaking HTTP/1.0 (the default of nginx's `proxy_pass`) and with
`Vary: Accept-Encoding`; a server of your own should compress them once as well
instead of for every visitor. The log of the
visits (the output of the container) has no whole address of a visitor: nginx leaves
out the last part of an IPv4 address and all but the first three groups of an IPv6
address (`log_format visits` of `website.conf`); its error log, which is seldom
written, has them. The page needs no server
side: nothing a visitor does leaves the browser.
