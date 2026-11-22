# Plugins

OpenECraft wird über das Crate `openecraft-api` erweitert. Ein Plugin fügt hinzu:

| Erweiterung | Was sie ist | Editor | Kommandozeile / MCP |
| --- | --- | --- | --- |
| Befehl | Funktion auf dem Projekt mit typisierten Parametern, liefert JSON | Bereich *Plugin-Befehle* (Registerkarte *Plugins*: jeder Befehl nach Plugin mit einer Suche; Favoriten, zuletzt verwendete Befehle und ein Menü je externem Plugin auf der Registerkarte; ein Formular für die Parameter) und die Befehlssuche | ein eigener Befehl / ein eigenes Werkzeug |
| Prüfung | fügt der Projektprüfung Meldungen hinzu | Bereich *Meldungen* | `check` |
| Bericht | aus dem Projekt erzeugte Tabelle | Bereich *Berichte*, Berichtsseiten und Auswertungsblöcke | `report --kind <name>` |
| Exporter | aus dem Projekt erzeugte Datei | *Datei › Export* | `export --format <name>` |

Namen sind snake_case und unter allen Plugins eindeutig; Befehle dürfen nicht den Namen
eines eingebauten Werkzeugs tragen. Änderungen, die ein Befehl am Projekt vornimmt, sind
ein Rückgängig-Schritt. `openecraft-cli plugins` listet die geladenen Plugins,
*Werkzeuge › Plugins verwalten* im Editor aktiviert und deaktiviert sie (vom Editor
gemerkt).

Geben Sie in *Plugins › Plugins verwalten* einen Plugin-Ordner ein, der `plugin.json`
enthält, oder einen übergeordneten Ordner, dessen Unterordner Plugins enthalten, und
klicken Sie auf *Plugins laden*. Der Editor lädt sie für die aktuelle Sitzung und merkt
sich brauchbare Ordner für spätere Starts. Ein fehlgeschlagenes Nachbar-Plugin
verhindert nicht, dass geladene Plugins gemerkt werden. `OPENECRAFT_PLUGINS` bleibt für
die Konfiguration über die Umgebung verfügbar.

## Rust-Plugins

Eingebaute Plugins liegen in `crates/openecraft-api/src/builtin.rs` und seinen
Untermodulen und werden in jedes Frontend kompiliert (auch in die Web-Version): die
Projektprüfungen, Berichte und Exporte von `openecraft-core`, Projektstatistik,
Schaltschrankaufbau-Prüfungen, Schaltschrankfertigung, Artikel-Lebenszyklus und ein
Plugin je Bereich von Projektbefehlen (Seiten, Blätter, Zeichnung, Geräteboxen,
Betriebsmittel, Stecker, SPS, Potenziale, Verfahrenstechnik, Übersichtsschaltpläne,
Tabellen, Engineering, Elektrotechnik, Schaltschrankaufbau, Räume, Gebäude,
19-Zoll-Racks, Verlegung, Feldverdrahtung, Verdrahtung, Kabelbäume, Integration,
Sprachen, Struktur, Eigenschaften, Konfigurator, Auswertungsblöcke, Projektverwaltung).
Ein Bereich listet seine Befehle in einer Tabelle `COMMANDS` von `Spec`s
(`builtin/<area>.rs`); die Kommandozeile und der MCP-Server bieten sie als Werkzeuge
neben den Sitzungswerkzeugen von `cli/` an. Die Projektprüfungen selbst sind
Registry-Einträge von `openecraft_core::checks` (ein `Check` je Bereich, mit einem
snake_case-Namen), und die Arten, Parameter und Tabellen der Registry sind in
`openecraft_core::{ext, params, table}` definiert. Ein Plugin implementiert `Plugin`:

```rust
use openecraft_api::{param, Info, Kind, Plugin, Registry, Table, Text};
use openecraft_core::issue::{Issue, Severity};
use openecraft_i18n::tr_const;

pub struct Spares;

impl Plugin for Spares {
    fn info(&self) -> Info {
        Info { id: "spares".into(), name: Text::Borrowed(tr_const!("Spare parts")), version: "1.0".into(), description: Text::Borrowed("") }
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        reg.command("set_spares", tr_const!("Sets the spare quantity of a part"), vec![param("number", Kind::Str, tr_const!("Part number"))], |prj, args| {
            // change `prj`, return JSON
            Ok(serde_json::json!({ "part": args.req("number") }))
        })
        .check("spares_missing", tr_const!("Spare parts"), |prj, analysis| Ok(vec![Issue::plain(Severity::Info, "nothing missing", None)]))
        .report("spares", tr_const!("Spare parts"), |prj, analysis| Ok(Table::new([tr_const!("Part")])))
        .keywords(["IEC 60300-3-4", "Spares"])
        .exporter("spares_csv", tr_const!("Spare parts (CSV)"), "csv", |prj| Ok(String::new()));
        Ok(())
    }
}
```

`keywords` sind weitere Suchbegriffe des zuletzt hinzugefügten Berichts, nicht
übersetzt: die Normen, denen er folgt, und andere Namen. Der Bereich *Berichte*,
`list_reports` (`search`) und die Website finden einen Bericht über jedes Wort der Suche
in seinem Titel wie gezeigt, seinem Namen oder einem Schlüsselwort, ohne Rücksicht auf
Groß- und Kleinschreibung, Leerzeichen und Satzzeichen (`Report::matches`), sodass
`VDE 0100 600` das Prüfprotokoll über `DIN VDE 0100-600` findet.

Als `&'static str` angegebene Texte sind englische Katalogeinträge (`tr_const!`, siehe
AGENTS.md) und werden übersetzt gezeigt. Meldungen einer Prüfung tragen entweder einen
fertigen Text (`Issue::plain`) oder, wie die eingebauten Prüfungen, einen Katalogeintrag
mit seinen Argumenten
(`openecraft_core::issue!(Severity::Warning, target, "Cable {} defined {} times", tag, n)`),
den `Issue::message()` in der aktuellen Sprache ausgibt; das Werkzeug `check` meldet die
Vorlage als `key` neben der Meldung.

## Externe Plugins

Ein externes Plugin ist ein Programm in beliebiger Sprache in einem eigenen Ordner:

```
my_plugin/
  plugin.json
  main.py
```

```json
{
  "id": "my_plugin",
  "name": "My plugin",
  "version": "1.0",
  "description": "What it does",
  "api": 1,
  "run": ["python3", "main.py"]
}
```

Die Ordner, die Plugin-Ordner enthalten, werden in der Umgebungsvariable
`OPENECRAFT_PLUGINS` (getrennt wie `PATH`) für den Editor und die Kommandozeile
angegeben, oder mit `openecraft-cli --plugins <folder>`. Externe Plugins brauchen
Prozesse und sind im Browser nicht verfügbar.

### Protokoll

Für jede Anfrage startet OpenECraft das Programm (`run`, im Plugin-Ordner), schreibt ein
JSON-Objekt auf dessen stdin und liest ein JSON-Objekt von dessen stdout, beide in UTF-8
(Python wird mit `PYTHONUTF8=1` und `PYTHONIOENCODING=utf-8` gestartet; wo `python3`
nicht installiert ist, wie unter Windows, wird stattdessen `python` gestartet). Jede
Anfrage hat `"api": 1` und eine `method`. Eine Antwort `{"error": "message"}` (oder ein
Exit-Code ungleich null, mit der Meldung auf stderr) ist ein Fehler. `project` ist das
Projekt wie in einer `.oec`-Datei. Ein Programm, das nach 5 Minuten nicht geantwortet
hat, wird beendet und die Anfrage schlägt fehl. Plugins laufen mit den Rechten des
Benutzers: Laden Sie nur Plugin-Ordner, denen Sie vertrauen.

Anfragen enthalten außerdem `project_features`, ein Array, das die Modellunterstützung
des laufenden Hosts bekannt gibt (`PROJECT_FEATURES` in `openecraft-api`); jedes Merkmal
ist mit den Daten beschrieben, die es betrifft, zum Beispiel `directional_pin_contacts`,
`explicit_wire_junctions`, `directed_junction_paths`, `fluid_port_roles`,
`wire_hide_labels` und `scoped_connections`. Importer können Merkmale verlangen, bevor
sie ein geändertes Projekt zurückgeben; die Verwendung einer getrennten aktualisierten
CLI gibt einem älteren Editor diese Merkmale nicht.

