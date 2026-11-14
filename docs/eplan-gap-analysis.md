# OpenECraft versus EPLAN: gap analysis

State of the code: master at commit a77311a (October 2026). "EPLAN" means the
current EPLAN Platform: Electric P8 with Pro Panel, Fluid, Preplanning, Harness proD,
FieldSys, Smart Wiring / Smart Mounting / Smart Production, Cogineer / eBUILD, eVIEW,
eMANAGE, the Data Portal and the API. The comparison was made by reading the
workspace (model, core modules, editor, CLI, servers, README) and matching every
area against what EPLAN offers in that area. Where a statement is an inference
rather than something read in the code, it is marked "not verified".

**Update (branch `eplan-parity`, October 2026):** most of the Tier 1 and Tier 2 items of
the roadmap and several Tier 3 / 4 items have been implemented since. Section 6 lists
what was done and what is still open; the scorecard below has a column for the state
after that work, and section 4 now has measured performance figures. Sections 2 and 3
describe the state at the time of the analysis; section 7 covers the six gaps left after
that work.

## 1. Summary

OpenECraft covers an unusually wide slice of the EPLAN Platform for a 130-commit
project: the core crate alone holds about 42,000 lines without tests, 41 report
kinds, roughly 150 distinct check messages, 215 command line / MCP tools and a parts
data portal with its own maintenance app. Whole EPLAN add-ons have a counterpart:
Pro Panel (3D layout, routing, NC data, copper, climate), Smart Wiring (wiring
assistant), Smart Production (CNC programs, hand-off, feedback), FieldSys (field
route network), Fluid (fluid nets, hoses), Cogineer (configurator), eVIEW (comments
and redlining), the Data Portal (parts server) and the API (plugins). Some things
EPLAN does not have at all are here: electrical calculations (short circuit,
selectivity, loads, ISO 13849-1), rule-set verification, an MCP server for AI agents
and a browser build.

The gaps are therefore not in breadth but in three places:

1. **Depth of the daily editing experience.** The editor (`crates/openecraft-canvas/src/editor.rs`,
   about 1,400 lines) is thin compared with the engine behind it. Texts cannot be
   rotated, aligned or formatted; device tags are only numbered online; pages cannot
   be renumbered; connection numbering has three fixed schemes; cross-reference
   formats are fixed; there is one grid size, no coordinate input, no project-wide
   find, no printing dialog, no page tree by structure, no terminal-strip, PLC,
   potential, interruption-point or parts navigators with drag-and-drop placement.
   An EPLAN user would notice most of these within the first hour.
2. **Forms, checks and master data as configurable systems.** EPLAN's report forms
   are graphical drawings (terminal diagrams with drawn terminals and jumpers, cable
   diagrams with cores, PLC card diagrams); OpenECraft's are tables with columns,
   filters and sorting. EPLAN's check run has about 900 numbered messages with
   schemes, severity overrides and a message-management dialog where false
   positives are acknowledged; OpenECraft lists issues without ids, schemes or
   acknowledgement. EPLAN has project templates, settings schemes, system master
   data (symbols, forms, frames, macros) synchronised into projects and rights
   management; OpenECraft has libraries as files and a token on the project server.
3. **Content and ecosystem.** About 320 IEC symbols plus small NFPA, fluid and P&ID
   sets against EPLAN's several thousand across IEC, NFPA, GB, GOST, JIS and fluid
   libraries; eight curated manufacturer catalogs against a Data Portal with over a
   million parts carrying macros, 3D data and drilling patterns; no importer for
   EPLAN projects; no manual beyond the README; PDF output limited to Latin-1
   characters; four UI languages against about twenty.

