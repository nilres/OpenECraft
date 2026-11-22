# Schaltplaneditor

Seiten, Betriebsmittel und ihre Struktur, mitlaufende Querverweise, Bearbeitungswerkzeuge, Navigatoren und Projektspeicherung. Die [Benutzeroberfläche](../user-interface.md) listet die Befehle und Tasten.

![Der Schaltplaneditor mit einem Stern-Dreieck-Hauptstromkreis, dem ausgewählten und hervorgehobenen Potenzial L1 und dessen automatischer Aderfarbe und Querschnitt](../../images/schematic.png)

## Projekte mit Seiten

Allpolige Schaltplanseiten (A3, 10 Spalten / 6 Zeilen, Schriftfeld),
Deckblatt und erzeugte Berichtsseiten. Seitennavigator mit Umsortieren.

## Betriebsmittelorientiertes Arbeiten

Betriebsmittelkennzeichen (`-K1`, `-S3`, …) werden automatisch vergeben.
Hauptfunktion (Spule) + Hilfsfunktionen (Kontakte) bilden ein Betriebsmittel. Eine Spule
auswählen und Kontakte platzieren: Sie werden ihr zugeordnet und nummeriert (13/14,
21/22, …).

## Geräteboxen und Geräteanschlusspunkte

Netzteile, Messgeräte und Steuerbaugruppen ohne Bibliothekssymbol:
*Einfügen › Gerätebox* zeichnet einen gestrichelten Kasten beliebiger Größe als neues
Betriebsmittel (`-A1`, mit Funktionstext und Artikeln wie jedes andere Betriebsmittel),
*Einfügen › Geräteanschlusspunkt* platziert seine Anschlüsse Klick für Klick. Ein Punkt,
der auf dem Umriss einer Box platziert wird, gehört zum Betriebsmittel der Box, führt
von der Kante weg, auf der er liegt, und wird fortlaufend nummeriert (1, 2, 3 …; auf
`13` folgt `14`); seine Bezeichnung (`L`, `N`, `PE`, `+24V`, …) wird in seinen
Eigenschaften bearbeitet. Jeder Punkt ist ein eigener Anschluss, anders als bei Klemmen:
Zwei Punkte einer Box werden durch die Box nie verbunden, sie erscheinen als
`-A1:L`, `-A1:N` in den Verbindungs- und Geräteanschlussberichten, und die Prüfung meldet
eine Bezeichnung, die an einem Betriebsmittel doppelt verwendet wird. Die Eigenschaften
eines Punkts nennen seine Gerätebox (eine andere Box weist ihn jenem Betriebsmittel zu);
ein Punkt abseits seiner Box zeigt sein Betriebsmittelkennzeichen, auf seiner Box zeigt
es die Box. Seine Bezeichnung lässt sich an jede Stelle ziehen und links, zentriert oder
rechts ausrichten. Die Eigenschaften einer Box legen ihre Breite und Höhe fest: Punkte
auf einer Kante, die sich bewegt, gehen mit, ihre Leitungen folgen. Verschieben, Drehen,
Kopieren, Ausschneiden, Löschen und Speichern als Makro nehmen eine Box mit ihren
Punkten; Kopien werden ein neues Betriebsmittel mit eigenen Punkten. Befehle:
`add_device_box`, `update_device_box`,
`add_device_connections` (mehrere in einem Abstand, Bezeichnungen angegeben oder
fortlaufend nummeriert), `update_device_connection`, `list_device_boxes`.