| Anfrage | Antwort |
| --- | --- |
| `{"method": "describe"}` | `{"commands": [{"name", "help", "params": [{"name", "kind", "required", "help"}]}], "checks": [{"name", "title"}], "reports": [{"name", "title", "keywords": ["…"]}], "exporters": [{"name", "title", "extension", "params"}]}` (alle Listen optional, `params` von Exportern und `keywords` von Berichten ebenfalls) |
| `{"method": "command", "name", "params": {…}, "project": {…}}` | `{"result": <any JSON>, "project": {…}}` – `project` nur, wenn der Befehl es geändert hat |
| `{"method": "check", "name", "project"}` | `{"issues": [{"severity": "error" \| "warning" \| "info", "message", "page": <page index>, "item": <item id>}]}` – `page` / `item` optional; `page` allein zeigt auf die Seite, eine Seite, die das Projekt nicht hat, auf nichts |
| `{"method": "report", "name", "project"}` | `{"headers": ["…"], "rows": [["…"]]}` |
| `{"method": "export", "name", "params": {…}, "project"}` | `{"content": "file content"}` (Text; `params` enthält, was das Frontend übergeben hat: nach Konvention `page`, der Name einer Seite, und `file`, der Dateiname) |

Parameterarten: `str`, `int`, `num`, `bool`, `ids` (Liste von Element-IDs), `points`
(Liste von `[x, y]` in mm), `object` (JSON-Objekt). `required` ist standardmäßig `true`.

Ein Befehl, der mit einem Fehler antwortet, lässt das Projekt unverändert, auch im
Editor und im MCP-Server.

`describe` wird einmal gefragt, wenn das Plugin geladen wird; Prüfungen laufen im Editor
nach jeder Änderung des Projekts, sollten also schnell antworten. Die
Prüfeinstellungen des Projekts gelten auch für die Meldungen von Plugins: Eine Prüfung
kann über ihren Namen abgeschaltet werden, und ihre einfachen Meldungen teilen sich den
Namen der Prüfung als Code (Änderungen der Stufe gelten dann für alle) und können
einzeln akzeptiert werden.

Artikel können `symbol` auf den Schlüssel einer Symboldefinition des Projekts setzen.
Die gemeinsame Logik zum Platzieren von Artikeln verwendet diese Definition in Editor,
CLI und MCP. Ein leerer Schlüssel behält die bisherige vorlagenbasierte Auswahl.
`source_data` an Artikeln und Symboldefinitionen kann Kennungen des Importers und
Herstellereigenschaften als JSON aufbewahren. `Project.import_sources` ordnet stabilen
Quell-IDs JSON-Datensätze zu, einschließlich Datensätzen, die keine eigene
Bearbeitungsoberfläche haben. Diese Felder sind für ältere Dateien optional.

Portable native Makro-Clips können `symbol_defs` tragen, ein Array von
Symboldefinitionen des Projekts. Das Einfügen eines Clips installiert fehlende
Definitionen und benennt kollidierende eingehende Schlüssel um, wobei lokale Änderungen
erhalten bleiben und die eingehenden Instanzen aktualisiert werden. Die optionale Map
`devices` und das Array `parts` tragen Betriebsmittelzuordnungen und Katalogdatensätze.
Das Einfügen benennt Betriebsmittelzuordnungen mit ihren Instanzen um und installiert
fehlende Artikel, ohne lokale Katalogdatensätze zu überschreiben. `Part.symbol` folgt
umbenannten Definitionen. Makrovorschauen lösen mitgelieferte Definitionen auf, bevor
sie das Zielprojekt befragen.
Der
Befehl `insert_macro` nimmt ein natives Makro als JSON-Zeichenkette sowie `page`, `x`
und `y` an. Editor, CLI und MCP stellen denselben Befehl bereit. Große
CLI-Parameterobjekte können als `--json @parameters.json` übergeben werden, um die
Grenzen für Shell-Argumente zu umgehen.
Anschlüsse von Benutzersymbolen können `internal: true` für Anschlussknoten von Brücken
setzen. Sie nehmen weiterhin an lokalen Brücken teil, erscheinen aber nicht als
physische Endpunkte des Betriebsmittels. Ein Leiter, der durch einen internen Anschluss
läuft, verbindet sich mit ihm; andere Anschlüsse brauchen dort ein Leiterende.
`list_nets` gibt die native Zugehörigkeit nach Symbol-ID und nullbasiertem
Anschlussindex aus; sein Array `pins` enthält physische Endpunkte von Betriebsmitteln.
Das zusätzliche Array `connection_points` enthält Verbindungsknoten und interne
Anschlusspunkte von Brücken, die zu einem verdrahteten oder physischen Netz gehören,
ohne Endpunkte von Betriebsmitteln oder leere Netze hinzuzufügen. Importer können
Pfade, die an diesen Punkten enden, über denselben Befehl in Editor, CLI und MCP
prüfen.
`bridge_terminals` ist standardmäßig true und kann für einzelne Leiterzüge abgeschaltet
werden.
`list_connections` liefert physische Leiterziele von Anschluss zu Anschluss als Objekte
`from` und `to`, die eine Symbol-ID und einen nullbasierten Anschlussindex enthalten,
zusammen mit Potenzial, Leiternummer, Farbe, Querschnitt, Kabel und Ader. Interne
Brücken von Betriebsmitteln erhalten die elektrische Durchgängigkeit, ohne externe
Fertigungsleiter zu erzeugen.
Benutzersymbole für Verbindungsknoten können `directed_junction: true` setzen: Ihre
`bridges` beschreiben dann auch physische Zielpfade zwischen auseinanderliegenden
Anschlüssen. `set_symbol_junction_paths` nimmt einen Symbolschlüssel und
`paths: {"links": [[0, 1], [0, 2]], "directed": true}` an; der Symboleditor stellt
dieselben Pfade bereit. Die Seiteneigenschaft `page.use_drawn_wire_targets` erhält
explizite Abzweige an Anschlusspunkten von Betriebsmitteln und verfolgt Leiter durch die
internen Brücken von Betriebsmitteln. Auf solchen Seiten und auf Seiten mit expliziten
Verbindungsknoten zählen Anschlüsse eines Symbols an derselben Position als ein
Anschluss.
Benutzersymbole können `pin_directions` angeben, eine Map von nullbasierten
Anschlussindizes auf lokale Richtungsmasken (oben 1, rechts 2, unten 4, links 8).
Ausgelassene Einträge erlauben jede Seite; null schaltet automatische Seitenkontakte ab.
Richtungen folgen der Drehung und Spiegelung der Instanz. Gespeicherte Leiterendpunkte
verbinden immer. `set_pin_connection_directions` nimmt `symbol`, `pin` und `directions`
an (`"up,right"`, `"all"` oder `"none"`); der Symboleditor stellt dieselben Richtungen
bereit. Neue automatische Leiter verwenden diese Richtungen.
Die Seiteneigenschaft `page.explicit_wire_junctions` macht Kreuzungen von Endpunkt zu
Leiterinnerem nichtleitend, sofern kein Verbindungsknoten sie verbindet, und Anschlüsse
von Betriebsmitteln verbinden nur mit gezeichneten Leiterendpunkten. Interne
Schienenanschlüsse und Anschlüsse von Verbindungsknoten behalten Seitenkontakte.
Verbindungspunkte und Prüfungen offener Enden verwenden dieselben Regeln.
`device_box_roles` fügt Geräteboxen `SymbolInst.box_role` hinzu: `extension` fügt dem
Betriebsmittel des Kennzeichens der Box Anschlusspunkte hinzu, `view` zeigt Anschlusspunkte, die es schon hat, und verbindet sie über
Seiten hinweg nach Betriebsmittel und Bezeichnung; der Standard `device` wird in der Datei ausgelassen.
Eine Instanz eines Unterbrechungspunkts kann `function.connection_scope` in ihren
`values` speichern. Punkte verbinden und querverweisen nur andere mit demselben
Geltungsbereich und derselben Beschriftung; ein leerer Geltungsbereich behält das
bisherige projektweite Verhalten. Die Beschriftung bleibt der gezeigte Name. Diese
Eigenschaft ist über die Eigenschaftsdatenbank im Editor und die gemeinsamen
Eigenschaftsbefehle von CLI/MCP bearbeitbar.
Benutzersymbole können `fluid_ports` angeben, eine Map von nullbasierten
Anschlussindizes auf `supply`, `pressure`, `working`, `exhaust`, `return`, `pilot`,
`suction`, `leakage`, `measuring` oder `passage`. Ein ausgelassener Eintrag behält die
Ableitung aus der Bezeichnung. Diese Rollen überstehen Bibliothekskopien, portable
Makros und die Serialisierung des Projekts; `list_symbols` gibt sie aus und
`set_fluid_port_role` bearbeitet oder leert einen Eintrag (`role: "automatic"`) über den
Bereich Plugins des Editors, CLI und MCP. Der Symboleditor bietet die Rolle auch am
gewählten Fluidanschlusspunkt an.
`FieldKind::TagName` zeigt den Betriebsmittelnamen ohne geerbte Struktur und bleibt
dabei mit dem Kennzeichen der Instanz verknüpft. Plotrahmen können `visible: false`
setzen, wenn importierte Grafik ihren eigenen Rand und ihr eigenes Schriftfeld enthält;
die Papiergeometrie bleibt verfügbar. `ZeroBasedNumbers` wird für Spaltenbeschriftungen
von Quellzeichnungen unterstützt. PNG-Zeichnungsbilder, einschließlich Palettenbildern
und Transparenz, werden mit einer weichen Bildmaske nach PDF exportiert.
`Func::Graphic` hält dekorative Grafik aus der Betriebsmittelkennzeichnung und der
Netzlistenauswertung heraus und behält dabei bearbeitbare Symbolprimitive.