Scorecard (coverage of the EPLAN function area by OpenECraft; "gap" is what is
missing relative to an EPLAN user's expectations):

| Area | Coverage | Gap | Section | After `eplan-parity` |
| --- | --- | --- | --- | --- |
| Schematic editor: texts, graphics, editing aids | partial | large | 3.1 | good (printing through the browser) |
| Device tags, numbering, cross references | partial | large | 3.2 | good |
| Symbol libraries and symbol editor | partial | large (content) | 3.3 | partial (NFPA, fluid extended, GB / GOST / JIS letter sets); content |
| Terminals, plugs, cables, potentials | good | medium | 3.4 | good (multi-level, jumper kinds) |
| Navigators, find, dialogs | partial | large | 3.5 | good (find, page tree, unplaced, PLC, strips / plugs with drag-and-drop) |
| Reports and forms | good (tables), missing (graphical forms) | large | 3.6 | good (graphical terminal, cable, plug and PLC forms; drawn forms for every report) |
| Project check and message management | good (messages), missing (management) | medium | 3.7 | good |
| Pages, plot frames, title blocks, templates | good | small | 3.8 | good (project templates) |
| Structure identifiers (IEC 81346) | good | small | 3.8 | good |
| Macros, placeholders, variants, configurator, options | good | small | 3.9 | very good (macro boxes, update) |
| Parts management and data portal | good (software), missing (content) | large (content) | 3.10 | good (typed mounting data, EPLAN XML); content |
| Panel layout (Pro Panel) | very good | small | 3.11 | very good (mounting assistant) |
| PLC, bus, AutomationML | good | medium | 3.12 | good (6 vendor formats, PLC navigator) |
| Fluid, P&ID, pre-planning | partial | medium | 3.13 | good (pre-planning module) |
| Revisions, review, compare, change requests | very good | small | 3.14 | very good (PDF comments, property changes) |
| Multi-user, rights, cloud | partial | medium | 3.15 | good (accounts, roles, viewer) |
| Import / export, migration | good (formats), missing (EPLAN, Excel, PDF import) | medium | 3.16 | good (xlsx, EPLAN parts XML); EPLAN project import open |
| Translation and languages | good | medium | 3.17 | good |
| API, scripting, automation | good | small | 3.18 | good |
| Enterprise integration | good | small | 3.18 | good |
| Harness, field wiring, mounting instructions | partial | medium | 3.19 | good (nailboards, mounting assistant) |
| Performance and scale | unknown | risk | 4 | measured; incremental analysis, parallel checks, IndexedDB (sections 4, 7) |
| Documentation, training, ecosystem | missing | large | 4 | missing |

## 2. What is already at EPLAN level or beyond

For calibration, these areas need no catch-up work and are not discussed further:

- Device-oriented data model with main / auxiliary functions, function definitions,
  function templates on parts, device selection, reserve contacts and "apply
  designations" (`model.rs`, `functions.rs`, `properties.rs`, `parts.rs`).
- Property database with 434 built-in property rows, user-defined properties,
  indexed properties and placement schemes.
- Structure identifiers with four aspects, page inheritance, relative display,
  structure management, templates and work areas (`structure.rs`).
- Potentials with definitions, presets, wire colours and cross-sections inherited by
  connections, connection and cable definition points (`potentials.rs`,
  `conndef.rs`, `cabledef.rs`).
- Panel layout: enclosure systems, door, walls, rails, ducts, busbars, cutouts,
  drilling pattern and NC export, virtual wiring, duct fill, 3D routing with
  segregation, bundles, shields, frozen routes, line-ups, room view, climate
  (IEC/TR 60890), thermal map, fastening systems, 3D macros and assemblies, CNC
  programs, copper bar bending with cut optimisation, hand-off and feedback, 3D
  export as glTF / STEP / IGES / IFC / JT (`panel.rs`, `routing3d.rs`,
  `production.rs`, `manufacturing/`, `export3d.rs`). This is at Pro Panel level and
  in copper and CNC beyond the base Pro Panel licence.
- Electrical calculations and rule sets: sizing (IEC 60204-1), short circuit and
  protection coordination (IEC 60909-0 for radial networks with transformer, cable
  chain and motor contribution, IEC 60364), selectivity with manufacturer curves and
  coordination tables, load
  calculation (IEC 61439-1), EMC, functional safety (ISO 13849-1), customer rule
  sets (`sizing.rs`, `shortcircuit.rs`, `protection.rs`, `loads.rs`, `emc.rs`,
  `safety.rs`, `rules.rs`). EPLAN has none of this natively.
- Revisions with approval workflow, baselines, compare and merge, comments,
  redlining and change requests (`revision.rs`, `compare.rs`, `review.rs`).
- Project management: project index, batch properties, subprojects, archives with
  format conversion, verification and repair (`projects.rs`, `subproject.rs`,
  `archive.rs`, `verify.rs`).
- Model options (EPLAN project options) and the configurator (`options.rs`,
  `config.rs`).
- Translation of project texts with a dictionary and per-language PDF output
  (`translation.rs`).
- Plugin API with built-in and external plugins, 215 CLI tools, MCP server
  (`openecraft-api`, `cli/`).
- Enterprise integration layer for ERP, PDM, PLM, ECAD, MCAD, procurement and wiring
  machines with conflict detection (`integration.rs`).
- Parts server as a data portal: SQLite or JSON storage, users and roles, catalogs,
  history, lifecycle and successor, attachments, faceted search, web page, BMEcat
  import; parts manager app; STEP / IGES / IFC / JT model import.

## 3. Gaps by area

Each gap lists what EPLAN has, what OpenECraft has, where the work would go, and a
size: S (days), M (weeks), L (months), XL (open-ended or content-driven).

### 3.1 Schematic editor: texts, graphics and editing aids

**Texts.** EPLAN texts have rotation (0 / 90 / 180 / 270 and free angle), horizontal
and vertical alignment, font, size, bold / italic / underline, colour, text box with
frame and fill, line spacing, paragraph width with word wrap, "path function text"
(a function text on a path that applies to every device in that column), "special
texts" (any project or page property placed as text on any page), hyperlinks and
multi-language text entry. OpenECraft's `TextItem` has position, text, size, layer
and colour; rendering is left-anchored, horizontal, multi-line by line break
(`render.rs` line 205). Function text is a per-symbol field; project properties can
only be shown in title blocks (`{prop:Name}`). Symbol texts stay horizontal when a
symbol is rotated (`rotated_anchor`), so vertical terminal strip names or rotated
labels are not possible.

- Gap: rotation, alignment, styles, text boxes, word wrap, path function texts,
  special texts anywhere on a page, hyperlinks.
- Where: `model.rs` (`TextItem`, `PlacedProp`), `render::Draw` trait (rotated
  text, styles), `svg.rs`, `pdf.rs`, `dxf.rs`, the text tool in `editor.rs` and
  `properties_ui.rs`; path function texts need a new item kind evaluated by
  `index.rs` / `reports.rs`.
- Size: M (text formatting), M (path function texts and special texts).

**Graphics.** EPLAN has lines, polylines, polygons, rectangles with rounded corners,
circles, arcs, ellipses, splines, hatching, images, text boxes, line types
(continuous, dashed, dotted, dash-dot, …), line ends (arrows), layer management with
hundreds of predefined layers (colour, line type, font, text size per layer) and
dimension types (linear, chain, baseline, angular, radius, diameter). OpenECraft has
line, rect, circle, ellipse, arc, polyline, polygon (width, colour, dashed, fill),
aligned / horizontal / vertical dimensions, PNG / JPEG images and user layers with
visible / locked / printed / colour.

- Gap: splines (imported only), hatching, rounded rectangles, line types beyond
  dashed, arrowheads, angular / radius / chain dimensions, per-layer line type and
  text size, graphic groups on schematic pages.
- Where: `model.rs` (`Shape`, `Dimension`), `graphics.rs`, `render.rs`,
  `drawing_ui.rs`, `dxf.rs` / `dxf_import.rs`.
- Size: S to M each.

**Editing aids.** EPLAN offers five grid sizes (A to E, default 4 mm), object snap,
coordinate input (typed X / Y, relative and absolute), increment moves, "design
mode" handle editing, stretching, align / distribute, insertion of several symbols in
a row ("insert multiple"), symbol variant switching with Tab during placement, group /
ungroup on schematic pages, and a status bar with coordinates. OpenECraft has one
fixed grid (`geom::GRID`, 4 mm), snapping to that grid, move by one grid step,
rotate, mirror, rubber-band wires, auto-connect and route-around.

- Gap: selectable grid, coordinate input, object snap, align / distribute, insert
  multiple, groups, stretch.
- Where: `editor.rs`, `geom.rs`.
- Size: M in total.

**Printing.** EPLAN prints directly with page ranges, scaling and printer settings.
OpenECraft exports PDF, SVG and an HTML print document; there is no print dialog
(desktop) and no page selection for output.

- Gap: native print, page ranges on export, print of the panel layout faces.
- Where: `openecraft/src/io.rs`, `pdf.rs` (page selection).
- Size: S to M.

### 3.2 Device tags, numbering and cross references

**Device tag numbering.** EPLAN numbers online (as here) and offline with configurable
schemes: identifying letter plus counter, per page, per page and column, per
structure, start values, increments, NFPA line-number based tags (`1CR1`, `203PB`),
selection of what is renumbered (whole project, pages, selection), and "keep
existing" rules. OpenECraft assigns `next_tag` per structure and letter when placing
(`model.rs`); renaming a device is manual. No bulk renumbering of devices and no
numbering schemes were found (`grep renumber` finds strips and wires only).

- Gap: offline DT numbering with schemes, NFPA style numbering, renumber selection.
- Where: new `numbering.rs` in core, tool in `cli/src/tools/devices.rs`, dialog in
  `openecraft/src/app`.
- Size: M.

**Page numbering.** EPLAN renumbers pages (start, increment, sub-pages, per structure).
OpenECraft pages have a free name and can be moved up / down; no renumbering tool was
found.

- Size: S.

**Connection numbering.** EPLAN connection numbering uses format strings with
placeholders (page, column, row, potential, counter, cross-section, …), counters per
potential type, numbering by potential or by signal, sorting by position, exclusion
of potential kinds, and schemes stored per project. OpenECraft has three schemes
(sequential, page / column, potential name) with prefix and start (`NumberScheme`,
`wirenum.rs`).

- Gap: format strings, per-kind counters, sort rules, exclusion rules, schemes.
- Where: `wirenum.rs`, `model.rs` (`WireNumbering`).
- Size: M.

**Cross references.** EPLAN cross-reference display is configurable: format
(`/3.4`, `=A1+CAB1/3.4`, `3.4`, with or without row), contact image on the coil or
in the page margin ("below the symbol" vs "in path"), pair cross references, same-DT
cross references for every function of a device, device cross references for black
boxes, cable and terminal cross references, interruption point star (one source,
many targets) and chain (sequence) with sorting, and cross references on overview /
single-line symbols. OpenECraft has contact images on coils with reserve contacts,
contact → coil references, interruption point references to every other location
with the same name (`index.rs`), single-line references and electro-fluid cross
references; the format is fixed.

- Gap: configurable formats, margin contact image, pair / same-DT / black box cross
  references, star / chain interruption points with sequence numbers.
- Where: `index.rs`, `render.rs` (`draw_annotations`), project settings.
- Size: M.

### 3.3 Symbol libraries and symbol editor

EPLAN ships IEC_symbol (about 1,500 symbols, each with 8 variants), IEC_single_symbol,
NFPA_symbol (several hundred), GB_symbol, GOST_symbol, JIS, SPECIAL, GRAPHICS, fluid
libraries (hydraulics, pneumatics, hundreds of valve spool configurations) and the
P&ID libraries of Preplanning. Symbols carry representation types (multi-line,
single-line, pair cross reference, overview, graphics, panel layout) and connection
point logic (direction, allowed targets, internal / external). OpenECraft has about
320 IEC symbols (50 generated families in `symbols/iec.rs` plus 124 entries in
`symbols.rs`), 16 NFPA, 11 fluid, 7 P&ID, 8 single-line and black boxes; the symbol
editor draws lines, rectangles, circles, texts, fields and pins and assigns one of 18
function kinds.

- Gap (content): comprehensive NFPA, GB, GOST, JIS, electronics (logic gates,
  op-amps, semiconductors), building installation (IEC 60617 part 11), telecom,
  hydraulics / pneumatics (ISO 1219 complete), P&ID (ISO 10628 / ISA 5.1 complete).
- Gap (software): the symbol editor draws lines, polylines, circles, texts and
  fields only (`Prim` in `symbols.rs`): no arcs and no filled areas; no import of EPLAN
  `.slk` symbol libraries or of symbols from DXF blocks; connection point logic
  (direction, internal / external target) is implicit in geometry; function
  definitions per symbol are richer in EPLAN (hundreds of definitions with
  categories, groups and sub-groups).
- Where: `symbols/` (new library modules), `symlib.rs` and `symbol_editor.rs`.
- Size: XL (content); M (editor features).

### 3.4 Terminals, plugs, cables and potentials

**Terminals.** EPLAN has multi-level terminals (2-, 3-, 4-tier with level
designations), terminal function definitions (feed-through, PE, N, disconnect, fuse,
component, diode, sensor terminals), jumper types (saddle jumper, wire jumper,
insertion bridge, plug-in jumper) with graphical representation, terminal strip
definition symbols on the page, terminal sorting by potential, by target or by
number, internal / external target determination with rules, accessory handling
through the parts database, and a terminal strip navigator for generating terminals
from parts and dragging unplaced terminals onto pages. OpenECraft has strip order,
renumbering, numeric jumper groups (`StripData::jumpers`), spare terminals, part per
terminal (PE through tech data `Terminal type: PE`), automatic end brackets, end
plates and plug-in jumpers, and the terminal diagram, line-up and strip overview
reports. No terminal levels were found (`grep level strips.rs` finds nothing).

- Gap: multi-level terminals, jumper types, terminal function kinds as definitions
  rather than tech-data strings, sorting rules, strip definition symbol, navigator.
- Where: `strips.rs`, `symbols.rs` (terminal pins with level), `reports/diagrams.rs`.
- Size: M.

**Plugs.** Good coverage (pins, mated pairs, pin order, unused pins, plug diagram).
Gap: plug definition symbols, pin function definitions (shield, PE pin), multi-pin
plug macros from parts, plug navigator. Size: S.

**Cables.** Good coverage (cores, shields, bus kind, diameter, cable definition
lines, cable diagram and connection diagram, sizing). Gap: automatic cable
selection by core count and cross-section from the parts database with
"cable selection" dialog, shield as a connection point (shield connected to PE at a
terminal), twisted pair / core pairing, cable "potential / signal" classification
beyond segregation class. Size: S to M.

**Potentials.** Good coverage. Gap: potential definition point and net definition
point symbols (define a potential on any wire, not only at connection points),
signal tracking separate from potential tracking, potential overview per page
column. Size: S.

### 3.5 Navigators, find and dialogs

EPLAN's working style is built on navigators (tree and list views, filters, "place"
by drag and drop, synchronised selection, "go to counterpart") for pages, devices,
terminal strips, plugs, cables, connections, PLC, interruption points, potentials,
parts (BOM), macros, placeholders, pre-planning, layout spaces and project options,
plus a project-wide Find (texts and properties with filters) and Edit in table on
nearly every navigator. OpenECraft has navigator tabs for pages (a flat list with
reorder, `app.rs` line 1122, no grouping by structure), devices (grouped by
structure, filter, place further functions, rename), connections, cables, layers,
check and review, and separate windows for strips, plugs, PLC, potentials,
harnesses, macros, revisions and more. Edit in table covers devices, connections
and terminals. A symbol filter exists (`lib_filter`); no project-wide text search was
found.