Eine Box kann auch ein anderswo platziertes Betriebsmittel fortsetzen: Man gibt ihr das
Kennzeichen des Betriebsmittels und wählt ihre *Rolle* in den Eigenschaften (`role` von
`add_device_box` und `update_device_box`). Eine *Erweiterung* fügt dem Betriebsmittel
Anschlusspunkte hinzu, für ein Betriebsmittel, das für eine Box zu groß ist, oder ein
Bibliothekssymbol, dem einige Anschlüsse fehlen. Ihre Punkte sind neue Anschlüsse, und
eine Bezeichnung, die das Betriebsmittel bereits hat, wird als doppelt verwendet
gemeldet. Eine *Anschlussansicht* zeigt einige Anschlusspunkte, die das Betriebsmittel
bereits hat, zum Beispiel nur die Versorgung einer SPS-Karte auf der Einspeisungsseite;
man hakt sie in den Eigenschaften an (`show_device_pins`) oder platziert Punkte mit
ihren Bezeichnungen. Ein Punkt auf einer Ansicht ist der Anschlusspunkt, den er zeigt:
Dort gezeichnete Leitungen gehören auf jeder Seite zum Potenzial dieses Punkts, Berichte
führen den Anschluss einmal auf, vom eigenen Punkt des Betriebsmittels aus, und die
Prüfung meldet die Ansicht eines Punkts, den das Betriebsmittel nicht hat. Keine der
beiden Rollen ist eine zweite Hauptfunktion. Die Box zeigt einen Querverweis auf das
Betriebsmittel und die Hauptfunktion des Betriebsmittels auf ihre Boxen; ohne das
Betriebsmittel kopiert, bleiben diese Boxen bei ihm. Eine Box mit einer Rolle, aber ohne
platziertes Betriebsmittel wird gemeldet.

## Strukturkennzeichen (IEC 81346)

Seiten tragen eine Funktion (`=A1`) und einen Ort
(`+CAB1`). Betriebsmittel, die auf einer Seite platziert werden, erhalten deren Struktur
(`=A1+CAB1-K1`); auf der Seite zeigen Kennzeichen und Querverweise nur, was von der
Seitenstruktur abweicht (EPLAN-Stil), Kennzeichen werden je Struktur nummeriert,
Kopieren / Einfügen verschiebt Betriebsmittel in die Zielstruktur, und das Ändern einer
Seitenstruktur kann ihre Betriebsmittel mitnehmen. Der Betriebsmittelnavigator gruppiert
nach Struktur; das Schriftfeld zeigt den vollständigen Seitennamen (`=A1+CAB1/3`).

**Strukturkästen** (*Einfügen › Strukturkasten*) geben einem Bereich einer Seite eine
eigene Funktion, einen eigenen Ort (oder Aufstellungsort, Produkt): ein Rechteck zeichnen
und in den Eigenschaften die Aspekte einstellen, die es überschreibt; leere Aspekte
werden von der Seite oder einem umschließenden Kasten geerbt, verschachtelte Kästen
lösen zum innersten auf. Ein Betriebsmittel gehört zu dem Kasten, in dem der Einfügepunkt
seiner Hauptfunktion liegt: Es erhält die Struktur des Kastens (`=SYS+FIELD-S1` auf einer
Seite `=SYS+CAB1`), sein Kennzeichen auf der Seite lässt weg, was der Kasten zeigt, und
sein vollständiges Kennzeichen ist das, was der Navigator, die Nummerierung, die
Querverweise und die Berichte verwenden. Ein Aspekt eines Kennzeichens, der gleich der
Struktur um das Symbol ist, ist geerbt, jeder andere ist der eigene des Betriebsmittels:
Das Verschieben der Hauptfunktion in einen Kasten hinein oder aus ihm heraus oder das
Verschieben, Ändern der Größe, Ändern oder Löschen des Kastens benennt das Betriebsmittel
(alle seine Funktionen) in seinen geerbten Aspekten im selben Rückgängig-Schritt um und
behält seine eigenen; ein Kennzeichen, das bereits existiert, wird abgelehnt und
gemeldet, statt zwei Betriebsmittel zu verschmelzen. Die Symboleigenschaften zeigen,
woher jeder Aspekt kommt. Kästen werden mit ihren Betriebsmitteln kopiert, eingefügt und
in Makros gespeichert; die Prüfung
*Strukturkennzeichen* meldet Kästen, die sich überlappen, ohne dass einer im anderen
liegt, und Betriebsmittel in einem Kasten, deren Kennzeichen für einen Aspekt, den der
Kasten setzt, einen anderen Wert hat.
Befehle `add_structure_box`, `update_structure_box`, `list_structure_boxes`.

## Mitlaufende Querverweise

