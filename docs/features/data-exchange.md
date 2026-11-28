# Output and data exchange

PDF, DXF / DWG, SVG and 3D exports, DXF / DWG import, the enterprise integration layer (ERP, PDM, PLM, ECAD, MCAD, procurement, wiring machines), the exchange with KiCad and FreeCAD, and Autodesk Vault.

## Output / data exchange

**PDF** of all pages or a page selection (`1-3, 5`;
also for the HTML print document; *File › Print* opens all or the current page in the
browser with its print dialog) with bookmarks per page, clickable cross
references and report rows (links to pages left out are dropped); texts beyond Latin-1 such as Polish, Czech, Turkish,
Greek or Cyrillic are written with an embedded subset of the Ubuntu font, searchable
and copyable, **DXF** (R12, layers by kind) and **DWG** (AutoCAD 2004,
the same drawing) per page or all pages side by side, SVG per page, printable HTML, production wire list (CSV with wire numbers,
ends, colour, cross-section, routed length, stripping length and termination of
both ends (ferrule per DIN 46228-4, twin ferrule, ring, fork, pin, flat plug or
receptacle terminal) with their parts, a twin ferrule named with the other wire and
listed at the first of them, cable cores; panel wires sorted by
cross-section, colour and length for wire processing machines; ferrules and terminals
also in the BOM; with a UTF-8 byte order mark, as
the report CSVs of the editor, so that spreadsheets show `mm²` and umlauts), report CSVs, PLC symbol tables,
NC data of mounting plates, the **panel layout in 3D** as **glTF 2.0** (viewers, web,
PLM; metres, one node per enclosure and item, colours), **STEP AP214** (MCAD;
faceted solids with colours, millimetres, z up), **IGES 5.3** (MCAD; B-rep solids
with planar faces, open shells for non-closed meshes, colours and names), **IFC4**
(BIM; a distribution board per enclosure with its components as triangulated
elements), **JT** (PLM viewers; tri-strip meshes per item, JT 8.1 layout) and
**Parasolid XT** (MCAD such as SolidWorks, NX, Solid Edge, Onshape; transmit text of
schema V13 in metres, every closed part a faceted solid body whose coplanar
triangles are merged into planar faces, open meshes a sheet body) –
walls, door, mounting plate, rails, ducts, busbars and devices with the 3D models of
their parts.
**DXF / DWG import** (*File › Import › Import DXF / DWG as page…*): old drawings as graphics
on a new schematic page – lines, polylines, circles, arcs, ellipses, splines, texts,
MTEXT, block references and dimensions, in their units, scaled down to the drawing
area when they are larger. DWG files of AutoCAD R12 … 2018 are read (and written)
with the pure Rust [acadrust](https://crates.io/crates/acadrust) crate, also in the
browser. EDZ files of the EPLAN Data Portal are read as parts (see [parts management](parts.md)); their 2D window macros become symbols and their 3D macros the 3D
models of the parts.

## Enterprise integration

**Enterprise integration** (*Tools › Enterprise integration…*): one integration
layer with connectors for **ERP** (item master and BOM, CSV), **PDM** (project and
page metadata, JSON), **PLM** (engineering BOM, JSON), **ECAD** (components and
netlist for PCB design, JSON), **MCAD** (device placement, CSV), **procurement**
(order list per supplier, stock and pack sizes considered, CSV) and **wiring
machines** (job list with cut length, stripping and ferrules, CSV). All connectors
work in both directions and share one neutral field vocabulary: exports use the
field codes as columns / keys, imports recognise the codes, common English and
German column names (`Material`, `Bestand`, `refdes` …) and the column mapping of a
*profile* (e.g. `MATNR` → `erp_number` for SAP). Imports are shown as a list of
changes that are reviewed and taken over: ERP numbers, prices, stock and lead
times, PLM item IDs, revisions and lifecycle states (obsolete parts in use are
flagged by the check), PDM document numbers and states, part assignments from the
ECAD system, device positions from the MCAD system, order confirmations, and the
wire processing feedback as wiring marks. Each export stores the exported values,
so values changed on both sides since are reported as conflicts and values the
other system left unchanged never overwrite local edits. A neutral exchange file
(`openecraft.exchange/1`, JSON) holds everything for systems without a connector;
the exchange log is kept in the project.

## KiCad and FreeCAD

OpenECraft exchanges data with the free tools for PCB design, [KiCad](https://www.kicad.org),
and for mechanical design, [FreeCAD](https://www.freecad.org), in their own file formats.

| What | Direction | How |
| --- | --- | --- |
| A board as a device | KiCad → OpenECraft | *File › Import › Import KiCad board as device…* reads the netlist of the board (`.net`) |
| A circuit for a board | OpenECraft → KiCad | *File › Export*, *KiCad netlist*; in KiCad's PCB editor *File › Import › Netlist* |
| Symbols | both | *File › Export*, *KiCad symbol library*; *Import symbol library…* reads `.kicad_sym` |
| Part numbers and descriptions | KiCad → OpenECraft | connector *KiCad* of the enterprise integration: a netlist or a bill of materials (CSV) |
| The panel layout in 3D | OpenECraft → FreeCAD | *File › Export*, *Panel layout in 3D for FreeCAD*: a macro that builds the layout |
| Positions of the devices | FreeCAD → OpenECraft | connector *FreeCAD* of the enterprise integration: the document saved in FreeCAD (`.FCStd`) |
| 3D models of parts and boards | KiCad, FreeCAD → OpenECraft | STEP files as the 3D model of a part |
| Drawings | both | DXF |

**A board designed in KiCad as a device.** Write the netlist in KiCad's schematic
editor (*File › Export › Netlist…*, format KiCad, or `kicad-cli sch export netlist`).
*Import KiCad board as device…* adds a schematic page with a device box for the board
and a device connection point for every pin of its connectors (`J1.1`, `J1.2` …), the
net of the board written at each pin (`+24V`, `IN1`). The board is then wired like any
other device and appears in the connection lists, cable plans and the bill of
materials. Connectors are the components with the references J, P, X, CN, CON, TB or
XS and those of KiCad's libraries *Connector* and *TerminalBlock*, without test
points (TP) and jumpers (JP); a board with many
pins goes on to further boxes and pages. The board gets a part (`PCB.<name>`) with
the title, company and revision of the title block and the list of its components.
The tool `import_kicad` of the command line and the MCP server also takes the board
file (`.kicad_pcb`), whose outline on the layer `Edge.Cuts` gives the size of the
part, the device tag and the components whose pins are the connections. The 3D model
of the board is the STEP file KiCad writes (*File › Export › STEP*,
`kicad-cli pcb export step`), assigned to the part like every model.

**The circuit as a KiCad netlist.** Every device with electrical connections is a
component (reference: the device tag, value: the type number of its part, fields
*Part number*, *Manufacturer* and *Order number*), every potential a net of
connection points. Footprints are assigned in KiCad; their pads are named like the
connection points (`A1`, `13`). The components keep their identity from export to
export, so the PCB editor updates a layout instead of starting again. A netlist or a
bill of materials (CSV with the column *Reference*) that comes back from KiCad
assigns part numbers and descriptions through the connector *KiCad*, as a list of
changes to review.

**Symbols.** The export writes the symbols placed in the project and its user symbols
(with the parameter `all` the whole library) as a KiCad symbol library: 4 mm here are
2.54 mm (100 mil) there, so the connection points lie on KiCad's grid. A `.kicad_sym`
file is read as user symbols in a library named after the file, 1.27 mm (50 mil)
there as 4 mm here: a symbol per unit
(`LM358 A`, `LM358 B`), the pin names inside, derived symbols with the graphics of
their parent, power symbols as potentials. Reading a library again replaces its
symbols.

**The panel layout in FreeCAD.** The export is a macro: run it in FreeCAD (*Macro ›
Macros…*) and it builds a document with a part container per enclosure and an object
per component, rail, duct and device, named with the device tags, as solids (objects
with very many triangles stay meshes; two settings at the top of the macro decide).
Every device carries its tag, part number, enclosure, mounting face and place as
properties of the group *OpenECraft*. Move devices in FreeCAD, save the document and
read the `.FCStd` file with the connector *FreeCAD*: the positions come back as a
list of changes, with the conflict detection of every connector. Models designed in
FreeCAD are assigned to parts as STEP files.

```bash
openecraft-cli -p plant.oec import_kicad --path relay-io.net --board relay-io.kicad_pcb
openecraft-cli -p plant.oec import_kicad --path Device.kicad_sym
openecraft-cli -p plant.oec export --format kicad_netlist --path plant.net
openecraft-cli -p plant.oec export --format kicad_symbols --path plant.kicad_sym
openecraft-cli -p plant.oec integration_export --connector freecad --path plant.FCMacro
openecraft-cli -p plant.oec integration_import --connector freecad --path plant.FCStd --apply
```

## Autodesk Vault

OpenECraft connects to Autodesk Vault Professional 2025.2 or newer through the Vault
Data API (REST, `/AutodeskDM/Services/api/vault/v2`), so projects, documents and 3D
models are exchanged with the PDM system directly instead of through files.

- In the editor: *Tools › Autodesk Vault…* – enter the server address, the knowledge
  vault, user name and password (or the access token of a Vault session or an Autodesk
  account) and *Sign in*. Browse the folders and search the file properties below the
  folder shown.
- **Import**: a selected file is opened as the project (`.oec`, `.oecz`), imported into
  the parts database (CSV, JSON, BMEcat, EPLAN XML, EDZ, enclosure packages as ZIP), assigned as 2D / 3D model of a
  part (SVG, STEP, IGES, Parasolid, JT, IFC, STL, OBJ – e.g. models of the mechanical design kept
  in the vault), inserted as a page (DXF, DWG) or as a document page (PDF, Office files,
  images), read with a connector of the enterprise integration layer, or saved as a
  local file. The file name suggests the use.
- **Export**: the project file, every export of *File › Export* (PDF, DXF / DWG, the
  panel layout in 3D as STEP, IGES, Parasolid, JT, glTF or IFC, reports, PLC tables, exports of
  plugins) and the connector exports go into the folder shown. A file of the same name
  there gets a new version (checked out, uploaded in parts, checked in with the
  comment; the checkout is undone when something fails), otherwise the file is added.
  A project opened from the vault is checked in again with *File › Save (Check in … to Vault)*.
- **Items**: *Update part links from Vault items* finds the Vault item of every project
  part (by item number or by its *Part Number* property) and shows the item numbers,
  revisions and lifecycle states as changes of the PLM connector for review, with the
  same conflict detection as file imports.
- Command line / MCP: `vault_connect`, `vault_browse`, `vault_items`, `vault_download`,
  `vault_open`, `vault_upload`, `vault_sync_items`. An MCP session signs in once with
  `vault_connect`; the command line starts a session per call and takes server, token
  and vault id from `OPENECRAFT_VAULT_URL`, `OPENECRAFT_VAULT_TOKEN` and
  `OPENECRAFT_VAULT_ID` (`vault_connect` shows the token and vault id of a sign-in).

```bash
openecraft-cli vault_connect --url https://vault.example.com --user anna --password …
export OPENECRAFT_VAULT_URL=https://vault.example.com OPENECRAFT_VAULT_TOKEN=V:… OPENECRAFT_VAULT_ID=101
openecraft-cli vault_browse --query "Plant 1"
openecraft-cli -p plant.oec vault_upload --folder 42 --format step --comment "Panel layout"
```

- Passwords are not stored and the token is kept in memory only. The browser version
  needs a Vault server (or a reverse proxy in front of it) that allows cross-origin
  requests from the editor's address.
