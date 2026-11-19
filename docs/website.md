# Website

`website/` is the public page of the project, online at
[openecraft.org](https://openecraft.org) with the web editor at
[editor.openecraft.org](https://editor.openecraft.org): what OpenECraft does, shown with
live demos instead of screenshots. The demos are the editor's own code compiled to
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
| Tools (`tools/…`) | `src/tools.rs`: the voltage drop of the installation circuits, the DXF and DWG import with the SVG and PDF exports, the import of parts files |
| Symbol library, example pages, documentation, news | Written by the build: `crates/openecraft-site` ([below](#pages-the-build-writes)) |

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
- `content/`: the pages written as content files, which the build puts into the
  frame of the site ([below](#pages-the-build-writes)).
- `tools.js`, `src/tools.rs`: the tools ([below](#tools)).
- `src/main.rs`: the exports for the page (`call`, `export_file`, `start`,
  `search_routing` for the worker, `tool` and `tool_file` for the tools). The native build
  is a tool for working on the demos: `cargo run -p openecraft-website` shows both
  canvases in a window, `cargo run -p openecraft-website -- call state` prints an
  answer as the page gets it, `… -- export pdf` writes the file of an exporter,
  `… -- tool voltage_drop '{"current":"16",…}'` prints the answer of a tool.

The module is 28 MB (10 MB compressed; a release build is optimised for size, profile `web`), so the page is shown first: trunk only names
the module in the page (`pattern_script` of `Trunk.toml`) and `site.js` loads it when
the visitor scrolls, clicks or types, and otherwise once the page is there and the
browser has time. A visitor who asked the browser to save data gets it only with the
first click or key, not by scrolling. Until then the first screen shows `sheet.svg`.

The page is laid out for phones and tablets too (`site.css`): below 1080 px the
sections are one column and the links of the bar a menu (`#nav-toggle`), below 700 px
the canvases follow the width of the screen. What is said to a finger instead of a
mouse is a second element beside the first, `class="for-touch"` next to
`class="for-mouse"`; the style sheet shows the one of the device (`pointer: coarse`),
which also gets larger buttons and list rows.

Where the links of the page lead is set in `config.js` beside it, which the page
loads first; the Docker image takes the same settings from environment variables:

| `config.js` | Variable of the image | Leads to | Empty |
| --- | --- | --- | --- |
| `editor` | `OPENECRAFT_EDITOR_URL` | The web editor: a folder beside the page (`editor/`, the default) or any address | Hides the links |
| `repository` | `OPENECRAFT_REPOSITORY_URL` | The source repository | Hides the links |
| `downloads` | `OPENECRAFT_DOWNLOADS_URL` | The downloads | The latest release of the repository |
| `documentation` | `OPENECRAFT_DOCUMENTATION_URL` | The folder with all documents (*All documents in the repository*; the site itself has the tutorial, the installation guide and the feature documents as pages, [below](#pages-the-build-writes)) | The `docs` folder of the repository |
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

The labels follow the language of the page; the values are shown as they are set. The
web editor at `/editor/` of the Docker image shows the same notice in *Help › Legal and
privacy*; a web editor hosted alone and the web pages of the parts and project servers
take the same fields and variables ([installation.md](installation.md#legal-notice-of-a-web-editor),
[parts-server.md](parts-server.md#legal-notice-disclaimer-and-privacy-notice),
[project-server.md](project-server.md#legal-notice-disclaimer-and-privacy-notice)).
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
`site address` of `index.html` and of every other page; the feed of the news gets
the address its links start from (`xml:base`); without it they are left out, as an address in the
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
alternates (`hreflang`), and the sitemap lists them. The footer of every page links
the page in all languages, so a search engine finds them without the sitemap too.

### Pages beside the main page

Four plain pages say for whom OpenECraft is made and how it compares, for visitors
and for searches such as an open source alternative to EPLAN:

| Folder | Page |
| --- | --- |
| `panel-builders/` | Panel builders and control cabinet shops: schematic, panel layout, production data, wiring assistant |
| `electricians/` | Building installation: floor plans on paper, outlets, switches and lights, final circuits, distribution boards |
| `education/` | Schools and training: free, in the browser, tutorial and examples simplest first |
| `compare/` | OpenECraft beside EPLAN Electric P8, AutoCAD Electrical and QElectroTech, with what OpenECraft lacks yet and a trademark note |

Each is `<folder>/index.html`, plain HTML without the module, with a title,
description, Open Graph tags and structured data (`WebPage` about the
`SoftwareApplication`) of its own; it uses the style sheet, `config.js` and `site.js`
of the main page (`../site.css`, copied by trunk as it is), whose dialogs it links
(`../#privacy`). Its texts are listed in `src/pages.rs` for the catalog (area
`website`), and the build writes the page in every language beside it, `de.html`, …
(`pages.sh`, on Windows `pages.ps1`, a hook of `Trunk.toml` that runs the example
`pages` of `openecraft-i18n` on each folder). The menu of the languages on such a page
leads to the file of the language chosen; links marked `data-page` lead to the pages
in the language of the page they are on (`site.js`). The main page links them from
the section *Made for your work* (`#uses`) and its footer. nginx sends `/compare` to
`/compare/` (`website.conf`), as the links of a page are relative to its folder.

What is said about other products stays general and public (license, platforms,
what they are known for) and what is said about OpenECraft is what the
documentation describes; a new page is a folder, an entry of `PAGES` in
`src/pages.rs`, of both hook scripts and of the locations of `website.conf`, and a link
from the bars of the pages (the tests check all of it).

`website-site.sh` gives these pages the tags of the public address as well (every
folder, one or two deep, whose `index.html` has the markers `site address`): each
language of a page names itself as canonical and the page's other languages as
alternates, and the sitemap lists them; nginx answers them from
`/tmp/openecraft/<folder>/`.

### Pages the build writes

Most pages of the site are not written by hand. After trunk built the main page, a
hook of `Trunk.toml` runs `cargo run --release -p openecraft-site --example site`
(`crates/openecraft-site`, no UI, unit tested), which writes into the page:

| Folder | Pages | From |
| --- | --- | --- |
| `symbols/` | The symbol library: an overview and a page for every category, each symbol with its drawing (`symbols/art/<key>.svg`, drawn by the SVG export), its name, key, identifying letter and connection points | `openecraft_core::symbols::library` |
| `examples/` | An overview and a page for every example project: what it shows, its first sheets as SVG, the list of its pages, its PDF and a link that opens it in the live demo | `openecraft_core::sample::EXAMPLES`, the SVG and PDF exports |
| `docs/` | The documentation: the tutorial, the installation guide, the feature documents, the command line, the plugins and the servers (`DOCUMENTS` of `docs.rs`) | The Markdown files of `docs/` |
| `news/` | A page for every released version, an overview and the feed `news/feed.xml` (Atom) | `CHANGELOG.md` |
| `faq/`, `how-to/…`, `tools/…`, `compare/<product>-alternative/` | Questions and answers, guides for single tasks, the tools and the comparison with one product | The content files of `website/content/` |
| `build.js` | What the build knows: the module of the site for the tools, the screen recording and the numbers of the project ([below](#screen-recording-and-numbers-of-the-project)) | The built page, `website/tour.webm`, the environment |

Every page gets the frame of the site (`frame.rs`: head with title, description, link
preview, structured data and the markers of the public address; bar; footer), the
same as the pages written by hand, which a test of the website compares. The symbol,
example and content pages are written in every language (`index.html`, `de.html`, …)
with their links leading to the pages of the same language; their texts are in the
catalog area `website`. The documentation and the news are in English, as their
sources are. Links between the documents lead to their pages, links to other files
of the repository to the repository (`data-link="repository"`).

A **content file** is `website/content/<folder>.html` (a folder of pages has its own
page as `index.html`). It starts with the lines `title: …`, `description: …` and
`kind: page`, `faq` or `tool`, then an empty line and the content of `<main>`, written
in English with its texts marked `data-t`. Links to the site start with `@/` for a
page that exists in every language (`@/compare/`, `@/#studio`,
`@/?example=conveyor#studio`) and with `~/` for a file or a page in English only
(`~/docs/tutorial/`). A page of the kind `faq` names its questions to search engines
as structured data (`FAQPage`): `<details><summary data-t>…</summary><p data-t>…</p></details>`.
Its texts are listed in `crates/openecraft-site/src/texts.rs` for the catalog (the
tests compare). A new top-level folder is also an entry of `WRITTEN` in
`website/src/pages.rs`, of `SECTIONS` in `frame.rs` (the footers), of the locations
of `website.conf` and of `check-website.sh`.

What a page says stays true to the program: the guides name the commands as the
tutorial does, and the comparisons say about other products only what is general and
public, with a trademark note.

To write the pages into a folder without trunk (for a look at them, with any static
web server): `cargo run --release -p openecraft-site --example site -- <folder>`.

### Links into the demos

An address can say what the demos of the main page show: `?example=conveyor` opens
an example, `&page=3` its third page, `&place=1&view=front` a place of the panel
layout (`site` or the number of an enclosure or room; `3d` or `front`) and
`&report=terminal_diagram` a report; `#studio`, `#layout` or `#reports` scrolls
there. The module is then loaded at once. *Copy link* in the toolbars of the demos
and beside the title of a report writes such an address into the clipboard; it opens
the example as it comes, without what the visitor changed. The guides and the example
pages link the demos this way.

### Tools

`tools/voltage-drop-calculator/`, `tools/dxf-dwg-viewer/` and `tools/edz-viewer/` are
content files of the kind `tool`: a form with values or a file of the visitor, and
places for the answer. `tools.js` loads the module of the site when the tool is used
(its address is in `build.js`; such a page allows WebAssembly in its policy, no other
page beside the main one does), hands the values and the file to `tool` of the module
(`src/tools.rs`, `tools::call`) in the language of the page and shows the JSON
answer: a summary, facts, a table, a drawing, and buttons for the files `tool_file`
writes from the file given (SVG, PDF, DXF, DWG, CSV, JSON). Nothing the visitor
enters or opens leaves the browser. A new tool is a function of `tools.rs` (`TOOLS`,
`FILES`), a content file with `data-tool="<name>"` and a card on `tools/index.html`.

### Screen recording and numbers of the project

Both are shown only when the build has them, so the page never promises what is
not there:

- A screen recording for the first screen: `website/tour.webm` (and `tour.jpg` as
  its still picture). The build copies them and names them in `build.js`; the main
  page shows the video below the sheet. It is loaded only when the visitor starts it
  (`preload="none"`). Both files are not kept in the repository (`.gitignore`): put
  them into `website/` before the build. `website/tour.mjs` records them from the
  live demos of the built site (headless Edge and Node, no other tool): record them
  again when the page or the demos changed visibly.
- The stars and the contributors of the repository: the variables `OPENECRAFT_STARS`
  and `OPENECRAFT_CONTRIBUTORS` of the build (numbers; for example from `gh api
  repos/<owner>/<name> --jq .stargazers_count`). With a number above 0 the facts of
  the first screen show it. The page itself asks no other server for them.

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
website image, without the rest of CI, start the workflow *Docker image of the
website* (`.github/workflows/docker-website.yml`) by hand under *Actions*, with the
branch, release tag or commit to build: like the workflows of the other images
([installation](installation.md#docker-images-of-the-servers)) it builds the image,
starts it once (`installer/docker/check-website.sh`) and pushes it with the tags CI
gives that commit (`edge` for the tip of master, `sha-<commit>`, the version of a
release tag) and the tags of your choice, or only builds and checks it with *push*
off. From a terminal:

```bash
gh workflow run docker-website.yml -f ref=master -f tags=test
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
put a TLS reverse proxy in front of it for a public site. The WebAssembly (28 MB,
10 MB compressed), the scripts and the routings have a compressed copy from the
build of the image (`installer/docker/compress.sh`), which nginx sends as it is
(`gzip_static`), also to a reverse
proxy speaking HTTP/1.0 (the default of nginx's `proxy_pass`) and with
`Vary: Accept-Encoding`; the WebAssembly and its script, of the page and of the web
editor, have a Brotli copy as well, a good third smaller, for the browsers that ask
for it (over HTTPS only). A server of your own should compress them once as well
instead of for every visitor. The log of the
visits (the output of the container) has no whole address of a visitor: nginx leaves
out the last part of an IPv4 address and all but the first three groups of an IPv6
address (`log_format visits` of `website.conf`); its error log, which is seldom
written, has them. The page needs no server
side: nothing a visitor does leaves the browser.