Kontakte zeigen, wo ihre Spule ist (`/3.4`), Spulen zeigen einen
Kontaktspiegel mit allen Kontakten und deren Orten, Unterbrechungspunkte verweisen auf
ihre Gegenstücke auf anderen Seiten. *Verbindungen › Querverweise…* legt das Format fest
(Präfix, Trennzeichen, Zeile: `/3.4`, `3-4`, `/3.4.B`), wann die Struktur der Zielseite
gezeigt wird (wenn sie abweicht, immer, nie), worauf Unterbrechungspunkte verweisen (alle
anderen Punkte, Stern vom ersten Punkt aus oder Paare in Schaltplanreihenfolge; die
Prüfung meldet einen Punkt ohne Partner) und ob Spulen einen Kontaktspiegel zeichnen.
Befehl:
`set_cross_references`.

Der Name und der Querverweis eines Unterbrechungspunkts folgen dem Pfeil: bei
waagerechten Punkten der Name über und der Querverweis unter dem Leiter, am
Anschlusspunkt endend, sodass sie am Seitenrand innerhalb der Seite bleiben; bei
senkrechten Punkten neben dem Pfeil (Querverweis unter dem Namen). *Darstellung* in den
Eigenschaften setzt sie stattdessen über, unter, links oder rechts vom Pfeil, für alle
ausgewählten Punkte auf einmal (rechts von einem Pfeil nach rechts: in einer Zeile
hinter seiner Spitze, für Leiter im Abstand von 4 mm); ist ein Punkt
ausgewählt, lässt sich jeder Text für sich ziehen, ohne das Symbol zu verschieben, und
die Schaltfläche zum Zurücksetzen kehrt zur automatischen Platzierung zurück. Die Plätze
werden mit dem Projekt gespeichert und von den PDF-, SVG- und DXF-Exporten verwendet;
der Querverweis bleibt ein Link auf seine Seite.
Befehl: `place_interruption_texts`.

## Verbindungen / Potenziale

Leitungen, T-Abzweige (mit Verbindungspunkten), Klemmen,
Potenzialanschlusspunkte (L1, N, PE, +24V, …) und Unterbrechungspunkte werden
seitenübergreifend zu Netzen ausgewertet. Das Auswählen einer Leitung hebt ihr ganzes
Potenzial hervor.

## Bearbeiten

Auswählen / Fensterauswahl / Verschieben mit Leitungen, die folgen und rechtwinklig
bleiben, Drehen, Spiegeln, Kopieren / Ausschneiden / Einfügen (über die Zwischenablage,
Betriebsmittel erhalten neue Kennzeichen), Rückgängig / Wiederholen, Texte, Fangen am
Raster, Verschieben und Zoomen. Eine Leitung, die an etwas hängt, das sich bewegt,
behält ihre Richtung: Eine Ecke oder ein Abzweig an ihrem fernen Ende gleitet dort an der
Leitung entlang (ein Abzweig auf einer Schiene bleibt auf der Schiene), sonst erhält die
Leitung einen Versatz oder einen Weg um das, was im Weg ist, und eine Leitung, die nicht
rechtwinklig werden kann, ohne eine andere Verbindung zu berühren, bleibt schräg.
Verbindungsdefinitionspunkte und Kabeldefinitionslinien bleiben auf ihren Leitungen,
sofern sie nicht selbst verschoben werden; eine Linie, die nicht genau über ihren
Leitungen bleiben kann, bleibt, wo sie ist. Mit *Verbindungen verschobener Elemente neu
verlegen* (*Verbindungen › Leitungskreuzungen…*, standardmäßig aus) wird eine solche
Leitung zusammen mit den Ecken nach ihr neu verlegt, bis zum nächsten Anschlusspunkt,
Abzweig oder zur nächsten Verzweigung, auf dem Weg des geringsten Gewichts: ihre Länge in
Rasterschritten, 1,2 je Knick und 1,5 je gekreuzter Leitung; eine Verbindung ohne solchen
Weg wird wie zuvor rechtwinklig gemacht.

