# Demo video: an AI agent draws a motor starter (60 s)

DRAFT – storyboard for a screen recording, nothing recorded yet.

**Goal:** one prompt in Claude Code produces a checked, two-page motor starter. The
result opens in the OpenECraft editor and as a PDF. The video shows a real run, sped up
where the agent works, and says so on screen. Use no voice-over; captions carry the
story, so the video also works muted in a feed.

## Before recording

- **Software:**
  - OpenECraft from the latest release, with the *Full* installation type on Windows so
    that the MCP server comes with it.
  - Claude Code with the server registered in an empty folder, e.g. `C:\demo\pump`:
    `claude mcp add openecraft -- "%LOCALAPPDATA%\Programs\OpenECraft\openecraft-cli.exe" mcp`.
  - Check with `claude mcp list` that `openecraft` shows *connected*.
- **Rehearse.** Run the prompt below at least once before recording.
  - Keep the run whose check ends with **0 errors** and whose pages look tidy. A model
    lays out by coordinates and some runs come out crooked.
  - Note how long it took. The captions must give the real time and the real number of
    tool calls; read both from the transcript, Claude Code's `/cost` or the tool counter.
  - If a run needs a second prompt to fix the layout, show that prompt too, or record
    another run. Never cut it out.
  - Rehearsing on your own model account costs tokens: about 130 tool calls with answers
    of a few kB each.
- **Fallback.** If no live run is clean, record the replay
  (`python docs/launch/ai/mcp_demo.py`) in a terminal instead. The captions must then say
  "replayed MCP calls" and must not show a model at work.
- **Screen:**
  - 1920 × 1080, dark terminal theme, font size 18 or larger, notifications off.
  - The editor window maximised, light theme, opened on an empty start page.
  - Desktop clear of other files.
- **Audio:** quiet music or none. Captions burned in, bottom third, white on a 70 %
  black bar, 1–2 lines each.

## The prompt

Paste it in one go; it is the first example prompt of the [README](README.md):

```text
Create an OpenECraft project pump_dol.oec from the template "Empty (IEC 60204-1)", with a
cover sheet as page 1. Draw a direct-on-line starter for a 2.2 kW pump. Page 2, power
circuit: L1 L2 L3 N PE rails, main switch -Q1, motor protection switch -Q2, contactor -K1,
terminals -X1, motor -M1, a 4G1.5 motor cable -W1 of 15 m; a 24 V DC supply -F1 / -T1 with
interruption points +24V and 0V. Page 3, control circuit: NC auxiliary contact of -Q2,
stop -S1, start -S2 with a self-holding contact of -K1, coil -K1 and a "pump running" lamp
-P1; the push buttons and the lamp are in the door, wired through terminal strip -X2.
Parts: -Q1 GEN.MS-3P-40A, -Q2 GEN.MPS-4-6.3A + GEN.MPS-AUX-1NO1NC, -K1 GEN.CT-9A-24DC +
GEN.AUX-2NO2NC, -M1 GEN.MOT-2.2KW, -X1 GEN.TB-2.5 + GEN.TB-PE-2.5, -X2 GEN.TB-2.5, -F1
GEN.MCB-1P-C6, -T1 GEN.PS-24DC-5A, -S1 GEN.PB-RD-1NC, -S2 GEN.PB-GN-1NO, -P1 GEN.LMP-GN-24,
-W1 GEN.CBL-4G1.5. Add a terminal diagram, cable overview, device list and bill of
materials, number the wires, run the check and fix everything it reports, then export
pump_dol.pdf.
```

On screen, show the prompt typed (pasted) in full, then scroll it once. The part list is
honest: without a parts server the agent cannot look up part numbers.

## Shots

| Time | Picture | Caption |
| --- | --- | --- |
| 0:00–0:04 | Terminal, `claude mcp list` with `openecraft … connected` highlighted. | **An AI agent draws an electrical schematic.** Claude Code + the OpenECraft MCP server |
| 0:04–0:12 | The prompt is pasted and sent. Zoom on the first lines. | One prompt: a direct-on-line starter for a 2.2 kW pump |
| 0:12–0:30 | Claude Code at work, sped up: `list_symbols`, `place_symbol`, `connect`, `add_wire` scroll past. Freeze for 1 s on one `connect` call `{"from": "-Q2:2", "to": "-K1:1"}` and its answer. | The agent places symbols and connects pins by name (`-Q2:2 → -K1:1`). Real run, [N] tool calls in [M] minutes, shown [X]× faster |
| 0:30–0:36 | The `check` call and its answer `"errors": 0, "warnings": 0`; then `export` → `pump_dol.pdf`. | The project check: 0 errors, 0 warnings |
| 0:36–0:48 | The editor: *File › Open project* → `pump_dol.oec`. Page 2 (power circuit) fills the screen, slow zoom from the rails down to the motor and cable `-W1`. Follow the cross reference `/3.1` under the main contacts of `-K1` to its coil on page 3 (rehearse how the editor jumps; otherwise `Page Down`), where the contact image under the coil points back to `/2.2`. | Opened in the OpenECraft editor: devices, cross references, wire numbers |
| 0:48–0:55 | The PDF (any viewer): page 3 control circuit, then flick through terminal diagram and bill of materials. | Also a 7-page PDF: schematic, terminal diagram, bill of materials |
| 0:55–1:00 | End card: logo, `openecraft.org`, `github.com/nilres/OpenECraft`, "MIT / Apache-2.0". | Open source. Try it: openecraft.org · MCP setup: docs/launch/ai |

Fill in [N], [M] and [X] from the run you recorded.

### Optional 3D shot (replaces 0:48–0:55)

The demo prompt does not lay out the panel. If you want to show 3D, cut to *Panel
layout* of an example project (*File › New › Example projects*, "4 · Conveyor with PLC and control cabinet") and caption it
**"Example project, not drawn by the agent: the panel in 3D"**. Without that caption the
shot claims more than the agent did. A second prompt that places the devices on a
mounting plate (`add_enclosure`, `place_panel_device`) would make it the agent's work;
rehearse it first, since the panel checks are strict.

## After recording

- **Length:** 60 s or less for X / LinkedIn. Keep a 30 s cut (shots 1, 3, 4, 5) for
  Bluesky and Reddit.
- **Files:**
  - Export 1080p H.264, under 50 MB.
  - Add a GIF of 0:36–0:48 (editor zoom, max 10 MB) for the README.
  - Use a still of the power page as the thumbnail (or `demo/power.png`).
- **Alt text:** "Screen recording: Claude Code calls OpenECraft MCP tools to draw a
  motor starter; the editor then shows the power and control pages and a PDF with
  terminal diagram and bill of materials."
- **Keep the evidence:** the transcript of the recorded run (Claude Code `/export` or
  the session log) next to the video, so that anyone who asks can see the calls.
