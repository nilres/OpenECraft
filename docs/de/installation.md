# Installation und Bauen

Pakete für Windows, Linux und macOS, die Web-Version und das Bauen aus dem Quellcode.

Veröffentlichte Versionen stehen auf der
[Release-Seite](https://github.com/nilres/OpenECraft/releases)
mit einer Datei `SHA256SUMS`; wie sie entstehen, steht in [releasing](../releasing.md).

Das Bauen aus dem Quellcode braucht eine aktuelle stabile Rust-Toolchain
(`rustup update stable`); Releases werden mit der Version gebaut, die in
`.github/workflows/ci.yml` festgelegt ist. Die Installer-Skripte unten und
`trunk build --release` brauchen außerdem
[cargo-about](https://github.com/EmbarkStudios/cargo-about)
(`cargo install cargo-about@0.9.3 --locked --features cli`) für
`THIRD-PARTY-LICENSES.html`, das jedes Paket neben `LICENSE-MIT` und
`LICENSE-APACHE` enthält.

## Desktop (Windows / Linux / macOS)

```bash
cargo run --release -p openecraft
```

Unter Linux werden die üblichen Fenster- / Dialogbibliotheken gebraucht, z. B. unter
Debian/Ubuntu:

```bash
sudo apt install libxkbcommon-dev libgtk-3-dev libwayland-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

Eine auf der Befehlszeile angegebene Projektdatei wird beim Start geöffnet
(`openecraft plant.oec`).

## Windows-Installer

`OpenECraft-<version>-setup.exe` installiert den Editor mit einem Eintrag im Startmenü,
ohne Administratorrechte (oder für alle Benutzer unter Programme), in Englisch, Deutsch,
Französisch oder Spanisch nach der Windows-Sprache. Optional: die Artikelverwaltung, die
Befehlszeile / der MCP-Server (auf Wunsch in den `PATH` eingetragen), der Artikel- und
der Projektserver, eine Desktop-Verknüpfung und das Öffnen von `.oec`- / `.oecz`- /
`.oecp`-Dateien per Doppelklick. Deinstalliert wird in *Einstellungen › Apps*; Projekte
und Einstellungen bleiben erhalten. CI baut ihn bei jedem Push (Artefakt
`openecraft-windows-installer`); lokal braucht er
[Inno Setup 6](https://jrsoftware.org/isinfo.php) (`winget install JRSoftware.InnoSetup`):

```bash
powershell -ExecutionPolicy Bypass -File installer/windows/build.ps1   # → target/installer/
```

Die Programme binden die C-Laufzeit statisch, es wird also kein Visual C++
Redistributable gebraucht.
Für die unbeaufsichtigte Installation (Softwareverteilung) das Setup mit
`/VERYSILENT /SUPPRESSMSGBOXES` starten, `/ALLUSERS` für alle Benutzer ergänzen und
Teile mit `/COMPONENTS="main,partsmanager,cli,servers"` und
`/TASKS="fileassoc,addtopath"` auswählen.
Mit `WINDOWS_SIGN_CERT=<.pfx file>` und `WINDOWS_SIGN_PASSWORD` signiert das Skript die
Programme, das Setup und das Deinstallationsprogramm (signtool des Windows SDK); sonst
sind sie unsigniert und SmartScreen fragt beim ersten Start nach einer Bestätigung (siehe
[releasing](../releasing.md)).
`installer/icon.py` zeichnet das Anwendungssymbol (`assets/icon.png`, `assets/icon.ico`,
`assets/icon.icns`, `assets/icon.svg`).

## Linux-Pakete

`installer/linux/build.sh` baut die Pakete für die Architektur der Build-Maschine nach
`target/installer/`. Jedes enthält den Editor und die Artikelverwaltung (mit
Menüeinträgen in Englisch, Deutsch, Französisch und Spanisch), die Befehlszeile / den
MCP-Server sowie den Artikel- und den Projektserver. Mit deb, rpm, Flatpak und tar.gz
öffnen sich `.oec`- / `.oecz`- / `.oecp`-Dateien per Doppelklick.

```bash
bash installer/linux/build.sh                        # tar.gz, deb, rpm, AppImage
bash installer/linux/build.sh flatpak snap           # sandboxed packages
```

| Paket | Datei | Installation | Build-Werkzeug |
| --- | --- | --- | --- |
| Debian, Ubuntu | `openecraft_<version>_amd64.deb` | `sudo apt install ./openecraft_*.deb` | `dpkg-dev` |
| Fedora, RHEL, openSUSE | `openecraft-<version>-1.x86_64.rpm` | `sudo dnf install ./openecraft-*.rpm` | `rpmbuild` |
| Jede Distribution | `OpenECraft-<version>-x86_64.AppImage` | `chmod +x`, dann starten | Debian oder Ubuntu, `patchelf`, `zsync`, `appimagetool` und dessen Laufzeit (heruntergeladen, Prüfsummen geprüft) |
| Jede Distribution | `OpenECraft-<version>-x86_64.flatpak` | `flatpak install --user OpenECraft-*.flatpak` | `flatpak-builder` |
| Jede Distribution | `openecraft_<version>_amd64.snap` | `sudo snap install --dangerous openecraft_*.snap` | `snapcraft` |
| Jede Distribution | `OpenECraft-<version>-linux-x86_64.tar.gz` | `./install.sh` (nach `~/.local`) | `tar` |

Die Programme heißen `openecraft`, `openecraft-parts-manager`, `openecraft-cli`,
`openecraft-parts-server` und `openecraft-project-server`. Im AppImage werden sie mit
`OpenECraft-*.AppImage parts-manager|cli|parts-server|project-server …` gestartet, im
Flatpak mit `flatpak run --command=openecraft-cli io.github.nilres.OpenECraft …`
und im Snap als `openecraft.cli`, `openecraft.parts-manager`, … CI baut sie bei jedem
Push (Artefakte `openecraft-linux-packages`, `openecraft-flatpak`, `openecraft-snap`);
deb, rpm, AppImage und tar.gz werden unter Debian 12 gebaut und laufen mit glibc 2.36
oder neuer.
Das AppImage bringt die Fensterbibliotheken mit, die nicht auf jedem System vorhanden
sind (xkbcommon, Xcursor, Xi: `appimage_libs` in `installer/linux/build.sh`); der Build
weist eines zurück, das eine andere Bibliothek oder eine neuere glibc als die Programme
braucht. Es enthält außerdem Update-Informationen für AppImageUpdate und ähnliche
Werkzeuge: das neueste GitHub-Release des Repositorys, das es gebaut hat
(`GITHUB_REPOSITORY`, sonst `nilres/OpenECraft`), dessen
`OpenECraft-<version>-x86_64.AppImage.zsync` der Build neben das AppImage schreibt
(`APPIMAGE_UPDATE_INFO` setzt andere Update-Informationen, ein leerer Wert keine).
Die Desktop-Einträge, MIME-Typen und AppStream-Daten (`io.github.nilres.OpenECraft`)
liegen in `installer/linux/`; eine neue Version braucht ein `<release>` in der
Metainfo-Datei.

## macOS-Disk-Image

`installer/macos/build.sh` baut `OpenECraft-<version>-macos.dmg` (macOS 11 oder neuer,
Universal-Binaries für Apple Silicon und Intel) mit den Apps *OpenECraft* und
*OpenECraft Parts Manager*; sie werden nach *Programme* gezogen. Die Befehlszeile und
die Server liegen im App-Bundle:

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
bash installer/macos/build.sh                        # → target/installer/
/Applications/OpenECraft.app/Contents/MacOS/openecraft-cli help
```

Ohne `MACOS_SIGN_IDENTITY` werden die Apps ad hoc signiert, sodass Gatekeeper beim
ersten Start nach einer Bestätigung fragt (*Systemeinstellungen › Datenschutz &
Sicherheit › Dennoch öffnen*). Mit einem Developer-ID-Zertifikat
(`MACOS_SIGN_IDENTITY="Developer ID Application: …"`) und einem
`notarytool`-Schlüsselbundprofil (`MACOS_NOTARY_PROFILE`, `MACOS_NOTARY_KEYCHAIN`, wenn
es nicht im Anmelde-Schlüsselbund liegt) wird das Image signiert und notarisiert.
Projekte werden mit *Datei › Projekt öffnen* geöffnet; das Öffnen per Doppelklick im
Finder wird noch nicht unterstützt. CI baut das Image bei jedem Push (Artefakt
`openecraft-macos-dmg`).

## Updates

Beim Start fragt der Editor im Hintergrund bei GitHub nach dem neuesten Release
(`https://api.github.com/repos/nilres/OpenECraft/releases/latest`); ohne Verbindung
arbeitet er ohne Meldung weiter. Ein neueres Release wird heruntergeladen, gegen die
`SHA256SUMS` des Release geprüft und installiert:

| Installation | Update |
| --- | --- |
| Windows-Setup | `OpenECraft-<version>-setup.exe` läuft still (`/SILENT`), wenn der Editor geschlossen wird, im Ordner und mit den Komponenten der Installation; *Neu starten und aktualisieren* startet den Editor danach wieder |
| AppImage | die AppImage-Datei wird sofort ersetzt; die neue Version startet beim nächsten Mal (AppImageUpdate und ähnliche Werkzeuge aktualisieren es ebenfalls) |
| macOS-App | `OpenECraft.app` (und `OpenECraft Parts Manager.app` daneben) werden sofort aus dem Disk-Image ersetzt |
| deb, rpm, Flatpak, Snap, tar.gz, eigener Build | die Statusleiste meldet nur die neue Version und verlinkt deren Release-Seite |

Die Statusleiste zeigt den Download und *Neu starten und aktualisieren*; *Datei › Hilfe ›
Nach Updates suchen* sucht erneut und meldet jedes Ergebnis. *Einstellungen › Updates*
schaltet die Prüfung beim Start ab, die Umgebungsvariable `OPENECRAFT_NO_UPDATE_CHECK`
(beliebiger Wert) tut das für alle Benutzer eines Rechners. Die Web-Version hat kein
eigenes Update.

Beim ersten Start einer neuen Version zeigt das Fenster *Neu in OpenECraft* die
Änderungen der Versionen seit der zuletzt verwendeten, wie `CHANGELOG.md` sie aufführt
(auf Englisch). *Datei › Hilfe › Versionsverlauf* zeigt sie wieder: eine Seite je
Version, nach Reihen (`0.1`) gegliedert im Fenster Hilfe.

## Docker-Images der Server

Der Artikelserver und der Projektserver werden bei jedem Release und bei jedem Push auf
`master` auch als Docker-Images veröffentlicht, gebaut von `installer/docker/Dockerfile`
(Debian 12, `linux/amd64`, laufen als Benutzer `openecraft`, Daten im Volume
`/data`, Lizenzen in `/usr/share/doc/openecraft/`):

| Image | Port | Tags |
| --- | --- | --- |
| `ghcr.io/nilres/openecraft-parts-server` | 8090 | `<version>`, `<major>.<minor>`, `latest` (Releases), `edge` (master), `sha-<commit>` |
| `ghcr.io/nilres/openecraft-project-server` | 8091 | dieselben |

```bash
docker run -d --name parts -p 127.0.0.1:8090:8090 -e OPENECRAFT_TOKEN=<secret> -v parts:/data ghcr.io/nilres/openecraft-parts-server
docker run -d --name projects -p 127.0.0.1:8091:8091 -e OPENECRAFT_PROJECT_TOKEN=<secret> -v projects:/data ghcr.io/nilres/openecraft-project-server
```

Die Einstellungen jedes Images stehen in `/etc/openecraft/parts-server.toml` und
`project-server.toml` (`bind = "0.0.0.0:<port>"`, `data` in `/data`); eine eigene Datei
dort einhängen oder Befehlszeilenoptionen nach dem Image-Namen anfügen
(`… openecraft-parts-server --install-catalogs all`). Auch der erste Administrator kommt
aus der Umgebung: `-e OPENECRAFT_ADMIN_USER=admin -e
OPENECRAFT_ADMIN_PASSWORD=<password>` (`OPENECRAFT_PROJECT_ADMIN_USER` /
`OPENECRAFT_PROJECT_ADMIN_PASSWORD` für den Projektserver) legt das Konto bei jedem
Start an oder setzt sein Passwort zurück.
Ein Bind-Mount anstelle eines Volumes muss für den Benutzer des Images beschreibbar sein
(uid 1000, oder den Container mit `--user` starten). `installer/docker/compose.yaml`
startet beide Server mit Docker Compose. Die Server sprechen unverschlüsseltes HTTP: Sie
bleiben auf `127.0.0.1` hinter einem TLS-Reverse-Proxy, wenn andere Rechner sie erreichen
([Projektserver](project-server.md)). Die Images selbst bauen:

```bash
docker build -f installer/docker/Dockerfile --target parts-server -t openecraft-parts-server .
docker build -f installer/docker/Dockerfile --target project-server -t openecraft-project-server .
```

CI baut beide Images bei jedem Push, startet sie einmal und prüft, dass sie antworten;
auf `master` und für ein Release-Tag schiebt es sie in die GitHub Container Registry
(Job `docker`), zusammen mit dem Image des Web-Editors (unten).

Jedes Image hat einen Workflow, der es allein neu baut, ohne den Rest von CI, von Hand
gestartet unter *Actions* (› Run workflow):

| Image | Workflow | Prüfung |
| --- | --- | --- |
| `openecraft-parts-server` | *Docker image of the parts server* (`docker-parts-server.yml`) | `check-server.sh parts` |
| `openecraft-project-server` | *Docker image of the project server* (`docker-project-server.yml`) | `check-server.sh project` |
| `openecraft-web` | *Docker image of the web editor* (`docker-web.yml`) | `check-editor-web.sh` |
| `openecraft-website` | *Docker image of the website* (`docker-website.yml`) | `check-website.sh` |

Alle vier nehmen dieselben Eingaben und teilen ihre Schritte
(`.github/workflows/docker-image.yml`): einen Branch, ein Release-Tag (`v0.1.2`, auch
eines, das älter ist als der Workflow) oder einen Commit, der gebaut wird (`ref`),
weitere Tags (`tags`), ob gepusht wird (`push`) und ob ohne den Build-Cache gebaut wird
(`no-cache`). Das Image wird wie in CI geprüft (die Skripte in `installer/docker/`) und
mit den Tags gepusht, die CI diesem Commit gibt (`sha-<commit>`, `edge` für die Spitze
von master, `<version>` sowie `<major>.<minor>` und `latest` nur, wenn kein neueres
Release sie hat), dazu die angegebenen Tags. Von der Befehlszeile:

```bash
gh workflow run docker-parts-server.yml -f ref=v0.1.2
gh workflow run docker-project-server.yml -f ref=master -f no-cache=true
gh workflow run docker-web.yml -f tags=test
gh workflow run docker-website.yml -f push=false
```

## Web (WebAssembly)

Die Web-Version läuft ohne Installation unter
[editor.openecraft.org](https://editor.openecraft.org); die Website des Projekts,
[openecraft.org](https://openecraft.org), verlinkt sie. Um sie selbst zu hosten:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk@0.22.1 --locked
cd openecraft && trunk serve            # development server on http://127.0.0.1:8080
cd openecraft && trunk build --release  # static files in openecraft/dist/ – host them on any web server
```

Beide Web-Builds legen `LICENSE-MIT`, `LICENSE-APACHE` und `OFL-NotoSans.txt` neben die
Seite, der Release-Build auch `THIRD-PARTY-LICENSES.html` (`installer/web/licenses.sh`,
unter Windows `licenses.ps1`, von trunk nach dem Build ausgeführt); diese Dateien
gehören beim Hosten der Seite dazu.

Im Browser lädt „Speichern“ die Projektdatei herunter und „Öffnen“ verwendet die
Dateiauswahl.

### Impressum eines Web-Editors

*Hilfe › Rechtliches und Datenschutz* des Editors zeigt den Haftungsausschluss von
OpenECraft (keine Gewährleistung, die genannten Produktnamen gehören ihren Inhabern) und
seine Datenschutzerklärung: was der Editor speichert und wo, und mit welchen Servern er
spricht. Ein Web-Editor, der für andere gehostet wird, zeigt außerdem, wer ihn betreibt,
wie es viele Länder verlangen (in Deutschland das *Impressum* nach § 5 DDG), und was der
Betreiber der Datenschutzerklärung hinzufügt (das Hosting, die Protokolle des
Webservers, die Ansprechperson). Beides wird in `legal` der `config.js` neben der Seite
eingestellt (`openecraft/config.js`, vom Build kopiert; nach einem Build bearbeiten), mit
den Feldern der [Website](../website.md#legal-notice): `name` (leer: kein Impressum),
`address`, `email`, `phone`, `representative`, `register`, `vat`, `responsible`, `text`
und `privacy`; Zeilen der Adresse und der Texte werden mit `\n` oder `|` getrennt. Der
Web-Editor, den die Website unter `/editor/` mitbringt, zeigt das Impressum der Website.
Der Desktop-Editor hat keinen Betreiber und zeigt nur den Haftungsausschluss und die
Datenschutzerklärung.
Die Artikelverwaltung hat dieselben Texte hinter *Rechtliches und Datenschutz* ihrer
oberen Leiste, mit einer eigenen Datenschutzerklärung; ihr Web-Build verwendet dieselbe
`config.js`.
Die Texte müssen stimmen: Eine Änderung des Editors, die etwas anderes speichert oder
sendet, ändert auch sie (`crates/openecraft-ui/src/legal.rs`).

### Docker-Image des Web-Editors

`ghcr.io/nilres/openecraft-web` (dieselben Tags wie die Server-Images) liefert den
Release-Build des Web-Editors mit nginx auf Port 8080 aus, als unprivilegierter Benutzer,
mit den Lizenzdateien neben der Seite; `installer/docker/compose.yaml` startet es mit
den Servern. Die Server werden vom Browser aus erreicht, nicht aus dem Container: Im
Editor werden sie mit der Adresse angegeben, die der Browser verwendet
(`http://127.0.0.1:8090` mit der Compose-Datei); beide Server erlauben Anfragen von
jedem Ursprung.

```bash
docker run -d --name openecraft-web -p 127.0.0.1:8080:8080 ghcr.io/nilres/openecraft-web
docker build -f installer/docker/Dockerfile --target editor-web -t openecraft-web .
```

Wer den Editor betreibt, wird mit den Variablen `OPENECRAFT_LEGAL_NAME`,
`OPENECRAFT_LEGAL_ADDRESS`, `OPENECRAFT_LEGAL_EMAIL`, `OPENECRAFT_LEGAL_PHONE`,
`OPENECRAFT_LEGAL_REPRESENTATIVE`, `OPENECRAFT_LEGAL_REGISTER`, `OPENECRAFT_LEGAL_VAT_ID`,
`OPENECRAFT_LEGAL_RESPONSIBLE`, `OPENECRAFT_LEGAL_TEXT` und `OPENECRAFT_LEGAL_PRIVACY`
eingestellt (unten, [Impressum eines Web-Editors](#impressum-eines-web-editors)): Das
Image schreibt bei jedem Start die `config.js` daraus.

Die nginx-Site ist `installer/docker/editor-web.conf` (eine eigene unter
`/etc/nginx/conf.d/default.conf` einhängen): `index.html` und die anderen Dateien werden
bei jedem Besuch geprüft, das WebAssembly und sein Skript, nach ihrem Hash benannt,
werden ein Jahr lang zwischengespeichert. Das WebAssembly ist 32 MB groß (ein
Release-Build der Seite ist auf Größe optimiert: das Profil `web` der `Cargo.toml` im
Wurzelverzeichnis, das etwa eine Viertelstunde braucht); der
Build des Images komprimiert es einmal
(`installer/docker/compress.sh`) und nginx sendet die Kopie, wie sie ist: 8 MB mit Brotli
an Browser, die danach fragen, was sie nur über HTTPS tun, sonst 12 MB mit gzip.
Ein eigener Server sollte solche Kopien ebenfalls senden, statt die Datei für jeden
Besucher zu komprimieren. Für andere Rechner gehört ein
TLS-Reverse-Proxy davor, wie bei den Servern. Neben der Seite liegen
`worker.js`, ein Worker, der die 3D-Verlegung eines geöffneten Projekts sucht, damit die
Seite währenddessen bedienbar bleibt, `routing/`, die Verlegungen der Beispiele aus dem
Build, damit ein Beispiel sofort öffnet, und `i18n/`, die Texte der anderen Sprachen
als Englisch, Deutsch, Französisch und Spanisch (`<code>.tsv`): Die Seite holt die
Datei einer Sprache, wenn sie verwendet wird, statt alle mitzuführen, und zeigt die
Sprache in Englisch, wenn die Datei fehlt. Während das WebAssembly geladen wird, zeigt
die Seite den Fortschritt (`loading.js`).

Der Workflow *Docker image of the web editor* (`.github/workflows/docker-web.yml`)
baut dieses Image allein neu, ohne den Rest von CI
([oben](#docker-images-der-server)).

### Website

Die Website des Projekts mit ihren Live-Demos, [openecraft.org](https://openecraft.org),
ist ein eigener Web-Build (`website/`),
der von Hand als statische Dateien oder mit dem Docker-Image
`ghcr.io/nilres/openecraft-website` bereitgestellt wird, das den Web-Editor unter
`/editor/` mitbringt: siehe [website.md](../website.md).

## Tests

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo clippy -p openecraft -p openecraft-parts-manager -p openecraft-website --target wasm32-unknown-unknown
```

Alle drei müssen ohne Warnungen durchlaufen (CI führt sie mit `-D warnings` aus).
