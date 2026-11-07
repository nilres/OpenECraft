# UX workplan: an EPLAN-style editor shell

Date: 2026-10-25. Baseline: `master` at 545e6be. Branch: `worktree-ux-refactoring`.

Method: read the editor shell (`openecraft/src/app.rs` and its 44 submodules, `dock.rs`,
`editor.rs`, `panel_ui.rs`, `wiring_ui.rs`, `mounting_ui.rs`, `parts_ui.rs`,
`shortcuts.rs`, `workspaces.rs`), counted its menus, windows, navigator tabs, tools and
shortcuts, listed what the plugin registry publishes, and compared the result with the
working style of the EPLAN Platform (ribbon, backstage, navigators, Insert center,
property dialogs, message management, workspaces) as described in EPLAN's help pages
(`eplan.help`, Platform 2022 to 2026: "Background: user interface", "Insert center",
"Favorites"). Where a statement about EPLAN comes from memory rather than from those
pages it is marked "from memory".

The engine has outgrown its shell: the function set is at EPLAN level in most areas
(`docs/eplan-gap-analysis.md`), but it is reached through nine menus, one of them with
36 entries, and about 55 floating windows. This document is the plan for a shell that
keeps up: what is wrong today, what the target looks like, the work packages in order,
and a table that gives every existing function its place in the new shell so that
nothing is lost on the way.

## Status (2026-10-25)

All ten packages were carried out on `worktree-ux-refactoring`. What was built, and
where it differs from the plan below:

| Package | State | Notes |
| --- | --- | --- |
| 0 Foundations | done | `openecraft/src/commands.rs` (about 230 commands), `openecraft/src/ribbon.rs`, `openecraft/src/style.rs` (state colours, contrast tested), `openecraft_ui::icons` (143 vector icons), UI scale; the open flags of the tools stay as requests that `App::open_requests` turns into panes instead of a separate windows struct |
| 1 Shell | done | Ribbon with context tabs and minimise, backstage, quick access row with undo list and page switch, status bar segments and message history, command search; the menu bar and `toolbar.rs` are gone |
| 2 One dock | done | One `egui_tiles` tree per workspace (schematic, panel layout, wiring, mounting, review, pre-planning); panel layout and wiring are panes; page tabs; floating panes (pane menu › Float). The pane header is a helper of the app (`navigator_header`), not of `openecraft-ui`, since only the editor uses it |
| 3 Navigators | done | 33 tool windows became panes; Edit in table from the navigators; page row context menu; the Devices navigator follows the selection; context menu on the page; Edit in table rows for cables, plug pins, PLC points, pages and parts (also for the command line and MCP). Symbols (Insert center) and unplaced functions are dragged into the page; macros and devices from parts are placed by click |
| 4 Insert center | done | Search with `*` / `?`, favorites (export / import), recently used (20), symbols, macros, devices from parts with functions, unplaced functions; list and tile view |
| 5 Properties | done | Tabs Device, Display, Parts, Properties, Connections; window on F2 / double click; common fields (function text, mirror) for several symbols |
| 6 Panes and settings | done | Messages pane as a sortable table, Reports pane with a report list, Tables pane; Settings dialog with project and user nodes; tools that keep project settings are linked from it rather than embedded |
| 7 Layout space | done | Context tab with the panel tools, views, show options, NC data, 3D macros and accessories; the canvas keeps only the profile part chooser; tablet mode (scale and 40 pt targets) |
| 8 Start and help | done | Start page on first start, Help pane with quick guide, the rendered tutorial (English) and the shortcut list, F1 per pane; `docs/tutorial.md` and the README use the ribbon paths |
| 9 Keyboard | done | Key tips (Alt or F10, then letters), F6 / Shift+F6, Ctrl+Tab, Ins, F2, F4, Ctrl+Shift+P, every command bindable; screen-reader labels on the icon buttons; Tab / Shift+Tab move through the ribbon and Enter runs a command. Not done: arrow keys inside the ribbon |
| 10 Customisation | done | Hide groups, own groups, export / import, kept in saved workspaces; the new texts in all 20 languages, the CJK font subsets regenerated. README screenshots in docs/images. Not done: screenshots for the installers |

## 1. Where the UI stands today

### 1.1 Shell

