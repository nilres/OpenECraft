# Posts for the AI / MCP audience

DRAFT – not posted. These point to the MCP demo ([README](README.md), the
[transcript](demo/transcript.md) and the [video](demo-video.md) once it is recorded).

## Before posting

- **Links** resolve only after `docs/launch/ai/` is on `master`:
  - demo: https://github.com/nilres/OpenECraft/blob/master/docs/launch/ai/README.md
  - transcript: https://github.com/nilres/OpenECraft/blob/master/docs/launch/ai/demo/transcript.md
- **Numbers:** the claims below are from the scripted demo (129 tool calls; check 0
  errors / 0 warnings; 7-page PDF; over 400 tools). If you post the video of a live
  model run, use that run's numbers: tool calls, minutes, how many prompts.
- **Be clear what the demo is.**
  - The demo script replays the calls an agent makes; it is not a model.
  - If you only have the replay, say "replayed MCP calls", not "Claude drew this".
  - Keep the limits in, they earn trust: the agent cannot see the drawing, and it needs
    part numbers or a parts server.
- **Your own words:**
  - Hacker News asks that comments are not AI-generated. Rewrite the HN comment in your
    own words and answer the thread yourself.
  - Most subreddits expect the same.
- **Spacing:** don't post the same text in many places on one day. AI channels can go
  the same week as the main launch, a day or two apart (see the main posting plan in
  `docs/launch/`).

## X / Bluesky

One post with the video (or `demo/power.png`) attached, the link in the post: 295
characters as written (Bluesky's limit is 300); X counts the link as 23, so 245 there:

> An MCP server for electrical schematics: 129 tool calls draw a direct-on-line motor
> starter in OpenECraft, assign parts, run the project check (0 errors) and export a
> 7-page PDF. Open source, offline, Windows/macOS/Linux.
> https://github.com/nilres/OpenECraft/blob/master/docs/launch/ai/README.md

With the video of a live run, start instead with "From one prompt, Claude Code drew a
direct-on-line motor starter in OpenECraft in [M] minutes, …" and the numbers of that
run.

Reply under it, as a thread (2/2):

> How it works: the agent calls place_symbol and connect with device tags (-K1:A1), the
> server keeps the netlist, cross references and wire numbers. Every call of the demo is
> in the transcript. It can't see the drawing, so layout is coordinates – check the PDF.

## LinkedIn

With the video uploaded natively; links in the text.

> Can an AI agent draw a control circuit you would sign off? Not yet without you – but
> it gets surprisingly far.
>
> OpenECraft, the open source electrical CAE I'm building, has an MCP server: the
> protocol Claude, Cursor, VS Code and other AI tools use to call software. In the demo,
> 129 tool calls draw a direct-on-line starter for a 2.2 kW pump: main switch, motor protection,
> contactor, terminals, motor cable, a 24 V control circuit with self-holding contact,
> parts from the catalog, terminal diagram and bill of materials. The project check ended
> with 0 errors and 0 warnings, and the result is an ordinary project you open in the
> editor and change by hand.
>
> What it can't do: see the drawing (it lays out by coordinates), pick manufacturer parts
> without a parts server, or take responsibility for the design. That stays with a
> qualified person – the agent saves the typing.
>
> Setup for Claude Code, Claude Desktop, Cursor and VS Code, the full transcript of every
> tool call, and the PDF: https://github.com/nilres/OpenECraft/blob/master/docs/launch/ai/README.md
>
> #ElectricalEngineering #MCP #ControlPanel #OpenSource

## r/ClaudeAI

Flair: the subreddit's MCP / showcase flair; check the rules on self-promotion first.

**Title** (after a live run): I gave Claude an MCP server for electrical schematics – it
drew a motor starter that passes the project check

**Title** (replay only): An MCP server for electrical schematics – the tool calls of a
motor starter, from prompt to checked PDF

> I'm building OpenECraft, an open source electrical CAE (the kind of tool used to design
> control cabinets, like EPLAN). Its command line doubles as an MCP server
> (`openecraft-cli mcp`, stdio, no account, works offline).
>
> The demo ([a live Claude Code run, M minutes] / [a scripted replay of the tool calls an
> agent makes]): a DOL motor starter – power circuit from the L1/L2/L3 rails
> through main switch, motor protection switch and contactor to terminals, cable and
> motor; a 24 V control circuit with stop/start, self-holding contact and a lamp; parts,
> terminal diagram, BOM. 129 tool calls, project check 0 errors / 0 warnings, 7-page
> PDF. Pictures and the full transcript:
> https://github.com/nilres/OpenECraft/blob/master/docs/launch/ai/README.md
>
> Things I learned building the tools for an agent:
>
> - Connecting by name (`connect -K1:2 → -X1:1`) works far better than asking the model
>   for wire coordinates. Positions still matter for the symbols, and that is where runs
>   go crooked.
> - Every tool answer is JSON with the new ids and pin positions, so the agent rarely has
>   to call `get_page` again.
> - The check is the agent's feedback loop: "core 1 is not a core of the cable" is
>   something it can fix by itself.
> - 440+ tools are too many for the context of an agent. The server lists the ~50 a
>   schematic needs and reaches the rest through `find_tools` / `call_tool`.
>
> Setup: `claude mcp add openecraft -- openecraft-cli mcp`. Happy to hear where it falls
> over.

