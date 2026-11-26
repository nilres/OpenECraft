# Artikelverwaltung

Die Artikeldatenbank, die sich alle Projekte teilen (EPLAN-Stil): kaufmännische,
technische und Montagedaten, Funktionsvorlagen, Anschlusspunkte, 2D- / 3D-Grafik und der
Import von Herstellerdaten. Zwischen Installationen werden Artikel über den
[Artikelserver](../parts-server.md) geteilt.

## Artikeldatenbank

Eine globale **Artikeldatenbank**, die sich alle Projekte teilen (Baum aus
Produkthauptgruppe / Gruppe / Untergruppe, Filter), mit allgemeinen, kaufmännischen
(Preis, Preiseinheit, Verpackungseinheit, Mengeneinheit), technischen und Montagedaten
(Abmessungen, Gewicht), Zubehör und einer **Funktionsvorlage** je Artikel (z. B. Spule
A1/A2, Hauptkontakte, 13/14 Schließer). Die Datenbank wird
im App-Speicher (Desktop) / lokalen Speicher (Browser) gespeichert.

- **Zubehör**, das nur mit anderen Artikeln funktioniert (Hilfs- und Meldekontakte,
  Kontaktblöcke, Registerkarte *Zubehör*): Ein Artikel listet sein Zubehör, ein Zubehör
  kann die Artikel nennen, zu denen es passt (*Passt zu diesen Artikeln*) – jede Seite
  stellt das Paar her, sodass ein Zubehör vielen Artikeln dient und die Daten eines
  Herstellers eines hinzufügen können, ohne dessen Hauptartikel zu ändern (der
  Meldekontakt Hager CZ001 nennt den FI-Schutzschalter CCA216D, der CDA440D
  nennt den CZ001). Ein Zubehör ist als solches gekennzeichnet und sagt, wo es montiert
  wird: auf der Front (in der Tiefe gestapelt), auf der linken oder rechten Seite (neben
  dem Artikel auf der Schiene), unter dem Artikel als Sockel, auf dem er steht (das
  Bausteinmodul univers N unter dem
  NH1-Sicherungslasttrennschalter Hager LT153: Das Betriebsmittel übernimmt die
  Grundfläche des Moduls, der Artikel sitzt mittig davor), in den Artikel eingebaut (der
  Unterspannungsauslöser und der Hilfsschalter in den Steckplätzen des
  h3+-Leistungsschalters HNT161GR: kein eigener Platz, ihre Klemmen sind am Schalter, und
  ihre neben dem Schalter gezeichneten Spulen sind Funktionen des Zubehörs, keine zweite
  Hauptfunktion) oder gar nicht.
  Mehrere eines Sockelartikels liegen nebeneinander entlang der längeren Seite des
  Artikels (die Stromwandler Hager LVZW150, hinter den drei Polen einer
  NH-Sicherungslasttrennschalterleiste LVSG eingeklipst), verschiedene Sockelartikel aufeinander. In einem Projekt bietet die Registerkarte *Artikel* des Betriebsmittels das Zubehör an, das zu
  seinen Artikeln passt (*Zubehör hinzufügen*), und zeigt den Artikel, auf dem jedes
  sitzt; seine Funktionen kommen zu den Kontakten des Betriebsmittels hinzu; die Prüfung
  `accessories` meldet ein Zubehör an einem Betriebsmittel ohne einen Artikel, zu dem es
  passt; der Schaltschrankaufbau und die 3D-Ansicht zeichnen es an seinem Platz mit
  eigenen Klemmen, und das Betriebsmittel wird breiter oder tiefer. Befehlszeile / MCP:
  `list_accessories`, `add_accessory`; Artikelserver: `GET /parts/{number}/accessories`;
  CSV-Spalten `Accessory`, `Accessory mounting`, `Fits`; BMEcat-Referenzen der Arten
  `accessories` und `base_product`; EPLAN-XML `P_ARTICLE_ISACCESSORY`.
- **Baugruppen**: Ein Artikel kann aus Komponentenartikeln mit Mengen bestehen
  (Registerkarte *Zubehör*, CSV-Spalte `Components`: `NUM:2|NUM2`); die Stückliste führt
  sie unter der Baugruppe auf (verschachtelte Baugruppen aufgelöst, im Preis enthalten).