Symboldefinitionen des Projekts können ein Zeichnungsprimitiv `Image` enthalten. Es
trägt einen Datei-`name`, base64-kodierte `data` und vier `corners` in lokalen
Symbolkoordinaten, geordnet oben links, oben rechts, unten rechts, unten links.
Der Editor und die SVG-/PDF-Ausgabe wenden die Drehung und Spiegelung des Symbols auf
diese Ecken an. Bestehende Darstellungen von Zeichnungsprimitiven werden neben Bildern
unterstützt.

Benutzersymbole können eine `domain` von `Electrical`, `Fluid` oder `Process` angeben.
Ältere Symboldateien haben standardmäßig `Electrical`. Fluid- und Prozessanschlusspunkte
nehmen an Fluidnetzen teil, und ihre Verbindung mit einem elektrischen Anschluss wird
als gemischtes Netz gemeldet.

`Field`-Primitive nehmen einen optionalen `angle` in Grad an, mit derselben Konvention
wie Symboltext. Wird er ausgelassen, bleibt das bisherige Verhalten waagerechter
Beschriftungen. Ihr `kind` kann auch `{"Pin": 0}` sein (nullbasierter Anschlussindex),
das der Anschlussbezeichnung der platzierten Instanz folgt. Die gezeigte Bezeichnung
einer Klemme verwendet `Label`.

Die Funktion `Auxiliary` stellt eine zusätzliche Betriebsmittelfunktion mit elektrischen
Anschlüssen und einem Querverweis auf ihre Hauptfunktion dar, ohne einen Kontakttyp zu
erfinden. `Junction` hat kein Betriebsmittelkennzeichen und verbindet nur die in seinen
Brücken aufgeführten Pfade. Seine Anschlusspunkte sind lokale Routing-Knoten und sind
von den Endpunkten der Betriebsmittel in der Netzliste ausgenommen.

Leiter können `hide_labels: true` setzen, wenn ihre Leiternummern und
Kabel-/Aderbeschriftungen bereits als getrennte Anmerkungen gezeichnet sind. Elektrische
Eigenschaften und Berichte bleiben verfügbar. Das Feld der Leitereigenschaften und der
Befehl `update_wire` können die automatischen Beschriftungen wiederherstellen; neue
Leiter zeigen standardmäßig Beschriftungen.

Ein vollständiges Beispiel liegt in [`examples/plugins/page_notes`](../../examples/plugins/page_notes).

### CAD-Zeichnungskoordinaten

Die CLI/MCP-Werkzeuge `import_dxf` und `import_dwg` liefern `source_bounds` (Punkte
`min` und `max` in dekodierten Zeichnungsmillimetern, Y nach oben) und `page_origin`
(die Seitenposition der oberen linken Grenze der Quelle), neben `scale`. Für einen
importierten Seitenpunkt `p` sind seine Quellkoordinaten `source_bounds.min.x +
(p.x - page_origin.x) / scale` und `source_bounds.max.y -
(p.y - page_origin.y) / scale`. Dies schließt die Einheitenumrechnung der Zeichnung ein
und erlaubt externen Importern, ein gestaltetes Ansichtsfenster anzuwenden. Die
bestehende Seitenplatzierung und die Standardwerte der Parameter bleiben unverändert.

### EPLAN-Fenstermakros

Das Zeichnungs-Plugin nimmt PXF-Fenstermakro-XML mit `import_ema` an (Parameter `xml`). Sein Ergebnis listet native Symboldefinitionen, Funktionstypen, Varianten-IDs, Makros mit mehreren Betriebsmitteln und nicht aufgelöste Verweise. Ein erneuter Import erhält lokal bearbeitete Definitionen. Der Befehl `place_part` nimmt einen optionalen Symbolschlüssel `variant` an. Das Dateiwerkzeug `import_eplan_symbols` von CLI und MCP liest ein EMA, ein EDZ, ein begleitendes Artikel-XML oder einen Herstellerordner mit einer gewählten Artikelnummer.

`product_designation_fields` gibt die Art `{"Product":N}` von `Prim::Field` bekannt. Sie zeigt die letzten N Ebenen der Produktkennzeichnung des aktuellen Instanzkennzeichens, ohne Funktions- und Ortsaspekt; null verhält sich wie eins. Das Feld aktualisiert sich nach dem Umbenennen und übersteht die Serialisierung von Symbolbibliothek und Projekt. Der Symboleditor stellt die Anzahl der Ebenen bereit; Plugins verwenden dasselbe serialisierte Primitiv über Symboldefinitionen. Hosts ohne dieses Merkmal können diese Feldvarianten nicht laden.

`function_text_fields` gibt die Art `"FunctionText"` von `Prim::Field` und den optionalen `default_function_text` von Benutzersymbolen bekannt. Das gemeinsame Platzieren initialisiert den nativen Funktionstext der Instanz aus dem Standard der Bibliothek. Felder geben den aktuellen Wert der Instanz aus, einschließlich transformiertem mehrzeiligem Text; der automatische Kennzeichenblock lässt einen doppelten Funktionstext aus, wenn ein explizites Feld existiert. `list_symbols` liefert den Standard der Bibliothek.

`function_part_selection` gibt funktionsspezifische Verweise auf zugewiesene Artikel bekannt. Die Eigenschaft `function.part_index` wählt eine einsbasierte Position in den Artikeln des Betriebsmittels und hält deren Artikelnummer in `SymbolInst.values["function.part_number"]` fest. Das Entfernen früherer Artikel ändert diesen Verweis nicht. Ein entfernter gewählter Artikel ergibt eine leere Auswahl, Position null und eine Projektmeldung; das Setzen einer vorhandenen Position repariert dies, während das Leeren der Position die automatische Auswahl des ersten Artikels zurücksetzt. Eigenschaften `part.*` an einer Funktion lesen nur ihren gewählten Artikel; Eigenschaften des Betriebsmittels behalten ihr aggregiertes Verhalten. `function.manufacturer` unterstützt eine explizite bearbeitbare Überschreibung mit Rückfall auf den gewählten Artikel, und `function.selection_text` behält gestalteten Auswahltext unabhängig von den Typnummern des Katalogs. Der registrierte Befehl `list_function_parts` liefert den gewählten Artikel und die bearbeitbaren Attributwerte jeder Funktion zur Stapelprüfung.

`property_blocks` gibt `Prim::PropertyBlock` bekannt, mit geordneten Zeilen `Property(key)`, `Field(FieldKind)` und `Text(value)`, lokalen Vektoren `pos` und `step`, Schrift-`size`, waagerechtem `anchor`, Text-`angle`, `align` (`Start`, `Middle`, `End`) und optionaler RGB-`color`. Der gemeinsame Renderer von Zeichenfläche und Export löst Eigenschaften an der ursprünglichen Funktion mit vollem Kennzeichen auf, überspringt leere Werte und lässt jede Zeile fließen, bevor er Symboltransformationen anwendet. `default_values` von Benutzersymbolen initialisiert bearbeitbare Instanzeigenschaften beim gemeinsamen Platzieren; `list_symbols` gibt Standardwerte und Blöcke aus. Der Symboleditor unterstützt das Erstellen und Bearbeiten von Blöcken und Standardeigenschaften.

