# Kommandozeile, MCP-Server und Plugins

Projekte und der Artikelserver ohne Fenster, für Skripte und KI-Agenten.

`openecraft-cli` bearbeitet Projekte und pflegt den Artikelserver ohne Fenster. Dieselben
Befehle werden KI-Agenten über das [Model Context Protocol](https://modelcontextprotocol.io)
angeboten (`openecraft-cli mcp`, JSON-RPC über stdin / stdout), sodass ein Agent Projekte
anlegen, Symbole platzieren, Leiter zwischen Anschlusspunkten zeichnen, Artikel zuweisen,
die Prüfung ausführen, Berichte erzeugen und PDF / SVG / DXF / DWG exportieren kann.

```bash
cargo run --release -p openecraft-cli -- help                 # all commands
cargo run --release -p openecraft-cli -- help connect         # parameters of a command
cargo run --release -p openecraft-cli -- --version             # also -V or version
openecraft-cli new_project --name "Pump control" --path pump.oec
openecraft-cli new_project --example star_delta_starter --path star_delta.oec   # see list_examples
openecraft-cli -p pump.oec place_symbol --page 1 --symbol pb_no --x 200 --y 100   # -S1
openecraft-cli -p pump.oec place_symbol --page 1 --symbol coil --x 200 --y 160    # -K1
openecraft-cli -p pump.oec place_part --page 1 --part GEN.LMP-GN-24 --x 240 --y 160   # -P1 with its part
openecraft-cli -p pump.oec connect --page 1 --from -S1:14 --to -K1:A1
openecraft-cli -p pump.oec check
openecraft-cli -p pump.oec export --format pdf --path pump.pdf
openecraft-cli --server http://127.0.0.1:8090 --token <secret> parts_import --path parts.csv
openecraft-cli --server http://127.0.0.1:8090 --token <secret> parts_import --path 6ED1052-1CC08-0BA3.edz
openecraft-cli --project drawing.oec import_eplan_symbols --path drive.edz
openecraft-cli --project drawing.oec import_eplan_symbols --path Beckhoff --part BEC.EK1100
```

- Auf der Kommandozeile wird jede Änderung sofort nach `--project` zurückgeschrieben. In
  einer MCP-Sitzung bleibt das Projekt im Speicher geöffnet: `save_project` schreibt es,
  `undo` / `redo` nehmen einzelne Werkzeugaufrufe zurück (`help mcp`); die Kommandozeile
  lehnt `undo` / `redo` ab, da sie das Projekt für jeden Befehl neu öffnet. Ein als
  `--project` angegebenes Archiv bleibt unverändert: Speichern Sie das Projekt als
  Projektdatei (`save_project --path <file>.oec`) oder wandeln Sie das Archiv um
  (`convert_project`).
- **Projektvorlagen**: `list_project_templates` listet die mitgelieferten Vorlagen und
  die des Vorlagenordners (`--templates <Ordner>`, `OPENECRAFT_TEMPLATES`, sonst
  `OpenECraft/data/templates` im Datenordner des Benutzers, den auch der Editor nutzt).
  `new_project --template "Motor starter direct on line" --values '{"POWER_KW": 7.5}'`
  legt ein Projekt mit den Werten seiner Parameter an, `template_values` zeigt, was die
  Formeln berechnen, ohne eines anzulegen, `save_project_template` speichert das offene
  Projekt in den Ordner und `delete_project_template` entfernt eine Vorlage dort; eine
  Vorlage des Ordners mit dem Namen einer mitgelieferten tritt an deren Stelle.
  `new_project --edit` öffnet eine Vorlage selbst, `get_template`,
  `set_template_parameter`, `set_template_formula` und `set_page --condition` ändern sie
  ([Projektvorlagen](features/schematic.md#projektvorlagen)).
- Ein **geteiltes Projekt** hält das Projekt als viele kleine Dateien für die
  Versionsverwaltung (git): den Index `<name>.oecp` mit einem Ordner `pages` (eine Datei
  je Seite), `data` (eine Datei je Betriebsmittel, Artikel, Kabel … und je Bereich der
  projektweiten Daten; die Dateien einer Art sind auf 16 Ordner verteilt, damit keiner
  zu groß wird) und `files` (eingebettete Dokumente und Bilder als gewöhnliche Dateien)
  daneben. Es enthält dasselbe Projekt wie die `.oec`-Datei:
  `convert_project --path plant.oec --format oecp` schreibt `plant/plant.oecp`,
  `--format oec` oder `--format oecz` wandeln zurück, und
  `--project plant/plant.oecp` arbeitet darauf wie auf einer
  Projektdatei – eine Änderung schreibt nur die Dateien neu, die sie betrifft. Jede
  Datei ist eine Zeile JSON und hängt nur vom Inhalt des Projekts ab, sodass das
  Speichern eines unveränderten Projekts nichts ändert. Ein geteiltes Projekt braucht
  einen eigenen Ordner; andere Dateien darin (eine README, `.gitattributes`) bleiben
  unberührt. Ein Ordner, der keinen Index hat, aber schon JSON-Dateien in `pages`,
  `data` oder `files` enthält, wird abgelehnt: Sie gehören jemand anderem und würden
  ersetzt oder entfernt.
- Das Speichern eines großen geteilten Projekts kostet, was sich geändert hat, nicht
  was das Projekt enthält: Nur die Seiten und Daten, die sich von der letzten
  Speicherung unterscheiden, werden geschrieben (ein Projekt mit 500 Seiten und 20 000
  Betriebsmitteln speichert eine geänderte Seite in deutlich unter einer Sekunde). Eine
  Speicherung ist ein Schritt: Die geänderten Dateien werden neben die alten
  geschrieben, ein Journal (`.oecp-save`) nennt sie, und erst dann nehmen sie ihre
  Plätze ein; nach einem Absturz ist das Projekt so, wie es vor der Speicherung war,
  oder der nächste Zugriff schließt sie ab.
- **Git** braucht für ein geteiltes Projekt zwei Dinge. Die `.gitattributes`, die das
  Speichern in den Ordner schreibt (sofern es keine gibt), hält git davon ab,
  Zeilenenden umzuwandeln, was jede Datei ändern und die eingebetteten beschädigen
  würde; eine damit geschriebene `.gitignore` lässt das Journal und die temporären
  Dateien einer laufenden Speicherung aus. Und der Merge-Treiber führt die JSON-Dateien
  Datensatz für Datensatz zusammen, sodass Änderungen an verschiedenen Symbolen einer
  Seite oder an verschiedenen Feldern eines Symbols keinen Konflikt ergeben; richten
  Sie ihn einmal je Klon ein:

  ```bash
  git config merge.openecraft.name "OpenECraft split project"
  git config merge.openecraft.driver "openecraft-cli merge_project_file --base %O --ours %A --theirs %B --name %P"
  ```

  Wo beide Seiten denselben Wert geändert haben, behält die Datei den Wert des aktuellen
  Zweigs, der Befehl nennt die Stellen und git markiert die Datei als konfliktbehaftet:
  Öffnen Sie das Projekt, stellen Sie ein, was richtig ist, und führen Sie `git add` für
  die Datei aus. Ohne den Treiber führt git die Dateien als Text zusammen und schreibt
  beide Fassungen hinein; eine solche Datei wird genannt, wenn das Projekt geöffnet
  wird: Nehmen Sie eine Fassung (`git checkout --ours` oder `--theirs`) und richten Sie
  den Treiber ein.
- **IDs neuer Objekte.** Jede Kopie eines geteilten Projekts beginnt die IDs ihrer neuen
  Objekte an einer zufälligen Stelle oberhalb der verwendeten IDs, damit zwei Personen,
  die auf ihren Zweigen Objekte hinzufügen, ihnen nicht dieselben IDs geben (IDs wachsen
  daher in Sprüngen, um bis zu 67 Millionen je Bearbeitungssitzung, die Objekte
  hinzufügt). Dies ist eine Absicherung mit Grenzen, keine Garantie:
  - Die Wahrscheinlichkeit von etwa 1 zu 30 000 für zwei gleichzeitig bearbeitete Kopien
    gilt, solange jede weniger als 1 024 Objekte hinzufügt. Eine Sitzung, die ein großes
    Makro importiert oder viele Seiten kopiert, überspannt viele der Stellen, an denen
    IDs beginnen: Zwei Kopien, die je etwa 50 000 Objekte hinzufügen, treffen sich
    ungefähr 1 Mal in 650.
  - `openecraft-cli -p plant/plant.oecp check_project_data` meldet mehrfach verwendete
    IDs und `--repair` nummeriert sie neu: Führen Sie es nach einem Merge aus. Ein
    Verweis auf eine solche ID (eine Verbindung zu einem Objekt) kann nicht erkennen,
    welches der beiden Objekte gemeint war, und kann am falschen landen; sehen Sie sich
    daher an, was die Reparatur nennt.
  - Nur die Sitzung, die ein Projekt bearbeitet, beginnt ihre IDs neu; das Lesen (der
    Vergleich, eine Vorlage, die Projektdatenbank) tut es nicht, und ein aus einer
    Vorlage erstelltes Projekt zählt weiter.
- **Änderungen eines anderen Programms, während das Projekt geöffnet ist**
  (ein `git pull`, ein zweiter Editor): Das Speichern schreibt nur die Dateien, die diese Seite
  geändert hat, und behält die anderen; der Editor und eine MCP-Sitzung lesen das
  Projekt dann erneut, sodass es beides zeigt. Eine auf beiden Seiten geänderte Datei
  wird nicht überschrieben: Die Speicherung schlägt fehl und nennt die Dateien.
  Speichern Sie unter einem anderen Namen, oder öffnen Sie das Projekt erneut und
  wiederholen Sie die Änderungen. Der Editor behält die Rückgängig-Schritte (Rückgängig
  nimmt nicht zurück, was das andere Programm geändert hat) und merkt sich die Dateien
  zwischen den Läufen, sodass ein Pull, der gemacht wurde, während er geschlossen war,
  ebenso behandelt wird.
- Parameter sind `--name value` (auch `--name=value`, oder alle auf einmal mit
  `--json '{…}'`); Listen als `--ids 3,4` und Punkte als `--points "0,0 40,0"`; Listen von Codes,
  Kennzeichen oder Sprachen werden durch Kommas oder Leerzeichen getrennt. Werte
  außerhalb ihres Bereichs werden mit dem zulässigen Bereich abgelehnt, nicht begrenzt,
  und ein Befehl, der fehlschlägt, lässt das Projekt unverändert. Zahlen in Antworten
  werden auf die Genauigkeit gerundet, mit der sie gespeichert sind.
- Seit früheren Versionen umbenannte Parameter werden weiterhin unter ihren alten Namen
  angenommen: `rename_device` `old` / `new` (jetzt `tag` / `new_tag`), `rename` von
  `set_potential`, `set_fluid_group` und `set_pct_loop` (jetzt `new_name`),
  `replace_text` `text` / `with` (jetzt `find` / `replace`).
- Artikelserver: `--server` / `--token`, die Umgebungsvariablen `OPENECRAFT_SERVER` /
  `OPENECRAFT_TOKEN` oder das Werkzeug `parts_connect` (auch mit `user` / `password`);
  Standard ist der öffentliche Artikelserver `https://parts-server.openecraft.org`
  ([öffentlicher Artikelserver](parts-server.md#öffentlicher-artikelserver)).
  `parts_connect` mit `kind: Eplan` liest stattdessen das EPLAN Data Portal (das Token
  ist das persönliche Zugriffstoken; nur Suche, `parts_get`, `parts_file` und
  `parts_check_updates`). Die Werkzeuge des Data Portals suchen mit Facetten
  (`parts_search`), verwalten Kataloge, Dateien, Favoriten und Benutzer
  (`parts_catalog_save`, `parts_attach`, `parts_favorites`, `parts_user_save`, …),
  vergleichen die Projektartikel mit dem Server (`parts_check_updates`) und teilen die
  Formulare der Firma (`parts_company_forms` (`publish`)).
- Hilfe und Meldungen folgen `--lang` oder der Systemsprache; der MCP-Server antwortet
  auf Englisch, sofern `--lang` nicht angegeben ist.

- `OPENECRAFT_MCP_ROOT` ist der Ordner, in dem die Dateien der MCP-Werkzeuge liegen
  müssen (`path`, `output`, `folder`, `database`, `target` und Vorlagendateien);
  Standard ist der Arbeitsordner des Servers, `*` erlaubt jeden Ordner. Ein Agent, der
  durch gelesenen Text (ein Projekt, ein Artikel, eine Webseite) irregeführt wurde, kann
  anderswo keine Dateien lesen oder überschreiben, etwa andere Projekte oder
  Plugin-Ordner. Relative Pfade liegen in `OPENECRAFT_MCP_ROOT`, auch wenn der Client
  den Server in einem eigenen Ordner startet (Claude Desktop); die Anweisungen des
  Servers nennen dem Agenten den Ordner.
- `OPENECRAFT_MCP_TOOLS` wählt die Werkzeuge, die der MCP-Server anbietet. Standardmäßig
  bietet er die Kernwerkzeuge für Schaltpläne (`core`, etwa 50: Projekte, Seiten,
  Symbole, Leiter, Kabel, Artikel, Prüfung, Berichte, Exporte; etwa 30 kB) und erreicht
  die anderen (alle zusammen beschreiben sich in etwa 260 kB) über drei eigene
  Werkzeuge: `find_tools` (Namen, oder die Werkzeuge, deren Name oder Beschreibung die
  Wörter einer Anfrage enthält), `describe_tool` (Beschreibung und Parameter) und
  `call_tool` (ruft ein beliebiges Werkzeug über seinen Namen auf). Durch Kommas
  getrennte Namen wählen andere Werkzeuge, `parts_*` alle Werkzeuge, die mit `parts_`
  beginnen, `*` alle (z. B. `core,panel*,parts_*`).
- Jedes Werkzeug des MCP-Servers hat einen Titel (seinen Namen in Wörtern) und
  Annotationen, nach denen ein Client entscheidet, ob er seinen Benutzer vor einem
  Aufruf fragt: `readOnlyHint` für ein Werkzeug, das nichts ändert (`list_…`, `get_…`,
  `check`, `parts_search` und andere), `destructiveHint` für eines, das ändern, ersetzen
  oder entfernen kann, was da ist (ein Werkzeug, das eine Datei schreiben kann, gehört
  dazu), `openWorldHint` für eines, das den Artikelserver, Autodesk Vault oder das Programm
  eines externen Plugins erreicht.
- Ohne Artikelserver verwenden `parts_search` und `parts_get` die Artikel des Projekts
  und den eingebauten Katalog (`parts_search --local` immer), und `assign_part` nennt
  ähnliche Artikel für eine Nummer, die es nicht kennt.
- `new_project --path` ersetzt keine vorhandene Datei, sofern nicht `overwrite`
  angegeben ist. `search_projects` durchsucht den aktuellen Ordner, sofern keine
  Datenbank und kein Ordner angegeben ist.

Der MCP-Server ist `openecraft-cli` der Pakete: unter Windows die Setup-Komponente
*Kommandozeile und MCP-Server* (Installationsart *Vollständige Installation*, nicht Teil
von *Editor*), in `%LOCALAPPDATA%\Programs\OpenECraft\` (oder
`C:\Program Files\OpenECraft\` für alle Benutzer); unter macOS
`/Applications/OpenECraft.app/Contents/MacOS/openecraft-cli`; unter Linux
`/usr/bin/openecraft-cli` (deb, rpm), `~/.local/bin` (tar.gz), `<AppImage> cli`,
`flatpak run --command=openecraft-cli io.github.nilres.OpenECraft` oder `openecraft.cli`
(snap). Registrieren Sie ihn bei einem Agenten, z. B. Claude Code (gestartet im Ordner
der Projekte):

```bash
claude mcp add openecraft -- /path/to/openecraft-cli mcp
claude mcp add --scope user -e OPENECRAFT_MCP_ROOT=/home/me/projects openecraft -- openecraft-cli mcp
```

oder in der `mcpServers`-Konfiguration anderer Clients (Claude Desktop:
`claude_desktop_config.json` in `%APPDATA%\Claude\`, `~/Library/Application Support/Claude/`
oder `~/.config/Claude/`; Cursor: `~/.cursor/mcp.json`; VS Code: `.vscode/mcp.json` mit
`servers` statt `mcpServers`):

```json
{ "mcpServers": { "openecraft": { "command": "openecraft-cli", "args": ["mcp"],
    "env": { "OPENECRAFT_MCP_ROOT": "/home/me/projects" } } } }
```

Ohne Installation gibt es den Server als MCP-Bundle des Releases
(`OpenECraft-MCP-<version>-windows-x86_64.mcpb`, `-linux-x86_64.mcpb`,
`-macos-universal.mcpb`): Claude Desktop und andere Clients, die das Format kennen,
installieren es per Doppelklick und fragen nach dem Projektordner
(`OPENECRAFT_MCP_ROOT`) und den Werkzeugen (`OPENECRAFT_MCP_TOOLS`). Jedes Release trägt
diese Bundles als `io.github.nilres/openecraft` in die offizielle MCP Registry ein.

Der Agent kann jede Datei lesen und schreiben, auf die der Serverprozess zugreifen kann;
starten Sie ihn in einem Arbeitsverzeichnis (und mit einem Artikelserver-Token), dessen
Nutzung durch den Agenten für Sie in Ordnung ist.
[Lassen Sie einen KI-Agenten Ihren Schaltplan zeichnen](../launch/ai/README.md) enthält
ein ausgearbeitetes Beispiel mit seinem Protokoll, Beispiel-Prompts und der Einrichtung
weiterer Clients.

## Plugins

Plugins fügen dem Editor (Registerkarte *Plugins*: *Plugin-Befehle* listet jeden Befehl
nach Plugin mit einer Suche, der Stern heftet einen an die Registerkarte; Registerkarte
*Prüfung*), der Kommandozeile und dem MCP-Server gleichzeitig Befehle, Prüfungen,
Berichte und Exporte hinzu. Eingebaute Plugins sind Rust-Code in
`crates/openecraft-api`; externe Plugins sind Programme in beliebiger Sprache, die
JSON-Anfragen über stdin / stdout beantworten:

```bash
OPENECRAFT_PLUGINS=examples/plugins openecraft-cli -p pump.oec plugins
openecraft-cli --plugins examples/plugins -p pump.oec report --kind page_notes
```

Siehe [docs/plugins.md](plugins.md) für die API und das Protokoll.