Auch eine verschobene Leitung bleibt verbunden: Ihr Ende an einem Anschlusspunkt eines
Symbols, das bleibt, erhält ein neues Stück von diesem Punkt aus (stattdessen einen
Versatz, wo das Stück einen anderen Anschlusspunkt oder eine andere Leitung berühren
würde), ihr Ende auf einer durchlaufenden Leitung gleitet an dieser Leitung entlang, und
andere Leitungen an dem Punkt bleiben dort; eine gerade Leitung behält ihre Richtung und
landet dort, wohin sie gezogen wurde, eine an beiden Enden gehaltene Leitung bewegt sich
nicht längs zu sich selbst. Abgänge von einer verschobenen Schiene folgen ihr nur quer
zur Schiene, sie behalten also ihren Platz, wenn die Schiene längs zu sich selbst
verschoben wird. *Bearbeiten › Verbindungen halten* (standardmäßig ein) schaltet dies
zusammen mit dem Dehnen angehängter Leitungen ab, sodass sich die Auswahl allein bewegt
und ihre Verbindungen verlässt; das Halten von Alt beim Beginn eines Ziehens kehrt die
Einstellung für dieses Ziehen um. Um ein Ende zu lösen, die Leitung auswählen und den
quadratischen Griff an diesem Ende ziehen: Nur dieses Ende bewegt sich, es verbindet sich
mit dem, worauf es abgelegt wird, und die Leitung bleibt rechtwinklig. Befehle:
`move_items` (`keep_connections`), `move_wire_end`.

Bearbeitungshilfen: Rastergrößen 1, 2, 4, 8 und 16 mm (Ansicht);
Anschlusspunkte und Leitungsenden abseits dieses Rasters (importierte Zeichnungen,
Geräteboxen beliebiger Größe) fügen unsichtbare Rasterlinien durch sie hinzu, sodass der
Mauszeiger dort an ihnen fängt, wo sie näher sind als das Raster, Leitungen an solchen
Punkten beginnen können und sich Dinge mit solchen Leitungen verbinden lassen; ein
Symbol, das platziert oder verschoben wird, fängt von seinen eigenen Anschlusspunkten aus
(und eine verschobene Leitung von ihren Enden), sodass sie auf solchen Linien landen und
Autoverbinden sie
verbindet; ein
Koordinatenfeld in der Werkzeugleiste (`x y` absolut oder `@dx dy` relativ zum letzten
Punkt, die Eingabetaste wirkt wie ein Klick des aktuellen Werkzeugs); *Start ›
Ausrichten* (Kanten oder Mitten), *Verteilen* (gleichmäßig, horizontal oder vertikal),
*Auswahl verschieben um…* und *Mehrfach kopieren…* (n Kopien in festem Abstand, neue
Kennzeichen und Klemmennummern);
angehängte Leitungsenden folgen. Befehle: `align_items`, `distribute_items`, `copy_items`.

## Texte

Freie Texte mit Drehung (0 … 359°, z. B. aufwärts lesend entlang einer
Klemmenleiste), Ausrichtung links / zentriert / rechts, einem vertikalen Anker,
Zeilenumbruch bei einer festgelegten Breite, einem Breitenfaktor der Buchstaben (0,8:
Engschrift; so gezeichnet auf dem Bildschirm und in den SVG-, PDF- und DXF- /
DWG-Exporten), fett / kursiv / unterstrichen, einer Umrandung und einem deckenden
Hintergrund sowie einem Hyperlink
(Strg + Klick öffnet ihn; anklickbar im PDF- und SVG-Export). **Sondertexte** zeigen
Projekt- und Seiteneigenschaften an beliebiger Stelle einer Seite: Platzhalter wie
`{page}`, `{project}` oder `{prop:Name}` (dieselben wie in Schriftfeldern, Einfügen ›
Sondertext) werden ausgefüllt, wenn die Seite gezeichnet wird. Ein
**Pfad-Funktionstext** (Einfügen › Pfad-Funktionstext) ist der Funktionstext aller
Funktionen in seiner Spalte, die keinen eigenen haben; die Betriebsmittelliste, der
Klemmen-, Stecker- und SPS-Plan verwenden ihn. Texte drehen sich mit einer gedrehten
Auswahl; der DXF- / DWG-Import behält Textdrehung und -ausrichtung. Befehlszeile /
MCP: `add_text`, `update_text`.

## Schaltplangrafik

Linien, Rechtecke, Kreise, Ellipsen, Bögen, Polylinien und
Polygone (Breite, Farbe, gestrichelt, gefüllt), Bemaßungen (ausgerichtet / horizontal /
vertikal, Maßstab, Einheit), PNG- / JPEG-Bilder (auch im PDF- und SVG-Export). **Ebenen**
(Navigator › Ebenen): sichtbar, gesperrt, gedruckt, Farbe; DXF- / DWG-Export je Ebene.