Das Projektmerkmal `wire_type` erhält den Leiteraufbau in `Wire.wire_type`.
`update_wire` nimmt `wire_type` an; `list_connections` liefert ihn für jede physische
Verbindung. Die bearbeitbare Verbindungstabelle hängt Leitertyp nach ihren
ursprünglichen neun Spalten an; Verbindungsberichte enthalten denselben Wert. Leer
bedeutet nicht angegeben.

Das Merkmal `physical_wire_routes` gibt die kontaktbewusste Leiterverfolgung bekannt.
`list_connections` enthält `route_traced` und `wires` (native Seiten-ID und Leiter-ID).
Wenn `route_traced` false ist, ist die Leiterliste der bisherige netzweite
Geltungsbereich der Eigenschaften, kein nachgewiesener Pfad von Punkt zu Punkt. Seiten
mit expliziten Verbindungsknoten halten leiterlokale Eckpunkte getrennt, bis ein
gültiger Endpunkt, Verbindungsknoten oder gerichteter Anschlusskontakt sie verbindet.
Diese Regeln gelten auch, wenn eine Kreuzung über einen anderen Weg zu einem Netz
gehört.

`trace_wire_routes` nimmt `routes: {"pairs": [{"from": {"symbol": 12, "pin": 0},
"to": {"symbol": 34, "pin": 0}}]}` an und liefert ein geordnetes Ergebnis für jedes
Paar. Es verfolgt exakte lokale gezeichnete Endpunkte, einschließlich virtueller
Unterbrechungspunkte, ohne zwischen gleichen Potenzial- oder Unterbrechungsnamen zu
springen. Jedes Ergebnis hat `route_traced` und `wires` mit nativen Seiten-/Segment-IDs
und ihren Leitereigenschaften. Dies prüft einen bestimmten Abzweig, auch wenn die
physische Verdrahtungsliste einen anderen gültigen Pfad durch ein benanntes Netz wählt.
Ein ungültiger oder nicht verbundener Endpunkt hat keinen verfolgten Weg.

### Vollständige SPS-Modulkanäle

Das Projektmerkmal `multi_channel_plc` gibt `UserSymbol.plc_channels` bekannt, das auch `list_symbols` ausgibt. Jede Kanaldefinition enthält einen stabilen `key`, einen nullbasierten richtungsbezogenen `index`, eine optionale Richtung `input`, `analog`, `data_type`, `designation`, nullbasierte physische `pins` und einen optionalen `signal_pin`. Bestehende einkanalige Metadaten `plc_channel` werden weiter unterstützt.

Instanzwerte von Kanälen verwenden die einsbasierten Eigenschaftsschlüssel `function.plc_address[1]`, `function.plc_symbol[1]`, `function.plc_comment[1]` und `function.plc_data_type[1]`. Bibliotheksdefinitionen sollten konfigurierte Adressen und Signalnamen leer lassen. Eigenschaftsblöcke können auf diese Schlüssel verweisen, um die aktuelle Konfiguration zu zeigen. `get_plc_points` liefert eine Zeile je Kanal mit seinen physischen Anschlussindizes; `set_plc_channel` bearbeitet einen Kanal über eine Symbol-ID und einen nullbasierten Kanalindex. Zeilenschlüssel der SPS-Tabelle verwenden `symbol_id:channel_index` für vollständige Module und behalten numerische Symbol-IDs für einzelne Punkte alter Art. Leere und richtungslose Kanäle sind von SPS-Variablenexporten ausgenommen.

### Verknüpfte Darstellungen von SPS-Kanälen

Das Merkmal `linked_plc_channels` fügt `SymbolInst.plc_bindings` hinzu, eine Map von nullbasierten Kanalindizes auf `{group, primary}`. Eine nicht leere Gruppe kennzeichnet denselben logischen Kanal über Darstellungen mit demselben vollen Betriebsmittelkennzeichen hinweg. Setzen Sie `primary` an der elektrischen Darstellung, deren physische Anschlüsse und Ort für die E/A-Navigation verwendet werden sollen. Verschiedene volle Betriebsmittelkennzeichen halten kopierte Betriebsmittel unabhängig.

Die SPS-Übersicht, Hardwarezählungen, Tabellen und Variablenexporte verwenden logische Kanäle. Kanalbearbeitungen, Bearbeitungen indizierter Kanaleigenschaften, die Neuadressierung von Karten und der Import von Symboltabellen aktualisieren alle verknüpften Ansichten; eine freigegebene Ansicht weist die ganze Kanalbearbeitung zurück. Bestehende gestaltete Werte bleiben an jeder Ansicht gespeichert. Leere gezeigte Felder können auf eine andere verknüpfte Ansicht zurückfallen. `get_plc_points` liefert standardmäßig logische E/A-Zeilen; `representations:true` liefert alle Schaltplandarstellungen mit ihren eigenen gespeicherten Werten, physischen Anschlussindizes und Bindungsmetadaten zur verlustfreien Importprüfung.

Das Merkmal `plc_channel_indices` fügt den optionalen `ChannelBinding.index` hinzu, die kartenbezogene Hardware-Kanalnummer. Speichern Sie die Hardwarenummer der Modulübersicht an jeder verknüpften Ansicht, auch wenn eine elektrische Ansicht nur einen lokalen Kanal enthält. `get_plc_points.hardware_index` gibt diese Nummer getrennt vom symbollokalen Index `channel` aus, den `set_plc_channel` verwendet. Die Neuadressierung und die logische Kanalbezeichnung verwenden die Hardwarenummer. Ältere Bindungen ohne explizite Nummer fallen auf die vollständigste verknüpfte Moduldefinition zurück, sodass fortgeführte Projekte nach dem Umordnen von Blättern nutzbar bleiben.

Artikel, die vollständige SPS-Module darstellen, verwenden ein `FunctionTemplate` je Kanal, mit `PlcInput`, `PlcOutput`, `PlcAnalogInput` oder `PlcAnalogOutput` und den physischen Anschlussbezeichnungen dieses Kanals. Lassen Sie wiederverwendbare Signalbeschreibungen leer. Die native Funktionsverwendung und Reserveberechnungen zählen verknüpfte Schaltplankanäle einmal und schließen Schaltschrankgrafik aus. `list_device_descriptions` enthält analoge Funktionskapazitäten; `list_unplaced` erkennt belegte Modulkanäle über die Hardware-Identität, unabhängig vom konfigurierten Adresstext.

`set_plc_card` erhält gestaltete Kanaladressen bei Änderungen, die nur Metadaten betreffen. Die Angabe von `start_byte`, `output_byte`, `readdress:true` oder eines Rockwell-`slot` baut die Adressen aus der Konfiguration neu auf. Eine importierte Kopfbeschreibung mit leeren Ports und ohne konfigurierten Bus behält eine leere Portliste; sie erhält keine abgeleitete PROFINET-Schnittstelle.

Die Umwandlung des Stationsschemas schließt digitale Kanäle vollständiger Module und ihre verknüpften Darstellungen ein. Sie behält jeden konfigurierten Adressversatz und Signalnamen. Eigener Adresstext, der keine unterstützte digitale Adresse ist, bleibt unverändert.

Das Merkmal `explicit_panel_mounts` unterstützt `EnclosureKind::Assembly`, eine offene Montagefläche ohne implizite Hüllen- oder Plattengeometrie, und `ItemKind::MountingPlate`, ein explizites Element mit gestalteter Position, Größe und Dicke. Explizite Plattenelemente unterdrücken auch die Standardplatte eines Schaltschranks. Koordinaten bleiben im gemeinsamen Bezugssystem des Gehäuses. `PanelItem.mounted_on` verweist auf ein anderes Element im selben Gehäuse; `mounting_explicit:true` unterscheidet ein explizit freies Element von der abgeleiteten Schienenplatzierung alter Art. `rail_mount_override:true` behält den Bezugspunkt der Instanz über das native Laden und die Artikelaktualisierung hinweg. `rail_vertical:true` misst den Bezugspunkt quer zur Schiene entlang x statt y.