- Gap: page tree by structure, terminal strip / plug / PLC / interruption point /
  potential / parts / macro / placeholder navigators with drag-and-drop placement of
  unplaced functions, synchronised selection, project-wide Find and Replace, Edit in
  table for cables, plugs, PLC, pages and parts, an "Insert Center" that searches
  symbols, macros and devices at once.
- Where: `openecraft/src/app/*_ui.rs`, `tables.rs` (more row kinds), new `search.rs`
  in core.
- Size: L in total; each navigator S to M.

### 3.6 Reports and forms

EPLAN report forms are drawings: a form editor places static graphics, placeholder
texts (any property), data areas with sub-forms, and the generated page draws them,
so a terminal diagram shows drawn terminals with jumpers, saddle jumpers, cables as
lines and targets left and right; cable diagrams draw cores and shields; PLC card
diagrams draw the card; plug diagrams draw pins; the parts list can be a form with
logos and multi-line descriptions. Forms are dynamic (rows per page follow the data)
or static. OpenECraft report pages are tables generated from a `Table` (headers and
rows); a `ReportForm` chooses title, columns, rows per page, filters, sort order and
grouping; report blocks regenerate pages per strip, plug, cable or device; title
blocks are graphical. The connection "diagrams" (`reports/diagrams.rs`) are tables as
well.

