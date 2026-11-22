# Klemmenleisten, Stecker, SPS und Beschriftung

Klemmenleisten mit Etagenklemmen und Zubehör, Stecker, SPS-Karten mit dem Austausch zu SPS-Programmierwerkzeugen und die Beschriftungen für die Werkstatt.

## Klemmenleisteneditor

Klemmenreihenfolge (nach oben / nach unten, sortieren), Neunummerierung 1 … n,
**Brücken** (gebrückte Klemmen teilen ihr Potenzial), Reserveklemmen (im Klemmenplan
und in der Stückliste);
jede Klemme hat ihren Artikel: PE-Klemmen nehmen den Artikel der Leiste mit den
technischen Daten `Terminal type: PE`, alle anderen deren Durchgangsklemmen-Artikel
(Stückliste, Länge der Leiste im Schaltschrank, Anschlusspunkte je Klemme);
**automatisches Zubehör**
aus den technischen Daten der Klemmenartikel (`End bracket: …`, `End plate: …`,
`Jumper: …` mit Brückenartikeln `Poles: n`): ein Endhalter an beiden Enden, ein
Abschlussdeckel nach der letzten Klemme jedes Artikels, Steckbrücken der kürzesten
passenden Länge (gekürzt, wenn länger, Zinken ausgebrochen bei Klemmen dazwischen) – in
der Stückliste, im Klemmenplan, im Schaltschrankaufbau (2D / 3D) und in der Prüfung;
das Fenster Klemmenleisten zeigt die Leiste auf der Schiene mit ihren Brücken und hat
einen Schalter für das automatische Zubehör.
**Etagenklemmen**: Eine Klemme auf Etage 2 oder höher sitzt auf dem Block der
Klemme davor (ein Artikel je Block, technische Daten `Levels: 2`); der Klemmenplan
listet jede Etage, der Aufbau auf der Schiene und die Stückliste zählen Blöcke,
und die Prüfung meldet mehr Etagen, als der Artikel hat. **Brückenarten**: Steckbrücken
(technische Daten `Jumper`), Sattelbrücken (`Saddle jumper`) und Drahtbrücken (jede
Etage, kein Artikel); Steck- und Sattelbrücken müssen auf einer Etage bleiben. Befehle:
`list_strips`, `get_strip`, `set_terminal`, `set_jumper_kind`, `set_strip`,
`renumber_strip`.

## Stecker

**Stecker** (*Betriebsmittel › Stecker…*, Symbolgruppe *Stecker*): Stift und Buchse
(gestecktes Paar, zwei Anschlusspunkte), Steckerstift und Steckerbuchse (je einer);
Kennzeichen = Stecker (`-X10`),
Bezeichnung = Pin. Stift und Buchse derselben Bezeichnung sind gesteckt: Das
Potenzial geht hindurch, die beiden Seiten sind getrennte Verbindungen
(Verbindungsnummern,
Kabel). Pins werden beim Platzieren je Stecker nummeriert; das Steckerfenster und der
Bericht *Steckerplan* listen die Pins in Steckerreihenfolge mit der Buchsenseite, der
Stiftseite und
dem Potenzial; Artikel mit Funktionen `Plug pin` geben die Pinzahl vor (unbenutzte Pins
werden als Reserve aufgeführt). Die Prüfung meldet doppelt platzierte Pins und mehr Pins,
als die Steckerartikel
haben; der KBL-Export nennt den Gehäuseartikel von Steckverbindern mit allen
Kammern des Artikels.

## Steckerbelegungen und Flachbandkabel

**Pinbelegung** (Steckerfenster, Export *Steckerbelegungen (SVG)* `pinouts`, Befehl
`get_pinout`): die Steckseiten der Stiftseite (male) und der Buchsenseite (female, das
Spiegelbild) mit jedem Pin des Steckerartikels an seinem Platz, gefüllt mit der Farbe
der Leitung oder Kabelader am Pin (Reserven leer), der Kodiernase und einer Pintabelle
mit
Potenzial, Zielen und Kabeladern; das Überfahren eines Pins zeigt seine Details, ein
Klick zeigt
sein Symbol. Die Steckseite kommt aus den technischen Daten des Steckverbinderartikels:
`Pin layout: circle 4` (Pins auf einem Kreis, Pin 1 rechts von der Nase oben, gezählt
gegen den Uhrzeigersinn auf der Stiftseite wie in IEC 61076-2-101), `circle 7+1`
(Mittelpins
zuletzt), `circle 9+3` / `circle 11+5+1` (ein innerer Ring, dann die Mitte), `row 8`,
`zigzag 10` (zwei Reihen, ungerade Pins über geraden Pins: IDC), `grid 2x5` (zeilenweise
nummeriert), `cols 20x2` (spaltenweise nummeriert), `dsub 9`, jeweils wahlweise `+PE`; für
jede
andere Steckseite die Pinmitten in mm: `points -3.5,2 3.5,2 0,-2` (rechteckige
Steckseite) oder
`round 0,9 -8,-3 8,-3` (runde Steckseite mit Nase). `Pitch`, `Coding` und `Gender`
(`male`,
`female`, `hermaphroditic`) vervollständigen sie. Nummerierte Pins eines kleineren
Artikels sitzen an ihrer
Nummer (M8 3-polig: 1, 3, 4); Stecker ohne Layout werden in einer Reihe gezeigt.