Verwenden Sie `add_enclosure` / `set_enclosure` mit `assembly:true`, `place_mounting_plate` und `set_panel_item` mit `mounted_on`, Abmessungen, `rail_offset`, `clip_on`, `rail_vertical` und `rail_mount_override`. Verschiebungen und Tiefenänderungen des Trägers nehmen Nachkommen einmal mit, einschließlich verschachtelter Träger. Explizit gelöste Elemente behalten ihre Position. Elementlisten geben die Montagefelder aus. Portable Kopien und Musterkopien ordnen interne Montageverweise neu zu; das Löschen eines Trägers löst überlebende Kinder. Sowohl waagerechte als auch senkrechte Schienen unterstützen Fangen und Anordnen. Ursprüngliche Projektdaten ohne diese optionalen Felder behalten das Verhalten alter Art.

Das Merkmal `individual_panel_terminals` fügt `PanelItem.terminal` hinzu: Eine nicht leere Bezeichnung bindet eine Grundfläche und ihren eigenen Artikel an eine Klemme einer Leiste. Leer behält das Layout der ganzen Leiste. Physische Punkte, 3D-Netze, virtuelle Verdrahtung und 3D-Kabelendpunkte lösen jede Klemme getrennt auf; mehrdeutige wiederholte Platzierungen wählen kein beliebiges Element. Funktionsspezifisch gewählte Artikel bestimmen auch das Layout der Klemmenleiste. `place_panel_device` erstellt ein explizit bemaßtes Betriebsmittel oder eine Klemme; `set_panel_item` kann seine Klemmenbezeichnung bearbeiten. `get_panel_points` gibt erzeugte oder exakte physische Punkte über die gemeinsame API aus.

Das Merkmal `explicit_panel_geometry` behält `PanelItem.single_part`: Die Grundfläche stellt nur ihren eigenen Katalogartikel dar, auch ohne Klemmenbezeichnung. Fehlende Katalogabmessungen können mit der Grundfläche der Instanz nicht exakte Anschlusspunkte erzeugen, ohne den Katalog umzuschreiben. Mehrere physische Ansichten werden nur dann über den gewählten Artikel aufgelöst, wenn er eindeutig ist. `Enclosure.no_implicit_plate` unterdrückt die Standardplatte, während `mounting_origin: [x,y,z]` einen expliziten Bezugspunkt der Montagefläche in Gehäusekoordinaten angibt. Offene Baugruppen haben standardmäßig den Bezugspunkt null und die volle Ausdehnung der Fläche; ihre Verlegeräume haben keine Schaltschrankwände. Die Befehle zum Hinzufügen/Setzen von Gehäusen stellen `implicit_plate`, `mounting_origin` (Objekt mit x/y/z; leer setzt zurück), `mounting_size` (Objekt mit width/height für eine Platte, die nicht zentriert ist; leer setzt zurück) und `door_hinge` (`left` oder `right`, die Kante, an der die Tür von vorn gesehen angeschlagen ist) bereit, und die Befehle zum Platzieren/Bearbeiten von Betriebsmitteln stellen `single_part` bereit. Explizite Platten tragen ihren eigenen Artikel zur Materialliste bei.


Das Merkmal `cached_impossible_routes` fügt `layout.routing.reuse_failed_searches` hinzu (Standard false), das auch der Befehl `routing_rules` bereitstellt. Ein Projekt kann unmögliche Ergebnisse der Vorabsuche nach Änderungen der Kanalfüllung ausdrücklich behalten, statt erschöpfende Suchen zu wiederholen. Gewöhnliche Projekte behalten das bisherige Wiederholungsverhalten. Erfolgreiche zwischengespeicherte Wege erfordern weiterhin Kapazitäts- und Abstandsprüfungen.

Das Merkmal `literal_part_quantities` fügt `PartRef.total` hinzu. Ein Wert true macht
`count` zur wörtlichen Menge für das ganze Betriebsmittel bzw. die ganze Leiste
(einschließlich einer expliziten Null), statt Klemmenartikel mit den Positionen der
Leiste zu multiplizieren. Der Standard false erhält die Zählung alter Art,
einschließlich des alten Standards, null als eins zu werten. Primäre automatische und
zusätzliche wörtliche Verweise können dieselbe Artikelnummer verwenden; ihre Mengen
addieren sich, sodass neu gezeichnete Klemmen weiterhin das primäre Material erhöhen,
ohne Reserven zu multiplizieren. Wörtliche reine Materialbetriebsmittel erscheinen auch
in `list_devices` und der Stückliste. Die gemeinsamen Befehle `add_device_part` und
`set_device_part_quantity` stellen sowohl Anzahl als auch Modus bereit; letzterer
verwendet einen 1-basierten Zuordnungsindex. Der Editor der Betriebsmittelartikel stellt
auch das Kennzeichen Gesamtmenge bereit. Wörtliche Verweise mit null tragen keine
Funktionskapazität, Stücke, Gewicht, Verlustleistung oder Materialmenge bei.

`Enclosure.device_tag` bindet einen physischen Körper an seine primäre
Betriebsmittelzuordnung, wie das `tag` an einem strukturellen Element des
Schaltschrankaufbaus. Ihr primärer automatischer Artikel wird vom physischen Layout
gezählt, zusätzliche wörtliche Artikel werden getrennt gezählt. Das Ändern des
physischen Artikels über `set_enclosure`, `set_panel_item` oder den Profilaustausch
aktualisiert seine primäre Zuordnung und die Artikelbindungen der Funktionen und behält
dabei wörtliche Reserven. Die Befehle zum Hinzufügen/Setzen von Gehäusen stellen
`device_tag` bereit; Elementbearbeitungen stellen `part` bereit.

Das Merkmal `material_ownership` fügt Elementen des Schaltschrankaufbaus und Gehäusen
`owner_device` und `bom_excluded` hinzu, bereitgestellt von den Abfrage- und
Bearbeitungsbefehlen des Schaltschrankaufbaus. Die Eigentümerschaft hält das
Betriebsmittel fest, das einen Artikel verbucht, und ändert nie `mounted_on`. Ein
Ausschluss einer Platzierung unterdrückt nur ihren physischen Materialeintrag; die
bearbeitbaren Betriebsmitteleigenschaften `device.material_owner` und
`device.exclude_from_bom` behalten reine Materialbetriebsmittel und können alles einem
Betriebsmittel zugewiesene Material unterdrücken. Eigentümerverweise folgen dem
Umbenennen von Betriebsmitteln. Wiederverwendbare Schaltschrankmakros leeren externe
Eigentümerkennzeichen und behalten den expliziten Ausschluss der Platzierung. Abfragen
des Schaltschrankaufbaus geben `mounting_height` über dem expliziten Träger aus (null,
wenn unbekannt); erzeugte Abstandshalter von Schienen auf einem expliziten Träger
verwenden diese Höhe und erfordern
`layout.hardware`, statt die Dicke des Trägers als Abstandshalter zu behandeln.

Betriebsmittelbindungen des Schaltschranks nehmen auch an `list_devices` und der
Materialliste teil, wenn keine Schaltplanfunktion gezeichnet ist, unabhängig von
zusätzlichen Artikeln. Eine einzeln platzierte Klemme, die noch nicht im Schaltplan
dargestellt ist, trägt ihren primären Artikel einmal je Klemmenbezeichnung bei.
Wiederholte physische Ansichten und das neue Zeichnen derselben Klemme fügen kein
weiteres Stück hinzu. Die bestehende Platzierung ganzer Leisten behält ihre Erweiterung
um die Schaltplanklemmen.

Das Merkmal `portable_macro_materials` garantiert, dass das Einfügen portabler Makros
Eigenschaftswerte von Betriebsmitteln und getrennte automatische
und wörtliche Verweise auf denselben Artikel erhält, einschließlich null. Beim Erweitern
eines vorhandenen Betriebsmittels werden fehlende Verweismodi und Eigenschaften
aufgefüllt, während lokale Mengen und Eigenschaftswerte unverändert bleiben. Artikel,
die das Betriebsmittel nicht verwendet, werden
nur hinzugefügt, wenn es noch keine hat oder wenn eine Funktion des Clips sie wählt, sofern unten nicht ausdrücklich zugestimmt wird. Materialeigentümer innerhalb eines Clips
folgen der Seitenstruktur des Ziels und dem Umbenennen bei Kollisionen; externe
Eigentümer werden geleert. Eingehende Betriebsmittelkennzeichen werden reserviert, bevor
Ersatz vergeben wird, einschließlich vorhandener reiner Materialkennzeichen. Leitende
Hilfsansichten folgen ihrem Hauptkennzeichen. Platzhalter für
Betriebsmitteleigenschaften werden zusammen mit Makrokennzeichen und Artikelnummern
aufgelöst.