- **Typisierte Montagedaten** je Artikel (Registerkarte *Montage*): Abstände (oben, unten,
  links, rechts, vorne, EMV, Serviceraum), Befestigungsbohrungen mit Gewinde, Tür- und
  Flächenausschnitte (mehrere je Artikel, jeder rund oder rechteckig mit seiner Mitte ab
  der Mitte der Frontansicht, einer Drehung und einem Eckenradius: Tabelle und Vorschau
  in der Registerkarte, Textform
  `197 x 141 @ 0/-6.5 R3 15°; Ø16 @ 80/60`), Verlustleistung, wie weit ein Zubehör übersteht, Befestigung, Flächen, Kollisionsklasse und
  Klemmentyp. Sie steuern die Schaltschrankprüfungen, das Bohren, die Ausschnitte und die
  Klimaberechnung; CSV hat eine Spalte je Wert (`Power loss`, `Mounting holes`, …).
  Ältere Artikel mit diesen Werten als Zeilen `Key: value` in den technischen Daten
  funktionieren weiterhin; Importe und der mitgelieferte Katalog verschieben solche
  Zeilen in die typisierten Felder, der Artikeleditor bietet es für vorhandene Artikel
  an.
- **Anschlusspunkte** je Artikel (Bezeichnung, Position in der Frontansicht, die
  Richtung, in der der Draht abgeht); Artikel ohne eigene erhalten sie aus den
  Funktionsvorlagen nach der IEC-Konvention (ungerade Bezeichnungen oben, gerade unten,
  Türgeräte von hinten, SPS-Module unten, Überspannungsableiter mit der Erde und
  dem Fernmeldekontakt unten). Die Betriebsmittel der mitgelieferten Kataloge,
  die nicht so verdrahtet werden (Victron, Pylontech, Beckhoff, Netzteile von Phoenix
  Contact), und die 19-Zoll-Geräte (Netzeingang hinten) haben eigene; erzeugte
  Modelle zeichnen ihre Klemmen an den Anschlusspunkten. Bearbeitet im Artikeleditor
  (Registerkarte *Anschlusspunkte*: Tabelle, in die Frontansicht klicken, um einen zu
  platzieren); der Schaltschrankaufbau
  verdrahtet zu ihnen; CSV-Spalte `Connection points`: `A1:22.5/0/55/up|A2:…`.

## Grafik und 3D-Modelle

2D-Frontansicht (SVG) und 3D-Modell je Artikel:

- **STL**, **OBJ**;
- **STEP**: Volumen- und Flächenmodelle aus AP203- / AP214- / AP242-Dateien, nach ihrer
  Baugruppenstruktur platziert (auch wo die Körper eines Teils an einer weiteren
  Darstellung hängen, wie in den CAD-Modellen von CADENAS, etwa denen von Rittal),
  tesselliert und in mm umgerechnet;
- **IGES 5.3**: Volumenkörper und getrimmte Flächen;
- **Parasolid** (`.x_t` / `.x_b`): Körper aus Transmit-Dateien ab V13, Ebenen,
  Zylinder, Kegel, Kugeln, Tori und B-Flächen exakt tesselliert, Verrundungen und andere
  prozedurale Flächen aus ihrer Berandung gefüllt;
- **IFC**: tessellierte, facettierte und extrudierte Körper aller Produkte, platziert und
  in mm umgerechnet;
- **JT**: die Tri-Strip-Netze von JT-8-Dateien.

Das Modell der Montageplatte eines Gehäuses, eine eigene Datei wie die Datei `CAD_MPL_…`
der CAD-Modelle von Rittal, wird mit *Ihr Modell importieren…* unter *Montageplatte*
gelesen: Seine Breite und Höhe werden die Artikeldaten `Mounting size: 699 x 1896`, und
ein Gehäuse des Artikels erhält eine Platte dieser Größe, mittig, sofern der Artikel
keinen `Mounting origin` nennt.

Artikel mit Abmessungen, aber ohne Modell erhalten ein erzeugtes, je nachdem, was sie
sind: Reiheneinbaugeräte mit ihren Klemmen, Kipphebel oder Anzeige, Klemmen in ihren
Farben (PE
grün, N blau), SPS-Module, Sicherungslasttrennschalter, wandmontierte Elektronik mit
Anzeige, Kabelverschraubungen (Sechskantkörper, Hals und Hutmutter, aus vernickeltem
Messing oder Polyamid
nach Material und Farbe), 19-Zoll-Geräte und Rack-Batterien, PV-Module, Pumpen, Motoren,
Behälter, Wärmepumpen, Kompressoren, Kältetrockner, Ventile mit Hebel oder Antrieb,
Rohrverbinder, Verteiler und Wartungseinheiten. Betriebsmittel der mitgelieferten Kataloge
werden nach ihren Produktfotos gezeichnet: Victron-Wechselrichter mit ihren schwarzen
Eckkappen
und dem orangefarbenen Band, Cerbo GX und SmartShunt, der Lynx-Verteiler, Pylontech-Rack-Batterien
mit Griffen und Klemmen, Hager-Sicherungslasttrennschalter, Zähler und
Reiheneinbaugeräte, DEHN-Ableiter mit ihren Steckmodulen, Beckhoff-EtherCAT-Klemmen,
 Netzteile von Phoenix Contact, Motorschutzschalter und