Der Katalog enthält, jeweils als Stift und Buchse:

| Familie | Artikel |
| --- | --- |
| Sensor / Aktor | M5, M8 (3 – 8 Pins), M12 A, B (PROFIBUS), D, X (10-Gbit/s-Ethernet) und die Leistungskodierungen S, K, T, L, 7/8" (3 – 6 Pins) |
| Rund | M16 nach IEC 61076-2-106 (3 – 24 Pins), M23 Signal (12, 17, 19 Pins), M23- und M40-Leistungssteckverbinder für Servomotoren, Rundsteckverbinder GX12 / GX16 / GX20 |
| Ventile | DIN EN 175301-803 Bauformen A, B und C, 2 + PE und 3 + PE |
| Rechteckig | Einsätze für schwere Steckverbinder der Baureihen A, D, E und Q mit PE, D-Sub 9 – 37, DIN 41612 Bauform C (32, 64, 96), SPS-Frontstecker (20, 40) |
| Leiterplatte | IDC-Buchsen und Wannenstecker 10 – 64, einreihig 2,5 mm, zweireihig 3,0 und 4,2 mm, steckbare Klemmenblöcke 3,5 / 5,08 / 7,62 mm |
| Daten | RJ11, RJ12, RJ45, BNC |
| Mobile Maschinen | abgedichteter Typ DT (2 – 12 Pins) und 1,5-mm-System (1 – 6 Pins) |
| Leistung | CEE IEC 60309 16 / 32 / 63 A mit 3, 4 und 5 Polen, Gerätesteckvorrichtungen IEC 60320 C5 – C20, Schuko, BS 1363, NEMA 5-15, L14-30, L21-30, Installationssteckverbinder (3 – 5 Pole), PV-Steckverbinder, Batteriesteckverbinder (hermaphroditisch, 50 – 175 A), Sicherheits-Bananenstecker, Thermoelementsteckverbinder |

**Flachkabel** (`Cable form: Flat` im Kabelartikel: Flachbandkabel 10 – 64 Adern mit
Raster 1,27 mm, Flachkabel für Leitungswagensysteme): `connect_flat_cable` macht die
Leitungen an
einer Seite eines Steckers zu Adern des Kabels, Pin n nimmt Ader n. Flachbandkabel
(Flachkabel
mit einem `Pitch`) zeigen auf den Steckseiten graue Adern mit Ader 1 in Rot. Die Prüfung
*Steckerbelegungen und Flachbandkabel* meldet Pins, die der Steckerartikel nicht hat,
Adern eines Flachkabels
an einem IDC-Steckverbinder (`Termination: IDC`), die auf einem anderen Pin landen, und
Flachkabel
mit einer anderen Aderzahl, als der IDC-Steckverbinder Pins hat.

## Navigator für Leisten und Stecker

**Navigator für Leisten und Stecker** (Navigator-Registerkarte *Klemmleisten / Stecker*):
die Klemmenleisten mit
ihren Klemmen in Schienenreihenfolge (Etagen von Etagenklemmenblöcken) und Reserven, die
Stecker mit
ihren Pins und Zielen sowie die unbenutzten Pins der Steckerartikel; ein Klick zeigt
ein platziertes
Element, Reserven und unbenutzte Pins werden in die Seite gezogen.

## SPS-Navigator

**SPS-Navigator** (Navigator-Registerkarte *SPS*): die Karten mit ihren Kanälen in
Adressreihenfolge, platzierte mit Funktionstext und angeschlossenen Betriebsmitteln (ein
Klick zeigt das
Symbol), freie Kanäle von Karten mit einer Gerätebeschreibung zum Ziehen in die Seite.

## SPS-Übersicht

Die importierte Karte einmal unter **Einfügezentrum → Betriebsmittel** auswählen, dann für jeden weiteren Kanal auf das Blatt klicken. Bei einer EL1008 teilen sich die Klicks eins bis acht ein Kartenkennzeichen. Klick neun beginnt Kanal 1 auf einer weiteren Karte. Esc beendet das Platzieren. **Darstellungsarten** aufklappen, wenn eine bestimmte Verdrahtungsvariante gebraucht wird.

Ein loses EMA mit erkannten Kanaldefinitionen hat einen Eintrag unter **Symbole** und verwendet dieselbe Platzierungsfolge. Für einen Katalogartikel, die Stücklistenzuordnung und die Schaltschrankmaße dessen Artikel-XML importieren.

