# Launch video: 60–90 s screen recording

DRAFT storyboard. One take per scene, cut together; no voice-over needed (captions
carry it), so the same file works muted on LinkedIn, Reddit and HN.

## Story

Star-delta starter → reports and bill of materials → panel in 3D → wiring assistant.

## Projects to start from (saves the drawing)

- Scenes 1–3: example **`star_delta_starter`** – *File › New › Example projects* →
  *3 · Star-delta motor starter* (built in code in
  `crates/openecraft-core/src/sample/star_delta.rs`). Pages: 1 Cover sheet, 2 Power
  circuit fan motor, 3 Control circuit fan motor, 4 Terminal diagram -X1, 5 Cable
  overview, 6 Cable diagram -W1, 7 Device list, 8 Bill of materials. It has **no panel
  layout**.
- Scenes 4–5: the **demo project** (`conveyor`, level 4) – *File › New › Open demo
  project*: PLC I/O, an enclosure with rails, ducts and door devices, routed wires. The
  wiring assistant's tests run on this project, so it has a full checklist.
- Alternative for a richer 3D shot: **`cabinet_lineup`** (*8 · Control cabinets in a
  line-up*, two bayed enclosures, busbars, 3D routing) – the README screenshot
  `docs/images/panel-3d.png` shows it. Its wiring is larger; use it for scene 4 only.
- Rehearse each scene twice; prepare the schematic edit of scene 2 so it takes one
  try.

## Recording settings

- Desktop build (`cargo run --release -p openecraft`) for smooth 3D; the web editor
  works too, but its first load shows the download progress. Show the browser at the
  very end (scene 6) so viewers see it runs there.
- Window 1920 × 1080, light theme (*View › Appearance › Theme*), *View › Appearance ›
  UI scale* 125 % so ribbon texts stay readable after downscaling; English UI. Close
  floating panes, *View › Panes › Reset pane layout* before each take.
- 60 fps capture (OBS or the OS recorder), cursor highlighted, clicks visualised; no
  notifications, clean desktop, title bar without personal paths (save nothing under
  your home directory during the take).
- Master: MP4, H.264, 1920 × 1080, 30 fps after the cut, ~8 Mbit/s; burned-in captions
  (sans serif, bottom third, max. two lines, 2.5 s minimum on screen). Also export a
  1080 × 1080 crop for LinkedIn / social if the ribbon stays readable; otherwise keep
  16:9.
- GIF for the README / social previews: 15–25 s cut (scenes 4 and 5 only, or 2 + 4),
  960 px wide, 12 fps, under 10 MB:

  ```bash
  ffmpeg -i master.mp4 -ss 00:00:38 -t 20 -vf "fps=12,scale=960:-1:flags=lanczos,palettegen=stats_mode=diff" palette.png
  ffmpeg -i master.mp4 -ss 00:00:38 -t 20 -i palette.png -lavfi "fps=12,scale=960:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=3" openecraft.gif
  ```

  On GitHub an MP4 dragged into an issue / README edit plays inline too and is much
  smaller than a GIF; keep the GIF for places that need one.

## Shot list

| # | Time | On screen (actual command names) | Caption |
| --- | --- | --- | --- |
| 1 | 0:00–0:08 | Start page. *File › New › Example projects* → *3 · Star-delta motor starter*. Page 2 *Power circuit fan motor* opens: main switch -Q0, motor protection -Q1, contactors -K1 / -K2 / -K3, motor -M1. | OpenECraft – open source electrical CAE. Desktop and browser. |
| 2 | 0:08–0:25 | *Page Down* to page 3 *Control circuit fan motor*. Zoom to the coil of -K1: the contact image under it lists the contacts with their pages. Edit: *Insert › Insert center* (Insert key), search `lamp`, place a lamp beside -P2, *Wire* (W) from the +24V line through it to the 0V line. Device tag `-P3` appears; *Edit › Properties* → function text "Running". | Device-oriented: coil and contacts are one device, cross-referenced across pages. |
| 3 | 0:25–0:40 | *Project data › Reports*: switch between *Bill of materials*, *Terminal diagram* and *Cable overview*. Then F4: the *Messages* pane with the project check. Click page 8 *Bill of materials* in the *Pages* navigator to show the report as a page. | Reports and bill of materials are generated from the schematic – live. |
| 4 | 0:40–0:58 | *File › New › Open demo project*. Workspace chooser in the quick access row → *Panel layout*. *Layout space › 3D view*, orbit with the mouse, *Layout space › Show › Wires*, then *Duct fill*. Optionally *Open cable ducts*. | The panel in 3D: devices at real size, every wire routed through the ducts. |
| 5 | 0:58–1:15 | Workspace *Wiring*. The *Checklist* pane, the *Workbench* with the current wire highlighted in 3D, its colour, cross-section, length, stripping length and ferrule. Click *✔ Wired*, then *▶* (next wire) twice; the progress counts up. Toggle *Wiring › Tablet mode*. | Wiring assistant: a tablet becomes a wiring station. |
| 6 | 1:15–1:25 | Browser window at `https://editor.openecraft.org` with the same demo project, then the GitHub page. | Try it in the browser: editor.openecraft.org · github.com/nilres/OpenECraft · MIT / Apache-2.0 |

Optional 5 s insert after scene 3 if time allows: *View › Language* → *Deutsch*, the
ribbon and the open report switch language at once; back to English.

## Check before recording

- The command names above were taken from `crates/openecraft-shell/src/commands.rs`,
  `ribbon.rs` and `openecraft/src/wiring_ui.rs` at ab8ee925. Click through once in the
  build you record: names change quickly in this project.
- In scene 2, make sure the placed lamp does not leave the project check with an error
  (connect both ends); if a warning shows, either fix it on screen or skip F4 in
  scene 3.
- The lamp has no part, so it will not show in the bill of materials. To show the BOM
  update, assign a part with *Devices › Parts › Part selection…* in scene 2 (rehearse,
  it costs ~5 s) – otherwise don't claim it in the caption.
- No personal data in the project properties of the title block (the examples are
  neutral; check the cover sheet).