Das Merkmal `explicit_macro_part_additions` fügt die optionalen `Clip.device_part_additions` hinzu, eine Map von Betriebsmittelkennzeichen auf Arrays von Artikelnummern. Sie fordert ausdrücklich an, fehlende **wörtliche** Zubehörverweise hinzuzufügen, die in den passenden `devices[tag].parts` des Clips bereits vorhanden sind. Sie kann keinen nicht zugewiesenen Artikel und keinen automatischen Verweis hinzufügen, keinen vorhandenen Artikel ersetzen und keine lokale Menge überschreiben. Ohne diese Map behalten Einfügen und Makroeinfügung die obigen Regeln für vorhandene Betriebsmittel. Kennzeichen und Nummern unterstützen Makroparameter, Zielstruktur und Umbenennen bei Kollisionen. Dies ist eine Einfügeabsicht, keine Anweisung, die gewöhnlichen Standardwerte des Projekts für Kopieren/Einfügen zu ändern.

Das Merkmal `per_wire_junctions` fügt die optionalen `Wire.explicit_junctions` hinzu:
ausgelassen erbt `page.explicit_wire_junctions`, true erfordert gezeichnete
Verbindungsknotenkontakte, und false erlaubt implizite T-Kontakte. Implizite
Verbindungen von Leiter zu Leiterinnerem erfordern, dass beide Leiter sie erlauben;
explizite Verbindungsknoten und gespeicherte Endpunkte verbinden weiterhin. Anschlüsse
an der Anschlussseite verwenden die Regel des einzelnen Leiters. Ein expliziter Leiter
aktiviert außerdem gezeichnete physische Ziele in seinem elektrischen Netz.
Die gemeinsamen Befehle add_wire, connect, route und update_wire nehmen
`junction_policy: inherit|explicit|implicit` an. update_wire liefert diese Regel;
Seitenabfragen und portable Clips behalten das optionale native Feld. Der Editor bietet
dieselben Wahlmöglichkeiten in den Leitereigenschaften. Das Einfügen eines expliziten
Makros ändert nie die Seiteneinstellung oder andere Leiter. Das Zwischenspeichern der
Abtastung verfolgt Regeländerungen, und das Bereinigen von Leitern sowie das
Zusammenführen kollinearer Leiter erhalten unterschiedliche Regeln und Eigenschaften.

`apply_cable_part` (`cable`, `part`) wendet einen Artikel des Projekts oder des
eingebauten Katalogs auf ein vorhandenes Kabel an. Es verwendet dieselbe Operation wie
der Editor, einschließlich Typ, gelieferter Adernamen und Querschnitt, und
`Outer diameter` aus den technischen Daten in Millimetern. Fehlende oder ungültige
Durchmesser setzen die Überschreibung auf null zurück, sodass die Verlegung ihre
Schätzung verwendet. Es liefert die konfigurierten Kabelfelder und `diameter_mm` (null
bedeutet geschätzt) und weist unbekannte Kabel- oder Artikelnamen zurück, ohne den
Zustand zu ändern.

`list_cable_parts` nimmt keine Parameter und liefert die verfügbaren Kabelartikel aus
den Projektartikeln und dem eingebauten Katalog, nach Nummer sortiert und ohne
Duplikate. Projektdefinitionen haben Vorrang, einschließlich einer lokalen Änderung der
Produktgruppe. Zeilen enthalten Nummer, Typ, Beschreibung, Hersteller, gelieferte Adern
und Querschnitt sowie einen positiven angegebenen `diameter_mm` oder null. Der Editor
verwendet denselben Vorrang mit seiner konfigurierten Artikeldatenbank.

Das Merkmal `portable_cables` trägt Kabelbestände und ihre referenzierten lokalen
Artikel in `Clip.cables`. Kopieren/Einfügen und Makroeinfügung vergeben frische
Kabel-IDs und kollisionsfreie Kennzeichen in der Zielstruktur und ordnen
Markierungssymbole, Leiter, Verbindungsdefinitionen und Kabeldefinitionslinien
einheitlich neu zu. Aderzuordnungen werden nur für Kabel behalten, die dem Clip
gehören. Clips alter Art ohne Kabeldefinitionen leeren weiterhin Leiter-/Aderverweise.
Kabeltextfelder unterstützen auch native Makroplatzhalter. Dieses Verhalten teilen sich
alle Frontends.

Ein expliziter Verweis auf einen Funktionsartikel kann den passenden Kabelartikel
auflösen, wenn dasselbe Kennzeichen auch einen gewöhnlichen Materialdatensatz eines
Betriebsmittels hat. Der gewählte Artikel muss zu einer tatsächlichen Zuordnung passen;
unbekannte Verweise bleiben ungültig. Kabelmaterial bleibt im Kabelbestand, statt in der
gewöhnlichen Stückliste der Betriebsmittel doppelt geführt zu werden.

Ergebnisse physischer Verbindungen können dasselbe Endpunktpaar mehr als einmal
enthalten, wenn getrennte Kabeladern unterschiedliche einfache gezeichnete Wege haben.
Jede Verbindung behält ihre eigenen Leiter-IDs und Kabel-/Adereigenschaften für Berichte
und Verlegung. Ein loser Stich wird nicht als zusätzliche vollständige Verbindung
behandelt. Die bestehenden Verbindungsbefehle und die Berichts-Registry stellen dieses
Verhalten in jedem Frontend bereit.

Der gemeinsame Befehl `apply_cable_part` aktualisiert explizite Verweise platzierter
Funktionen, die nur zum vorherigen Kabelartikel gehörten. Verweise auf einen
gewöhnlichen Betriebsmittelartikel unter demselben Kennzeichen sowie ungültige oder
nicht zugehörige explizite Verweise bleiben erhalten. Der Kabeleditor verwendet dieselbe
Kernoperation für Verweise bei der Katalogauswahl und bei direkten Bearbeitungen der
Artikelnummer. Länge und Leiterzuordnungen bleiben beim Anwenden von Katalogkabeldaten
erhalten.

Das Host-Merkmal `macro_definition_parameters` erweitert native Makroplatzhalter auf
Buchstaben portabler Symboldefinitionen, Standardbeschriftungen, Standardfunktionstexte
und Standardeigenschaften, Anschlussbeschriftungen, Textprimitive und wörtliche Zeilen von
Attributblöcken. Symbolschlüssel, Eigenschaftsschlüssel, kodierte Bilder und rohe
Quellmetadaten behalten ihre Identität. Die Auflösung arbeitet vor der Installation auf
einer Kopie des Clips und wird von Desktop, CLI und MCP geteilt. Importer, die
Parameter von Quellvorlagen umwandeln, sollten dieses Merkmal verlangen, damit ältere
Hosts keine statische Grafik hinterlassen, die nicht zu den aufgelösten Instanzen passt.

`insert_macro` nimmt optionale `values` an, ein Objekt, das Platzhalternamen auf
Zeichenketten abbildet. Es löst Instanzfelder und Felder portabler Definitionen vor dem
Einfügen zusammen auf; unbekannte Platzhalter bleiben wörtlich. Ungültige Werte, die
keine Zeichenketten sind, schlagen vor jeder Bearbeitung fehl. Wird der Parameter
ausgelassen, bleibt bestehender nicht aufgelöster Vorlagentext erhalten. Diese Option
ist über die gemeinsame Registry in jedem Frontend verfügbar.

Das Host-Merkmal `parallel_cable_cores` bestätigt, dass Verbraucher physischer
Verbindungen getrennte gestaltete parallele Aderwege behalten. Importer müssen den
laufenden Host ebenso prüfen wie eine getrennte Prüf-CLI, bevor sie Stromkreise mit
zugewiesenen Kabeladern annehmen; ein neuerer Prüfer kann die Berichts-Engine eines
älteren Editors nicht aktualisieren.

Das Merkmal `page_property_blocks` löst Projekt-/Seiteneigenschaften in
Attributblöcken von Symbolen über ihren platzierten Zeichnungskontext auf. Andere Felder
behalten die Funktionsauswahl, einschließlich des gewählten Artikels. Das
schreibgeschützte `page.local_name` liefert die Seitennummer ohne ihr Abschnittspräfix;
`page.section_pages` zählt die Seiten im selben Abschnitt (Seiten der obersten Ebene
bilden Abschnitt null). Beide aktualisieren sich nach Bearbeitungen.