Analoge Beckhoff-Makros mit einem erkennbaren Signal und dessen Begleitklemmen ergeben einen SPS-Punkt. Bei der EL3174 verwendet der Spannungseingang `U1` Klemme 1 für das Signal und Klemme 9 für dessen Bezug. Beide Klemmen lassen sich getrennt verdrahten. Darstellungen im Strommodus sind Alternativen für dieselben vier Kanäle. Über das Einfügezentrum wird eine bestimmte Darstellung gewählt. Die SPS-Übersicht meldet die Signalklemme, und das Neuadressieren der Karte lässt analoge Kanalbeschriftungen unberührt. TwinCAT-Variablendeklarationen behalten den gelieferten Datentyp. Kanäle ohne deklarierten Datentyp bleiben auf dem Blatt und in der Übersicht verfügbar.

Für Beckhoff EL1008 und EL2008 das Artikel-XML mit dem passenden EMA daneben importieren. Einen Artikel im Schaltplan platzieren, um seinen nächsten verfügbaren Kanal zu verwenden, oder im Einfügezentrum einen bestimmten allpoligen Kanal wählen. Kanäle aus derselben Makrofamilie teilen sich ein Kartenkennzeichen. Sind alle acht platziert, beginnt das nächste Standard-Ablegen eine weitere Karte. Der SPS-Navigator listet nach der ersten Platzierung die verbleibenden Kanäle. Die SPS-Übersicht bearbeitet Adressen und Funktionstexte. Das Neuadressieren verwendet die importierte Kanalreihenfolge, auch wenn Punkte außer der Reihe platziert werden. Zum Beispiel verwendet Kanal 2 die physische Klemme 5 und Bit 1. Für eine weitere Karte im selben Adressraum ein anderes Startbyte einstellen. Der Koppler EK1100 behält seine Versorgungs- und Busanschlüsse.

Alle Ein- / Ausgänge je Karte mit angeschlossenen Betriebsmitteln, bearbeitbaren
Adressen und Funktionstexten, Startbyte / Baugruppenträger / Steckplatz der Karte,
Neuadressierung (der
digitalen Punkte; analoge behalten ihre Wortadressen), **Export / Import der
Symboltabelle** (CSV: Name, Datentyp, Adresse, Kommentar) für SPS-Programmierwerkzeuge,
die Variablentabelle von **Siemens TIA Portal** (Excel-Blatt *PLC Tags*), der
Tag-Import von **Rockwell Studio 5000** (Aliase der Modulkanäle), die Symboltabelle
von **Siemens STEP 7 classic** (SDF), eine globale Variablenliste für **Beckhoff TwinCAT 3**
(TcGVL, `AT %I*` / `AT %Q*`), Variablen für **Schneider Control Expert** (XSY, `EBOOL`
mit topologischen Adressen), Operandenkommentare für **Mitsubishi GX Works** (CSV, `X` /
`Y`
oktal) und globale Variablen nach IEC 61131-3 (CODESYS); Adressen,
die auf mehreren Karten verwendet werden, werden gemeldet.

## Beschriftung

**Beschriftung** (*Verbindungen › Beschriftung…*: Text der Leitungsmarkierer, Material je
Art mit Stückzahlen
und Export; *Datei › Export › Beschriftung exportieren (CSV)…*, Bericht *Beschriftung*): Betriebsmittelschilder (Bezeichnungsschilder
für Türgeräte; der Ort des Gehäuses wird aus dem Kennzeichen weggelassen),
Klemmenmarkierer
in Schienenreihenfolge mit dem Leistenschild, Leitungsmarkierer (Verbindungsnummer an
beiden Enden oder
Zielbeschriftung: jedes Ende zeigt das andere Ende), Kabelmarkierer; gruppiert nach
Beschriftungsmaterial in der Reihenfolge des Anbringens, als CSV für
Beschriftungssoftware (UTF-8,
`;`, Kopfzeile `Material;Category;Text;Count;Enclosure;Group`); das Material steht in der
Stückliste. Texte und Materialien sind Projekteinstellungen (`marking`).
**Beschriftungsschemata** wählen die Markierer, die Spalten und ihre Reihenfolge, das
Trennzeichen,
die Kopfzeile und eine Zeile je gedrucktem Schild, für den CSV- / Text- und
Excel-Export; mitgelieferte
Importtabellen für Phoenix Contact CLIP PROJECT, Weidmüller M-Print PRO, Brady,
Murrplastik und Partex (Ausgangspunkte – kopieren und an die Programmversion anpassen),
eigene Schemata werden im Projekt gespeichert (Befehle `marking_schemes`,
`set_marking_scheme`, Exporter `marking` / `marking_xlsx` mit `scheme`).
