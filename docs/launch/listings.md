# Listings

Texts and steps for the places people find software besides the stores
(`installer/stores/README.md`): the GitHub repository itself, AlternativeTo and curated
lists. Everything here is for the maintainer to run or submit; every claim is taken from
the README and the documentation of version 0.1.2 and the current `master`.

## GitHub repository

Description (shown under the name and in search results, at most 350 characters),
website and topics (at most 20; lowercase, digits and hyphens):

```bash
gh repo edit nilres/OpenECraft \
  --description "Open source electrical CAE: device-oriented schematics, live reports and panel layout in 3D. Windows, Linux, macOS and the browser, with a command line and an MCP server for AI agents." \
  --homepage "https://openecraft.org/"

gh repo edit nilres/OpenECraft \
  --add-topic electrical-engineering,electrical-cad,ecad,cae,cad \
  --add-topic schematic,schematic-editor,wiring-diagram,control-panel,panel-layout \
  --add-topic industrial-automation,plc,iec-60617,eplan \
  --add-topic rust,egui,webassembly,mcp,mcp-server,3d
```

`eplan` is there because people search for it and OpenECraft imports EPLAN EDZ and XML
data; the README's trademark notice covers the naming. Leave it out if that is too
close to the trademark.

**Social preview** (the picture of links on social media and chats): there is no
command line for it. *Settings › General › Social preview › Edit › Upload an image*,
`website/social.jpg` (1200 × 630, 58 KB: logo, "Open source electrical CAE",
*Schematics · Reports · Panel layout in 3D* and the panel layout in 3D; GitHub asks for
at least 640 × 320 and less than 1 MB).

After the next push to `master`, GitHub shows the files of `.github/ISSUE_TEMPLATE/`
as forms under *New issue* and `CONTRIBUTING.md` in the sidebar and on the issue page.

## AlternativeTo

Add the app at <https://alternativeto.net> (account needed, *Add new app*); a moderator
approves it. Then open the page of each application below and use *Suggest
alternative* so that OpenECraft is listed there.

| Field | Text |
| --- | --- |
| Name | OpenECraft |
| Website | https://openecraft.org/ |
| Short description | Open source electrical CAE editor: device-oriented schematics, live reports and panel layout in 3D, on the desktop and in the browser. |
| Platforms | Windows, Mac, Linux, Online (web editor), Self-Hosted (parts and project servers, Docker images) |
| License | Free, Open Source (MIT or Apache-2.0) |
| Screenshots | `docs/images/schematic.png`, `panel-3d.png`, `wiring-assistant.png`, `terminal-diagram.png` |
| Tags / features | Electrical Engineering, Schematic Capture, Wiring Diagram, CAD, Panel Design, 3D, PLC, Offline, Portable (AppImage), Web-based, Multilingual, Self-Hosted, AI integration (MCP) |
| Alternative to | EPLAN Electric P8, EPLAN Pro Panel, AutoCAD Electrical, SEE Electrical, WSCAD SUITE, Zuken E3.series, PC\|SCHEMATIC Automation, ProfiCAD, QElectroTech |

Description:

> OpenECraft is an open source electrical CAE editor for control panels and machines.
> Coils and contacts form one device with live cross references and contact images;
> about 330 IEC 60617 symbols plus NFPA / JIC, GB, GOST and JIS sets, fluid power
> (ISO 1219), P&ID and installation plans. Terminal, cable and plug diagrams, the bill
> of materials and other reports update live, and a project check reports what is
> missing. The panel layout shows devices at real size in 3D with virtual wiring, duct
> fill levels, drilling data and an enclosure heat map; wiring and mounting assistants
> turn a tablet into a wiring station.
>
> Parts come from a shared data portal or are imported from BMEcat, EPLAN XML and EDZ
> files. Projects export to PDF, DXF / DWG and SVG, the panel to STEP, IGES, IFC, JT,
> glTF and Parasolid. A project server shares projects with page locks; a command line
> and an MCP server let scripts and AI agents edit projects.
>
> It runs on Windows, Linux and macOS and in the browser (editor.openecraft.org,
> nothing to install), in English, German, French, Spanish and 16 further languages.

## Curated lists

Each list wants one pull request per entry against its `README.md`, in alphabetical
order inside the section unless noted.

### awesome-rust (`rust-unofficial/awesome-rust`)

Only projects with **more than 50 stars** (or 2000 crates.io downloads) are accepted;
the repository has 31 today. Once it passes 50, section **Applications › Industrial
automation**, between `locka99/opcua` and `slowtec/tokio-modbus`:

```markdown
* [nilres/OpenECraft](https://github.com/nilres/OpenECraft) - Electrical CAE editor: device-oriented schematics, live reports and panel layout in 3D, with a command line and an MCP server; native and WebAssembly with egui
```

The list asks for a CI badge after the description when there is one; the `CI`
workflow of the repository is disabled at the moment, so leave the badge out until it
runs green on `master` again:
`[![CI](https://github.com/nilres/OpenECraft/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/nilres/OpenECraft/actions/workflows/ci.yml)`.

### awesome-mcp-servers (`punkpeye/awesome-mcp-servers`)

Section **🏭 Industrial & IoT**, between `LGDiMaggio/predictive-maintenance-mcp` and
`purinzan/gx3-cli-mcp` (Rust, local, all three systems):

```markdown
- [nilres/OpenECraft](https://github.com/nilres/OpenECraft) 🦀 🏠 🍎 🪟 🐧 - Electrical CAE for AI agents: create projects, place symbols, draw wires between connection points, assign parts, run the project check, generate reports and export PDF / SVG / DXF / DWG (`openecraft-cli mcp`).
```

Most entries also carry a score badge of glama.ai, which indexes public MCP servers on
its own; add it only once the server has a page there
(`https://glama.ai/mcp/servers/nilres/OpenECraft`).

### egui showcase

egui has no list in its README; apps are shown in the issue
[Showcase gallery: share your egui screenshots here!](https://github.com/emilk/egui/issues/996)
(open, over 240 comments) and in the *Show and tell* category of its discussions. A comment
with `docs/images/panel-3d.png` or `schematic.png` attached:

```markdown
**OpenECraft** – an open source electrical CAE editor built on egui / eframe: schematics with live cross references, reports and panel layout in 3D, wiring assistants for touch screens, 20 languages (CJK through bundled Noto subsets). One code base runs natively on Windows, Linux and macOS and in the browser.

Web editor: https://editor.openecraft.org · Code: https://github.com/nilres/OpenECraft
```

The list `vonnieda/awesome-egui` is archived and takes no entries.

### awesome-electronics (`kitspace/awesome-electronics`)

Its contributing rules forbid self-promotion ("only add projects … that you are not
personally affiliated with"), so the maintainer cannot add OpenECraft; a user may. The
list is about PCB electronics, the closest section is **Free EDA Packages** (new
entries at the bottom, title case):

```markdown
- [OpenECraft](https://openecraft.org/) - Open source electrical CAE for control panels: schematics, reports and panel layout in 3D, on the desktop or in the browser.
```
