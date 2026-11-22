#!/usr/bin/env python3
"""Builds what the MCP registries need of the MCP server (openecraft-cli mcp):

  python installer/mcp/build.py bundle [--binary <openecraft-cli>] [--notices <file>]
  python installer/mcp/build.py server-json <bundle.mcpb>... [--repository <owner/name>] [--output <file>]

`bundle` writes target/installer/OpenECraft-MCP-<version>-<platform>.mcpb, an MCP bundle
(a ZIP with manifest.json, the program and the licenses) for the platform of the build
machine, which Claude Desktop and other clients install with a double click. Without
--binary it builds openecraft-cli first; the tools the manifest lists are asked from
the program itself, which also shows that it starts. --notices is the
THIRD-PARTY-LICENSES.html of an installer script run before; without it cargo-about
writes one (cargo install cargo-about@0.9.3 --locked --features cli).

`server-json` writes target/installer/mcp-server.json, the entry of the official MCP
Registry (registry.modelcontextprotocol.io): installer/mcp/server.json with the version
and the bundles as files of the GitHub release v<version> with their SHA-256.
mcp-publisher publishes it once the release is public (docs/releasing.md).

Needs Python 3.8 or newer and nothing else.
"""

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = ROOT / "target" / "installer"
# The platform of the build machine as the manifest of a bundle names it.
PLATFORM = {"win32": "win32", "darwin": "darwin"}.get(sys.platform, "linux")


def die(message):
    sys.exit(f"error: {message}")


def version():
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    found = re.search(r'^\[workspace\.package\][^\[]*?^version\s*=\s*"([^"]+)"', cargo, re.M | re.S)
    if not found:
        die("no version in [workspace.package] of Cargo.toml")
    return found.group(1)


def machine():
    name = platform.machine().lower()
    return {"amd64": "x86_64", "arm64": "aarch64"}.get(name, name)


def platform_name(binary):
    """The platform in the file name of the bundle: clients tell the bundles apart by it."""
    if PLATFORM == "win32":
        return f"windows-{machine()}"
    if PLATFORM == "darwin":
        archs = subprocess.run(["lipo", "-archs", str(binary)], capture_output=True, text=True, check=True).stdout.split()
        return "macos-universal" if len(archs) > 1 else f"macos-{machine()}"
    return f"linux-{machine()}"


def ask_tools(binary, expected_version):
    """The tools the server offers by default, asked over the protocol."""
    requests = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize",
         "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "build", "version": "0"}}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
    ]
    env = dict(os.environ, OPENECRAFT_MCP_TOOLS="core")
    run = subprocess.run([str(binary), "mcp"], input="".join(json.dumps(r) + "\n" for r in requests),
                         capture_output=True, text=True, encoding="utf-8", timeout=120, env=env)
    answers = {}
    for line in run.stdout.splitlines():
        if line.strip():
            answer = json.loads(line)
            answers[answer.get("id")] = answer
    try:
        served = answers[1]["result"]["serverInfo"]["version"]
        tools = answers[2]["result"]["tools"]
    except KeyError:
        die(f"{binary} mcp did not answer initialize and tools/list:\n{run.stdout}\n{run.stderr}")
    if served != expected_version:
        die(f"{binary} is version {served}, Cargo.toml has {expected_version}")
    return [{"name": t["name"], "description": t["description"]} for t in tools]


def notices(given):
    if given:
        return Path(given)
    built = [OUT / "THIRD-PARTY-LICENSES.html", OUT / "windows" / "THIRD-PARTY-LICENSES.html", OUT / "macos" / "THIRD-PARTY-LICENSES.html"]
    newest = max((p for p in built if p.is_file()), key=lambda p: p.stat().st_mtime, default=None)
    if newest:
        return newest
    if not shutil.which("cargo-about"):
        die("cargo-about not found (cargo install cargo-about@0.9.3 --locked --features cli)")
    target = OUT / "mcp" / "THIRD-PARTY-LICENSES.html"
    target.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["cargo", "about", "generate", "--fail", "--manifest-path", str(ROOT / "Cargo.toml"),
                    "-c", str(ROOT / "installer" / "about.toml"), "-o", str(target), str(ROOT / "installer" / "about.hbs")], check=True)
    return target