- Gap: graphical form editor and graphical forms for terminal, cable, plug and PLC
  diagrams; placeholder texts for any property in forms; sub-forms; logos and
  images in forms; form-based parts list variants.
- Where: `forms.rs` (form items exist for title blocks and could be generalised to
  report forms with a data area), `report_blocks.rs`, `render.rs`, `forms_ui.rs`.
- Size: L.

Missing report kinds compared with EPLAN's standard set: structure identifier
overview, interruption point overview, parts list per device and per manufacturer /
supplier (the bill of materials is one summarised form grouped by part number,
`reports::bom`), placeholder object overview, project
options overview, PLC card overview and PLC diagram (graphical), pre-planning
segment lists, symbol / form / plot frame documentation, manufacturer and supplier
list, enclosure "drilling view" as a drawing (NC data exists as DXF / CSV). Each is
S once the data is there; graphical ones depend on the form editor above.

### 3.7 Project check and message management

EPLAN's check run has about 900 numbered messages in categories (001 to 036 and
beyond), check schemes (which messages run, offline or online, severity per message:
error / warning / note / off), a message management dialog with filters, "go to",
and a per-message "verified / accepted" flag that hides acknowledged messages
until the object changes. OpenECraft runs all checks after every change
(`Analysis::build` plus plugin checks, `app.rs` line 2249), lists roughly 150
distinct messages plus panel, rule-set, fluid, electrical and plugin checks in a
Check tab, and shows them translated with a target to jump to. Rule sets add
customer checks (good). No message ids, schemes, severity override or
acknowledgement were found.

- Gap: message ids, check schemes, severity override, acknowledge / verify flag
  stored in the project, filtering by page / structure / severity, export of the
  message list.
- Where: `reports.rs` (`Issue` gets an id), project settings, Check tab.
- Size: M.

### 3.8 Pages, plot frames, title blocks, templates, structure

Good coverage: page kinds (schematic, overview, graphics, cover, report, mounting
layout, document), plot frames for A0 to A4, Letter, Legal, Tabloid and custom with
columns / rows / labels, custom title blocks with placeholders, page templates,
structure templates and four structure aspects.

- Gap: document type aspect (`&` of IEC 61355) and user-defined aspect blocks
  (EPLAN allows up to 17 structure blocks), nested identifiers as a tree in the
  structure management (sub-levels exist in work area filters), sub-pages (`3.a`),
  page property "page type" variants of EPLAN (e.g. single-line vs multi-line vs
  topology vs panel layout as distinct types with their own evaluation), plot frame
  editor as a drawing (plot frames are parametric here: paper, border, columns, rows;
  EPLAN frames are free drawings with path areas), project templates (a complete
  "basic project" with settings, forms, frames, structure and master data; only
  page and structure templates exist here, no project templates were found).
- Where: `structure.rs`, `frame.rs`, `page_templates.rs`, `projects.rs`.
- Size: S to M each.

### 3.9 Macros, placeholders, variants, configurator, options

Good coverage: window and page macros, value sets, variants A to …, panel 3D macros
and assemblies, configurator with options and rules, model options.

- Gap: macro boxes on pages (define a macro inside the project so it can be
  re-exported after edits), macro projects and automatic macro generation,
  "update macros" (propagate a changed library macro into the projects that use it),
  representation types in one macro (multi-line, single-line, panel layout and
  pre-planning content of a device in one macro file), placeholder objects as
  placed symbols with a navigator, Cogineer-style designer (variables in Excel,
  "typicals", generation across projects). The macro library is stored with the
  application, not as files per macro in a directory that several users share.
- Where: `macros.rs`, `config.rs`, `panel/macros.rs`.
- Size: M.

### 3.10 Parts management and data portal

The software side is complete: part model with commercial, technical, mounting,
function template, accessory, connection point and attachment data, product groups
in EPLAN's numbering, lifecycle and successor, CSV / JSON / BMEcat import, 2D and 3D
models in many formats, project parts synchronisation, device selection, a data
portal server with accounts, catalogs, history and faceted search, a parts manager
app.

- Gap (content): eight curated catalogs against more than a million portal parts
  with manufacturer-made schematic macros, 3D macros with mounting points and
  placement areas, drilling patterns and connection point patterns; a manufacturer
  onboarding path (EPLAN works with manufacturers who publish their data);
  manufacturer configurators (Rittal RiPanel, Siemens TIA Selection Tool, Phoenix
  Project Complete, Weidmüller Configurator) reached from the part selection.
- Gap (data model): mounting and placement data live in free-text technical data
  (`Clearance: 25 mm`, `Mounting holes: M5: 9,12; 69,72`, `Door cutout: Ø30.5`,
  `Terminal type: PE`, `Power loss: 7.5 W`). This works but is fragile and hard to
  import; EPLAN keeps them as typed fields (mounting data, drilling pattern,
  connection point pattern, macros per representation type). Part variants, part
  assemblies / modules with sub-parts, and "part data per language" are missing;
  prices have one currency per part with no price lists.
