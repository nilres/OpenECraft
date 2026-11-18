# Stores and package managers

What OpenECraft needs to appear in the app stores and package managers people search,
prepared for version 0.1.2. Every file here points at the assets of the GitHub release
`v0.1.2` (checksums from the release's asset digests); the maintainer submits them. The
packages themselves are built by CI ([releasing](../../docs/releasing.md)), nothing here
builds anything.

| Store | Prepared | Repository / place | Notes |
| --- | --- | --- | --- |
| winget | `winget/manifests/o/OpenECraft/OpenECraft/0.1.2/` | PR to `microsoft/winget-pkgs` | open to anyone, no notability rule |
| Homebrew | `homebrew/Casks/openecraft.rb` | own tap `nilres/homebrew-openecraft` | `homebrew/cask` needs notarized apps and 225 stars for a self-submission |
| AUR | `aur/openecraft-bin/` (`PKGBUILD`, `.SRCINFO`) | `ssh://aur@aur.archlinux.org/openecraft-bin.git` | from the Linux tar.gz |
| Scoop | `scoop/openecraft.json` | own bucket or PR to `ScoopInstaller/Extras` | unpacks the Windows setup |
| Chocolatey | `chocolatey/` (`openecraft.nuspec`, `tools/`) | `choco push` to community.chocolatey.org | moderated |
| Snap Store | `../linux/snap/snapcraft.yaml` (unchanged) | snapcraft.io | the snap of the release, uploaded as is |
| Flathub | requirements below, **no manifest** | PR to `flathub/flathub`, branch `new-pr` | Flathub refuses AI-written manifests |

The editor updates itself from GitHub for the Windows setup, the AppImage and the
macOS app bundles (`openecraft/src/updater.rs`); every other installation only shows
that a new version exists. The notes per store say what that means there.

## winget

Package `OpenECraft.OpenECraft` (publisher `OpenECraft` as in `AppPublisher` of
`installer/windows/openecraft.iss`), an installer for the user (`/CURRENTUSER`, the
default of the setup) and one for all users (`/ALLUSERS`), English as default locale
and German, French and Spanish locales. `winget install OpenECraft.OpenECraft`
installs the editor only (the setup type *Editor*); the other components:

```powershell
winget install OpenECraft.OpenECraft --custom "/COMPONENTS=main,partsmanager,cli,servers /TASKS=fileassoc,addtopath"
```

Steps:

1. Check locally (Windows, `winget settings --enable LocalManifestFiles` once, as
   administrator):
   ```powershell
   winget validate --manifest installer\stores\winget\manifests\o\OpenECraft\OpenECraft\0.1.2
   winget install --manifest installer\stores\winget\manifests\o\OpenECraft\OpenECraft\0.1.2
   ```
2. Fork `microsoft/winget-pkgs`, copy the folder to the same path
   `manifests/o/OpenECraft/OpenECraft/0.1.2/` in the fork, open a PR (one package
   version per PR; fill in its checklist). Alternatively `wingetcreate submit <folder>`
   or `komac submit <folder>` do fork, branch and PR in one go.
3. The validation pipeline installs the setup in a VM and scans it; a bot labels the PR.
   An unsigned setup is accepted. A first package is merged by a moderator, usually
   within a few days.

## Homebrew

The disk image is ad hoc signed, not notarized. `homebrew/cask` only takes apps that
pass Gatekeeper and, for a submission by the author, a repository with 90 forks, 90
watchers or 225 stars (`docs/Acceptable-Casks.md`, `Package-Acceptance-Policy.md` of
`Homebrew/brew`). Until then the cask lives in an own tap:

1. Create the public repository `nilres/homebrew-openecraft` with `Casks/openecraft.rb`
   from here.
2. Check on a Mac:
   ```bash
   brew tap nilres/openecraft
   brew audit --cask --online nilres/openecraft/openecraft
   brew style nilres/openecraft/openecraft
   brew install --cask nilres/openecraft/openecraft
   ```
3. Users install with `brew install --cask nilres/openecraft/openecraft`; Gatekeeper
   asks once (the cask's caveats say how). Check the `zap` paths on a Mac: they are
   where eframe 0.37 keeps the state of the editor and the parts manager.

`auto_updates true` because the editor replaces `OpenECraft.app` itself; `brew upgrade`
then skips it unless `--greedy` is given. Homebrew asks contributors to disclose AI use
in PRs (`CONTRIBUTING.md` of `homebrew-cask`); that matters once the cask moves there.

## AUR

`openecraft-bin` installs the Linux archive (`OpenECraft-<version>-linux-x86_64.tar.gz`,
programs built on Debian 12) to `/usr`, the license files to
`/usr/share/licenses/openecraft-bin/`. The editor only shows a notice of new versions.

1. Account on <https://aur.archlinux.org> with an SSH key.
2. On Arch Linux:
   ```bash
   git clone ssh://aur@aur.archlinux.org/openecraft-bin.git
   cp installer/stores/aur/openecraft-bin/{PKGBUILD,.SRCINFO} openecraft-bin/
   cd openecraft-bin
   # fill in the Maintainer line of the PKGBUILD
   makepkg --printsrcinfo | diff - .SRCINFO   # must be empty
   namcap PKGBUILD && makepkg -si && namcap openecraft-bin-*.pkg.tar.zst
   git add PKGBUILD .SRCINFO && git commit -m "openecraft-bin 0.1.2-1" && git push
   ```
   The first push creates the package. There is no review; the AUR asks that the
   package follows the [AUR submission guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
   (`-bin` because it repackages a binary release).

A version with a suffix (`0.2.0-rc.1`) needs `pkgver=0.2.0rc.1` and an explicit
`_tag`; `-` is not allowed in `pkgver`.

## Snap Store

The release already contains a strictly confined snap (`openecraft_<version>_amd64.snap`,
`installer/linux/snap/snapcraft.yaml`, base core24, GNOME extension).

1. Ubuntu One account, then `snapcraft login` and `snapcraft register openecraft`.
2. Download the snap of the release and upload it:
   `snapcraft upload --release=stable openecraft_0.1.2_amd64.snap`. The automated
   review passes strict snaps with these plugs; a failing check is shown on the
   upload page.
3. Fill in the listing at <https://snapcraft.io/openecraft/listing>: title, summary,
   description (from the snap), screenshots (`docs/images/*.png`), icon
   (`assets/icon.png`), category *Science* (and *Art and design* or *Productivity*),
   website, contact, license.
4. `removable-media` is not connected automatically: users run
   `snap connect openecraft:removable-media`, or ask for auto-connection in the
   *store-requests* category of <https://forum.snapcraft.io>.

The editor in the snap only tells of new versions, which the store installs on its own;
`environment: OPENECRAFT_NO_UPDATE_CHECK: "1"` on the apps of `snapcraft.yaml` would
silence it, but also for the snap downloaded from the release page.

## Scoop

`scoop/openecraft.json` unpacks the Windows setup with innounp (`"innosetup": true`), so
all programs are there: shortcuts for the editor and the parts manager, shims for the
command line and the servers. Without the uninstaller next to it the editor does not
update itself, it only shows the notice.

1. Check: `scoop install installer\stores\scoop\openecraft.json`, start the shortcut and
   `openecraft-cli --version`. innounp must know the Inno Setup version of the CI build
   (Inno Setup 6); if it fails to unpack, the manifest needs `installer.script` running
   the setup instead.
2. Either an own bucket (repository `nilres/scoop-openecraft` with
   `bucket/openecraft.json`; `scoop bucket add openecraft https://github.com/nilres/scoop-openecraft`)
   or a PR adding `bucket/openecraft.json` to `ScoopInstaller/Extras` (GUI programs;
   follow its PR template). In Extras the bucket's bot updates the manifest from
   `checkver` / `autoupdate`.

## Chocolatey

`openecraft` installs the setup for all users, silently; the uninstaller of the setup
removes it.

1. Account on <https://community.chocolatey.org>, API key from the account page; set
   `<owners>` in the nuspec to the account name.
2. On Windows with Chocolatey:
   ```powershell
   cd installer\stores\chocolatey
   choco pack
   choco install openecraft --source . -y
   choco uninstall openecraft -y
   choco push openecraft.0.1.2.nupkg --source https://push.chocolatey.org/ --api-key <key>
   ```
3. The package goes through automatic validation and verification (VirusTotal, a test
   install) and then a human moderator; a first package can take days to weeks.

## Flathub

Flathub publishes only apps built from source in its own build service, and its
[requirements](https://docs.flathub.org/docs/for-app-authors/requirements) say that
Flathub manifests must not contain AI-generated or AI-assisted content and that AI
tools must not open submission PRs; AI use in the app's code, documentation or
packaging has to be disclosed in the PR. So no manifest is prepared here: the
maintainer writes it and opens the PR. `installer/linux/flatpak/io.github.nilres.OpenECraft.yml`
packages prebuilt binaries for the release's `.flatpak` file and cannot be submitted.

What the manifest has to cover, from the existing packaging:

| Topic | Facts |
| --- | --- |
| App id | `io.github.nilres.OpenECraft` (verified through the GitHub account `nilres`) |
| Runtime | `org.freedesktop.Platform` / `Sdk` `25.08` as today, plus the SDK extension `org.freedesktop.Sdk.Extension.rust-stable`; the code needs no GTK (the file dialogs use the portal) and no system C libraries (SQLite is bundled) |
| Source | the release tag as `git` source with `tag` and `commit` (v0.1.2: `5ac04ff59230264d6a9d0343f5c9a5272250db7b`) |
| Crates | no network during the build: `cargo-sources.json` from [flatpak-cargo-generator](https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo) on the `Cargo.lock` of the tag, `python3 flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json` (617 crates, no git dependencies; about 250 KB, it lives in the Flathub repository and is regenerated for every release); `CARGO_HOME` set to `cargo` in the module's build folder, `cargo --offline build --release --locked` |
| Programs | `-p openecraft -p openecraft-parts-manager -p openecraft-cli -p openecraft-parts-server -p openecraft-project-server`, installed like `stage()` in `installer/linux/build.sh`: the binaries, both desktop entries, the metainfo, the MIME types, the icons (256 px PNG and SVG) |
| Licenses | `LICENSE-MIT`, `LICENSE-APACHE`, `crates/openecraft-fonts/fonts/OFL.txt` and `THIRD-PARTY-LICENSES.html` to `/app/share/licenses/io.github.nilres.OpenECraft/`; the last one is generated by cargo-about, so either build cargo-about too or take the release asset (`THIRD-PARTY-LICENSES.html` of v0.1.2, sha256 `f74ac301ffdeb9466286af0dbd4a66bf5718305698fb7099154397aa53a628ac`) |
| Permissions | `--share=ipc`, `--socket=wayland`, `--socket=fallback-x11`, `--device=dri`; `--share=network` (parts server, project server, Vault, EPLAN Data Portal); `--filesystem=home` needs a reason in the PR (projects open other files by path: recent projects, linked documents, the folders of a split project) or has to give way to the portals |
| Updates | `--env=OPENECRAFT_NO_UPDATE_CHECK=1`: Flathub updates the app, the notice pointing at GitHub would only confuse |
| Other programs | `flatpak run --command=openecraft-cli io.github.nilres.OpenECraft mcp`, as today |

The metainfo (`installer/linux/io.github.nilres.OpenECraft.metainfo.xml`) now has what
the Flathub linter demands (screenshots with captions, releases, OARS rating, developer,
launchable) and what its quality guidelines ask for (summary of 34 characters, brand
colours, help and contribution links); `appstreamcli validate` passes. The metainfo of
the tag **v0.1.2 has no screenshots** (`appstream-missing-screenshots`, never
excepted), so the first submission uses the first release tag after this change.
The screenshots are pinned to `v0.1.2` and are app captures without window frame;
Flathub's quality review (not the linter) wants window captures on Linux of at most
1000×700 (2000×1400 HiDPI).

Steps: build and lint locally with `org.flatpak.Builder` (`flatpak run
--command=flathub-build org.flatpak.Builder --install <manifest>` and
`flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest <manifest>` /
`repo repo`), fork `flathub/flathub` with all branches, branch from `new-pr`, add the
manifest and `cargo-sources.json`, open the PR against `new-pr` titled
`Add io.github.nilres.OpenECraft`, keep the PR template, answer the review, and
comment `bot, build` for a test build. After the merge Flathub creates
`flathub/io.github.nilres.OpenECraft`; accept the invitation within a week (2FA
required).

## A new release

Every file above names the version, the download URL and the SHA-256 of the asset
(`gh release view v<version> --json assets` lists the `digest` of each). Per store:

| Store | Update | Could be automated by (suggestion, not set up) |
| --- | --- | --- |
| winget | new version folder, PR | `komac update OpenECraft.OpenECraft --version <v> --urls <setup url> --submit`, or the `vedantmgoyal9/winget-releaser` action on `release: published` with a PAT of a fork of winget-pkgs |
| Homebrew tap | `version` and `sha256` | `brew bump-cask-pr --version <v> nilres/openecraft/openecraft`, or a scheduled workflow in the tap running `brew livecheck` |
| AUR | `pkgver`, `pkgrel=1`, `sha256sums`, `.SRCINFO` | `KSXGitHub/github-actions-deploy-aur` on `release: published` with an AUR SSH key secret |
| Snap Store | upload the release's snap | `snapcore/action-publish` in the `publish` job of `release.yml` with `SNAPCRAFT_STORE_CREDENTIALS` (`snapcraft export-login --snaps=openecraft --acls package_access,package_push,package_update,package_release -`) |
| Scoop | `checkver` / `autoupdate` (Extras: its bot) | own bucket: the Excavator workflow of the Scoop bucket template |
| Chocolatey | `version`, URL, checksum, `choco push` | `chocolatey-au` update script |
| Flathub | `tag`, `commit`, new `cargo-sources.json` in the Flathub repository | Flathub's external data checker cannot regenerate the crate sources, so this stays by hand |

The release workflow publishes a draft; store updates start only after the release is
published, as the URLs of a draft are not public.