Schütze von Siemens. Erzeugte Modelle sind eingefärbt (eine Farbe
je Dreieck, `colors` des Netzes); importierte nehmen die Farbe, die die 3D-Ansicht
dem Betriebsmittel gibt. Bodenstehende Artikel stehen aufrecht auf dem Boden eines Raums.

## Artikel im Projekt

- **Projektartikel**: Artikel werden beim Zuweisen in das Projekt kopiert; Projekt ←
  Datenbank synchronisieren, Projekt → Datenbank übernehmen, nicht verwendete
  Projektartikel entfernen.
- Dialog **Artikelauswahl** mit *Gerätewahl* (nur Artikel, die die Hauptfunktion des
  Betriebsmittels bereitstellen); mehrere Artikel je Betriebsmittel mit Menge (Schütz +
  Hilfsschalterblock).
- Funktionsvorlagen steuern **Reservekontakte** im Kontaktspiegel, „Bezeichnungen
  übernehmen“ (Anschlussnummern aus dem Artikel) und die Projektprüfung (mehr
  Funktionen platziert, als die Artikel bereitstellen, im Projekt fehlende Artikel).

## Import und Export

- **CSV** (durch `;` getrennt, Kopfzeilennamen) und **JSON**, Import und Export.
- **BMEcat**-Kataloge (1.2 / 2005, wie sie Hersteller und Großhändler veröffentlichen,
  z. B. aus dem EPLAN Data Portal oder aus ETIM- / eCl@ss-Datenpools): Identifikation,
  Beschreibung, Preis, Datenblatt, die ETIM- / eCl@ss-Klasse und jedes Merkmal als
  technische Daten; Abmessungen, Gewicht, Bemessungsstrom / -spannung und Verlustleistung
  gehen in ihre Felder (nach Merkmalsname oder den ETIM-Codes für Breite, Höhe und Tiefe).
- **EPLAN-Artikel-XML**: Der Export der Artikelverwaltung von EPLAN (`<part>` mit
  `P_ARTICLE_*`-Attributen und Varianten, mehrsprachigen Texten, Produktgruppen nach ihren
  Austauschnummern) wird wie die anderen Formate importiert, und Artikel werden für den
  Rückweg als EPLAN-Artikel-XML exportiert (Artikelverwaltung,
  `parts_export --format eplan_xml`). Funktionsvorlagen
  werden Funktionsvorlagen (SPS-Ein- und -Ausgänge als SPS-Funktionen),
  Anschlusspunktdefinitionen Anschlusspunkte (Anschlusspunkte an Steckern als
  `X10.1`) mit den Drähten, die ihre Klemmstellen aufnehmen (kleinster und größter
  Querschnitt in mm², aus AWG umgerechnet, wo nur das angegeben ist, und die größte Zahl
  von Drähten), Bohrbilder
  Befestigungsbohrungen: Durchgangsbohrungen (`Ø3.3`) oder Gewindebohrungen (`M4`),
  während eine rechteckige
  Position oder eine Bohrung, die größer als eine Schraube ist, ein Ausschnitt des
  Artikels ist, mit Platz und
  Drehung (`197 x 141` eines HMI-Panels, 6,5 mm über der Mitte seiner Front); die
  Bohrdatensätze werden unverändert mit dem Artikel aufbewahrt (ihr Untertyp und ihre
  dritte Dimension werden noch nicht
  ausgewertet). Die Schienendaten von EPLAN (Rasthöhe, Mittenversatz) oder ein Text, der
  die Schiene nennt
  (*DIN rail*, *Hutschiene*, *TS 35*, *TS 15*, *35 x 15*, *G 32*), machen den Artikel zu
  einem für
  eine Tragschiene dieser Größe (*Tragschiene 35 mm*).