## Leitungskreuzungen und T-Verbindungen

Leitungen, die sich ohne Verbindung kreuzen, werden als einfache Kreuzung gezeichnet,
oder die senkrechte (oder waagerechte) Leitung springt mit einem **Bogen**, einer
eckigen **Brücke** oder einem **Abstand** über die andere; dicht beieinanderliegende
Sprünge auf einer Leitung werden zu einem. T-Verbindungen zeigen einen **Punkt**, einen
**großen Punkt** oder **keinen Punkt** (ein einfaches T); eine Verbindung von vier oder
mehr Leitungen behält ihren Punkt immer, damit sie nie wie eine Kreuzung aussieht.
*Verbindungen › Leitungskreuzungen…* stellt beides mit einer Vorschau ein; die
Einstellung wird mit dem Projekt und in Einstellungsschemata gespeichert und gilt für die
Zeichenfläche und den PDF-, SVG- und DXF-Export gleichermaßen. Befehlszeile / MCP:
`wire_settings` (`jump`: none, arc, bridge, gap; `jump_wire`: vertical, horizontal;
`tee`: dot, large_dot, none; `reroute`).

## Verlegung

Autoverbinden zeichnet Leitungen zwischen einander zugewandten freien
Anschlusspunkten, wenn Symbole platziert oder verschoben werden; während ein Symbol mit
der Maus platziert oder verschoben wird, zeigen schwache
gepunktete Linien die Leitungen, die es zeichnen würde; das Leitungswerkzeug kann um
Symbole und andere Leitungen herum verlegen
(A); Leitungsstücke in einer Linie lassen sich zusammenfassen. Auch als Befehle `route` /
`autoconnect`.

## Eigenschaftsplatzierung

Den Kennzeichenblock eines ausgewählten Symbols ziehen; weitere
Betriebsmitteleigenschaften (Beschreibung, Artikel- / Typnummer, Hersteller, Bezeichnung,
Anschlusspunkte, Struktur) daneben platzieren.

Anschlussbezeichnungen werden neben der Zuleitung ihres Anschlusspunkts platziert, an ihr
entlang, auf ihre andere Seite oder nach außen neben die Leitung verschoben, wo die
Grafik des Symbols, Leitungen, benachbarte Symbole oder die anderen Texte des Symbols im
Weg sind. Eine Bezeichnung eines ausgewählten Symbols ziehen, um sie von Hand zu
platzieren, oder in *Eigenschaftsplatzierung* für jede links, rechts, oben oder unten
wählen; *Zurücksetzen* stellt die automatische Platzierung wieder her. Die Platzierung
dreht und spiegelt sich mit dem Symbol und verschiebt nie den Anschlusspunkt. Befehl:
`place_pin_label`. Die Bezeichnung eines Symbols mit einem einzigen Anschlusspunkt (wie
eines Geräteanschlusspunkts) wird frei gezogen und behält ihre Ausrichtung.

Automatisch platziert sitzt der Kennzeichenblock links von einem senkrechten Symbol und
über einem waagerechten. Wo er dort in ein benachbartes Symbol laufen würde, geht er auf
die andere Seite, nach oben oder unten, je nachdem, wo er zuerst mit allen seinen Zeilen,
einschließlich der Querverweise, frei von Symbolen und Leitungen ist. Das Kennzeichen
einer Funktion direkt neben einer desselben Betriebsmittels in derselben Zeile (wie
Überspannungsableiter, die Pol für Pol gezeichnet sind) wird einmal gezeigt, an der
ersten von ihnen, wie das Leistenkennzeichen
einer Reihe von Klemmen. Ein von Hand platzierter Kennzeichenblock bleibt, wo er ist.

## Suchen

**Suchen** (*Start › Suchen…*, `Strg+F`): durchsucht das ganze Projekt (Seitennamen und
-beschreibungen, Betriebsmittelkennzeichen, Funktionstexte, freie Texte,
Verbindungsnummern, Klemmen und Pins, Kabel, Artikelnummern, Beschreibungen) mit Optionen
für Groß-/Kleinschreibung und ganze Wörter, springt zu den Treffern und ersetzt Text in
freien Texten, Funktionstexten und Beschreibungen
(freigegebene Seiten bleiben). Befehle: `find`, `replace_text`.