def add(archive, name, data, executable=False):
    """A file of the bundle with a fixed date and Unix rights: the same bytes on every
    machine, and the program stays executable where the bundle is unpacked."""
    info = zipfile.ZipInfo(name, date_time=(2020, 1, 1, 0, 0, 0))
    info.create_system = 3
    info.external_attr = (0o100755 if executable else 0o100644) << 16
    info.compress_type = zipfile.ZIP_DEFLATED
    archive.writestr(info, data, compresslevel=9)


def bundle(args):
    ver = version()
    exe = ".exe" if PLATFORM == "win32" else ""
    if args.binary:
        binary = Path(args.binary)
    else:
        subprocess.run(["cargo", "build", "--release", "--locked", "--manifest-path", str(ROOT / "Cargo.toml"), "-p", "openecraft-cli"], check=True)
        binary = ROOT / "target" / "release" / f"openecraft-cli{exe}"
    if not binary.is_file():
        die(f"{binary} not found")

    program = binary.read_bytes()
    # Signed programs (Windows, macOS) are taken as they are.
    if PLATFORM == "linux" and shutil.which("strip"):
        stripped = OUT / "mcp" / "openecraft-cli"
        stripped.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(binary, stripped)
        subprocess.run(["strip", str(stripped)], check=True)
        program = stripped.read_bytes()

    manifest = json.loads((HERE / "manifest.json").read_text(encoding="utf-8"))
    manifest["version"] = ver
    server = manifest["server"]
    server["entry_point"] += exe
    server["mcp_config"]["command"] += exe
    manifest["compatibility"]["platforms"] = [PLATFORM]
    manifest["tools"] = ask_tools(binary, ver)

    OUT.mkdir(parents=True, exist_ok=True)
    target = OUT / f"OpenECraft-MCP-{ver}-{platform_name(binary)}.mcpb"
    with zipfile.ZipFile(target, "w") as archive:
        add(archive, "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
        add(archive, "icon.png", (ROOT / "assets" / "icon.png").read_bytes())
        add(archive, server["entry_point"], program, executable=True)
        for name in ("LICENSE-MIT", "LICENSE-APACHE"):
            add(archive, name, (ROOT / name).read_bytes())
        add(archive, "OFL-NotoSans.txt", (ROOT / "crates" / "openecraft-fonts" / "fonts" / "OFL.txt").read_bytes())
        add(archive, "THIRD-PARTY-LICENSES.html", notices(args.notices).read_bytes())
    print(f"{target} ({target.stat().st_size} bytes, {len(manifest['tools'])} tools)")


def server_json(args):
    ver = version()
    entry = json.loads((HERE / "server.json").read_text(encoding="utf-8"))
    entry["version"] = ver
    for path in sorted(Path(b) for b in args.bundles):
        if not re.fullmatch(rf"OpenECraft-MCP-{re.escape(ver)}-[a-z0-9_]+-[a-z0-9_]+\.mcpb", path.name):
            die(f"{path.name} is not a bundle of version {ver}")
        entry["packages"].append({
            "registryType": "mcpb",
            "identifier": f"https://github.com/{args.repository}/releases/download/v{ver}/{path.name}",
            "fileSha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "transport": {"type": "stdio"},
        })
    if not entry["packages"]:
        die("no bundles given")
    target = Path(args.output) if args.output else OUT / "mcp-server.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(entry, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"{target} ({len(entry['packages'])} bundles)")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    b = commands.add_parser("bundle")
    b.add_argument("--binary")
    b.add_argument("--notices")
    b.set_defaults(run=bundle)
    s = commands.add_parser("server-json")
    s.add_argument("bundles", nargs="+")
    s.add_argument("--repository", default=os.environ.get("GITHUB_REPOSITORY", "nilres/OpenECraft"))
    s.add_argument("--output")
    s.set_defaults(run=server_json)
    args = parser.parse_args()
    args.run(args)


if __name__ == "__main__":
    main()