Das Merkmal `page_template_values` behält die Map `values` einer Seitenvorlage beim
Speichern, Austauschen und Einfügen. Leere Maps alter Art bleiben kompatibel. Der
gemeinsame Befehl list_page_templates liefert diese Standardwerte, und jedes Frontend
wendet sie über dieselbe Einfügeoperation an. Importer sollten beide Merkmale prüfen,
bevor sie wiederverwendbare Quellrahmen mit bearbeitbaren Kontextfeldern liefern.

Das Merkmal `page_frame_fields` fügt die berechneten Eigenschaften
`page.section_last_number`, `page.previous_local_name` und `page.next_local_name` hinzu.
Die letzte Nummer verwendet die natürliche Sortierung; benachbarte Seiten folgen der
bearbeitbaren Reihenfolge ihres Abschnitts, mit leeren Werten an beiden Enden. Für die
Blätter 11 bis 23 ist die Seitenzahl 13, während die letzte Nummer 23 ist. Einfügungen,
Löschungen und Neunummerierung aktualisieren diese Felder. `page.function_designation`
und `page.location_designation` enthalten die Aspektzeichen `=` und `+`, einschließlich
bloßer Zeichen für einen nicht konfigurierten Aspekt.
Diese schreibgeschützten Eigenschaften sind über die gemeinsame Eigenschafts-Registry,
Attributblöcke von Symbolen, Freitext-Platzhalter und Schriftfeldformulare verfügbar.


Das Host-Merkmal `function_cable_references` stellt die bearbeitbare Eigenschaft
`function.cable` der Klasse Funktion bereit, die das volle Kabelkennzeichen für eine
platzierte Markierung enthält. Das Umbenennen des Kabels über die Kabeltabelle oder den
Editor aktualisiert diese Verweise, passende Markierungskennzeichen, Leiterzuordnungen,
Anschlusspunkte und Definitionslinien, einschließlich inaktiver Modelloptionen.
Gewöhnliche Betriebsmittelsymbole mit demselben Kennzeichen bleiben getrennt. Das
Löschen des Bestandseintrags leert seine explizite Markierungszuordnung und seine
Aderzuordnungen, entfernt seine eigenen Weg- und Zubehördefinitionen und behält
gewöhnliche Endpunktstellen von Betriebsmitteln. Portabler Makro-/Zwischenablageinhalt trägt das referenzierte Kabel
und bildet die Zuordnung auf das frisch eingefügte Kabelkennzeichen ab. Verwenden Sie
`set_properties` auf `function:<symbol id>`, um die Zuordnung zu bearbeiten; bestehende
Registry-Operationen gelten gleichermaßen für Desktop, CLI und MCP.


Zellen mit Kabelkennzeichen (`table_set`, Tabelle `cables`, Spalte `Cable`) und die
Kabelverwaltung verwenden dieselbe atomare Operation `ops::rename_project_cable`. Sie
kürzt das eingegebene Kennzeichen, prüft seine Bezeichnung, lehnt Duplikate und
Änderungen an freigegebenen Zeichnungen ab und aktualisiert Verweise erst nach der
Prüfung. Die Kabelverwaltung hält Entwurfstext außerhalb des Projekts bis zur
Eingabetaste oder zum Verlust des Fokus; Esc verwirft den Entwurf.

Eine Funktion mit einem expliziten Verweis `function.cable` löst ihren gewählten und
zugewiesenen primären Artikel über diesen Eintrag des Kabelbestands auf. Ein geänderter,
geleerter oder fehlender Kabelartikel wird sofort wiedergegeben, einschließlich direkter
Bearbeitungen von Kabeleigenschaften und Tabellen. Zwischengespeicherte Artikelauswahlen
von Funktionen verdecken diese Zuordnung nicht. Die gewöhnliche Artikelauswahl von
Funktionen behält ihre bestehenden Regeln für explizite Verweise.


Das Merkmal `cable_property_blocks` löst Zeilen der Klasse Kabel in Attributblöcken von
Symbolen gegen den expliziten Verweis `function.cable` des Symbols auf. Ein fehlendes
explizites Ziel bleibt unaufgelöst; Symbole ohne diesen Verweis behalten den
Kabelkontext alter Art über dasselbe Kennzeichen. Dieser Kontext gilt auch für
platzierte Kabeleigenschaften und wird von Zeichenfläche und Exporten geteilt. Der
Symboleditor bietet Eigenschaften der Klasse Kabel an. Platzierte Eigenschaften der
Klasse Artikel lösen den gewählten Katalogartikel der Funktion auf, einschließlich des
aktuell verknüpften Kabelartikels und nicht primärer Betriebsmittelzuordnungen.

Hosts, die `placed_wire_labels` bekannt geben, behalten `Wire.labels`, auch wenn
`hide_labels` automatische Beschriftungen unterdrückt. Jedes Layout enthält `kind`
(`Core`, `Cable`, `Number` oder `Color`), den Befestigungsanteil `along` (0..1 von a
nach b), `offset` in Zeichnungs-mm, `size`, `anchor`, `angle` und optionale RGB-`color`.
Der Text kommt vom aktuell besitzenden Leiter. `update_wire` nimmt
`labels: {"items": [...]}` an, um die Layouts zu ersetzen, oder ein leeres Array items,
um sie zu entfernen. Ungültige Layouts weisen den ganzen Befehl zurück. Beschriftungen
folgen ihrem Leiter beim Bearbeiten und werden einmal neu verteilt, wenn die Verlegung
neue Abschnitte erzeugt.

Nicht angehängte Adermarkierungen können unabhängigen bearbeitbaren Text in den
indizierten Eigenschaften `function.cable_core[1]` bis `[64]` behalten. Importer sollten
platzierte Leiterbeschriftungen für Markierungen verwenden, die auf einen tatsächlichen
physischen Leiter verweisen, sodass Bearbeitungen der Leiterader den gezeigten Text
automatisch ändern.

`rename_symbol_device` nimmt die `id` eines platzierten Symbols und `new_tag` an. Ein
expliziter Verweis `function.cable`, der zu seinem gezeigten Kennzeichen passt, wählt
den Kabelbestand; gewöhnliche Betriebsmittel mit geteiltem Kennzeichen bleiben
unabhängig. Fehlende Kabelziele, doppelte Kabelkennzeichen und Verweise auf freigegebene
Zeichnungen weisen die Operation zurück. Das Umbenennen der gewählten Funktion im Editor
verwendet dieselbe Operation. `rename_device` folgt dem Kabelbestand, wenn keine
gewöhnliche Betriebsmittelidentität dieses Kennzeichen teilt. Verweise auf
Materialeigentümer folgen Kabelbearbeitungen nur, solange das alte Kennzeichen keine
gewöhnliche Betriebsmittelidentität hat.
Das Umbenennen eines Betriebsmittels oder Kabels kann ein unverändertes importiertes
Präfix mit Leerzeichen behalten; der neue endgültige Betriebsmittelname folgt weiterhin
den normalen Prüfregeln für Kennzeichen.

### Explizite Ausrichtung im Schaltschrank

Hosts, die `explicit_panel_rotations` bekannt geben, erhalten optionale numerische Werte
`PanelItem.rotation` von 0, 90, 180 und 270 (im Uhrzeigersinn in der Ebene der Fläche).
Negative Vielfache von 90 werden angenommen und normalisiert. Fehlende Werte erhalten
die Ableitung aus den Abmessungen alter Art; eine explizite Null unterdrückt diese
Ableitung. Die gespeicherten `x`, `y`, `w`, `h` bleiben die bereits ausgerichtete
Grundfläche. Importer dürfen diese Abmessungen nicht ein zweites Mal drehen. Grafik,
3D-Netze, Positionen und Austrittsrichtungen von Anschlusspunkten, Montagezubehör und
exakte Bohrbilder folgen dem Winkel.

`set_panel_item(rotation: ...)` ändert die Ausrichtung, ohne die Grundfläche zu
verschieben oder in der Größe zu ändern. `infer_rotation: true` leert den expliziten
Winkel; beides anzugeben ist ein Fehler. Elementabfragen liefern `rotation` und
`effective_rotation`.
`arrange_panel_items(action: "rotate")` dreht gewählte Elemente um 90 Grad um ihre
Mittelpunkte und tauscht Breite und Höhe, passend zur Aktion Drehen des Editors.