## r/mcp

**Title:** OpenECraft: MCP server for electrical CAE (schematics, terminals, BOM, PDF) –
native binary, stdio, offline

> Sharing an MCP server from a different corner: electrical engineering. OpenECraft is an
> open source electrical CAE in Rust; `openecraft-cli mcp` exposes the same commands as
> the editor – projects, pages, symbols, wires, devices, cables, parts, the check,
> reports and PDF / SVG / DXF export.
>
> Some details that may interest MCP folks:
>
> - stdio JSON-RPC, protocol 2025-06-18 (also 2025-03-26 and 2024-11-05). The
>   `instructions` field tells the model the page geometry and how device tags work.
> - **File sandbox:** every path parameter must lie in `OPENECRAFT_MCP_ROOT` (default:
>   the working folder). Text an agent reads in a project can't make it overwrite files
>   elsewhere.
> - **Tool list:** all 446 tools would describe themselves in ~260 kB, so the server
>   lists the ~50 for schematics (~30 kB) plus `find_tools`, `describe_tool` and
>   `call_tool` for the rest; `OPENECRAFT_MCP_TOOLS` chooses others (`*` for all).
> - The session keeps the project in memory, with `undo` / `redo` per tool call.
>
> Reproducible demo: a stdlib-only Python client replays 129 calls that draw a motor
> starter; the check ends clean. Transcript, PDF and setup for Claude Code / Desktop,
> Cursor and VS Code: https://github.com/nilres/OpenECraft/blob/master/docs/launch/ai/README.md
>
> Not in the official registry yet; packaging a native binary (MCPB per OS vs. an OCI
> image) is the open question. Opinions welcome.

## Hacker News comment

For a thread about MCP or AI agents in engineering, or as a reply in the Show HN thread
of OpenECraft. **Rewrite it in your own words before posting.**

> I've been building an open source electrical CAE (OpenECraft, Rust) and gave it an
> MCP server early on, mostly to see whether agents can do real engineering drawings.
> My experience so far: the hard part isn't the protocol but the tool design. Agents are
> bad at coordinates and good at names, so wires are drawn between pins by device tag
> (`-K1:A1`), and every answer returns the ids and pin positions the next call needs.
> The project check is the feedback loop – the agent reads "core 1 is not a core of the
> cable" and fixes it.
>
> A scripted demo (129 calls, a DOL motor starter, check clean) with the transcript is
> here: https://github.com/nilres/OpenECraft/blob/master/docs/launch/ai/README.md
>
> Limits: the agent can't see the page, so layouts need a pattern to follow; without a
> parts server it can't search parts; and nobody should build a cabinet from an agent's
> schematic without an electrician checking it – it's a drafting aid.
