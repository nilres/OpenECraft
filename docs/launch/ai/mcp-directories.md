# Listing the MCP server in directories

DRAFT – nothing has been submitted. Formats and rules were checked on 2026-11-18
against the repositories and pages linked below; check them again before you submit.

The MCP server is a native program, `openecraft-cli mcp`. Directories built for npm and
PyPI packages have no slot for "download the release, run the binary". They take it in
one of three ways:

- **metadata only:** a link to setup instructions
- **an MCPB bundle:** one per operating system, attached to the GitHub release
- **an OCI image:** `ghcr.io/nilres/openecraft-mcp`

| Where | What it needs from us | Status today | Effort |
| --- | --- | --- | --- |
| [Official MCP Registry](https://registry.modelcontextprotocol.io) | `server.json`, published with `mcp-publisher` | In "preview"; feeds VS Code, GitHub, PulseMCP and others | Metadata only: 15 min. With packages: a release step |
| [Glama](https://glama.ai/mcp/servers) | The repo is added there; a Dockerfile in its build settings; `glama.json` to claim it | Open | 30 min plus a build that must pass |
| [punkpeye/awesome-mcp-servers](https://github.com/punkpeye/awesome-mcp-servers) | One README line in a PR; in practice a Glama score badge | Open, very active | 15 min after Glama |
| [mcp.so](https://mcp.so/submit) | Web form: repository URL, name | Open (free review slow; $39 fast track) | 5 min |
| [mcpservers.org](https://mcpservers.org/submit) | Web form | Open (free review up to 2 weeks; $39 in 24 h) | 5 min |
| [PulseMCP](https://www.pulsemcp.com/submit) | Nothing: it imports from the official registry | Submissions paused | 0 |
| [Docker MCP Catalog](https://github.com/docker/mcp-registry) | PR with `servers/openecraft/server.yaml`; an image | Open | 1 h; questionable fit, see below |
| Smithery | – | Its CLI, API and accounts stopped on 2026-10-31 (smithery.ai/future) | Skip |
| Claude Desktop directory | Only as part of a Claude plugin; the directory no longer takes MCPB submissions | – | Skip for now; MCPB still installs by double-click |
| [cursor.directory](https://cursor.directory) | Not verified: the site refused automated requests | – | Check by hand |

Drafts in this folder:

- [`registry/server.json`](registry/server.json): the official registry entry for now,
  metadata only
- [`registry/server.packages.json`](registry/server.packages.json): the entry once there
  are MCPB bundles and an image
- [`mcpb/manifest.json`](mcpb/manifest.json): the manifest of the MCPB bundles
- [`registry/Dockerfile.mcp`](registry/Dockerfile.mcp): the image, for Glama, the
  registry and Docker
- [`registry/glama.json`](registry/glama.json): goes into the repository root

## Official MCP Registry

**Name:** `io.github.nilres/openecraft`. A GitHub login as `nilres` may publish below
`io.github.nilres/`. The alternative `org.openecraft/openecraft` needs proof of the
domain:

- a DNS TXT record on `openecraft.org`: `v=MCPv1; k=ed25519; p=<public key>`, or
- the same line at `https://openecraft.org/.well-known/mcp-registry-auth`,
- then `mcp-publisher login dns --domain=openecraft.org --private-key=<hex>`.

The GitHub name is enough. A name cannot be changed later, so choose once.

**Schema:** `https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json`.

- Required: `name`, `description` (at most 100 characters) and `version`.
- Each package needs `registryType`, `identifier` and `transport`.
- `version` must be new on every publish: use the release version.

### Now: metadata only

[`registry/server.json`](registry/server.json) has no `packages`. Its `websiteUrl`
points to the setup instructions. The registry documentation shows this form for
"MCP server embedded in a Desktop app", which is our case: the server ships inside the
OpenECraft packages. Clients cannot install from such an entry; it makes the server
findable. If `publish` refuses the entry, go straight to the packages below.

```bash
# mcp-publisher: brew install mcp-publisher, or the archive of your system from
# https://github.com/modelcontextprotocol/registry/releases/latest (mcp-publisher_<os>_<arch>.tar.gz)
mcp-publisher login github                     # device flow, sign in as nilres
mcp-publisher validate docs/launch/ai/registry/server.json
mcp-publisher publish docs/launch/ai/registry/server.json
curl "https://registry.modelcontextprotocol.io/v0/servers?search=io.github.nilres/openecraft"
```

Before you publish, check that the `websiteUrl` resolves on `master`, which is the case
once this folder is merged.

### Later: MCPB bundles and an image

[`registry/server.packages.json`](registry/server.packages.json) adds three `mcpb`
packages (Windows x64, macOS universal, Linux x64) and one `oci` package.

- **MCPB**
  - The `identifier` must be a GitHub release download URL that contains "mcp".
  - `fileSha256` is required; the registry does not check it, clients do.
  - The schema has no field for the operating system of a package. How a client picks
    among three bundles is not documented; that is the open question of this format.
    The file names say the system.
- **OCI**
  - The image must carry the label `io.modelcontextprotocol.server.name=io.github.nilres/openecraft`
    (in `Dockerfile.mcp`).
  - It runs the server in a container. The `--mount` runtime argument binds the
    projects folder to `/projects`, the only folder the server may use there.
- **cargo** (crates.io, also supported by the registry) would need every crate of the
  workspace published to crates.io, plus `mcp-name: io.github.nilres/openecraft` as
  visible text in the crate README. That is not worth it for this.

#### Building the MCPB bundles (a step for `release.yml`)

One bundle per system keeps them small. A bundle with all three programs would be over
150 MB.

1. For each of `windows-x64` (`openecraft-cli.exe`), `macos` (universal binary from
   `installer/macos/build.sh`, signed) and `linux-x64`, build a folder `stage/`:

   ```text
   stage/manifest.json      ← mcpb/manifest.json: "version" set, "compatibility.platforms" only this system
   stage/icon.png           ← assets/icon.png
   stage/server/openecraft-cli[.exe]
   stage/LICENSE-MIT, stage/LICENSE-APACHE, stage/THIRD-PARTY-LICENSES.html
   ```

2. `npx -y @anthropic-ai/mcpb validate stage/manifest.json`
3. `npx -y @anthropic-ai/mcpb pack stage openecraft-mcp-<version>-<system>.mcpb`
4. Attach it to the release, and add its `sha256sum` to `SHA256SUMS` and to
   `server.packages.json`.
5. `mcp-publisher publish docs/launch/ai/registry/server.packages.json`. In CI, use
   `mcp-publisher login github-oidc` with `permissions: id-token: write`.

The manifest asks the user for a **projects folder**. It becomes `OPENECRAFT_MCP_ROOT`,
because Claude Desktop starts the server in a folder of its own: relative paths do not
work there, and the agent has to use absolute paths in that folder. It also takes an
optional **tool list** (`OPENECRAFT_MCP_TOOLS`).

On macOS, a binary that is not notarized may be stopped by Gatekeeper when Claude
Desktop starts it. Bundle only the signed binary of a notarized build.

Then a double-click on the `.mcpb` installs the server in Claude Desktop (*Settings ›
Extensions*), with no JSON to edit. Claude Desktop runs on Windows and macOS; Linux
users keep the JSON configuration.

#### The image

```bash
docker build -f docs/launch/ai/registry/Dockerfile.mcp -t ghcr.io/nilres/openecraft-mcp:0.1.3 .
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | docker run -i --rm ghcr.io/nilres/openecraft-mcp:0.1.3 | head -c 300
```

For CI, add a target `mcp-server` to `installer/docker/Dockerfile` and a workflow like
`docker-parts-server.yml`, rather than keeping a second Dockerfile.

## Glama

Glama indexes public MCP repositories and gives each a score. awesome-mcp-servers
practically requires its badge.

1. On [glama.ai/mcp/servers](https://glama.ai/mcp/servers), use *Add Server* with
   `https://github.com/nilres/OpenECraft`. The fields of the form were not visible
   without an account.
2. In the server's build settings on Glama, give it the Dockerfile:
   `docs/launch/ai/registry/Dockerfile.mcp`, or the `mcp-server` target once it exists.
   The check builds the image, starts the server and lists its tools. The build compiles
   the workspace, which takes several minutes.
3. Commit [`registry/glama.json`](registry/glama.json) as `glama.json` in the
   repository root, then run *Claim* on the Glama page. This is the only file outside
   `docs/` in these steps; it lists `nilres` as maintainer.
4. When the checks pass, the badge is
   `https://glama.ai/mcp/servers/nilres/OpenECraft/badges/score.svg`.

## awesome-mcp-servers (punkpeye)

The category is **🏭 Industrial & IoT** (`#industrial--iot`): "machinery and
operational technology", which is what control cabinets are for. KiCad sits under
"📟 Embedded System", which is for PCB design, not this.

- Keep the entries in alphabetical order; `nilres/OpenECraft` goes between the entries
  starting with "n".
- Use only the emoji of the legend: 🦀 Rust, 🏠 local, 🍎 🪟 🐧 the systems.
- The CI of the list checks the `owner/repo` link text and the Glama badge. A PR
  without the badge gets the label `missing-glama` and stays open.

```markdown
- [nilres/OpenECraft](https://github.com/nilres/OpenECraft) [![nilres/OpenECraft MCP server](https://glama.ai/mcp/servers/nilres/OpenECraft/badges/score.svg)](https://glama.ai/mcp/servers/nilres/OpenECraft) 🦀 🏠 🍎 🪟 🐧 - Electrical CAE for control cabinets (IEC 60204-1): create projects, place symbols, connect pins by device tag, assign parts, run the project check, write terminal diagrams, bill of materials and PDF / DXF exports; the projects open in the OpenECraft editor.
```

PR title: `Add OpenECraft (electrical CAE)`. Body: one sentence, and a link to
`docs/launch/ai/README.md` with the transcript.

## mcp.so and mcpservers.org

The form fields of both:

- **Name:** OpenECraft
- **Type:** MCP Server (local)
- **Repository:** https://github.com/nilres/OpenECraft
- **Website:** https://openecraft.org
- **Category:** Developer tools / Engineering (mcpservers.org has no industrial
  category; pick the closest)
- **Official registry name:** io.github.nilres/openecraft, once published
- **Short description:** "Open source electrical CAE: an AI agent draws schematics,
  assigns parts, runs the project check and exports terminal diagrams, bill of
  materials and PDF. Native binary, works offline."
- **Tags:** electrical, schematic, CAE, EDA, PLC, IEC 60204, Rust
- **License:** MIT OR Apache-2.0
- **Contact email (mcpservers.org):** the maintainer's address, entered by the
  maintainer.

mcp.so also takes GitHub issues on `chatmcp/mcpso` ("Submit MCP Server: OpenECraft"),
but thousands are open; the form is quicker. Neither needs the paid fast track.

## Docker MCP Catalog

The catalog runs every server in a container. For OpenECraft that means the projects
are in a mounted folder, and the parts server is only reachable by URL. It works, but it
is not how an electrician runs the editor next to the agent. Do it after the image
exists and someone asks for it.

```yaml
# servers/openecraft/server.yaml in a fork of docker/mcp-registry
name: openecraft
image: ghcr.io/nilres/openecraft-mcp
type: server
meta:
  category: productivity
  tags: [electrical, schematic, cae, engineering]
about:
  title: OpenECraft
  description: Electrical CAE – draw schematics, assign parts, run checks, export reports and PDF
  icon: https://raw.githubusercontent.com/nilres/OpenECraft/master/assets/icon.png
source:
  project: https://github.com/nilres/OpenECraft
  commit: <commit of the release>
config:
  description: Mount the folder of your projects at /projects
  env:
    - name: OPENECRAFT_MCP_TOOLS
      example: new_project,open_project,save_project,place_symbol,connect,check,export
      value: '{{openecraft.tools}}'
```

Validate it with `task wizard`, or `task create -- --category productivity
https://github.com/nilres/OpenECraft --image ghcr.io/nilres/openecraft-mcp`, then
`task build -- --tools openecraft`.

## Install buttons for the README

These links need no directory. They work once `openecraft-cli` is on the `PATH`;
otherwise the user edits the command afterwards.

- **Cursor:**
  `cursor://anysphere.cursor-deeplink/mcp/install?name=openecraft&config=eyJjb21tYW5kIjoib3BlbmVjcmFmdC1jbGkiLCJhcmdzIjpbIm1jcCJdfQ==`
- **VS Code** (through the web redirect, because GitHub does not link `vscode:` URLs):
  `https://vscode.dev/redirect?url=vscode%3Amcp%2Finstall%3F%257B%2522name%2522%253A%2522openecraft%2522%252C%2522command%2522%253A%2522openecraft-cli%2522%252C%2522args%2522%253A%255B%2522mcp%2522%255D%257D`
- After registry publication, VS Code also takes
  `vscode:mcp/by-name/io.github.nilres/openecraft`.

## Steps only the maintainer can do

In this order:

1. **Merge this folder** (`docs/launch/ai/`) to `master`, so that the links in
   `server.json` and the posts resolve.
2. **Official registry, metadata only:** install `mcp-publisher`, `mcp-publisher login
   github` as `nilres`, then `validate` and `publish` with `registry/server.json`. This
   needs the GitHub account.
3. **Glama:**
   - *Add Server* with the repository and give it the Dockerfile.
   - Commit `glama.json` to the repository root and claim the listing.
   - Wait for the checks to pass.
   - This needs a Glama account (GitHub login).
4. **awesome-mcp-servers:** fork, add the line above under Industrial & IoT, open the PR
   from your account.
5. **mcp.so and mcpservers.org:** fill in the forms. They need your email; the paid
   fast track is optional.
6. **Next release:**
   - Build the MCPB bundles and the image; push the image to ghcr.io (CI already has the
     rights for the other images).
   - Fill in the hashes and publish `registry/server.packages.json` with the new
     version.
   - Make sure the Windows and macOS binaries in the bundles are signed, or Gatekeeper
     and SmartScreen will get in the way.
7. **Later, optional:** the Docker MCP catalog PR, and a Claude plugin that wraps the
   MCPB, if the Claude directory is wanted.