| Element | Today | Where |
| --- | --- | --- |
| Menu bar | 9 menus: File, Edit, Insert, Page, Project, Reports, Plugins, View, Help. The Project menu has 36 entries without grouping, in no recognisable order (Cables between Property database and Potentials, Symbol editor after Configurator) | `app/menus.rs` |
| Tool bar | One wrapping row: workspace switch, tools, coordinate field, draw menu, dimension menu, definition tools, layer, auto-connect, route, review tools, undo / redo, rotate, delete, fit, zoom, page switch, reports. Wraps to two or three rows below 1,600 px | `app/toolbar.rs`, `drawing_ui.rs`, `review_ui.rs`, `arrange_ui.rs` |
| Workspaces | Three shells that share nothing: *Schematic* (dock tree of panes), *Panel layout* (fixed left / right / centre panels), *Wiring* (fixed left panel and workbench). Saved "workspaces" store the dock tree and the open windows | `app.rs` (`ui`), `dock.rs`, `workspaces.rs` |
| Docked panes | `Pane`: the page (exactly one), 10 navigator tabs, symbol library, properties; `egui_tiles` 0.18 | `dock.rs`, `app/dock_ui.rs` |
| Navigator tabs | Pages, Devices, Connections, Cables, Layers, Check, Review, Unplaced, PLC, Strips / plugs. Each has its own header row with its own buttons; only Devices and Unplaced have a filter; no context menus except Devices and Check | `app/navigator.rs`, `drawing_ui.rs`, `check_ui.rs`, `review_ui.rs` |
| Floating windows | About 55 `egui::Window`s, each with its own `open` flag on `App` or a sub-struct, sizes from 420 to 1,150 px, opened from the Project menu; several (Cables, Potentials, Terminal strips, Plugs, PLC overview, Macros, Harnesses, Revisions, Change requests, Pre-planning) are navigators in all but name | `app/windows.rs` and 36 `*_ui.rs` |
| Properties | One scrolled column: selection summary, then a grid per object kind, then collapsing headers "Device properties", "Function properties", lists for parts, cross references and connections. Double-click on the page sets `open_properties`, which `page_pane` discards | `app/properties_panel.rs`, `app/dock_ui.rs:126` |
| Context menus | Device rows (navigator) and check messages only. None on the page, on wires, on page rows, on tabs | `editor.rs` (no `context_menu`) |
| Status bar | One monospace text from the tool plus file name and a six-second message. No coordinates, grid, snap, layer, zoom, page scale, language, lock or server state | `app.rs` (`ui`) |
| Shortcuts | 16 configurable actions, fixed keys for the rest; the shortcut window is a flat grid | `shortcuts.rs`, `app/windows.rs` |
| Plugin commands | About 190 commands in 26 built-in area plugins, shown under *Plugins › <area>* by their snake_case names (`set_wire_defaults`) with the help text as tooltip; the same functions have hand-made dialogs elsewhere, so a user meets them twice | `app/plugins_ui.rs`, `crates/openecraft-api/src/builtin/` |
| Reports | A window with 41 report kinds as a wrapped row of toggles, plus a *Reports* menu with the same list | `app/windows.rs` |
| Settings | Spread over eleven windows (wire numbering, cross references, check settings, marking, hose defaults, potentials, languages, structure, properties, numbering, settings scheme export) and the View menu (grid, theme, language) | various |
| Help | One window with 16 numbered paragraphs and the shortcut table; `docs/tutorial.md` is not reachable from the app | `app/windows.rs` |
| Start | The app opens on the last project or the demo; no start page, no recent files | `app.rs` (`new`) |
| Icons | Emoji glyphs of the bundled fonts (🗋 🗄 ✔ ⟳ 🗑 📊 ⬉ ✏), mixed with text buttons; the glyph test limits the set | `glyphs.rs` |
| Accessibility | `accesskit` is enabled in eframe, so widgets are exposed to screen readers; no keyboard access to menus by accelerator, no focus order design, no UI scale setting, no key tips | `Cargo.toml` |

### 1.2 What is good and stays

- The dock tree (`egui_tiles`) with saved arrangements and workspaces: EPLAN's dockable
  dialogs and workspaces work the same way.
- The navigator tabs that exist (pages by structure, devices by structure, unplaced
  functions, PLC and strips with drag-and-drop placement, check with accept / level /
  switch off, review).
- `Session` apart from `App`, `dock.rs` and `workspaces.rs` without UI code and unit
  tested: the pattern for the new shell state (section 5).
- The plugin registry as the single source of commands, checks, reports and exporters:
  the ribbon and the command search will read from it instead of bypassing it.
- Undo by snapshot, the `Draw` trait, the i18n catalog with the glyph and convention
  tests: none of this changes.

### 1.3 The problems, in the user's words

1. **"Where is it?"** Everything project-related is one 36-entry menu; the parts of a
   workflow (cables: Project › Cables…, then select wires, then Properties › Assign to
   cable) are in three places; plugin commands duplicate dialogs under technical names.
2. **"Why does this workspace look different?"** Panel layout and wiring lose the
   navigators, the properties pane and the tool bar; the dock arrangement does not apply.
3. **"Why can't I right-click?"** No context menu on the page, no properties on double
   click, no page tabs, no "go to counterpart", no "place" from most lists.
4. **"Too many windows."** A session with cables, strips, PLC and parts open has four
   overlapping 900-px windows over the page; they do not dock, do not remember their
   place in the dock and are closed by the workspace switch.
5. **"The tool bar wraps."** On a laptop the single tool bar takes three rows; the
   workspace switch, the tools and the page navigation compete for the same row.
6. **"Nothing tells me the state."** Coordinates, grid, snap, active layer, lock,
   routing state, server state: all missing from the status bar; messages vanish after
   six seconds.
7. **"Settings are everywhere."** Project settings are eleven dialogs; user settings
   (language, theme, shortcuts, servers) are menu items and dialog corners.
8. **"The first hour."** A new user sees the demo project and a 16-paragraph help
   window; the tutorial lives in the repository.

## 2. The EPLAN model to orient on

What an EPLAN user expects, and how OpenECraft takes it over. The aim is the working
style, not a pixel copy; names in *italics* are EPLAN's.

