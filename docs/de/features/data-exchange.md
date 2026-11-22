# Ausgabe und Datenaustausch

PDF-, DXF- / DWG-, SVG- und 3D-Exporte, DXF- / DWG-Import, die Schicht für die Unternehmensintegration (ERP, PDM, PLM, ECAD, MCAD, Beschaffung, Verdrahtungsautomaten) und Autodesk Vault.

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