- **EDZ** (Archive des EPLAN Data Portal, 7-Zip oder ZIP, wie ABB sie veröffentlicht): Das Artikel-XML darin wird wie oben
  gelesen, zusammen mit Abständen, Verlustleistung und dem Link zum Datenblatt.
  Eingebettete 2D-Fenstermakros werden Projektsymbole. Das Produktbild wird als kleines
  JPEG (höchstens 192 px) mit dem Artikel aufbewahrt und in den Artikeldetails mit seinen
  Montagedaten
  (Montage, Bohrungen, Ausschnitt, Abstände) und den Drähten gezeigt, die seine
  Anschlusspunkte aufnehmen.
  Ein 3D-Makro des Artikels (EPLAN Pro Panel, `…_3D.ema`, `P_ARTICLE_MACRO`) gibt ihm
  sein 3D-Modell, wenn er keines hat, auch für das Artikel-XML einer entpackten EDZ-Datei
  oder eines
  Herstellerordners; ein Makro, dessen Körper in einer Form vorliegen, die diese Version
  nicht liest, wird
  in der EPLAN-Diagnose des Artikels gemeldet.
  Parts Manager und `parts_import` speichern auch die Originaldateien als Anhänge. Die
  Artikeldatensätze des Projekts und des Servers tragen die umgewandelten Definitionen
  und die Diagnose.
  Das Artikel-XML einer entpackten EDZ-Datei (`items/partxml/…part.xml`) wird genauso
  gelesen: mit den Dateien für Anschlusspunkte und Bohrbilder daneben und den Makros
  und Bildern unter `items/`.
