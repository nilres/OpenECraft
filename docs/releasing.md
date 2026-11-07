# Releasing

How a version of OpenECraft is released. A tag `v<version>` starts
`.github/workflows/release.yml`: it checks the version, builds every package with the
jobs of `ci.yml` and creates a draft GitHub release with them.

## Steps

1. Set the version in `[workspace.package]` of the root `Cargo.toml` (all crates and
   installers take it from there) and run `cargo check` to update `Cargo.lock`.
2. Add `<release version="<version>" date="<yyyy-mm-dd>"/>` at the top of `<releases>`
   in `installer/linux/io.github.nilres.OpenECraft.metainfo.xml` (or correct its date).
3. Give the version's section in `CHANGELOG.md` the same date
   (`## [<version>] - <yyyy-mm-dd>`) and start a new `unreleased` section after the
   release.
4. Commit, then tag and push:

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

5. When the workflow is done, check the draft under *Releases*: the files, their
   `SHA256SUMS` and the notes (the changelog section). Publish it.

The workflow refuses a tag that does not match the version, a version without a
`<release>` in the metainfo file, a changelog section without a date (`unreleased`)
and a metainfo date that differs from the changelog's. A tag with a suffix
(`v0.2.0-rc.1`) becomes a pre-release, and its Docker images do not get the tag
`latest`.

The notes of the release are the changelog section, followed by how to check the
downloads, the Docker images of the servers and, without the signing secrets (see
below), that the Windows setup is not signed and the macOS apps are not notarized.
For a public repository the workflow also
attests the build provenance of every file (`gh attestation verify <file> --repo
nilres/OpenECraft`); private repositories need GitHub Enterprise Cloud for that.

## Pinned versions

Release builds are reproducible: cargo runs with `--locked` (`Cargo.lock` must be up to
date), and the versions of the tools are pinned. Raise them together:

| Tool | Where |
| --- | --- |
| Rust | `dtolnay/rust-toolchain@<version>` in `.github/workflows/ci.yml`; `ARG RUST_VERSION` in `installer/docker/Dockerfile` |
| cargo-about, trunk | `tool: cargo-about@…`, `trunk@…` in `ci.yml`; `cargo install cargo-about@…` in `installer/docker/Dockerfile`; the install hints of the installer scripts, `installer/web/` and the docs |
| appimagetool, AppImage runtime | versions and SHA-256 at `build_appimage` in `installer/linux/build.sh`; a new runtime needs `installer/linux/appimage/RUNTIME-LICENSES.txt` checked (its components and their versions) |

## Release files

| File | Built by |
| --- | --- |
| `OpenECraft-<version>-setup.exe` | `installer/windows/build.ps1` |
| `openecraft_<version>_amd64.deb`, `openecraft-<version>-1.x86_64.rpm`, `OpenECraft-<version>-x86_64.AppImage`, `OpenECraft-<version>-linux-x86_64.tar.gz` | `installer/linux/build.sh` |
| `OpenECraft-<version>-x86_64.flatpak`, `openecraft_<version>_amd64.snap` | `installer/linux/build.sh flatpak snap` |
| `OpenECraft-<version>-macos.dmg` | `installer/macos/build.sh` |
| `OpenECraft-<version>-web.zip`, `OpenECraft-Parts-Manager-<version>-web.zip` | `trunk build --release` |
| `THIRD-PARTY-LICENSES.html`, `SHA256SUMS` | the workflow |
| `ghcr.io/nilres/openecraft-parts-server:<version>`, `ghcr.io/nilres/openecraft-project-server:<version>` (also `<major>.<minor>` and `latest`) | `installer/docker/Dockerfile`, pushed by the job `docker` ([installation](installation.md)) |

## Licenses

OpenECraft is licensed under MIT (`LICENSE-MIT`) or Apache-2.0 (`LICENSE-APACHE`). The
installer scripts and the release build of the web pages (`installer/web/licenses.sh`,
run by trunk) generate `THIRD-PARTY-LICENSES.html` from the licenses of all libraries
in the programs with [cargo-about](https://github.com/EmbarkStudios/cargo-about)
(`cargo install cargo-about@0.9.3 --locked --features cli`; configuration
`installer/about.toml`, template `installer/about.hbs`), and ship it with both license
files and the license of the bundled Noto fonts (`OFL-NotoSans.txt`); the web ZIPs
have them next to `index.html`. The AppImage also has
`APPIMAGE-RUNTIME-LICENSES.txt` (`installer/linux/appimage/RUNTIME-LICENSES.txt`) for
the AppImage runtime it starts with.

cargo-about only knows the licenses a library declares. Files inside a library under
other terms need a notice in `installer/about.hbs` by hand: today the fonts of
`epaint_default_fonts` (Hack is under the Bitstream Vera License as well) and the note
on the MPL-2.0 library `acadrust` (used unmodified; a modified copy would need its
source published). Check the license files of a new library that bundles fonts, data
or C code.

A new dependency under a license that is not in `accepted` of `installer/about.toml`
stops the installer build. Add the license only if it allows distribution in closed
and open programs alike, like the ones listed. The package metadata (`project_license`
of the metainfo file, `license` of the snap, `License` of the RPM spec, the deb
`copyright` in `build.sh`) name the licenses of the programs as a whole; keep them in
step when a font or a library under another kind of license is added. The snap names
only licenses that snapd knows (`spdx/licenses.go` of snapd): `snap pack` refuses
`Ubuntu-font-1.0` and `Bitstream-Vera`, so those two are left out there.

## Code signing

Without the secrets below the packages are built unsigned: SmartScreen and Gatekeeper
then ask the user for confirmation on first start. Set them as repository secrets
(*Settings › Secrets and variables › Actions*) to sign in CI; pull requests from forks
never see them.

| Secret | Content |
| --- | --- |
| `WINDOWS_CERTIFICATE` | Code signing certificate as a `.pfx` file, base64 (`base64 -w0 cert.pfx`) |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password of the `.pfx` file |
| `MACOS_CERTIFICATE` | *Developer ID Application* certificate with its key as a `.p12` file, base64 |
| `MACOS_CERTIFICATE_PASSWORD` | Password of the `.p12` file |
| `MACOS_SIGN_IDENTITY` | Name of the certificate, `Developer ID Application: <name> (<team id>)` |
| `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD` | Apple account, team and app-specific password for notarization |

On Windows `build.ps1` signs the programs, the setup and the uninstaller with
`signtool` (`WINDOWS_SIGN_CERT=<.pfx file>`, `WINDOWS_SIGN_PASSWORD`). Certificate
authorities issue new code signing certificates only on hardware tokens or in cloud
signing services, which cannot be exported as a `.pfx`; such a certificate needs a
signing step of its provider in place of the `.pfx` (for Azure Trusted Signing the
`azure/trusted-signing-action`). On macOS `build.sh` signs the apps with the hardened
runtime and notarizes and staples the disk image (see [installation](installation.md)).