- Gap (storage): the local parts database is a JSON document in app storage /
  browser local storage. Large databases (tens of thousands of parts with models)
  will not fit browser local storage and load slowly on the desktop (not measured).
  EPLAN uses SQL Server / Access / SQLite with lazy loading.
- Where: `openecraft-parts/src/part.rs`, `catalog.rs`, `portal.rs`,
  `openecraft-parts/catalogs`.
- Size: XL (content), M (typed mounting data), M (lazy local database).

### 3.11 Panel layout (Pro Panel)

Coverage is at or above Pro Panel in most places (section 2). Remaining gaps:

- 3D macros of parts with mounting points, placement areas, handles and
  accessory placement points as defined by the part (here derived from dimensions
  and tech data).
- Mounting rail and duct "ejection" of items when a rail moves (devices travel
  with their rail here: covered), rail ends and cutting allowances.
- 2D panel layout pages drawn with symbols from parts (EPLAN "panel layout" page
  type, 2D without Pro Panel): the mounting page here is a generated model view, not
  an editable 2D layout page.
- Section views, exploded views, hidden-line and shaded rendering quality,
  dimensioning on model views by hand.
- Automatic placement proposals (none in EPLAN either, so not a gap).
- Export of wire lists in the formats of wire processing machines (Komax Zeta,
  Schleuniger, Rittal Wire Terminal, Cadenas) beyond the generic CSV; the
  integration connector covers one neutral CSV.
- Smart Mounting (step-by-step mounting instructions on a tablet, like the wiring
  assistant but for mounting) is missing; the wiring assistant exists.
- Imperial units in the panel layout (inches) not verified; millimetres throughout.

Size: S to M each.

### 3.12 PLC, bus and AutomationML

Good coverage: PLC cards with addressing, symbol table CSV import / export (IEC
61131-3 global variables), hardware configuration with stations, racks, modules and
bus systems, GSDML / ESI device descriptions, AutomationML AR APC import and export
(`aml.rs`), address schemes per PLC type.

- Gap: manufacturer formats besides AML: Siemens STEP 7 Classic (.sdf / .asc),
  Rockwell Studio 5000 (.csv / .L5X), Beckhoff TwinCAT (.xml / .tsm), Mitsubishi
  GX Works, Schneider Unity / Control Expert (.xsy), B&R, Phoenix PLCnext; symbolic
  addresses with data types per channel as a first-class property; PLC box symbols
  with device connection points and channel / bus port functions drawn from device
  descriptions; PLC navigator with drag-and-drop of addresses; "PLC card overview"
  and graphical "PLC diagram" reports; safety PLC signals (F-signals).
- Where: `plc.rs`, `plc_hw.rs`, `aml.rs`, `plc_ui.rs`.
- Size: S per format, M for navigator and graphical reports.

### 3.13 Fluid, P&ID and pre-planning

Fluid engine is solid (fluid nets, port functions, hoses and tubes, fittings, groups,
electro-fluid links, checks and reports). Pre-planning is limited to PCT loops.

- Gap (Fluid): symbol content (section 3.3); fluid power parts with flow and pressure
  data; hose / tube routing in the field as in the panel.
- Gap (Preplanning): planning objects and structure segments as a tree, segment
  templates, consumption / budget planning, pre-planning macros that generate
  schematic pages from the pre-planning, import from process engineering tools and
  Excel, pre-planning navigator and reports, P&ID symbol content.
- Where: `process.rs`, `fluid.rs`, new `preplanning.rs`.
- Size: L (pre-planning), XL (content).

### 3.14 Revisions, review, compare and change requests

Beyond EPLAN in workflow (approvals, change requests). Gap: import of PDF comments
made in Acrobat into the review layer (EPLAN does this since 2.9), revision
"change tracking per property" with old / new value in the revision overview (the
overview lists changed items, not verified for property level), and revision
markers on panel layout pages. Size: S to M.

### 3.15 Multi-user, rights and cloud

Page and project-data locks, merge on save, automatic reload and id ranges match
EPLAN's multi-user approach. Gaps:

- Rights management: the project server knows a bearer token and a free-text user
  name (`project-server/src/routes.rs`); there are no accounts, roles or rights per
  function (EPLAN Rights Management: user groups, rights for about 100 functions,
  project rights). The parts server has roles; the project server should get the
  same account model.
- TLS and single sign-on are left to a reverse proxy; no audit log of who changed
  what (the exchange log covers integrations only).
- A read-only viewer mode for the browser build (eVIEW: viewing, redlining and
  comments without editing rights) does not exist as a mode; the web build is the
  full editor.
- Cloud storage of projects, sharing links, notifications on comments.

Where: `project-server/`, `collab.rs`, `openecraft/src/app/remote_ui.rs`. Size: M.

### 3.16 Import, export and migration

Export is broad (PDF with bookmarks and links, DXF R12, DWG, SVG, HTML, CSV reports,
3D formats, AML, KBL, marking CSV, symbol tables, NC data). Import covers DXF / DWG as
graphics, BMEcat, AML, GSDML / ESI, CSV / JSON parts and tables, dictionaries.

- Done: EDZ files of the EPLAN Data Portal are imported as parts (parts XML with
  function templates, connection points and drilling patterns; pictures and macros
  stored with the part, macros not drawn).
- Gap: no importer for EPLAN projects. EPLAN P8 can export a
  project as XML (pages, functions, connections, parts) and EPLAN parts as
  parts-XML (`.xml` of the parts database); readers for both would be the migration
  path and are the single most asked-for feature of any EPLAN alternative. DXF
  import creates graphics only; an "intelligent" import that recognises blocks as
  symbols and attributes as tags is missing.
- Gap: Excel (.xlsx) import / export for tables, labels and reports (CSV only);
  labeling schemes with output templates for Brady, Phoenix Clip Project, Weidmüller
  M-Print Pro, Murrplastik, Partex (marking export is one CSV layout).
- Gap: PDF output uses the standard Helvetica font with WinAnsi encoding
  (`pdf.rs`), so every character outside Latin-1 (Polish, Czech, Turkish, Greek,
  Cyrillic, Chinese) becomes `?`; no font embedding, no PDF/A, no layers in PDF, no
  comments round trip. DXF export is R12 (no true type fonts, no layers with line
  types beyond the kind layers).
- Gap: PDF and image import of old drawings as vector / raster background on pages
  (only "document page" embedding exists, which shows the file name).
- Where: new `eplan_xml.rs` in core, `xlsx` writer / reader crate in
  `openecraft-core` (pure Rust, works in the browser), `pdf.rs` (TrueType subset
  embedding), `marking.rs`.