| EPLAN | Take over as | Note |
| --- | --- | --- |
| *Ribbon*: tabs with command groups; *Home* always shown, *Insert*, *Edit*, *View* context-dependent; can be minimised; customisable, exportable | Ribbon with fixed tabs and context tabs per workspace; groups with large and small buttons; collapses to tab titles; customisation in a later package | EPLAN's further tabs (*Devices*, *Connections*, *Tools*, *Master data*, *Pre-planning*, from memory) give the grouping of section 3.2 |
| *Backstage view* (File) with Open, Recent, New from basic project, Save, Export, Print, Settings, Info | Backstage: full-window page with a left column and content | Also home for the parts and project servers, Vault and the project database |
| *Quick access toolbar* above the ribbon: Previous page, Next page, Undo list | Row at the top: undo / redo with history list, page navigation, save, workspace switch | |
| *Find command*: typing a command name finds it with its tab and group | Command search (Ctrl+Shift+P and a box in the ribbon): every command of the editor and of the plugins, with its place and shortcut | Reaches functions that a user has not found in the ribbon |
| *Navigators*: page, device, terminal strip, plug, cable, connection, potential, interruption point, PLC, parts, macro, placeholder, pre-planning, layout space, project options, revision; tree and list view, filter, place by drag, synchronised selection, *Go to (graphic)*, *Edit in table*, context menus | Every project data list is a docked pane with the same header: view toggle, filter, scheme, context menu; selection follows the page and the page follows a double-click | Converts the navigator-like windows (section 6, column "Pane") |
| *Insert center* (Ins): one dialog for symbols, devices, window and symbol macros; search; recent (20); favorites (star, exportable); tiles with preview; pinned or as a vertical tab | Insert center pane replaces the symbol library: symbols, user symbols, macros, unplaced functions, devices from parts with function templates; favorites and recent per user | |
| *Properties* dialog with tabs (device data, display, symbol / function data, parts, properties, …) on double click | Docked properties pane with the same tabs, and the same content as a window on double click or F2 | The pane stays docked for direct editing; the window is for the user who wants it beside the page |
| *Message management*: table of messages with filters, *Go to*, accepted flag, check run settings | Messages pane: table with columns severity, code, message, page, object, check; filters and schemes; accept; settings | Replaces the Check tab's label list |
| *Settings*: one dialog with a tree *Projects / User / Company / Station* | Settings dialog with a tree: *Project* (numbering, potentials, cross references, checks, wire defaults, marking, hoses, languages, structure, properties, forms) and *User* (language, theme, UI scale, shortcuts, ribbon, servers, Vault) | Settings schemes export and import stay |
| *Workspaces*: saved arrangement of dialogs, reselectable | Keep; add presets: Schematic, Panel layout, Wiring, Mounting, Review, Pre-planning | |
| Page tabs and the *Page navigator* in tree or list view | Open pages as tabs of the page pane; page tree by structure stays | Several page panes side by side is a later package (section 7) |
| Status bar: coordinates, object snap, snap to grid, grid, page scale, zoom, language; main window: connection state, system messages | Same fields, as clickable segments (grid opens the grid menu, language the language menu) | |
| Context menus everywhere (page, navigators, tabs) | Same | |
| Dark theme, scalable UI | Keep dark theme; add UI scale | |

Not taken over: EPLAN's modal dialog chains (the property dialog closes before the
page can be clicked), the separate *Layout space* application window and the licence
dependent hiding of tabs. OpenECraft keeps direct editing in docked panes and shows
every function to every user.

## 3. The target shell

### 3.1 Layout

```
┌ Quick access: save · undo ▾ · redo · ◀ page ▶ · workspace ▾ ─── command search ─ 🔍 ┐
├ Ribbon tabs: File | Home | Insert | Edit | View | Devices | Connections | Project data | Tools | Master data | Plugins | Help   [context: Layout space / Wiring]
├ Ribbon groups of the active tab (large + small buttons, menus, toggles); minimisable  ┤
├──────────────┬──────────────────────────────────────────────┬────────────────────────┤
│ Navigators   │ Page tabs:  =A1+CAB1/1 ·  /2 ·  Layout +CAB1  │ Insert center          │
│ (tabs:       │                                              │ (search, favorites,    │
│ Pages,       │              page / 3D view / table          │  recent, tree)         │
│ Devices,     │                                              ├────────────────────────┤
│ Strips, …)   │                                              │ Properties (tabs)      │
├──────────────┴──────────────────────────────────────────────┴────────────────────────┤
│ Messages pane (dockable, usually at the bottom): table of check results              │
├ Status: x 112.5  y 40.0 │ grid 4 mm ▾ │ snap │ layer: Graphics ▾ │ 100 % │ =A1+CAB1/3 │ de ▾ │ 🔒 │ ● server ┤
```

Every region is a pane of the dock tree, including the 3D view and the tables; the
ribbon, the quick access row and the status bar are the only fixed parts.

### 3.2 Ribbon tabs and groups

The tab set follows EPLAN's order and vocabulary; the groups are OpenECraft's functions.
Each entry is a command of the command registry (section 3.3); the table in section 6
maps every existing menu entry to one of these groups.

