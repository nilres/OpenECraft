#!/usr/bin/env python3
"""An AI agent draws a schematic: the tool calls an agent makes over MCP, replayed.

Starts the OpenECraft MCP server (`openecraft-cli mcp`, JSON-RPC on stdin / stdout),
does the MCP handshake, lists the tools and then draws a direct-on-line (DOL) motor
starter for a 2.2 kW pump: main switch, motor protection switch, contactor, terminal
strip, motor cable and motor on the power page; control voltage 24 V DC with stop /
start push buttons, self-holding contact and a "running" lamp on the control page.
It assigns parts, numbers the wires, runs the project check, generates the bill of
materials and exports the pages as PDF and SVG.

Python 3.8+, standard library only. Run from anywhere:

    python docs/launch/ai/mcp_demo.py                 # target/release, target/debug or cargo run
    python docs/launch/ai/mcp_demo.py --exe /path/to/openecraft-cli
    python docs/launch/ai/mcp_demo.py --out /tmp/demo

Writes into the output folder (default `docs/launch/ai/demo/`): `pump_dol.oecz` (the
project as archive; `--keep-oec` also keeps the 1 MB project file `pump_dol.oec`),
`pump_dol.pdf`, `power.svg`, `control.svg`, `bill_of_materials.csv`, `transcript.jsonl`
(every JSON-RPC message; the answer of `tools/list` with the tool names only) and
`transcript.md` (the calls and short answers). The server runs in the output folder,
the only folder its tools may use (`OPENECRAFT_MCP_ROOT` default).
Exits with 1 if a tool call fails or the check reports errors.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", "..", ".."))
EXE = "openecraft-cli.exe" if os.name == "nt" else "openecraft-cli"


def server_command(exe):
    """The command that starts the MCP server."""
    if exe:
        return [exe, "mcp"]
    for profile in ("release", "debug"):
        path = os.path.join(ROOT, "target", profile, EXE)
        if os.path.isfile(path):
            return [path, "mcp"]
    if shutil.which("cargo") and os.path.isfile(os.path.join(ROOT, "Cargo.toml")):
        return ["cargo", "run", "--quiet", "--release", "--manifest-path", os.path.join(ROOT, "Cargo.toml"), "-p", "openecraft-cli", "--", "mcp"]
    if shutil.which("openecraft-cli"):
        return ["openecraft-cli", "mcp"]
    sys.exit("openecraft-cli not found: build it (cargo build --release -p openecraft-cli) or pass --exe")


class Client:
    """A minimal MCP client over stdio that records every message."""

    def __init__(self, command, cwd, log, md):
        self.proc = subprocess.Popen(command, cwd=cwd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, encoding="utf-8")
        self.out = cwd
        self.log = log
        self.md = md
        self.next_id = 0
        self.calls = 0

    def send(self, msg):
        self.log.write(json.dumps({"dir": "client", "msg": msg}, ensure_ascii=False) + "\n")
        self.proc.stdin.write(json.dumps(msg) + "\n")
        self.proc.stdin.flush()

    def request(self, method, params):
        self.next_id += 1
        self.send({"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params})
        line = self.proc.stdout.readline()
        if not line:
            sys.exit("the MCP server stopped")
        reply = json.loads(line)
        logged = reply
        if method == "tools/list" and "result" in reply:
            # The full list is about 270 kB (every tool with its JSON schema): only the names.
            tools = reply["result"]["tools"]
            logged = dict(reply, result={"tools": [{"name": t["name"]} for t in tools], "abridged": "names only; descriptions and inputSchema left out"})
        self.log.write(json.dumps({"dir": "server", "msg": logged}, ensure_ascii=False) + "\n")
        if "error" in reply:
            sys.exit(f"{method}: {reply['error']}")
        return reply["result"]

    def notify(self, method):
        self.send({"jsonrpc": "2.0", "method": method})

    def tool(self, tool_name, note="", **arguments):
        """Calls a tool; returns its answer as JSON (or text) and writes it to the transcript."""
        name = tool_name
        self.calls += 1
        result = self.request("tools/call", {"name": name, "arguments": arguments})
        text = result["content"][0]["text"]
        try:
            value = json.loads(text)
        except ValueError:
            value = text
        args = json.dumps(arguments, ensure_ascii=False)
        self.md.write(f"{self.calls}. `{name}` `{args}`" + (f" – {note}" if note else "") + "\n")
        self.md.write(f"   → {summary(name, value)}\n")
        if result.get("isError"):
            self.md.write("   **error**\n")
            sys.exit(f"{name} {args}: {text}")
        return value

    def close(self):
        self.proc.stdin.close()
        self.proc.wait()


def summary(name, value):
    """A short line for the readable transcript."""
    if isinstance(value, str):
        return value.splitlines()[0][:160] if value else "(empty)"
    if isinstance(value, dict):
        if name in ("place_symbol", "place_part"):
            pins = ", ".join(f"{p['pin'] or '·'} ({p['x']:g}, {p['y']:g})" for p in value.get("pins", []))
            return f"id {value.get('id')}, {value.get('tag') or value.get('label') or value.get('symbol')}: {pins}"
        if "points" in value and "ids" in value:
            return "wire " + " → ".join(f"({x:g}, {y:g})" for x, y in value["points"])
        keys = ("name", "pages", "devices", "errors", "warnings", "file", "id", "tag", "cable", "path", "bytes")
        short = {k: value[k] for k in keys if k in value}
        return json.dumps(short or value, ensure_ascii=False)[:200]
    if isinstance(value, list):
        return f"{len(value)} entries"
    return json.dumps(value)[:200]


def draw(c):
    """The tool calls of the agent."""
    c.md.write("\n## Project\n\n")
    c.tool("new_project", "an empty project with the IEC 60204-1 potentials (L1 L2 L3 N PE, +24V 0V)",
           name="Pump P1 – direct-on-line starter", template="Empty (IEC 60204-1)", path="pump_dol.oec", overwrite=True)
    c.tool("set_project_info", company="Demo Ltd.", creator="AI agent via MCP", date="2026-11-18",
           description="Pump P1, 2.2 kW\ndirect-on-line start\n24 V DC control")
    c.tool("set_wire_colors", "draw wires in the colours of their potentials", enabled=True)
    c.tool("set_wire_numbering", "wire numbers by page and column", scheme="page_column")
    c.tool("list_symbols", "look up symbol keys and connection points", query="motor protection")
    c.tool("add_page", "a cover sheet, moved in front of the schematic page of the template", name="cover", kind="cover", description="Cover sheet")
    c.tool("set_page", page="cover", before="1")
    c.tool("renumber_pages")
    c.tool("set_page", page="2", description="Power circuit pump P1")

    c.md.write("\n## Power circuit (page 2)\n\n")
    for y, key in ((24, "pot_l1"), (32, "pot_l2"), (40, "pot_l3"), (48, "pot_n"), (56, "pot_pe")):
        c.tool("place_symbol", page="2", symbol=key, x=20, y=y, rotation=270)
        c.tool("add_wire", page="2", points=[[24, y], [400, y]])
    c.tool("place_symbol", "main switch", page="2", symbol="disc3", tag="-Q1", x=60, y=72, function_text="Main switch")
    for x, y in ((60, 24), (68, 32), (76, 40)):
        c.tool("add_wire", "tap the phase rails", page="2", points=[[x, y], [x, 68]])
    c.tool("place_symbol", page="2", symbol="mps3", tag="-Q2", x=60, y=100, function_text="Motor protection")
    c.tool("place_symbol", page="2", symbol="main3", tag="-K1", x=60, y=136, function_text="Pump on")
    power_wires = []
    for a, b in ((1, 2), (3, 4), (5, 6)):
        power_wires += c.tool("connect", page="2", **{"from": f"-Q1:{b}", "to": f"-Q2:{a}"})["ids"]
        power_wires += c.tool("connect", page="2", **{"from": f"-Q2:{b}", "to": f"-K1:{a}"})["ids"]
    terminals = []
    for i, x in enumerate((60, 68, 76)):
        t = c.tool("place_symbol", page="2", symbol="term", tag="-X1", label=str(i + 1), x=x, y=164)
        terminals.append(t["id"])
    pe = c.tool("place_symbol", page="2", symbol="term_pe", tag="-X1", label="PE", x=84, y=164)["id"]
    for t, pin in zip(terminals, (2, 4, 6)):
        power_wires += c.tool("connect", "terminals of one strip share the tag: #id:pin", page="2", **{"from": f"-K1:{pin}", "to": f"#{t}:1"})["ids"]
    # Behind the main switch the nets have no potential: black power wires, 1.5 mm².
    for wire in power_wires:
        c.tool("update_wire", page="2", id=wire, color="BK", cross_section="1.5 mm²")
    c.tool("add_wire", "protective conductor to the PE terminal", page="2", points=[[84, 56], [84, 160]])
    c.tool("place_symbol", page="2", symbol="motor3", tag="-M1", x=60, y=196, function_text="Pump P1\n2.2 kW")
    motor_wires = []
    for t, pin in zip(terminals + [pe], ("U1", "V1", "W1", "PE")):
        motor_wires += c.tool("connect", page="2", **{"from": f"#{t}:2", "to": f"-M1:{pin}"})["ids"]
    c.tool("add_cable_definition", "the four wires to the motor become cores of the new cable -W1", page="2",
           points=[[52, 180], [92, 180]], cable="-W1")
    c.tool("apply_cable_part", "type and cores from the catalog; the wires take the new core names", cable="-W1", part="GEN.CBL-4G1.5")
    c.tool("set_cable", cable="-W1", length=15, description="Pump P1")

    c.md.write("\n## 24 V DC supply (page 2)\n\n")
    c.tool("place_symbol", page="2", symbol="cb1", tag="-F1", x=300, y=64, function_text="Control voltage")
    c.tool("add_wire", page="2", points=[[300, 24], [300, 60]])
    c.tool("place_symbol", page="2", symbol="psu", tag="-T1", x=300, y=80, function_text="24 V DC")
    for wire in c.tool("connect", page="2", **{"from": "-F1:2", "to": "-T1:L"})["ids"]:
        c.tool("update_wire", page="2", id=wire, color="BK", cross_section="1.5 mm²")
    c.tool("add_wire", page="2", points=[[308, 48], [308, 76]])
    c.tool("add_wire", page="2", points=[[316, 56], [316, 76]])
    c.tool("place_symbol", "interruption point: continues on page 3", page="2", symbol="intr", label="+24V", x=300, y=112)
    c.tool("add_wire", page="2", points=[[300, 96], [300, 112]])
    c.tool("place_symbol", page="2", symbol="intr", label="0V", x=324, y=112)
    c.tool("add_wire", page="2", points=[[308, 96], [308, 104], [324, 104], [324, 112]])

    c.md.write("\n## Control circuit (page 3)\n\n")
    c.tool("add_page", name="3", kind="schematic", description="Control circuit pump P1")
    c.tool("place_symbol", page="3", symbol="intr", label="+24V", x=20, y=24, rotation=90)
    c.tool("add_wire", page="3", points=[[20, 24], [400, 24]])
    c.tool("place_symbol", page="3", symbol="intr", label="0V", x=20, y=236, rotation=90)
    c.tool("add_wire", page="3", points=[[20, 236], [400, 236]])
    c.tool("place_symbol", "auxiliary contact of the motor protection switch", page="3", symbol="ol_nc", tag="-Q2", pins="21,22", x=40, y=40)
    c.tool("add_wire", page="3", points=[[40, 24], [40, 36]])
    # The push buttons and the lamp sit in the door: they are wired through terminal strip -X2.
    x2 = {}
    for n, x, y in ((1, 40, 56), (2, 40, 96), (3, 40, 120), (4, 40, 160), (5, 120, 168), (6, 120, 216)):
        x2[n] = c.tool("place_symbol", page="3", symbol="term", tag="-X2", label=str(n), x=x, y=y)["id"]
    c.tool("connect", page="3", **{"from": "-Q2:22", "to": f"#{x2[1]}:1"})
    c.tool("place_symbol", page="3", symbol="pb_nc", tag="-S1", x=40, y=76, function_text="Stop")
    c.tool("connect", page="3", **{"from": f"#{x2[1]}:2", "to": "-S1:11"})
    c.tool("connect", page="3", **{"from": "-S1:12", "to": f"#{x2[2]}:1"})
    c.tool("connect", page="3", **{"from": f"#{x2[2]}:2", "to": f"#{x2[3]}:1"})
    c.tool("place_symbol", page="3", symbol="pb_no", tag="-S2", x=40, y=140, function_text="Start")
    c.tool("connect", page="3", **{"from": f"#{x2[3]}:2", "to": "-S2:13"})
    c.tool("connect", page="3", **{"from": "-S2:14", "to": f"#{x2[4]}:1"})
    c.tool("place_symbol", page="3", symbol="coil", tag="-K1", x=40, y=200, function_text="Pump on")
    c.tool("connect", page="3", **{"from": f"#{x2[4]}:2", "to": "-K1:A1"})
    c.tool("add_wire", page="3", points=[[40, 204], [40, 236]])
    c.tool("place_symbol", "self-holding contact in parallel to the start button, both ends in the cabinet",
           page="3", symbol="no", tag="-K1", pins="13,14", x=64, y=140)
    c.tool("add_wire", page="3", points=[[40, 108], [64, 108], [64, 136]])
    c.tool("add_wire", page="3", points=[[64, 144], [64, 176], [40, 176]])
    c.tool("place_symbol", "lamp contact", page="3", symbol="no", tag="-K1", pins="53,54", x=120, y=128)
    c.tool("add_wire", page="3", points=[[120, 24], [120, 124]])
    c.tool("connect", page="3", **{"from": "-K1:54", "to": f"#{x2[5]}:1"})
    c.tool("place_symbol", page="3", symbol="lamp", tag="-P1", x=120, y=196, function_text="Pump running")
    c.tool("connect", page="3", **{"from": f"#{x2[5]}:2", "to": "-P1:X1"})
    c.tool("connect", page="3", **{"from": "-P1:X2", "to": f"#{x2[6]}:1"})
    c.tool("add_wire", page="3", points=[[120, 220], [120, 236]])

    c.md.write("\n## Parts, report pages, wire numbers\n\n")
    parts = [
        ("-Q1", "GEN.MS-3P-40A"), ("-Q2", "GEN.MPS-4-6.3A"), ("-Q2", "GEN.MPS-AUX-1NO1NC"),
        ("-K1", "GEN.CT-9A-24DC"), ("-K1", "GEN.AUX-2NO2NC"), ("-M1", "GEN.MOT-2.2KW"),
        ("-X1", "GEN.TB-2.5"), ("-X1", "GEN.TB-PE-2.5"), ("-X2", "GEN.TB-2.5"),
        ("-F1", "GEN.MCB-1P-C6"), ("-T1", "GEN.PS-24DC-5A"),
        ("-S1", "GEN.PB-RD-1NC"), ("-S2", "GEN.PB-GN-1NO"), ("-P1", "GEN.LMP-GN-24"),
    ]
    for tag, number in parts:
        c.tool("assign_part", tag=tag, number=number)
    for name, kind, text in (("4", "terminal_diagram", "Terminal diagram"), ("5", "cable_overview", "Cable overview"),
                             ("6", "device_list", "Device list"), ("7", "bill_of_materials", "Bill of materials")):
        c.tool("add_page", name=name, kind=kind, description=text)
    c.tool("renumber_wires")

    c.md.write("\n## Check, reports, export\n\n")
    result = c.tool("check", "errors, warnings and hints of the project")
    bom = c.tool("report", kind="bill_of_materials")
    csv = c.tool("report", "the same as CSV, which the agent saves for the user", kind="bill_of_materials", format="csv")
    with open(os.path.join(c.out, "bill_of_materials.csv"), "w", encoding="utf-8", newline="") as f:
        f.write(csv)
    c.tool("report", "terminal diagram as a table", kind="terminal_diagram")
    c.tool("export", format="pdf", path="pump_dol.pdf")
    c.tool("export", format="svg", page="2", path="power.svg")
    c.tool("export", format="svg", page="3", path="control.svg")
    c.tool("save_project")
    c.tool("convert_project", "a compressed archive (55 kB) to attach or share", path="pump_dol.oec", format="oecz")
    return result, bom


BROWSERS = [
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "google-chrome", "chromium", "chromium-browser", "microsoft-edge",
]


def screenshots(out, names, width=1600):
    """PNG pictures of the SVG pages, taken by a headless Chrome or Edge (if there is one)."""
    browser = next((b for b in BROWSERS if os.path.isfile(b) or shutil.which(b)), None)
    if not browser:
        print("no Chrome or Edge found: no PNG pictures")
        return
    for name in names:
        svg = os.path.join(out, name + ".svg")
        html = os.path.join(out, name + ".shot.html")
        with open(html, "w", encoding="utf-8") as f:
            f.write(f"<html><body style='margin:0;background:#fff'><img src='{name}.svg' "
                    "style='width:100vw;height:100vh;object-fit:contain;display:block'></body></html>")
        profile = tempfile.mkdtemp(prefix="oec-shot-")
        try:
            subprocess.run([browser, "--headless=new", "--disable-gpu", "--hide-scrollbars", f"--user-data-dir={profile}",
                            "--allow-file-access-from-files", f"--window-size={width},{width * 297 // 420}",
                            f"--screenshot={os.path.join(out, name + '.png')}", "file:///" + html.replace(os.sep, "/")],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=60)
        finally:
            os.remove(html)
            shutil.rmtree(profile, ignore_errors=True)
        print("picture", os.path.join(out, name + ".png"), "of", svg)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--exe", help="openecraft-cli executable (default: target/release, target/debug, cargo run, PATH)")
    parser.add_argument("--out", default=os.path.join(HERE, "demo"), help="output folder (default docs/launch/ai/demo)")
    parser.add_argument("--keep-oec", action="store_true", help="keep the uncompressed project file pump_dol.oec")
    parser.add_argument("--png", action="store_true", help="also take PNG pictures of the SVG pages with a headless Chrome or Edge")
    args = parser.parse_args()
    out = os.path.abspath(args.out)
    os.makedirs(out, exist_ok=True)
    command = server_command(args.exe)
    with open(os.path.join(out, "transcript.jsonl"), "w", encoding="utf-8", newline="\n") as log, \
            open(os.path.join(out, "transcript.md"), "w", encoding="utf-8", newline="\n") as md:
        md.write("# MCP transcript: an agent draws a DOL motor starter\n\n")
        md.write("Written by `docs/launch/ai/mcp_demo.py`; every JSON-RPC message is in `transcript.jsonl`.\n\n")
        # The server may only touch files in its working folder (OPENECRAFT_MCP_ROOT).
        c = Client(command, out, log, md)
        init = c.request("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                        "clientInfo": {"name": "openecraft-mcp-demo", "version": "1.0"}})
        c.notify("notifications/initialized")
        info = init["serverInfo"]
        md.write(f"## Handshake\n\n`initialize` → {info['title']} {info['version']}, protocol {init['protocolVersion']}\n\n")
        md.write("> " + init.get("instructions", "").replace("\n", " ") + "\n\n")
        tools = c.request("tools/list", {})["tools"]
        md.write(f"`tools/list` → {len(tools)} tools, e.g. " + ", ".join(f"`{t['name']}`" for t in tools[:12]) + " …\n")
        check, bom = draw(c)
        c.close()
        rows = bom if isinstance(bom, list) else bom.get("rows", [])
        md.write(f"\n## Result\n\n{c.calls} tool calls. Check: {check.get('errors')} errors, {check.get('warnings')} warnings. "
                 f"Bill of materials: {len(rows)} rows. Files: `pump_dol.oecz`, `pump_dol.pdf`, `power.svg`, "
                 "`control.svg`, `bill_of_materials.csv`.\n")
    if not args.keep_oec:
        os.remove(os.path.join(out, "pump_dol.oec"))
    print(f"{c.calls} tool calls, output in {out}")
    errors = check.get("errors", 0) if isinstance(check, dict) else None
    print("check:", json.dumps(check, ensure_ascii=False)[:400])
    print(f"bill of materials: {len(rows)} rows")
    if args.png:
        screenshots(out, ["power", "control"])
    if errors:
        sys.exit(1)


if __name__ == "__main__":
    main()