- Size: L (EPLAN XML), M (xlsx), M (PDF fonts), S (labeling schemes).

### 3.17 Translation and languages

Project text translation with dictionary and per-language output exists and is
well done. Gaps: the dictionary is per project (EPLAN has a system dictionary
shared by all projects, automatic translation while typing, a missing-word list and
import of EPLAN dictionaries); UI in four languages against about twenty (Italian,
Portuguese, Dutch, Polish, Czech, Russian, Turkish, Chinese, Japanese, Korean);
bundled fonts lack CJK and partly Cyrillic glyphs, so those project texts render as
boxes (the glyph test enforces this for literals, not for user input). Size: S
(system dictionary), M per language family (fonts, catalog).

### 3.18 API, scripting, automation and integration

The plugin API (commands, checks, reports, exporters; built-in and external process
plugins) and the MCP server exceed EPLAN's script interface in reach for automation.
Gaps noted in AGENTS.md itself: no extension points for symbols, importers and
project events; plugins cannot add UI (dialogs, navigators, toolbar buttons) or
react to events (page opened, project saved, property changed); no in-process
scripting language for end users (EPLAN: C# scripts without a licence, full API with
licence). Enterprise integration is broad and Autodesk Vault is connected directly
(Vault Data API: browsing, imports, check-ins, item links); other connector-specific
adapters (SAP IDoc, Teamcenter, Windchill) are left to the CSV / JSON profile. Size: M.

### 3.19 Harness, field wiring and mounting instructions

Harness: assignment, lengths, connectors, bundle diameter, nailboards to scale, a 3D
topology (derived from the panel routes or edited: nodes, segments, bend radii),
tape / tube / sleeve coverings, clips and grommets, end treatment, a KBL export
following the KBL 2.4 SR-1 schema and a JSON / CSV interface for machines exist.
EPLAN Harness proD adds interactive 3D harness routing on the machine model (MCAD
geometry) and splices inside the bundle. Field
wiring (route network, bonding, accessories) matches FieldSys. Smart Mounting is
missing (section 3.11). Size: L (nailboards).

## 4. Non-feature gaps and risks

**Performance at scale.** Undo works by cloning the whole project and comparing it
every frame (`history.rs`); the analysis (netlist, index, all checks) is rebuilt
completely after every committed change (`app.rs`); the project is one JSON document
kept in memory and in browser local storage. EPLAN projects routinely have 500 to
5,000 pages with tens of thousands of connections.

Measured on branch `eplan-parity` with a synthetic 1,000-page project (copies of the
demo schematic pages, release build, desktop):

| Step | Time / size |
| --- | --- |
| Analysis (netlist, index, routing) | 232 ms |
| All checks, before / after the fixes | 2.8 s / about 0.25 s |
| of which terminal strips and PLC, before / after | 2.27 s / 24 ms |
| of which devices, before / after | 224 ms / 49 ms |
| Project clone and compare (undo snapshot) | 15 ms |
| Save / load (JSON) | 32 ms / 42 ms |
| Project as JSON | 9 MB |
| PDF of all pages | 2.3 s |

