# r/PLC and r/ElectricalEngineering

DRAFT – not posted. Post a few days after the developer channels (posting-plan.md),
when the first round of bugs from HN / r/rust is fixed.

## Self-promotion norms – check before posting

I could not open Reddit from here, so the rules are **not verified**. Read the sidebar
and the pinned posts of each subreddit on the day and adjust:

- Both subreddits are communities of practitioners; posts that read like an ad get
  removed or downvoted. Common Reddit norms: disclose that it is your project, don't post
  the same text to many subreddits on the same day, be an active member of the
  community (comments, not only your own links), and answer in the thread.
- r/PLC: look for a rule on self-promotion / vendor posts and whether software
  announcements go into a weekly or megathread. If unsure, message the moderators first
  with two sentences and the link – an approved post does far better than a removed one.
- r/ElectricalEngineering: check whether project / software posts are allowed outside
  a designated thread, and whether it requires a flair (e.g. "Project Showcase" or
  "Design"). Many of its readers are students and electronics people; panel design is
  a niche there, so lead with what it is.
- Being free and open source helps, but is not an exemption from the rules.
- No AI-written comments in the thread: write the answers yourself.

## r/PLC

**Title**

> I'm building a free, open source EPLAN-style ECAD that runs in the browser – looking for feedback from panel designers

**Body**

Hi all, [name] here, [one line: your background – controls engineer / panel builder /
developer].

I've been working on OpenECraft, a free and open source electrical CAE for control
panels. It runs in the browser with nothing to install
(https://editor.openecraft.org), and there are desktop versions for Windows, macOS and
Linux. Full disclosure: it's my project, it's a month old, and most of the code was
written with AI coding tools [adjust: under my direction / with my review]. That's why I
want people who do this for a living to try to break it.

**Five minutes in the browser:**

1. Open https://editor.openecraft.org → *File › New › Example projects* →
   *3 · Star-delta motor starter*: power and control circuit on separate pages, three
   interlocked contactors, timer, a 7-core motor cable.
2. Under each contactor coil the contact image lists its contacts with the page and
   column they are on; the contacts point back to the coil. *Project data › Reports* → *Bill of materials*, *Terminal
   diagram*, *Cable overview*. Press F4 for the project check.
3. *File › New › Open demo project* (a conveyor with PLC I/O): *Devices › PLC overview*
   shows the I/O per card. The tag table exports for TIA Portal (Excel *PLC Tags*),
   Studio 5000 (tag import), STEP 7 classic (SDF), TwinCAT 3 (GVL), Schneider Control
   Expert, Mitsubishi GX Works and CODESYS.
4. Switch the workspace (chooser at the top) to *Panel layout* → *Layout space ›
   3D view*: the cabinet with rails, ducts and devices at real size, every wire routed
   through the ducts, *Duct fill* colours the ducts by fill level.
5. Workspace *Wiring*: the wiring assistant – a checklist of every wire with colour,
   cross-section, length, stripping length and ferrule, the current wire highlighted in
   3D. Open `https://editor.openecraft.org/#wiring` on a tablet and it starts there in
   tablet mode.

**What else is in it:** multi-level terminals with accessories, plugs with pin-outs,
structure identifiers (`=A1+CAB1-K1`), macros, revisions with redlining, PDF with
clickable cross references, DXF / DWG export and import, wire lists for wire processing
machines, EDZ import from the EPLAN Data Portal for parts, a shared parts server, IEC
60617 symbols (~330) plus NFPA / JIC, a command line and 20 UI languages.

**Honest limits:**

- No import of EPLAN projects (.elk / .zw1). Parts come in via EDZ, EPLAN XML,
  BMEcat or CSV.
- Young: expect bugs, especially in the 3D routing, rooms / buildings and STEP import.
  The file format is versioned and old files are converted, but keep your PDFs.
- The bundled manufacturer catalogs are compiled from public data sheets – check part
  numbers before ordering. The calculations (conductor sizing, short circuit, voltage
  drop) are aids, not a certified tool.
- The browser version downloads 8–12 MB the first time (compressed WebAssembly).

**What I'd like to know:**

- What would stop you from drawing a real panel in it?
- Which report or export does your shop need that is missing or wrong?
- Does the wiring assistant match how wiring is actually done on the bench?

Bugs: https://github.com/nilres/OpenECraft/issues (or *Report a bug* in the editor,
which fills in the GitHub form; nothing is sent by itself). Code:
https://github.com/nilres/OpenECraft (MIT / Apache-2.0).

## r/ElectricalEngineering

**Title**

> Open source tool for control panel schematics and 3D panel layout, runs in the browser – feedback wanted

**Body**

Hi, [name] here. I'm working on OpenECraft, a free, open source electrical CAE: the
type of software (EPLAN, AutoCAD Electrical, …) used to design the control cabinets of
machines and buildings. It's my project and a month old; most of the code was written
with AI coding tools [adjust], so I'm looking for engineers to check it against
practice.

Unlike a drawing program it is device-oriented: a contactor is one device whose coil
and contacts sit on different pages with live cross references, wires carry numbers,
colours and cross-sections, and the bill of materials, terminal and cable diagrams and
PLC tag tables are generated from the schematic. The panel layout places the devices at
real size in 3D and routes the wires through the ducts.

Try it in the browser, no sign-up: https://editor.openecraft.org – *File › New ›
Example projects* has 16 projects from a lamp circuit to a building with PV, heat
pump and main distribution; *3 · Star-delta motor starter* is a good first one.

There are also calculation tools (conductor sizing, short circuit and selectivity, load
calculation, voltage drop of final circuits, ISO 13849-1 safety functions). They are
aids and need checking, like everything else in a month-old project. There's no import
of EPLAN projects.

I'd value feedback on what is wrong or missing from an engineering point of view – in
the comments or as issues: https://github.com/nilres/OpenECraft/issues