Optionale boolesche Parameter im Editor bieten Plugin-Standard verwenden, Aktiviert und
Deaktiviert an. Das erste lässt das Argument aus, sodass externe Plugins ihre
dokumentierten Standardwerte behalten; ein explizites false bleibt in jedem Frontend von
der Auslassung verschieden.

### Revisionen von Symboldefinitionen

`merge_symbol_definitions` nimmt `definitions: {"items": [<UserSymbol>, ...]}` an.
Der vollständige Stapel wird vor jeder Änderung deserialisiert und geprüft. Es liefert
`keys`, einen aufgelösten Schlüssel je Eingabe in Quellreihenfolge, und `added`, die
Anzahl neuer Definitionen. Identische typisierte Definitionen verwenden ihren
vorhandenen Schlüssel wieder. Konflikte versuchen `<original>_import_1`, `_import_2`
und so weiter und verwenden eine gleiche vorhandene Revision wieder. Vorhandene
Definitionen und Seiteninstanzen bleiben unverändert. Zwei Eingaben mit demselben
ursprünglichen Schlüssel können verschiedene Revisionsschlüssel liefern, daher sollten
Importer Verweise innerhalb jedes eingehenden Clips abbilden, statt eine globale Map der
ursprünglichen Schlüssel aufzubauen.

Editor, CLI, MCP und externe Plugins verwenden alle denselben registrierten Befehl.
Die native Typisierung normalisiert Standardfelder und numerische Genauigkeit vor dem
Vergleich; Importer sollten diese Operation verwenden, statt rohe gespeicherte
JSON-Objekte zu vergleichen. Dies ist dieselbe Kernoperation, die beim Einfügen
portabler Makrodefinitionen verwendet wird.

### Selbst gestaltete Plotrahmen

Hosts, die `plot_frame_artwork` bekannt geben, nehmen `forms.frames[].artwork` an, ein
Array von Symbol-Zeichnungsprimitiven in Papiermillimetern. Nicht leere Grafik ersetzt
den erzeugten Rand und das erzeugte Schriftfeld; `visible` steuert die gesamte
Rahmenzeichnung. Eigenschaftsblöcke lösen aktuelle Projekt- und Seiteneigenschaften für
jede Seite auf, die ihn verwendet. Die Grafik gehört zum wiederverwendbaren Rahmen und
ist kein auswählbarer Schaltungsinhalt. Primitive mit einer `color` (Pfade, Kreise, Texte, Eigenschaftsblöcke: ein farbiges Logo) behalten sie; die anderen werden in der Rahmenfarbe gezeichnet.
Der gemeinsame Befehl `set_frame` nimmt optionales JSON `artwork: {"items": [...]}` an; ein leeres Array items
stellt den erzeugten Rahmen wieder her. `list_frames` enthält die Grafik. Zeichenfläche,
Rahmenvorschau und Exporte verwenden denselben Renderer. Ältere Projektdateien behalten
erzeugte Rahmen.

### Schriftfeldbilder und Textstile

Hosts, die `title_block_images` bekannt geben, nehmen Elemente `Image` in Schriftfeldern
(`forms.title_blocks[].items`) und in gezeichneten Auswertungsformularen
(`layout.header`, `layout.row`) an:
`{"type": "Image", "a": {…}, "b": {…}, "picture": {"file": {"name", "data"}, "px": [w, h]}}`
mit einem base64-kodierten PNG oder JPEG, das unter Beibehaltung seines
Seitenverhältnisses in das Rechteck `a`..`b` eingepasst wird. Ohne `picture` zeigt das
Element `info.logo` (gleiche Form wie `picture`), das Projektlogo, das auch das
Deckblatt zeigt: über dem Projektnamen oder eingepasst in `info.cover_logo` (`x`, `y`,
`width`, `height` in mm von der oberen linken Ecke der Zeichenfläche; Befehl
`set_cover_logo`). Elemente `Text` nehmen optional `bold`, `italic` und `angle` an
(Grad, gegen den Uhrzeigersinn). Die Platzhalter `{page.previous}`, `{page.next}` und
`{rev.N.name|date|author|description}` (N-letzte freigegebene Revision) werden wie die
anderen gefüllt. Die gemeinsamen Befehle `list_title_blocks`, `set_title_block`
(Elemente als `items: {"items": [...]}`), `delete_title_block`,
`set_default_title_block` und `set_page` (`title_block`) bearbeiten sie; die
Kommandozeile fügt `add_title_block_image` und `set_project_info` (`logo`) für
Bilddateien hinzu.

### Formularbedingungen, Stile, QR-Codes und Stempel

Hosts, die `form_customization` bekannt geben, nehmen diese weiteren Felder von
`forms` an (alle optional, ältere Dateien und Plugins lassen sie aus):

- Jedes Element eines Schriftfelds oder gezeichneten Formulars nimmt
  `show: {"filled": "…", "page_types": [...]}` an: Es wird nur gezeichnet, wenn `filled`
  (mit Platzhaltern) nicht leer ist, und nur auf Seiten der angegebenen Typen
  (`schematic`, `cover`, `report`, `mounting`, `drilling`, `overview`, `graphic`, `document`).
  `Line`, `Rect` und `Text` nehmen `color: [r, g, b]` an, `Line` und `Rect` `dashed`.
  `{"type": "Qr", "a": {…}, "size": mm, "text": "…"}` zeichnet einen QR-Code
  (Byte-Modus, Stufe M) des gefüllten Texts.
- `forms.page_type_title_blocks` ordnet Seitentypen Schriftfeldnamen zu, und
  `forms.frames[].title_block` nennt das Schriftfeld eines Plotrahmens. Eine Seite
  verwendet ihr eigenes Schriftfeld, sonst das ihres Typs, ihres Rahmens, den
  Projektstandard, Standard.
- `forms.stamp` (`draft`, `review`, `approved`, `released`, `size`, `angle`, `color`)
  ist ein Text, der je nach Zustand des Revisionsablaufs quer über jede Seite gezeichnet
  wird; die Platzhalter `{rev.status}`, `{rev.approved_by}`, `{rev.approved_date}` und
  `{rev.N.approved_by|approved_date}` geben den Zustand an und wer eine Revision
  freigegeben hat.
- Auswertungsformulare nehmen `totals` (unter der letzten Zeile summierte Spalten),
  `shade_rows` und `align` (`{column: "Left" | "Center" | "Right"}`) an; eingefrorene
  Berichtsseiten behalten sie in `totals`, `shade_rows` und `align` ihrer Tabelle.

Befehle: `set_stamp`, `set_default_title_block` (`page_type`), `set_frame`
(`title_block`); `set_title_block` nimmt die neuen Elementfelder an. Die
Formularbibliothek (Datei und die Bibliothek `forms` des Artikelservers, `GET` /
`PUT /api/v1/libraries/forms`) trägt auch `frames`; die Kommandozeile lädt und
veröffentlicht sie mit `parts_company_forms` (mit `publish`).

### Live-Felder für Funktionsverweise

Hosts, die `function_cross_references` bekannt geben, lösen die schreibgeschützte
Eigenschaft `function.cross_references` aus dem aktuellen nativen Index der
Betriebsmittel und Unterbrechungspunkte und den Verweiseinstellungen auf. Zeilen von
Eigenschaftsblöcken behalten die gestaltete Platzierung und den Fluss, während ihr Text
umbenannten Seiten, Abschnitten und geänderten Betriebsmittelzielen folgt.
Ihre Links auf die Zielseite je Zeile überstehen den PDF-Export; Text auf der
Zeichenfläche und im SVG verwendet denselben Resolver. Ein explizites Feld ersetzt
den automatischen Verweistext; gewöhnliche Symbole behalten automatische Anmerkungen.
Bibliotheks-/Geistervorschauen ohne Projektkontext lassen die Verweiszeile leer.

Der gemeinsame Befehl `list_cross_references` liefert für jede Funktion `id`, `page`,
`tag`, mehrzeiligen `text` und `references` (Text und Zielseite, null für fehlende
Ziele). Die optionalen Filter `id` und `page` verwenden native Symbol-IDs und
Seitennamen. Die Eigenschaft ist auch über `get_properties` und Eigenschaftsauswahlen
verfügbar; `set_properties` weist Versuche zurück, ihren berechneten Wert einzufrieren.