The check hot spots were quadratic searches (fixed in "Speed up the project check on
large projects"); the 9 MB JSON would exceed browser local storage, so the autosave
now stores the project deflate-compressed ("Autosave the project compressed"). The
analysis after every committed change (about a quarter second plus the checks) is the
remaining limit for very large projects; per-page dirty tracking for the analysis and
IndexedDB for browser storage were the next mitigations and are done (section 7).

**Robustness of free-text conventions.** Several features key off strings in
technical data (`Terminal type: PE`, `Clearance:`, `Mounting holes:`,
`Characteristic:`, `Power loss:`). The mounting and placement values (clearances,
holes, cutouts, power loss, fastening, faces, collision class, terminal type) are now
typed fields with CSV columns and migration of the old lines ("Add typed mounting data
to parts"); electrical values such as `Characteristic:` or `Coil power:` are still
free text.

**Documentation and training.** EPLAN has a help system, training courses,
certification, a user forum and consultants. OpenECraft has the README (long and
good), `docs/plugins.md`, an in-app shortcut list and now a getting-started tutorial
(`docs/tutorial.md`). A user manual with pictures,
tutorials from first page to BOM, a reference of properties, checks and tools, and
sample projects beyond the one demo are needed for adoption.

**Content ecosystem.** Symbols, macros, parts and forms are what EPLAN customers pay
for as much as the software; see sections 3.3, 3.6 and 3.10. A community library
format and a contribution path (pull requests of catalogs exist: ABB, Beckhoff,
DEHN, Hager, MDT, Phoenix Contact, Siemens, Weidmüller) are the start.

**UI customisation.** Keyboard shortcuts can be changed (*View › Keyboard
shortcuts…*) and window arrangements saved as workspaces (*View › Workspaces*);
panels of the schematic workspace dock (section 7); no toolbar customisation, no ribbon.
Minor for parity, matters for migration comfort.

## 5. Prioritised roadmap

Ordered by what an EPLAN user hits first; sizes as in section 3.

**Tier 1: daily-use parity (the editor)**

1. Text formatting and placement: rotation, alignment, styles, text boxes, path
   function texts, special texts, hyperlinks (M + M).
2. Offline device tag numbering with schemes, page renumbering, connection numbering
   formats (M + S + M).
3. Graphical report forms and a form editor; graphical terminal, cable, plug and PLC
   diagrams (L).
4. Check schemes, message ids, severity override and acknowledgement (M).
5. Navigators with drag-and-drop placement (terminal strips, PLC, plugs,
   interruption points, potentials, parts), page tree by structure, project-wide
   Find (L).
6. Multi-level terminals and jumper types (M).
7. Editing aids: grid sizes, coordinate input, align / distribute, insert multiple
   (M); printing (S to M).
8. PDF with embedded Unicode fonts and UI font fallback (M).

**Tier 2: engineering depth**

9. Configurable cross references, star / chain interruption points (M).
10. EPLAN XML project and parts importer (L) and xlsx import / export (M).
11. Typed mounting / placement / drilling data on parts instead of tech-data strings
    (M); part variants and assemblies (S).
12. Project templates, settings schemes, master data synchronisation, macro update
    and macro boxes (M).
13. Accounts, roles and rights on the project server; read-only viewer mode (M).
14. PLC exchange formats of the major vendors; PLC navigator (S each, M).
15. Pre-planning module with planning objects and segments (L).
16. PDF comment import, labeling schemes (S each).

**Tier 3: content**

17. Symbol libraries: full NFPA, GB, GOST, JIS, fluid, P&ID, electronics,
    installation (XL, community-driven).
18. Manufacturer catalogs with macros and 3D data; manufacturer onboarding (XL).
19. User manual, tutorials, sample projects (L, ongoing).
20. More UI languages and fonts (M each).

**Tier 4: platform**

21. Measure and fix performance at 1,000+ pages; browser storage beyond local
    storage (M to L).
22. UI customisation: shortcuts, workspaces, docking (M).
23. Harness nailboard drawings, Smart Mounting instructions (L, M).

## 6. Progress on branch `eplan-parity`

One commit per item, in roadmap order. "Open" lists what of the item is still missing.

| # | Roadmap item | State | Commit | Open |
| --- | --- | --- | --- | --- |
| 1 | Text formatting and placement | done: rotation, alignment, styles, text boxes, path function texts, special texts, hyperlinks | 0da4422 | – |
| 2 | Offline numbering | done: device tags with schemes, page renumbering, wire number formats | 820fc98 | – |
| 3 | Graphical report forms | done: graphical terminal, cable and plug diagrams and PLC assignment; drawn forms with a form editor for every report (section 7) | d7e57f2, eca317c | several row kinds per drawn form |
| 4 | Check message management | done: message codes, levels per check, acceptance, check switches, check schemes as files | 42d8f5b, 9f7367b | – |
| 5 | Navigators and find | partly: project-wide find and replace, page tree by structure, navigator of unplaced PLC channels, part functions and potentials, PLC and strip / plug navigators, all with drag-and-drop placement | 0d77ed5, 57c6af6, 44732b7, 84181bd, 1463540 | – |
| 6 | Multi-level terminals, jumper kinds | done | 787dc48 | – |
| 7 | Editing aids, printing | done: grid sizes, typed points, align, distribute, multiple copy; page selection for PDF / print; File › Print through the browser's print dialog; mounting layout pages show all faces with items | b0ae357, 9ec50c4, 8b876cb | print without the browser |
| 8 | PDF with Unicode fonts, UI font fallback | done: embedded font subset for texts beyond WinAnsi; system CJK fonts as fallback on the desktop, on screen and embedded in PDF exports | c336a16, 9eca2c8 | – |
| 9 | Cross references, interruption points | done: configurable cross references, star / chain interruption points | f10f102 | – |
| 10 | EPLAN XML and xlsx | partly: EPLAN parts XML import / export; xlsx export and import of tables and reports | df94fd2, 7da1077 | EPLAN project import (needs sample exports, section 7) |
| 11 | Typed mounting data, assemblies | done: typed mounting data; assemblies with components in the bill of materials | 11794d0, ed86e62 | part variants |
| 12 | Templates, master data, macros | partly: project templates; settings schemes; macro boxes with update from the library (device tags kept) and saving back | 57c6af6, 341a108, c76948c | master data synchronisation |
| 13 | Accounts and roles | done: accounts, roles per project, read-only viewer | a14895e | – |
| 14 | PLC exchange formats, navigator | done: PLC navigator; TIA Portal, Studio 5000, STEP 7 classic (SDF), TwinCAT 3 (TcGVL), Control Expert (XSY), GX Works device comments | 93fdeb6, 7e3fb7b, 294e3dc, 84181bd | PLCnext |
| 15 | Pre-planning | done: segment tree, planned devices, templates, power budget, state against the schematic, page generation, CSV / Excel | 0328637 | pre-planning macros with several representation types |
| 16 | PDF comment import, labeling schemes | done | aab1be8, 53db461 | – |
| 17 | Symbol libraries | partly: more NFPA / JIC and fluid power symbols, basic electronics, installation plan symbols; GB, GOST and JIS sets with national device letters (section 7) | 31f4166, b2b4e88, cceabce, 3e46498 | ESKD proportions, older JIS symbols, more symbols per set |
| 18–20 | Catalogs, manual, languages | partly: getting-started tutorial (docs/tutorial.md) | 5eeb5a1 | catalogs, full manual with pictures, more languages |
| 21 | Performance, browser storage | done: measured; check hot spots fixed; compressed autosave; incremental netlist per page, parallel checks; IndexedDB (section 7) | 0a895b0, f492586 | incremental checks, analysis off the UI thread |
| 22 | UI customisation | partly: configurable keyboard shortcuts, workspaces (saved window arrangements), docking panels (section 7) | 9898711, 738440b | toolbar customisation |
| 23 | Nailboards, Smart Mounting | done: nailboard drawings to scale; mounting assistant | a2fcb96, 7f8e7bb | 3D harness routing |

Also done beyond the roadmap list: property level change tracking in the revision
overview (old and new value per property, c465254), parameters of exports in the
editor (a form for exporter parameters) and on the command line (`--options`), and
sub-pages (`3.1`, *Page › New sub-page*, kept by page renumbering) from section 3.8.

Notes on the vendor formats: the PLC exports (SDF, TcGVL, XSY), the labeling scheme
templates (CLIP PROJECT, M-Print PRO, Brady, Murrplastik, Partex) and the PDF comment
import follow the published or commonly used layouts of these programs, but were not
tested against the vendor software; the labeling schemes are starting points meant to
be copied and adjusted.

## 7. The remaining gaps (October 2026)

After the work of section 6 six gaps were left, each large or in need of design
decisions: a graphical form editor, an importer for EPLAN projects, incremental
analysis for very large projects, browser storage beyond local storage, docking panels,
and GB / GOST / JIS symbol sets. They were taken in this order: what limits users of
large projects first (measurable, so the effect can be shown), then what an EPLAN user
misses in daily work, then content, and last what cannot be built without material from
outside the project.

| Order | Gap | State | Why in this order |
| --- | --- | --- | --- |
| 1 | Incremental analysis | done | Every committed change paid for the whole analysis and all checks (half a second at 1,000 pages); a risk for adoption with large projects |
| 2 | Browser storage | done | A large project does not fit local storage (about 5 MB); small and self-contained |
| 3 | Docking panels | done | Migration comfort; `egui_tiles` supports egui 0.37 (`egui_dock` does not yet) |
| 4 | Form editor | done (drawn forms) | Builds on the title block editor; works for every report, also those of plugins |
| 5 | GB / GOST / JIS | partly (letters and names) | Content; GB/T 4728 and JIS C 0617 adopt the IEC graphics, so the national device letters are what users miss first |
| 6 | EPLAN project import | open | Needs export files from an EPLAN installation to build and test against (see below) |

**Incremental analysis.** The netlist is built from a scan per page (its connection
points joined by wires and symbol bridges), kept in `analysis::Cache` with a fingerprint
of the page's wires and symbols; only changed pages are scanned again, the assembly of
the nets over the pages stays global. The 3D routing is kept there too and reused while
its routes, the wiring, the language and the project apart from its pages stay the same
(an edit of the schematic: 25 ms instead of 3.8 s for the building services example).
After an edit that changes routes the editor builds the analysis without searching
them (`Analysis::build_kept`: the earlier cables and tubes stand in, `routes_pending`)
and searches them on another thread; the result goes into the cache and the analysis
is built again with it (a layout edit there: 30 ms, the routes follow about 4 s later).
The point-to-point connections are computed once
(the panel wiring took its own copy before), the two netlists are built at the same
time and the checks run on several threads where the platform has them (sequential in
the browser, without a `cfg` branch: `std::thread::available_parallelism` fails there).
Measured with `cargo run --release -p openecraft-api --example scale -- <pages>`
(copies of the demo schematic pages with device tags of their own, release build,
desktop with 32 threads):

| Step | 1,000 pages before | 1,000 pages after | 5,000 pages after |
| --- | --- | --- | --- |
| Analysis, first build | 252 ms | 152 ms | 829 ms |
| Analysis after a change of one page | 252 ms | 104 ms | 786 ms |
| All checks | 246 ms | 62 ms | 352 ms |

The netlist itself is incremental: after a change of one page of the 5,000-page
project it takes 73 ms instead of 259 ms. Still global after a change: the assembly of
the nets over the pages, the point-to-point connections (322 ms at 5,000 pages, linear
in the number of connections), the panel wiring, the 3D routing and every check; the
analysis runs on the UI thread, so at 5,000 pages a committed change costs about 1.3 s. Per-check caching would need checks to declare what they read (a change of
the `Check` registry kind); moving the analysis to a worker thread needs the editor to
cope with an analysis of an older project state (page and symbol indices). Both are
the next steps if projects beyond 5,000 pages appear. A pathological case remains: when
the ducts are far over their fill limit the panel wiring reroutes every wire (seconds at
10,000 routed wires).

**Browser storage.** The autosaved project and parts database go to the app's store
(`openecraft_ui::io::Store`): IndexedDB in the browser, a file per key in the app's data
folder on the desktop. Values are read back asynchronously at start-up and replace the
start-up ones only if nothing was changed meanwhile; until the first write succeeds
both stay in local storage too (private windows without IndexedDB keep working as
before). The File System Access API (saving in place in the browser) is not used: it
exists only in Chromium browsers.

**Docking panels.** In the schematic workspace the page, each navigator tab, the symbol
library and the properties are panes of a tile tree: tabs are dragged beside, above or
below another pane or among its tabs, panes can be closed and shown again (*View ›
Panels*, also *Reset pane layout*). The arrangement is kept by the app and in saved
workspaces. Open: the panel layout and wiring workspaces keep their fixed panels;
toolbar customisation. *Update (UX workplan, `docs/ux-workplan.md`):* both are done; every
workspace and every tool is a pane, and the ribbon that replaced the menus and the tool
bar can be customised.

**Form editor.** A report form can be an own drawing: graphics drawn once per page and
a row drawing repeated for every row with a step that runs downwards (lists) or sideways
(e.g. terminals side by side); texts take the report's columns as placeholders, by
their English name or the one shown, besides `{report}` and the title block
placeholders. The editor starts from the report's table, shows the page with its real
rows and edits lines, rectangles and texts by list, properties and dragging. Drawn
forms work for every report, also those of plugins, and in report blocks, PDF and
print. Open: several row kinds per form (group headers, sub-rows), symbols in forms
(EPLAN draws terminals with symbols), and a library of ready-made drawn forms.

**GB / GOST / JIS.** Three sets of 26 symbols each (coils, contacts, push buttons,
switches, protective devices, motor, lamp, horn, transformer, valve coil, terminal) with
the device letters of GB/T 7159, GOST 2.710 and JIS C 0401, so placed devices are
numbered `-KM1`, `-QF1`, `-MCCB1`, and terminals start strips `-XT1` / `-TB1`. They
are drawn like the IEC symbols, which GB/T 4728 and JIS C 0617 adopt; the ESKD
proportions of GOST 2.755 / 2.756 and the older JIS symbols still common in Japanese
sequence diagrams are not reproduced. More symbols per set and drawings checked by
users of these standards are the next steps (community content, section 4).

**EPLAN project import: what it needs.** EPLAN keeps projects in a proprietary database
(`.elk` with the `.edb` folder) without a published format; reading it directly would
mean reverse engineering and is not planned. The viable route is EPLAN's own export of
the project data as XML (and the `.zw1` / `.pxf` exchange archives), whose structure is
documented only inside EPLAN. An importer needs sample exports from an EPLAN
installation (several projects: IEC and NFPA, with terminals, cables, PLC and macros)
to build and test against, and a mapping of EPLAN symbol libraries (symbol names and
variants) to OpenECraft symbols. The import would then map pages with their structure
identifiers, placed functions to symbols, the connections (EPLAN connects by geometry
as OpenECraft does; connection definition points carry number, colour and
cross-section), devices with their parts (the EPLAN parts XML importer exists),
terminal strips, cables and PLC cards. Until then archived EPLAN projects can be
migrated in parts: the pages as DWG / DXF (*File › Import DXF / DWG as page*), device,
terminal and connection lists as Excel exports (table import), parts as EPLAN parts
XML. Decision for the maintainers: obtaining sample exports (an EPLAN licence is needed
to produce them) and which symbol libraries the mapping covers first.

## Appendix: inventory used for this comparison

| Item | Value |
| --- | --- |
| Core crate lines (without tests) | 41,806 |
| Editor application lines (without tests) | 18,332 |
| Report kinds | 41 built-in + 5 from built-in plugins |
| Distinct check messages in `reports.rs` | about 146 (plus panel, fluid, electrical, rule set and plugin checks) |
| CLI / MCP tools | 215 |
| Built-in property definitions | 434 |
| Symbols | about 320 IEC, 16 NFPA, 11 fluid, 7 P&ID, 8 single-line, black boxes |
| Paper formats | A0 to A4, Letter, Legal, Tabloid, custom |
| Structure aspects | function, installation site, location, product |
| UI languages | English, German, French, Spanish |
| Catalog translation lines | 5,128 |
| Bundled manufacturer catalogs | 8 |
| Commits | 130 (to 19 October 2026) |