## Navigatoren

Seiten (wahlweise nach Struktur gruppiert, mit den Beschreibungen der
Strukturkennzeichenverwaltung), Betriebsmittel (auf der Seite auswählen, weitere
Funktionen platzieren, umbenennen), Verbindungen (filtern, zeigen, auswählen, eine
Verbindungsdefinition hinzufügen), Kabel (verwendete und freie Adern,
Kabeldefinitionen, eine Kabeldefinition zeichnen), Ebenen und die Prüfung.
Die Registerkarte *Nicht platziert* listet, was die Projektdaten haben, der Schaltplan
aber nicht: die Kanäle von SPS-Karten mit einer Gerätebeschreibung, die kein Ein- /
Ausgangssymbol haben (mit ihrer Adresse), die Funktionen der zugewiesenen Artikel, die
kein Symbol verwendet (Spulen, Kontakte mit ihren Bezeichnungen), und die im Projekt
definierten Potenziale, die noch kein Potenzialanschlusspunkt trägt; eines davon in die
Seite ziehen (oder anklicken, dann in die Seite klicken), um es mit Kennzeichen, Adresse
und Bezeichnungen zu platzieren. Befehl:
`list_unplaced`.

## Sprachen

Englisch, Deutsch, Französisch, Spanisch, Portugiesisch (Brasilien), Italienisch,
Niederländisch, Polnisch, Tschechisch, Russisch, Ukrainisch, Türkisch, Schwedisch,
Dänisch, Finnisch, Ungarisch, Rumänisch, Chinesisch (vereinfacht), Japanisch und
Koreanisch (*Ansicht › Sprache*; der Editor und die Artikelverwaltung merken sich die
Wahl und starten in der System- / Browsersprache, sofern sie unterstützt wird).
Chinesisch, Japanisch und Koreanisch bringen eigene Schriften mit (Teilmengen von Noto
Sans SC / JP / KR); Projekttexte in diesen Schriften verwenden auf dem Desktop auch die
CJK-Schriften des Systems, auf dem Bildschirm und in PDF-Exporten (als Teilmenge
eingebettet).

## Projekttexte in mehreren Sprachen

Funktionstexte, Seitenbeschreibungen, freie
Texte und Beschreibungen werden in einer Quellsprache geschrieben; das
Übersetzungswörterbuch des Projekts (*Projektdaten › Projektsprachen…*) übersetzt sie
Zeile für Zeile in beliebig viele Zielsprachen, mit CSV-Export / -Import für Übersetzer
und einer Prüfung auf unübersetzte Texte. *Projektdaten › Als PDF in Sprache exportieren*
(Befehlszeile: `export` mit `language`) gibt die Dokumentation je Sprache aus –
Projekttexte aus dem Wörterbuch, Berichte und Schriftfelder in dieser Sprache (Englisch,
Deutsch, Französisch, Spanisch; andere Sprachen erhalten englische
Berichtsüberschriften).

## Projektvorlagen

**Projektvorlagen** (im Sinne der Basisprojekte von EPLAN): *Datei › Neu › Neues Projekt
aus Vorlage* beginnt mit einer mitgelieferten Vorlage (leeres Projekt nach IEC 60204-1;
Maschine mit Deckblatt, Inhaltsverzeichnis, strukturierten Schaltplanseiten und
Berichtsseiten; Verteiler mit einpoliger Übersicht, Klima- und Kurzschlussberichten) oder
mit einer beliebigen Projektdatei; *Als Projektvorlage speichern…* schreibt das aktuelle
Projekt ohne Revisionen, Baselines, Kommentare, Verdrahtungsmarkierungen und akzeptierte
Prüfmeldungen.
Befehlszeile: `new_project --template Machine` (oder eine Datei); `new_project --example
<name>` beginnt mit einem [Beispielprojekt](../../../README.md#example-projects).

## Speicherung

JSON-Projektdateien (`.oec`); das aktuelle Projekt und die
Artikeldatenbank werden außerdem automatisch (komprimiert) im Datenordner der App auf
dem Desktop und in IndexedDB im Browser gespeichert, dessen lokaler Speicher nur etwa
5 MB fasst (bis der erste Schreibvorgang in IndexedDB gelingt, werden sie auch im lokalen
Speicher gehalten).
