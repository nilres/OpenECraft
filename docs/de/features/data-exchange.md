# Ausgabe und Datenaustausch

PDF-, DXF- / DWG-, SVG- und 3D-Exporte, DXF- / DWG-Import, die Schicht für die Unternehmensintegration (ERP, PDM, PLM, ECAD, MCAD, Beschaffung, Verdrahtungsautomaten), der Austausch mit KiCad und FreeCAD und Autodesk Vault.

## Ausgabe / Datenaustausch

**PDF** aller Seiten oder einer Seitenauswahl (`1-3, 5`;
auch für das HTML-Druckdokument; *Datei › Drucken* öffnet alle Seiten oder die aktuelle
Seite im Browser mit dessen Druckdialog) mit Lesezeichen je Seite, anklickbaren
Querverweisen und Auswertungszeilen (Links auf ausgelassene Seiten entfallen); Texte jenseits von Latin-1 wie Polnisch, Tschechisch, Türkisch,
Griechisch oder Kyrillisch werden mit einer eingebetteten Teilmenge der Schrift Ubuntu
geschrieben, durchsuchbar und kopierbar, **DXF** (R12, Layer nach Art) und **DWG**
(AutoCAD 2004, dieselbe Zeichnung) je Seite oder alle Seiten nebeneinander, SVG je Seite, druckbares HTML, Fertigungs-Leitungsliste (CSV mit Drahtnummern,
Enden, Farbe, Querschnitt, verlegter Länge, Abisolierlänge und Anschlussart beider
Enden (Aderendhülse nach DIN 46228-4, Zwillingsaderendhülse, Ring-, Gabel-,
Stiftkabelschuh, Flachstecker oder Flachsteckhülse) mit ihren Artikeln, eine
Zwillingsaderendhülse mit dem anderen Draht benannt und beim ersten der beiden
aufgeführt, Kabeladern; Schaltschrankdrähte nach Querschnitt, Farbe und Länge sortiert
für Drahtkonfektionierautomaten; Aderendhülsen und Kabelschuhe auch in der Stückliste;
mit einer UTF-8-Byte-Order-Mark, wie
die Auswertungs-CSVs des Editors, damit Tabellenkalkulationen `mm²` und Umlaute anzeigen), Auswertungs-CSVs, SPS-Symboltabellen,
NC-Daten von Montageplatten, der **Schaltschrankaufbau in 3D** als **glTF 2.0** (Viewer,
Web, PLM; Meter, ein Knoten je Gehäuse und Element, Farben), **STEP AP214** (MCAD;
facettierte Volumenkörper mit Farben, Millimeter, z nach oben), **IGES 5.3** (MCAD;
B-Rep-Volumenkörper mit ebenen Flächen, offene Schalen für nicht geschlossene Netze,
Farben und Namen), **IFC4** (BIM; ein Verteiler je Gehäuse mit seinen Komponenten als
triangulierte Elemente), **JT** (PLM-Viewer; Tri-Strip-Netze je Element, Aufbau nach
JT 8.1) und **Parasolid XT** (MCAD wie SolidWorks, NX, Solid Edge, Onshape;
Transmit-Text des Schemas V13 in Metern, jedes geschlossene Teil ein facettierter
Volumenkörper, dessen koplanare Dreiecke zu ebenen Flächen zusammengefasst sind, offene
Netze ein Flächenkörper) –
Wände, Tür, Montageplatte, Schienen, Kanäle, Sammelschienen und Betriebsmittel mit den
3D-Modellen ihrer Artikel.
**DXF- / DWG-Import** (*Datei › Import › DXF / DWG als Seite importieren…*): alte Zeichnungen als Grafik
auf einer neuen Schaltplanseite – Linien, Polylinien, Kreise, Bögen, Ellipsen, Splines,
Texte, MTEXT, Blockreferenzen und Bemaßungen, in ihren Einheiten, auf die
Zeichenfläche verkleinert, wenn sie größer sind. DWG-Dateien von AutoCAD R12 … 2018
werden mit dem reinen Rust-Crate [acadrust](https://crates.io/crates/acadrust) gelesen
(und geschrieben), auch im Browser. EDZ-Dateien des EPLAN Data Portal werden als Artikel gelesen (siehe [Artikelverwaltung](parts.md)); ihre 2D-Fenstermakros werden zu Symbolen und ihre 3D-Makros zu den
3D-Modellen der Artikel.

## Unternehmensintegration

**Unternehmensintegration** (*Werkzeuge › Unternehmensintegration…*): eine
Integrationsschicht mit Konnektoren für **ERP** (Artikelstamm und Stückliste, CSV),
**PDM** (Projekt- und Seitenmetadaten, JSON), **PLM** (Konstruktionsstückliste, JSON),
**ECAD** (Bauteile und Netzliste für den Leiterplattenentwurf, JSON), **MCAD**
(Platzierung der Betriebsmittel, CSV), **Beschaffung** (Bestellliste je Lieferant,
Bestand und Verpackungseinheiten berücksichtigt, CSV) und **Verdrahtungsautomaten**
(Auftragsliste mit Zuschnittlänge, Abisolierung und Aderendhülsen, CSV). Alle
Konnektoren arbeiten in beide Richtungen und teilen sich ein neutrales Feldvokabular:
Exporte verwenden die Feldcodes als Spalten / Schlüssel, Importe erkennen die Codes,
gängige englische und deutsche Spaltennamen (`Material`, `Bestand`, `refdes` …) und die
Spaltenzuordnung eines *Profils* (z. B. `MATNR` → `erp_number` für SAP). Importe werden
als Liste von Änderungen angezeigt, die geprüft und übernommen werden: ERP-Nummern,
Preise, Bestand und Lieferzeiten, PLM-Element-IDs, Revisionen und Lebenszyklusstatus
(verwendete abgekündigte Artikel werden von der Prüfung gemeldet),
PDM-Dokumentnummern und -status, Artikelzuweisungen aus dem ECAD-System,
Betriebsmittelpositionen aus dem MCAD-System, Auftragsbestätigungen und die
Rückmeldung der Drahtkonfektionierung als Verdrahtungsmarkierungen. Jeder Export
speichert die exportierten Werte, so dass Werte, die seither auf beiden Seiten geändert
wurden, als Konflikte gemeldet werden und Werte, die das andere System unverändert
gelassen hat, lokale Änderungen niemals überschreiben. Eine neutrale Austauschdatei
(`openecraft.exchange/1`, JSON) enthält alles für Systeme ohne Konnektor;
das Austauschprotokoll wird im Projekt geführt.

## KiCad und FreeCAD

OpenECraft tauscht Daten mit den freien Werkzeugen für den Leiterplattenentwurf, [KiCad](https://www.kicad.org),
und für die mechanische Konstruktion, [FreeCAD](https://www.freecad.org), in deren eigenen Dateiformaten aus.

| Was | Richtung | Wie |
| --- | --- | --- |
| Eine Platine als Betriebsmittel | KiCad → OpenECraft | *Datei › Import › KiCad-Platine als Betriebsmittel importieren…* liest die Netzliste der Platine (`.net`) |
| Eine Schaltung für eine Platine | OpenECraft → KiCad | *Datei › Export*, *KiCad-Netzliste*; im Platineneditor von KiCad *Datei › Importieren › Netzliste* |
| Symbole | beide | *Datei › Export*, *KiCad-Symbolbibliothek*; *Symbolbibliothek importieren…* liest `.kicad_sym` |
| Artikelnummern und Beschreibungen | KiCad → OpenECraft | Konnektor *KiCad* der Unternehmensintegration: eine Netzliste oder eine Stückliste (CSV) |
| Der Schaltschrankaufbau in 3D | OpenECraft → FreeCAD | *Datei › Export*, *Schaltschrankaufbau in 3D für FreeCAD*: ein Makro, das den Aufbau erzeugt |
| Positionen der Geräte | FreeCAD → OpenECraft | Konnektor *FreeCAD* der Unternehmensintegration: das in FreeCAD gespeicherte Dokument (`.FCStd`) |
| 3D-Modelle von Artikeln und Platinen | KiCad, FreeCAD → OpenECraft | STEP-Dateien als 3D-Modell eines Artikels |
| Zeichnungen | beide | DXF |

**Eine in KiCad entworfene Platine als Betriebsmittel.** Die Netzliste im
Schaltplaneditor von KiCad schreiben (*Datei › Exportieren › Netzliste…*, Format
KiCad, oder `kicad-cli sch export netlist`). *KiCad-Platine als Betriebsmittel
importieren…* fügt eine Schaltplanseite mit einer Gerätebox für die Platine und einem
Geräteanschlusspunkt für jeden Pin ihrer Steckverbinder ein (`J1.1`, `J1.2` …), an
jedem Pin steht das Netz der Platine (`+24V`, `IN1`). Die Platine wird dann wie jedes
andere Betriebsmittel verdrahtet und erscheint in den Verbindungslisten, Kabelplänen
und der Stückliste. Steckverbinder sind die Bauteile mit den Referenzen J, P, X, CN,
CON, TB oder XS und die der KiCad-Bibliotheken *Connector* und *TerminalBlock*, ohne
Testpunkte (TP) und Steckbrücken (JP); eine
Platine mit vielen Pins wird auf weiteren Boxen und Seiten fortgesetzt. Die Platine
bekommt einen Artikel (`PCB.<Name>`) mit Titel, Firma und Revision des Schriftfelds
und der Liste ihrer Bauteile. Das Werkzeug `import_kicad` der Kommandozeile und des
MCP-Servers nimmt außerdem die Platinendatei (`.kicad_pcb`), deren Umriss auf der Lage
`Edge.Cuts` die Größe des Artikels ergibt, das Betriebsmittelkennzeichen und die
Bauteile, deren Pins die Anschlüsse sind. Das 3D-Modell der Platine ist die
STEP-Datei, die KiCad schreibt (*Datei › Exportieren › STEP*,
`kicad-cli pcb export step`), dem Artikel zugewiesen wie jedes Modell.

**Die Schaltung als KiCad-Netzliste.** Jedes Betriebsmittel mit elektrischen
Anschlüssen ist ein Bauteil (Referenz: das Betriebsmittelkennzeichen, Wert: die
Typnummer seines Artikels, Felder *Part number*, *Manufacturer* und *Order number*),
jedes Potenzial ein Netz aus Anschlusspunkten. Footprints werden in KiCad zugewiesen;
ihre Pads heißen wie die Anschlusspunkte (`A1`, `13`). Die Bauteile behalten ihre
Identität von Export zu Export, so dass der Platineneditor ein Layout aktualisiert,
statt neu zu beginnen. Eine Netzliste oder eine Stückliste (CSV mit der Spalte
*Reference*), die aus KiCad zurückkommt, weist über den Konnektor *KiCad*
Artikelnummern und Beschreibungen zu, als Liste von Änderungen zur Durchsicht.

**Symbole.** Der Export schreibt die im Projekt platzierten Symbole und seine
Benutzersymbole (mit dem Parameter `all` die ganze Bibliothek) als
KiCad-Symbolbibliothek: 4 mm hier sind 2,54 mm (100 mil) dort, so dass die
Anschlusspunkte auf dem Raster von KiCad liegen. Eine `.kicad_sym`-Datei wird als
Benutzersymbole in einer nach der Datei benannten Bibliothek gelesen, 1,27 mm
(50 mil) dort als 4 mm hier: ein Symbol je
Einheit (`LM358 A`, `LM358 B`), die Pinnamen innen, abgeleitete Symbole mit der Grafik
ihres Elternsymbols, Spannungssymbole als Potenziale. Erneutes Lesen einer Bibliothek
ersetzt ihre Symbole.

**Der Schaltschrankaufbau in FreeCAD.** Der Export ist ein Makro: in FreeCAD
ausgeführt (*Makro › Makros…*), erzeugt es ein Dokument mit einem Part-Container je
Gehäuse und einem Objekt je Komponente, Schiene, Kanal und Gerät, benannt mit den
Betriebsmittelkennzeichen, als Volumenkörper (Objekte mit sehr vielen Dreiecken
bleiben Netze; zwei Einstellungen am Anfang des Makros entscheiden). Jedes Gerät
trägt Kennzeichen, Artikelnummer, Gehäuse, Montagefläche und Platz als Eigenschaften
der Gruppe *OpenECraft*. Geräte in FreeCAD verschieben, das Dokument speichern und die
`.FCStd`-Datei mit dem Konnektor *FreeCAD* lesen: die Positionen kommen als Liste von
Änderungen zurück, mit der Konflikterkennung jedes Konnektors. In FreeCAD
konstruierte Modelle werden Artikeln als STEP-Dateien zugewiesen.

```bash
openecraft-cli -p plant.oec import_kicad --path relay-io.net --board relay-io.kicad_pcb
openecraft-cli -p plant.oec import_kicad --path Device.kicad_sym
openecraft-cli -p plant.oec export --format kicad_netlist --path plant.net
openecraft-cli -p plant.oec export --format kicad_symbols --path plant.kicad_sym
openecraft-cli -p plant.oec integration_export --connector freecad --path plant.FCMacro
openecraft-cli -p plant.oec integration_import --connector freecad --path plant.FCStd --apply
```

## Autodesk Vault

OpenECraft verbindet sich über die Vault Data API (REST,
`/AutodeskDM/Services/api/vault/v2`) mit Autodesk Vault Professional 2025.2 oder neuer,
so dass Projekte, Dokumente und 3D-Modelle direkt mit dem PDM-System statt über Dateien
ausgetauscht werden.

- Im Editor: *Werkzeuge › Autodesk Vault…* – die Serveradresse, den Tresor (Knowledge
  Vault), Benutzername und Passwort (oder das Zugriffstoken einer Vault-Sitzung oder
  eines Autodesk-Kontos) eingeben und *Anmelden*. Die Ordner durchblättern und die
  Dateieigenschaften unterhalb des angezeigten Ordners durchsuchen.
- **Import**: Eine ausgewählte Datei wird als Projekt geöffnet (`.oec`, `.oecz`), in
  die Artikeldatenbank importiert (CSV, JSON, BMEcat, EPLAN XML, EDZ, Gehäusepakete als ZIP), als 2D- / 3D-Modell eines
  Artikels zugewiesen (SVG, STEP, IGES, Parasolid, JT, IFC, STL, OBJ – z. B. Modelle der mechanischen Konstruktion, die
  im Tresor liegen), als Seite (DXF, DWG) oder als Dokumentseite (PDF, Office-Dateien,
  Bilder) eingefügt, mit einem Konnektor der Integrationsschicht gelesen oder als
  lokale Datei gespeichert. Der Dateiname schlägt die Verwendung vor.
- **Export**: Die Projektdatei, jeder Export von *Datei › Export* (PDF, DXF / DWG, der
  Schaltschrankaufbau in 3D als STEP, IGES, Parasolid, JT, glTF oder IFC, Auswertungen, SPS-Tabellen, Exporte von
  Plugins) und die Konnektor-Exporte kommen in den angezeigten Ordner. Eine Datei
  gleichen Namens dort erhält eine neue Version (ausgecheckt, in Teilen hochgeladen,
  mit dem Kommentar eingecheckt; das Auschecken wird rückgängig gemacht, wenn etwas
  fehlschlägt), andernfalls wird die Datei hinzugefügt.
  Ein aus dem Tresor geöffnetes Projekt wird mit *Datei › Speichern (… in Vault einchecken)* wieder eingecheckt.
- **Elemente**: *Artikelverknüpfungen aus Vault-Elementen aktualisieren* findet das
  Vault-Element jedes Projektartikels (über die Elementnummer oder seine Eigenschaft
  *Part Number*) und zeigt die Elementnummern, Revisionen und Lebenszyklusstatus als
  Änderungen des PLM-Konnektors zur Prüfung an, mit derselben Konflikterkennung wie bei
  Dateiimporten.
- Kommandozeile / MCP: `vault_connect`, `vault_browse`, `vault_items`, `vault_download`,
  `vault_open`, `vault_upload`, `vault_sync_items`. Eine MCP-Sitzung meldet sich einmal
  mit `vault_connect` an; die Kommandozeile startet je Aufruf eine Sitzung und nimmt
  Server, Token und Tresor-ID aus `OPENECRAFT_VAULT_URL`, `OPENECRAFT_VAULT_TOKEN` und
  `OPENECRAFT_VAULT_ID` (`vault_connect` zeigt Token und Tresor-ID einer Anmeldung).

```bash
openecraft-cli vault_connect --url https://vault.example.com --user anna --password …
export OPENECRAFT_VAULT_URL=https://vault.example.com OPENECRAFT_VAULT_TOKEN=V:… OPENECRAFT_VAULT_ID=101
openecraft-cli vault_browse --query "Plant 1"
openecraft-cli -p plant.oec vault_upload --folder 42 --format step --comment "Panel layout"
```

- Passwörter werden nicht gespeichert, und das Token wird nur im Arbeitsspeicher
  gehalten. Die Browser-Version benötigt einen Vault-Server (oder einen Reverse-Proxy
  davor), der Cross-Origin-Anfragen von der Adresse des Editors zulässt.