- **Gehäusepakete** (ZIP-Archive von Gehäusekonfiguratoren wie dem
  Rittal-Konfigurator): Die Stückliste (`bill-of-material.csv`) liefert die Artikel, der
  erste ist das konfigurierte Gehäuse (Herstellerpräfix und Name aus dem
  EPLAN-Makro, z. B. `RIT.1360000`, Größe aus *WHD: 600x600x350 mm*, GTIN, Konfigurations-ID
  und Kompaktgehäusesystem in seinen technischen Daten) mit den weiteren Zeilen als seinen
  Komponenten. Die Vorschau des EPLAN-Pro-Panel-Makros wird sein Bild. Die
  Körper des Makros (Gehäuse, Tür, Montageplatte, Flanschplatte, Scharniere und Schloss)
  werden sein 3D-Modell, aufrecht gestellt mit der Front in Richtung +z; ein Makro ohne
  Körper überlässt es dem STEP-Modell, ohne dessen Sperrflächen für Ausschnitte und
  Platzierung
  (Parts Manager und der Editor; `parts_import` hängt nur die STEP-Datei an). Die
  Front der Montageplatte (oder sonst die größte zur Front gewandte Fläche in der
  hinteren
  Hälfte des Gehäuses) liefert die Artikeldaten `Mounting origin: 25/12.5/20` und
  `Mounting size: 550 x 575`, die Seite der Scharnierblöcke neben der Tür
  `Door hinge: right`, sodass
  ein Gehäuse des Artikels diese Platte erhält (siehe
  [Schaltschrankaufbau](panel-layout.md)).
  Die Sperrflächen des Makros werden die *Sperrflächen* des Artikels (Montagedaten,
  `placing plate 0/0 550 x 20; cutout left 10/20 30 x 40`: Art, Fläche, Platz und
  Größe in den Koordinaten der Fläche), jede auf der Fläche, in deren Ebene sie liegt;
  die auf
  dem Rahmen um die Tür werden weggelassen. Der Schaltschrankaufbau zeichnet und prüft sie
  (siehe
  [Schaltschrankaufbau](panel-layout.md#sperrflächen)). Das Makro, das EPLAN-Projekt
  (`.zw1`), die STEP-Datei und die PDF-Dokumente werden als Anhänge gespeichert.
- **Ein schon bekannter Artikel:** eine EDZ-Datei, ein EPLAN-Artikel-XML oder ein
  Gehäusepaket, dessen Artikel schon in der Artikeldatenbank, auf dem Artikelserver
  oder in einem mit OpenECraft gelieferten Katalog ist, wird mit diesem Artikel
  zusammengeführt, statt ihn zu ersetzen (Editor, Parts Manager und `parts_import`).
  Der bekannte Artikel ist der mit derselben Nummer, sonst der mit derselben
  Bestellnummer vom selben Hersteller (oder der einzige Artikel mit dieser
  Bestellnummer); er behält seine Nummer, und die Dateien und Verweise des Imports
  folgen ihm. Die Werte des Imports gewinnen; was ihm fehlt, bleibt, etwa die
  Artikeldaten `Fields`, `Rows` und `Module spaces` der Schränke und Felder von
  Striebel & John. Gruppe und Untergruppe bleiben, die technischen Daten werden Zeile
  für Zeile zusammengeführt (das `Schlüssel: Wert` des Imports für denselben
  Schlüssel), Zubehör wird vereinigt. CSV-, JSON- und BMEcat-Dateien ersetzen die
  Artikel ihrer Nummern weiterhin. Parts Manager und `parts_import` laden eine große
  Datei in mehreren Anfragen hoch; eine Datei darin, die größer ist, als der
  Artikelserver annimmt (32 MB, etwa manche 3D-Makros von ABB), wird nicht gespeichert
  und genannt, der Artikel behält das daraus gelesene Modell.

### Ein EPLAN-Symbol platzieren

**Artikel importieren** öffnen und die EDZ-Datei oder das Artikel-XML des Herstellers auswählen; ein Kasten unten rechts meldet das Ergebnis (Nummer und Beschreibung des Artikels oder die Zahl der Artikel) und schließt sich nach einigen Sekunden oder bei einem Klick. Auf dem Desktop kann das Artikel-XML ein begleitendes EMA aus demselben Verzeichnis laden. Unter **Einfügezentrum → Betriebsmittel** den Artikel suchen, auswählen und dann vor dem Platzieren die Variante wählen. Jede Zeile hat eine Vorschau und listet nicht aufgelöste Eigenschaften oder Bibliotheksverweise. Ein loses EMA lässt sich auch mit **Symbolbibliothek importieren** öffnen. Ausdrücklich gekennzeichnete Fenster mit mehreren Betriebsmitteln lassen sich als native Makros platzieren.

Der Decoder liest PXF-Fenstermakros, eingebettete Zeichnungsblöcke, affine Transformationen, einfache Vektorgrafik, Text und ausgewählte Eigenschaftsfelder. Seine eingebauten SPECIAL-Zuordnungen decken die SPS- und Gerätekontakte 13, 17 und 20, die Stecker- und Buskontaktformen 34, 35 und 37, die zweiseitige Form 22 mit gemeinsamer Bezeichnung und die Übersichtsform 40 ab. Diese Formen verwenden die Kontaktzeichnungen der EPLAN-IEC-Symbolübersicht von 2007, S. 1–2. Physische XYZ-Anschlusspunkte gehören weiterhin zum Schaltschrankmodell.

Der Importer liest lokale Pin-Slot-Verweise und in Anführungszeichen gesetzte Verweise zur Potenzialweiterleitung über Stecker/Bezeichnung. Wechselseitige, eindeutige Verweise zwischen zugeordneten statischen Kontakten desselben Betriebsmittels erzeugen native Brücken. Einseitige Regeln, fehlende Ziele, unsichere Betriebsmittelidentitäten und nicht unterstütztes Betriebsmittelverhalten bleiben Diagnosen. Der Importer hält Autoverbinden-Unterbrechungen, Kabel-/Schirm-Metadaten und Verbindungsdefinitionspunkte getrennt von Leitungsstücken. Das Quell-XML enthält nicht unbedingt eine verlegte Drahtliste von Quelle zu Ziel. [EPLAN-Anschlusslogik](https://www.eplan.help/en-us/Infoportal/Content/Plattform/2022/Content/htm/xfctdefbrowsergui_d_anschlusslogik.htm)

**Nicht aufgelöste Platzierungen** einer Artikelvariante aufklappen, um den Master, das Quellobjekt sowie die Stecker- und Klemmenbezeichnungen zu sehen. Für eine Datei oder einen ganzen Herstellerordner gibt `cargo run -p openecraft-edz --example inspect -- PATH --references` JSON-Zeilen mit nativen Pins, offenen Verweisen, Verbindungssteuerungen und dem Weiterleitungsstatus je Darstellung aus. Koordinaten an nicht aufgelösten Platzierungen sind Einfügepunkte. Eine geprüfte native Definition mit dem Schlüssel `user:eplan-lib:LIBRARY:NUMBER:VARIANT` liefert ausdrückliche Pin-Anker. Nicht registrierte PDF-Grafik kann zum Vergleich unter `user:pdf-review:LIBRARY:NUMBER` aufbewahrt werden und nimmt nicht an der Zuordnung teil.

Unvollständige Varianten lassen sich prüfen und platzieren. Fehlende Mastersymbole, nicht unterstützte Eigenschaften, Platzhalterzustände und andere nicht behandelte Objekte werden gemeldet. Eingebettete Ebeneneinstellungen sowie ausdrückliche Stiftbreiten und vordefinierte Farben werden übernommen. Eigene Paletten, exakte Strichmuster und Füllungen können von EPLAN abweichen. Die automatische Rekonstruktion der Verdrahtung eines vollständigen EPLAN-Projekts und die Umwandlung von 3D-Modellen liegen außerhalb des 2D-Decoders. Ein erneuter Import aktualisiert unbearbeitete Grafik mit kompatiblen Endpunkten. Lokale Änderungen und vorhandene Pin-Geometrien haben Vorrang vor einer widersprechenden Revision.