| Tab | Groups |
| --- | --- |
| **File** (backstage) | New (blank, templates, from template file, demo), Open (file, recent, project database, project server, Vault), Save / Save as / Download, Save as template, Import (DXF / DWG page, settings scheme, parts, macros, symbols, documents), Export (every exporter of the registry with its parameter form, PDF per language), Print, Info (project properties, verification), Settings, Help, Quit |
| **Home** | Clipboard (cut, copy, paste, multiple copy), Page (new page kinds, page properties, renumber, templates), Edit (undo, redo, delete, rotate, mirror, align, distribute, move by), Find (find and replace, command search), Select (select all, select by filter), Reports (open the Reports pane) |
| **Insert** | Symbols (Insert center, categories), Devices (device from part, unplaced function), Macros (window / page macro, save as macro), Connections (wire, connection definition point, cable definition line, interruption point, potential), Text (text, path function text, special text), Graphics (line, rectangle, circle, ellipse, arc, polyline, polygon, image), Dimensions, Review (comment, redline, greenline) |
| **Edit** | Routing (auto-connect, route, join collinear, apply connection definitions), Arrange (align, distribute, grid), Text (rotation, alignment), Layer (active layer, layers pane), Properties (properties window, placement, property placement) |
| **View** | Zoom (fit, 100 %, zoom window, previous page, next page), Show (grid, connection points, revision marks, macro boxes, wire colours), Panes (every pane as a toggle), Workspaces (presets, saved ones, save, reset), Appearance (theme, UI scale, language) |
| **Devices** | Numbering (device numbering, rename, apply designations), Terminals (strips, jumpers, multi-level, sort, renumber), Plugs, PLC (overview, hardware and bus, addresses, exchange), Parts (parts management, part selection, data portal, synchronise), Structure (structure management, work areas), Pre-planning (segments, planned devices, single-line / multi-line) |
| **Connections** | Numbering (wire numbering, schemes), Cables (cables, assign, cable definitions), Potentials (potentials, definitions), Harnesses (harnesses, nailboards, KBL), Field wiring (plant layout, routes, field cables, bonding), Marking, Cross references (format, interruption points, contact images) |
| **Project data** | Check (run, messages pane, settings, schemes), Reports (reports pane, report blocks, update blocks, forms), Revisions (revisions, approval, release, baselines, compare, change requests), Review (comments and markup pane), Languages (project languages, translate, dictionary), Options (model options, configurator), Tables (edit in table: devices, connections, terminals, cables, plugs, PLC, pages, parts), Management (project management, subprojects, archives, verification, settings scheme) |
| **Tools** | Calculations (sizing, short circuit, selectivity, loads, EMC, functional safety, rule sets), Fluid and process (hoses, PCT loops), Integration (ERP / PDM / PLM / ECAD / MCAD, wiring machines), Collaboration (project server, locks, Vault), Automation (plugins, command line and MCP hint) |
| **Master data** | Symbols (symbol editor, libraries import / export), Forms (title blocks, plot frames, page templates, report forms, drawn forms), Macros (macro library, import / export), Parts (database, catalogs, import EDZ / XML / CSV), Properties (property database, function definitions, placement schemes), Project templates |
| **Plugins** | One group per enabled plugin with commands that no built-in group claims; *Manage plugins* | Built from the registry; empty groups are hidden |
| **Help** | Getting started (start page), Tutorial (rendered `docs/tutorial.md`), Shortcuts, About |
| **Layout space** (context, panel workspace) | Enclosures (new, properties, line-up), Place (devices to place, accessories, 3D macros), Draw (rail, duct, busbar, cutout, routing path, reservation), View (front, door, walls, 3D, room, thermal), Routing (route, segregation, duct fill), Production (NC data DXF / DWG / CSV, CNC, copper, hand-off), Reports |
| **Wiring** (context, wiring and mounting workspaces) | Mode (wiring, mounting), Order, Progress, Export CSV, Tablet mode |

A context tab is shown and activated when its workspace is active and stays available
as a plain tab otherwise, so nothing depends on the workspace switch.

### 3.3 The command registry of the editor

The ribbon, the backstage, the quick access row, every context menu, the command
search, the shortcut editor and the saved customisation all read one table:

```rust
pub struct Command {
    pub id: &'static str,          // "page.new_schematic"
    pub title: &'static str,       // tr_const!("New schematic page")
    pub icon: Icon,                // see 3.8
    pub group: Group,              // tab and group of the ribbon
    pub shortcut: Option<Shortcut>,
    pub enabled: fn(&App) -> bool,
    pub run: fn(&mut App, &egui::Context),
}
```

`openecraft/src/commands.rs` holds the table and the lookup (no UI code, unit tested);
`openecraft/src/app/ribbon.rs` draws it. Plugin commands of `openecraft_api` are wrapped
into `Command`s at start-up (and when plugins change) under the *Plugins* tab or, where
a built-in area claims them (`drawing`, `pages`, `terminals`, …), into that area's
group, so a function exists once. The dialogs that duplicate a plugin command keep
their forms but call the command.

Tests (`openecraft/src/commands/tests.rs` and `conventions/tests.rs`):

- every `Command` is in exactly one ribbon group or backstage page;
- every `open` flag of a window and every `Pane` is reached by a command;
- every command title and group title is in the catalog in the four languages;
- shortcuts have no conflicts in the defaults;
- every `Pane` has a title, an icon and a toggle under *View › Panes*.

This is the mechanism behind "all functions accessible": the test fails when a feature
is added without a place in the shell.

### 3.4 Panes

`Pane` grows from 13 to about 30 variants; the navigator-like windows become panes
(section 6). All panes share one header widget (`openecraft_ui::pane_header`): title,
view toggle (tree / list where it applies), filter box, scheme chooser, pane menu
(close, float, help). Panes of project data share one behaviour contract:

- single click selects and highlights on the page (synchronised selection, both ways);
- double click or Enter goes to the object (page, 3D view or table);
- right click opens the context menu of the object (the same entries as on the page);
- drag of an unplaced function, a device or a macro into the page places it;
- *Edit in table* of the pane opens the Tables pane filtered to its rows;
- the pane keeps its filter, view and expanded nodes in the saved workspace.

Panes that keep being windows: wizards and one-shot forms (save as macro, rename
device, move by, multiple copy, new enclosure, export parameters, plugin command
parameters), the Settings dialog and the Properties window. The parts management, the
data portal, the symbol editor and the forms editor are large editors; they become
panes of a tab beside the page ("document panes") so they can be put side by side
with the page or maximised.

### 3.5 Insert center

Replaces the symbol library pane (`app/toolbar.rs::library`). Content:

- a search box (symbol name in English and in the UI language, device letter, macro
  name, part number, function text of unplaced functions; `*` and `?` placeholders);
- rows *Favorites* and *Recent* (tiles with preview, at most 20 recent, favorites
  stored by the app and exportable);
- a tree: Symbols (built-in categories and user libraries), Macros (window, page,
  3D), Devices (parts with function templates: placing a device places its first
  unplaced function and creates the device), Unplaced (today's Unplaced tab, kept as
  a pane of its own too), Graphics and texts;
- variants of a symbol (rotation, mirror) chosen with R / M as today, shown as a row
  of tiles;
- Ins opens or focuses it; Esc returns to the page; a click starts placement, drag
  places at the drop point (`PlacePayload` exists).

The previews use `symbol_icon` and the macro preview; tiles are a fixed grid so the
pane works at 220 px width (list) and at 600 px (grid).

### 3.6 Properties

The pane keeps direct editing and gains tabs that group what today is one column:

| Tab | Content |
| --- | --- |
| Device / function | Tag with structure, assign to device, label (terminal, pin, address, potential), function text, function definition, connection points, orientation, visibility of texts |
| Display | Placement of tag and property texts (drag or table), text rotation and alignment, layer, colour, line width, hidden |
| Parts | Assigned parts with quantities, part selection, synchronise, function templates |
| Properties | The property grid of the object (user and indexed properties) |
| Connections | Connections of the object with targets and states; cross references, contact image |
| Page | For an empty selection: name, description, kind, title block, frame, structure, scale, document |

Double click, F2 and *Edit › Properties* open the same content as a window beside the
page (`out.open_properties` is honoured). Multi-selection shows the common fields
with "(several)" where values differ and writes to all, instead of today's reduced set.

### 3.7 Messages, reports, tables

- **Messages pane**: table with sortable columns (severity, code, message, page,
  object, check, accepted by), filter row, the toggles of today, context menu (go to,
  accept, level, switch off check), buttons for check settings and schemes, CSV
  export. The tab title keeps the error count.
- **Reports pane**: the report list as a tree on the left (built-in groups and plugin
  reports), the table on the right with the actions of today (CSV, Excel, insert as
  page, form, report blocks). Replaces the Reports window and menu.
- **Tables pane**: *Edit in table* as a pane with a row kind chooser; every navigator
  offers *Edit in table* for its rows (devices, connections, terminals today; cables,
  plugs, PLC, pages and parts need `tables.rs` row kinds, each S).

### 3.8 Appearance

- **Icons**: one icon set drawn as vector paths (`openecraft_ui::icons`, a table of
  small polyline / polygon definitions rendered with the painter at 16 and 24 px) for
  the ribbon, panes and context menus; emoji glyphs disappear from buttons. No icon
  font, so the glyph test and the bundle size stay as they are. The set is about 120
  icons; symbols keep `symbol_icon`.
- **Typography and spacing**: one `Style` for the whole app (text sizes for ribbon,
  pane headers, tables; spacing of groups and grids), applied in `App::new`; the panel
  layout and wiring workspaces use the same style.
- **Theme**: light, dark and system as today, chosen in Settings and the status bar;
  the canvas palette (`editor::Palette`) and the 3D view follow.
- **UI scale**: a setting (75 to 200 %) stored by the app; tablet mode of the wiring
  workspace sets 150 % and larger hit targets.

### 3.9 Keyboard and accessibility

- Every command has a place in the ribbon and may have a shortcut; the shortcut editor
  lists all commands (grouped by tab, searchable) instead of 16 actions.
- Alt shows key tips on the ribbon tabs and groups (letters drawn over the buttons);
  Alt+letter opens a tab, a second letter runs the command; Esc backs out. F10 focuses
  the ribbon, arrows move inside it (egui's focus handling, `accesskit` roles set).
- F6 cycles the focus through the panes, Ctrl+Tab through the page tabs; Ins opens the
  Insert center, F2 the properties, F4 the messages, Ctrl+Shift+P the command search.
- Every icon button has a text label for the screen reader (`on_hover_text` is not
  enough; `accesskit` label via `Button::new(...).widget_info`), every pane a landmark.
- Contrast: the colours used for states (errors, warnings, locks, pens) are defined
  once with light and dark variants that meet 4.5:1 against their background; the
  canvas palette already has both.
- Touch: the wiring and mounting workspaces in tablet mode have 44 px targets and
  no hover-only information.

### 3.10 Start page and help

- **Start page** (shown when no project was restored, and from *File › Home*): recent
  projects, templates, the demo, open from server / Vault, the tutorial steps as cards,
  "what is new".
- **Help**: `docs/tutorial.md` and the shortcut list rendered in a pane (a small
  Markdown renderer for headings, lists, code and links; `egui_commonmark` is an
  option if it supports egui 0.37); F1 opens the help page of the active pane or tab.
- The 16 numbered paragraphs of the help window go into the tutorial.

## 4. Work packages

Sizes: S = up to three days, M = one to two weeks, L = three to five weeks. Order by
dependency and by what the user feels first; packages marked ∥ can run in parallel
with the one above them in separate sessions, because they touch different files.

| # | Package | Size | Depends on | Files (new or changed) |
| --- | --- | --- | --- | --- |
| 0 | **Foundations**: command registry with tests; `Windows` struct that collects the `open` flags and the pane list; icon set; app style; UI scale setting | M | – | `openecraft/src/commands.rs` (+ tests), `openecraft/src/windows_state.rs`, `crates/openecraft-ui/src/icons.rs`, `style.rs`, `app.rs` |
| 1 | **Shell**: ribbon with tabs, groups, minimise and context tabs; backstage; quick access row with undo list; status bar segments; command search; the menu bar goes | L | 0 | `app/ribbon.rs`, `app/backstage.rs`, `app/quick_access.rs`, `app/status_bar.rs`, `app/command_search.rs`; `app/menus.rs` and `app/toolbar.rs` removed |
| 2 | **One dock for all workspaces**: the panel layout (enclosures navigator, 3D / front view, properties) and the wiring workbench become panes; workspace presets; page tabs in the page pane; pane header widget | M | 0 | `dock.rs`, `app/dock_ui.rs`, `panel_ui.rs` (split into `panel_ui/{navigator,canvas,properties,dialogs}.rs`), `wiring_ui.rs`, `mounting_ui.rs`, `workspaces.rs`, `crates/openecraft-ui/src/pane_header.rs` |
| 3 ∥ | **Navigators**: the navigator-like windows become panes with the shared behaviour (selection sync, go to, context menu, drag to place, edit in table); context menus on the page and on page rows; Edit in table row kinds for cables, plugs, PLC, pages, parts | L | 0, 2 | `app/navigator.rs` (one file per pane under `app/navigators/`), `cables_ui.rs`, `potentials_ui.rs`, `engineering.rs` (strips, PLC, macros, revisions), `plugs_ui.rs`, `harness_ui.rs`, `requests_ui.rs`, `preplanning_ui.rs`, `editor.rs` (context menu, selection events), `crates/openecraft-core/src/tables.rs` |
| 4 ∥ | **Insert center**: search, favorites, recent, tree, tiles, devices from parts, Ins key | M | 0, 2 | `app/insert_center.rs` (+ `insert_center/model.rs` tested: search, favorites, recent), replaces `library` in `toolbar.rs` |
| 5 ∥ | **Properties**: tabs, properties window on double click and F2, multi-selection with common fields, display tab (text placement, rotation, alignment) | M | 0 | `app/properties_panel.rs` → `app/properties/{device,display,parts,grid,connections,page}.rs`, `app/dock_ui.rs` |
| 6 | **Messages, Reports, Tables, Settings**: the three panes; the Settings dialog with the Project and User trees collecting the eleven settings windows; settings schemes | M | 1, 2 | `app/messages.rs`, `app/reports_pane.rs`, `app/tables_ui.rs`, `app/settings/{mod,project,user}.rs`; removes `check_settings_window`, `xref_window`, `numbering_window`, the settings parts of `cables_ui.rs`, `engineering.rs` (marking), `process_ui.rs`, `languages_ui.rs` |
| 7 | **Layout space and wiring in the shell**: context tabs, the panel tool bar as ribbon groups, the enclosure dialogs, tablet mode for wiring and mounting | M | 1, 2 | `app/ribbon_layout.rs`, `app/ribbon_wiring.rs`, `panel_ui/*`, `wiring_ui.rs`, `mounting_ui.rs` |
| 8 | **Start page, help, onboarding**: start page, tutorial pane, F1, first-run tips, "what is new" | S–M | 1 | `app/start_page.rs`, `app/help_pane.rs`, `docs/tutorial.md` |
| 9 | **Keyboard and accessibility**: key tips, F6 / Ctrl+Tab cycles, shortcut editor over all commands, screen-reader labels, contrast pass, keyboard-only walkthrough as a test checklist | M | 1, 3, 5 | `shortcuts.rs`, `app/ribbon.rs`, `app/key_tips.rs`, every pane |
| 10 | **Ribbon customisation and polish**: hide / show groups, own groups from commands, export / import; saved workspaces include it; screenshots for README and the installers; translations of the new texts into the 16 further languages | M | 1–9 | `app/ribbon_custom.rs`, `workspaces.rs`, catalogs |

Milestones a user can see:

1. After 0 + 1: the ribbon, the backstage and the status bar, every old menu entry in
   a group, the command search. The windows are still windows.
2. After 2 + 3 + 4 + 5: the panel layout and wiring share the shell, the project data
   lives in navigators with context menus and drag-and-drop, the Insert center and
   the tabbed properties pane.
3. After 6 + 7 + 8: messages, reports and tables as panes, one Settings dialog, the
   layout space tab, the start page.
4. After 9 + 10: keyboard-complete, customisable, translated, documented.

Each package is one pull request (or two for L), each passing the three gate commands
(`cargo test --workspace`, `cargo clippy --workspace --all-targets`, the wasm clippy)
and the acceptance criteria of section 8. The old menu bar is removed in package 1,
not kept as an option: two shells double the catalog and the tests.

## 5. Rules for the work

From `AGENTS.md`, applied to the shell:

- **State without UI.** The command table, the pane list, the ribbon model (tabs,
  groups, customisation), the Insert center model (search, favorites, recent), the
  messages filter and the settings tree are plain modules with tests, like `dock.rs`
  and `workspaces.rs`; the `*_ui.rs` modules only draw and dispatch.
- **Extensibility.** The ribbon shows registry commands, checks, reports and exporters;
  a plugin's commands appear in the *Plugins* tab or in the group its area names
  without any front-end change. Where a feature needs a new registry kind (panes or
  navigator rows contributed by plugins), add it to `Registry` first and document it
  in `docs/plugins.md`.
- **Languages.** Every tab, group, command title, pane title, tooltip, status segment
  and key tip is a catalog entry in all four languages; the command titles use
  `tr_const!` in the table and `tr` where drawn. Expect 300 to 500 new lines in
  `catalog/editor/required.tsv`; add them to the 16 further catalogs in package 10 and run the font
  subset script for the CJK catalogs.
- **Tests layout.** `src/<module>/tests.rs` for every new module; the conventions test
  gets the completeness rules of section 3.3.
- **Platform.** No `cfg(target_arch)` outside `openecraft-ui::io` and `main.rs`; the
  backstage asks `FileIo::can_save_in_place()` and `has_own_window()` as the File menu
  does today.
- **Compatibility.** Saved dock trees and workspaces from before are read; unknown
  panes are dropped, missing ones added at the default place (`dock::from_json`
  already validates). Keyboard shortcuts stored by id keep working; new commands get
  new ids.
- **Icons and glyphs.** Vector icons, no new characters in string literals; the glyph
  test stays as it is.
- **Undo.** Nothing new: the shell mutates `Project` as today.

## 6. Every function and its place in the new shell

The table gives each entry of the current shell a home. "Pane" means a docked pane,
"Window" a floating dialog kept as such, "Backstage" a page of the File tab,
"Settings" a node of the Settings dialog. Where a function exists only as a plugin
command today it is listed under its area.

### 6.1 File menu

| Today | New place |
| --- | --- |
| New project | Backstage › New |
| New project from template (built-in, from file) | Backstage › New › templates |
| Open… | Backstage › Open; quick access |
| Save as project template… | Backstage › Save as template |
| Open demo project | Backstage › New › Demo; start page |
| Save / Save to project server / Check in to Vault / Download project / Save as… | Backstage › Save (label as today); quick access Save |
| Project server… | Backstage › Open › Project server and Tools › Collaboration |
| Autodesk Vault… | Backstage › Open › Vault and Tools › Collaboration |
| Export › every exporter | Backstage › Export (list with parameter form) |
| Export as PDF in language | Backstage › Export › PDF › language chooser |
| Print › all pages / current page | Backstage › Print |
| Import DXF / DWG as page… | Backstage › Import |
| Quit | Backstage › Quit |

### 6.2 Edit menu, tool bar and arrange

| Today | New place |
| --- | --- |
| Undo, Redo | Quick access (with history list); Home › Edit |
| Rotate, Mirror, Delete | Home › Edit; page context menu |
| Find… | Home › Find |
| Select all | Home › Select |
| Copy / cut / paste hint | Home › Clipboard as real buttons |
| Align, Distribute, Move selection by…, Multiple copy… | Home › Edit (Arrange menu); page context menu |
| Auto-connect selection, Join collinear wires, Apply connection definitions | Edit › Routing |
| Save selection as macro… | Insert › Macros; page context menu |
| Select / Wire / Text tools | Insert › Connections and Insert › Text; Esc returns to Select; the active tool is shown in the status bar |
| Coordinate field | Status bar segment (click or Ctrl+G to type) |
| Draw menu (line … polygon), Dimension menu | Insert › Graphics, Insert › Dimensions |
| Conn. def., Cable def. | Insert › Connections |
| Layer combo | Edit › Layer and status bar segment |
| Auto-connect, Route toggles | Edit › Routing (toggles) |
| Comment, pen, markup tools | Insert › Review |
| Placing: name | Status bar |
| Fit, zoom percentage | View › Zoom; status bar segment |
| Previous / next page, page title | Quick access; page tabs |
| Reports button | Project data › Reports (opens the Reports pane) |

### 6.3 Insert menu

| Today | New place |
| --- | --- |
| Macro… | Insert › Macros; Insert center › Macros |
| Wire, Text, Path function text, Special text | Insert › Connections / Text |
| Graphic line / rectangle, Graphics submenu, Dimension, Image… | Insert › Graphics / Dimensions |
| Connection definition point, Cable definition | Insert › Connections |
| Symbol categories, user symbols | Insert center (tree, search, favorites, recent) |

### 6.4 Page menu

| Today | New place |
| --- | --- |
| New schematic page, sub-page, cover sheet, overview, graphics, document page | Home › Page › New (menu); Pages pane header; page tab context menu |
| Renumber pages… | Home › Page; Pages pane context menu |
| New page from template, New report page, New mounting layout page | Home › Page › New |
| Page properties… | Home › Page; Properties pane › Page tab; page tab context menu |
| Save page as template… / as macro… | Home › Page; Insert › Macros |
| Move page up / down, Delete page | Pages pane (drag, context menu); Home › Page |

### 6.5 Project menu

| Today (36 entries) | New place |
| --- | --- |
| Settings scheme › export / import | Settings dialog › Project › Scheme; Backstage › Import / Export |
| Project properties… | Backstage › Info; Properties pane with nothing selected (link) |
| Project management… | Project data › Management (pane "Projects") |
| Parts management… | Devices › Parts (document pane) |
| Data portal… | Devices › Parts (document pane); Master data › Parts |
| Structure management… | Devices › Structure (pane) |
| Property database… | Master data › Properties (document pane) |
| Cables… | Connections › Cables (pane "Cables", today's tab and window merged) |
| Potentials… | Connections › Potentials (pane) |
| Electrical calculations… | Tools › Calculations (document pane) |
| Wire numbering… | Connections › Numbering (command) and Settings › Project › Wire numbering |
| Cross references… | Settings › Project › Cross references; Connections › Cross references |
| Device numbering… | Devices › Numbering |
| Harnesses… | Connections › Harnesses (pane) |
| Field wiring… | Connections › Field wiring (document pane) |
| Terminal strips… | Devices › Terminals (pane "Terminal strips", merged with the Strips / plugs tab) |
| Plugs… | Devices › Plugs (pane) |
| Project languages… | Project data › Languages; Settings › Project › Languages |
| Fluid and process… | Tools › Fluid and process (pane); hose defaults in Settings |
| Pre-planning… | Devices › Pre-planning (pane) |
| Single-line / multi-line… | Devices › Pre-planning |
| Marking… | Connections › Marking; Settings › Project › Marking |
| PLC overview… | Devices › PLC (pane "PLC", merged with the PLC tab: cards, channels, overview) |
| PLC hardware and bus topology… | Devices › PLC (document pane) |
| Enterprise integration… | Tools › Integration (document pane) |
| Compare and baselines… | Project data › Revisions (pane) |
| Change requests… | Project data › Revisions (pane) |
| Review: comments and markup | Project data › Review (pane, today's tab) |
| Revisions… | Project data › Revisions (pane) |
| Edit in table… | Project data › Tables (pane), and from every navigator |
| Macros… | Master data › Macros (pane); Insert center › Macros |
| Configurator… | Project data › Options |
| Symbol editor… | Master data › Symbols (document pane) |
| Title blocks, plot frames and page templates… | Master data › Forms (document pane) |
| Report blocks… | Project data › Reports |
| Check project | Project data › Check (Messages pane) |

### 6.6 Reports, Plugins, View, Help menus

| Today | New place |
| --- | --- |
| Reports › 41 kinds | Reports pane (tree); Project data › Reports |
| Report blocks…, Update report blocks | Project data › Reports |
| Plugins › <area> › command | Plugins tab (groups per plugin) or the area's group; command search |
| Plugins › reports | Reports pane (plugin group) |
| Manage plugins | Plugins |
| Fit page | View › Zoom |
| Grid, grid size | View › Show; status bar segment |
| Connection points, revision marks, macro boxes, wire colours | View › Show |
| Keyboard shortcuts… | Settings › User › Shortcuts |
| Panels | View › Panes |
| Workspaces | View › Workspaces; quick access workspace chooser |
| Language | Settings › User; status bar segment |
| System / Light / Dark | Settings › User › Appearance; View › Appearance |
| Shortcuts & usage | Help › Shortcuts, Help › Tutorial (pane) |

### 6.7 Navigator tabs

| Today | New place |
| --- | --- |
| Pages (list, tree, lock, properties, move) | Pane "Pages" (unchanged content, shared header, context menu) |
| Devices | Pane "Devices" |
| Connections | Pane "Connections" |
| Cables | Pane "Cables" (merged with the window) |
| Layers | Pane "Layers" |
| Check | Pane "Messages" |
| Review | Pane "Review" |
| Unplaced | Pane "Unplaced" and Insert center › Unplaced |
| PLC | Pane "PLC" (merged with the overview window) |
| Strips / plugs | Panes "Terminal strips" and "Plugs" (merged with the windows) |
| Work area chooser above every tab | Status bar segment and Devices › Structure |

### 6.8 Panel layout and wiring workspaces

| Today | New place |
| --- | --- |
| Workspace switch (tool bar) | Quick access workspace chooser; View › Workspaces |
| Enclosures navigator, devices to place, placed | Panes "Layout spaces" and "Unplaced" (devices with mounting data) |
| Panel properties column | Properties pane (layout tabs) |
| Panel tool bar (select, rail, duct, busbar, cutout, routing path, reservation, profile part, accessory, 3D macro) | Layout space › Draw / Place |
| Views (front, door, walls, 3D, room, thermal) | Layout space › View; the view is a pane "Layout" |
| New enclosure, Pattern, Save as 3D macro windows | Windows (kept) |
| NC export, CNC, copper, hand-off (properties column) | Layout space › Production |
| Panel reports button | Layout space › Reports |
| Wiring / mounting mode switch, checklist, workbench, CSV export | Wiring context tab; panes "Checklist" and "Workbench" |
| Web app `#wiring` start | Unchanged: opens the Wiring workspace preset in tablet mode |

### 6.9 Windows that stay windows

Save as macro, Rename device, Move selection by, Multiple copy, Renumber pages, Save
page as template, Save workspace, Export parameters, Plugin command parameters, New
enclosure, Pattern, Save as 3D macro, Part selection (for a device), Properties
(window form), Settings. All others of the list in section 1.1 become panes.

## 7. Open decisions and risks

| Topic | Decision proposed | Risk |
| --- | --- | --- |
| Ribbon widget | Own widget on egui (tabs as `selectable_label`s, groups as horizontal frames with large / small buttons); no crate offers one for egui 0.37 | Drawing cost per frame: measured in package 1 with the demo project; the ribbon draws only the active tab |
| Menu bar | Removed in package 1; Alt key tips replace accelerators | Users of the old menus; the command search and the mapping table are the answer |
| Floating panes | `egui_tiles` has no floating tiles; a pane "floats" by opening it as an `egui::Window` showing the same content (pane menu › Float) | Two code paths for one pane; kept behind one function per pane |
| Several page panes | One page pane with tabs in package 2; two pages side by side (EPLAN's undocked editors) later: `Session` keeps one current page, a second pane needs its own page index and selection | Scope creep; left out of this plan on purpose |
| Document panes (parts, symbols, forms, calculations) | Panes in a tab beside the page; they are wide, so the default workspace opens them maximised in the page area | Their internal layouts assume a window of fixed size; package 3 adapts them to the available size |
| Plugin commands versus dialogs | A dialog that duplicates a command calls the command; the command keeps its snake_case name in the search, with the dialog title as its title | Parameter forms of commands stay generic (`params`) |
| Icons | Vector icon table in `openecraft-ui`; about 120 icons to draw | Drawing effort; a consistent 1.5 px stroke style keeps it tractable; the parts manager uses the same set |
| Web build | Same shell; the backstage replaces browser-specific labels (*Download project*); tablet mode for wiring | Size of the wasm binary grows little (no new crates) |
| Translations | 300 to 500 new catalog lines in four languages, then 16 more | Done per package for the four, in package 10 for the rest |
| Compatibility of stored state | Dock JSON, workspaces, shortcuts and disabled plugins are read by their old keys; new keys for ribbon customisation, favorites, recent, UI scale | A reset exists (*View › Workspaces › Reset*) |

## 8. Acceptance criteria

A package is done when, besides the gate commands and the tests of section 3.3:

1. Every function in the tables of section 6 that the package touches is reached
   from the ribbon or the backstage in at most two clicks, and by the command search.
2. At 1,366 × 768 px and 100 % scale nothing wraps: ribbon, quick access, status bar
   and pane headers fit; at 200 % UI scale the layout still works at 1,920 × 1,080.
3. The keyboard-only walkthrough of the tutorial (`docs/tutorial.md`, sections 1 to
   9) succeeds without a mouse: tabs, groups and panes reached by key tips, F6 and
   shortcuts; focus is visible at every step.
4. Switching the language in Settings changes every text of the package at once;
   the screenshots of the package are taken in the four languages and attached to
   the pull request.
5. The dark theme shows no hard-coded light colours (checked by switching the theme
   while the package's panes are open).
6. A saved workspace from before the package is restored without an error, with the
   new panes at their default place.
7. No new emoji or other character outside the bundled fonts (glyph test), no plain
   string literal in a widget (conventions test), no plugin or built-in area
   special-cased in the shell (the ribbon reads the registry).

## 9. What to start with

Package 0 and the ribbon of package 1 come first; they unblock everything else and
are the change a user sees first. A reasonable split for parallel work after that:
package 2 (dock, one session), then packages 3, 4 and 5 in three sessions (navigators,
Insert center, properties: different files), then 6 and 7 together, 8, 9 and 10 last.
Every package updates this document's section 6 when a function moves, and
`docs/eplan-gap-analysis.md` section 3.5 when a gap listed there closes.
