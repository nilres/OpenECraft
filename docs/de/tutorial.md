# Erste Schritte: von der ersten Seite bis zur Stückliste

Dieses Tutorial führt durch eine kleine Steuerung in OpenECraft: Seiten, Symbole,
Leitungen, Betriebsmittel mit Artikeln, Klemmen, eine SPS, die Projektprüfung, Berichte,
den Schaltschrankaufbau und die Ausgabe. Befehle werden als *Registerkarte › Befehl*
geschrieben: Die Registerkarten des Menübands oben enthalten die Befehle nach Thema,
*Datei* öffnet die Backstage-Ansicht mit Neu, Öffnen, Speichern, Import, Export und
Drucken. Jeder Befehl wird auch über seinen Namen mit *Befehl suchen* (Strg+Umschalt+P)
gefunden, Alt zeigt die Zugriffstasten des Menübands und F1 öffnet die Hilfe des
verwendeten Fensters. Die Oberfläche gibt es in Englisch, Deutsch, Französisch,
Spanisch und 16 weiteren Sprachen (*Ansicht › Sprache*); die folgenden Namen sind die
deutschen.

Am schnellsten sieht man alles auf einmal mit *Datei › Neu › Demoprojekt öffnen*: eine
Förderbandsteuerung mit Haupt- und Steuerstromkreisen, Klemmen, einem Kabel, einer SPS,
einem Kabelbaum und einem Schaltschrankaufbau. Die meisten der folgenden Schritte lassen
sich daran ausprobieren.

## 1. Ein neues Projekt

- *Datei › Neu › Neues Projekt* beginnt ein leeres Projekt mit einer Schaltplanseite; die
  Vorlagen darunter beginnen mit einer mitgelieferten Vorlage (Rahmen im IEC- oder
  NFPA-Stil, Struktur, Potenziale) und *Aus einer Vorlagendatei…* mit einem beliebigen
  gespeicherten Projekt. Die Startseite zeigt dieselbe Auswahl und die zuletzt
  geöffneten Projekte.
- *Projektdaten › Projekteigenschaften…* enthält die Projektdaten, die in den
  Schriftfeldern stehen (Name, Kunde, Ersteller), sowie den Standard-Plotrahmen und das
  Standard-Schriftfeld.
- Das Projekt wird als eine `.oec`-Datei gespeichert (*Datei › Speichern*, Strg+S). Im
  Browser liegt das Projekt im Browserspeicher und wird mit *Datei › Projekt
  herunterladen* heruntergeladen. Für die Versionsverwaltung (git) speichert die
  Desktop-Version es auch als geteiltes Projekt (*Datei › Als geteiltes Projekt
  speichern…*): ein Ordner mit dem Index `<name>.oecp` und einer kleinen Datei je Seite,
  Betriebsmittel und Artikel, sodass eine Änderung nur die Dateien berührt, die sie
  betrifft.

## 2. Seiten und Struktur

- Der Navigator *Seiten* links listet die Seiten; *Start › Neue Schaltplanseite* fügt
  eine nach der aktuellen Seite ein, *Start › Neue Seite* die anderen Arten: Deckblatt,
  Übersicht, Grafik, Dokument (ein eingebettetes PDF oder Bild), Berichts- und
  Montageplanseiten. Die zuletzt geöffneten Seiten sind Registerkarten über der Seite
  (Strg+Tab wechselt).
- *Start › Seiteneigenschaften…* (oder ein Rechtsklick auf eine Seite) gibt einer Seite
  ihre Beschreibung und ihre Struktur: die Funktion (`=A1`) und den Ort (`+CAB1`).
  Betriebsmittel, die auf der Seite platziert werden, übernehmen diese Struktur
  (`=A1+CAB1-K1`); auf der Seite wird nur der Teil gezeigt, der abweicht.
- Die Baumschaltfläche des Navigators *Seiten* gruppiert die Seiten nach Struktur;
  *Start › Seiten neu nummerieren…* nummeriert sie neu. *Start › Neue Unterseite* fügt
  `3.1`, `3.2`, … nach der aktuellen Seite ein; beim Neunummerieren bleiben sie
  Unterseiten ihrer Seite.
- *Start › Neuer Abschnitt* gliedert die Seiten wie Verzeichnisse: Ein Abschnitt enthält
  Seiten und Unterabschnitte, und die Seitennamen beginnen mit seiner Nummer (`2.3.1`
  ist Seite 1 des Abschnitts 2.3). Im Navigator *Seiten* zieht man eine Seite zwischen
  zwei Seiten oder auf einen Abschnitt (oder wählt ihren *Abschnitt* in den
  Seiteneigenschaften), um sie dorthin zu verschieben, und einen Abschnitt zwischen
  Abschnitte oder auf einen anderen; die Linie oder der Rahmen zeigt, wo er landet, die
  Beschriftung am Mauszeiger den Namen, den er bekommt, ein zugeklappter Abschnitt
  öffnet sich, wenn man beim Ziehen über ihm verweilt, und die freie Fläche unter der
  Liste ist die oberste Ebene. Ein Rechtsklick auf einen Abschnitt fügt Seiten und
  Unterabschnitte hinzu, verschiebt ihn, nummeriert ihn neu oder löscht ihn, und die
  Seiten folgen.
  *Seiten neu nummerieren…* zählt in jedem Abschnitt neu und kann zuvor die Abschnitte
  mit 1, 2, 3 … nummerieren. Sobald ein Projekt Abschnitte hat, erhalten Unterseiten
  Buchstaben (`2.3.1.a`).

## 3. Symbole platzieren

- Das *Einfügezentrum* rechts (*Einfügen › Einfügezentrum*, Einfg) findet Symbole aller
  Bibliotheken (IEC 60617, NFPA, Fluidtechnik, R&I, Installationsplan, …), Makros,
  Betriebsmittel aus Artikeln und nicht platzierte Funktionen; `*` und `?` sind
  Platzhalter. Einen Eintrag anklicken, dann in die Seite klicken, um ihn zu platzieren.
  Das Platzieren geht weiter bis Esc oder einem Rechtsklick. Der Stern hält einen
  Eintrag unter den Favoriten; die zuletzt verwendeten stehen vorn.
- R dreht und M spiegelt das Symbol, das gerade platziert wird (oder die Auswahl). Die
  Tasten lassen sich in *Datei › Einstellungen › Tasten und Gesten* ändern, ebenso die
  Maustasten und Zusatztasten.
- Betriebsmittelkennzeichen werden automatisch aus dem Kennbuchstaben vergeben (`-K1`,
  `-S2`). Um Kontakte eines Schützes zu platzieren, zuerst dessen Spule auswählen: Danach
  platzierte Kontakte gehören zu ihm und erhalten die nächsten freien Nummern (13/14,
  21/22, …). Kontakte und Spulen zeigen ihre Querverweise (`/3.4`) mitlaufend.
- Ein ausgewähltes Symbol wird im Fenster *Eigenschaften* rechts bearbeitet: Dessen
  Registerkarten enthalten das Kennzeichen, den Funktionstext und die
  Anschlussbezeichnungen (*Betriebsmittel*), die Platzierung seiner Texte
  (*Darstellung*), die Artikel des Betriebsmittels (*Artikel*), weitere Eigenschaften
  und die Verbindungen mit den Querverweisen. F2 oder ein Doppelklick zeigt dasselbe in
  einem eigenen Fenster; ein Rechtsklick auf die Seite bietet die Befehle für die
  Auswahl.

## 4. Leitungen und Potenziale

- W (*Einfügen › Leitung*) startet das Leitungswerkzeug: Anfang, Zwischenecken und Ende
  anklicken; die Leertaste wechselt die Eckenrichtung, die Eingabetaste oder ein
  Doppelklick beendet die Leitung, Esc oder ein Rechtsklick beendet das Werkzeug. Eine
  Leitung, die im Leeren endet, meldet die Prüfung *Offene Verbindungsenden*. Ist
  *Bearbeiten › Autoverbinden* eingeschaltet, werden Symbole, die in einer Flucht mit
  anderen platziert werden, automatisch verbunden.
- Potenziale (`L1`, `+24V`, `0V`, …) sind Potenzialsymbole (*Einfügen › Potenzial* oder
  die Bibliothek *Potenziale*); *Verbindungen › Potenziale* legt ihre Art, Spannung,
  Aderfarbe und ihren Querschnitt fest. Der Navigator *Nicht platziert* listet
  definierte Potenziale, die noch auf keiner Seite stehen.
- Unterbrechungspunkte (*Einfügen › Unterbrechungspunkt*) führen ein Netz über Seiten
  hinweg; ihre Querverweise werden in *Verbindungen › Querverweise…* eingestellt (Stern
  oder Kette).
- *Verbindungen › Leitungsnummerierung…* nummeriert die Verbindungen (je Potenzial, je
  Seite, Formate wie `{page}.{n}`); eine Nummer, die in die Eigenschaften einer Leitung
  eingetragen wurde, bleibt erhalten.

## 5. Artikel

- *Betriebsmittel › Artikelverwaltung* öffnet die Artikeldatenbank (Produktgruppen,
  Filter, Artikeleditor mit kaufmännischen, technischen und typisierten Montagedaten,
  Funktionsvorlagen, Anschlusspunkten, 2D- / 3D-Modellen, Baugruppen). Artikel werden
  aus CSV, BMEcat, EPLAN-Artikel-XML, EDZ-Dateien des EPLAN Data Portal (*Datei ›
  Import*) oder aus dem Data Portal (*Stammdaten › Data Portal*) importiert.
- Artikel werden in der Registerkarte *Artikel* der Eigenschaften eines Betriebsmittels
  zugewiesen (*Artikel hinzufügen…*); die Gerätewahl bietet nur Artikel an, die die
  Hauptfunktion des Betriebsmittels bereitstellen. *Bezeichnungen übernehmen* übernimmt
  die Anschlussbezeichnungen aus dem Artikel. Der Abschnitt *Betriebsmittel* des
  Einfügezentrums platziert ein neues Betriebsmittel aus einem Artikel mit Funktionen.
- Ein Artikel lässt sich auch aus dem Baum der *Artikelverwaltung* oder aus den
  Ergebnissen des *Data Portal* auf eine Schaltplanseite ziehen: Das Symbol seiner
  Hauptfunktion wird dort als neues Betriebsmittel mit zugewiesenem Artikel platziert.
  Ein Artikel, der noch nicht in der Artikeldatenbank ist, wird zuerst importiert (aus
  dem Beispielkatalog oder vom Artikelserver heruntergeladen). Die Befehlszeile macht
  dasselbe mit `place_part`.
- Der Navigator *Nicht platziert* listet dann die Funktionen der zugewiesenen Artikel,
  die noch nicht im Schaltplan sind (zum Beispiel die Reservekontakte eines Schützes);
  eine davon in die Seite ziehen, um sie mit Kennzeichen und Bezeichnungen zu platzieren.

## 6. Klemmen und Stecker

- Klemmen werden wie andere Symbole platziert (Bibliothek *Klemmen*); Klemmen mit
  demselben Kennzeichen bilden eine Leiste. *Betriebsmittel › Klemmenleisten* ordnet
  sie, fügt Reserven, Brücken (Steck-, Sattel-, Drahtbrücken), Endhalter und
  Abschlussdeckel sowie Etagenklemmenblöcke hinzu.
- Der Navigator *Klemmleisten / Stecker* zeigt jede Leiste in Schienenreihenfolge und
  jeden Stecker mit seinen Pins; ein Klick zeigt eine platzierte Klemme oder einen
  platzierten Pin, Reserven und unbenutzte Pins werden in die Seite gezogen.
- *Verbindungen › Kabel* definiert Kabel; Leitungen auswählen und *Kabel zuweisen*
  verwenden oder eine Kabeldefinitionslinie über sie zeichnen (*Einfügen ›
  Kabeldefinition*).

## 7. SPS

- SPS-Ein- und -Ausgänge werden aus der Bibliothek *SPS* mit Kartenkennzeichen und
  Adresse platziert. *Betriebsmittel › SPS-Hardware und Bustopologie* fügt Stationen,
  Baugruppenträger und Module aus Gerätebeschreibungen (GSDML, ESI) hinzu;
  *Betriebsmittel › SPS-Übersicht* listet alle Punkte mit ihren angeschlossenen
  Betriebsmitteln und adressiert Karten neu.
- Der Navigator *SPS* zeigt die Karten mit ihren Kanälen; freie Kanäle von Karten mit
  einer Gerätebeschreibung werden in die Seite gezogen.
- *Datei › Export* schreibt die Symboltabelle für das SPS-Programmierwerkzeug: TIA
  Portal, Studio 5000, STEP 7 classic, TwinCAT 3, Control Expert, GX Works oder
  IEC 61131-3.

## 8. Die Projektprüfung

- Das Fenster *Meldungen* unter der Seite führt die Prüfungen laufend aus
  (Betriebsmittel, Verbindungen, Klemmen, Kabel, SPS, Schaltschrankaufbau, …) und listet
  die Meldungen als Tabelle; ein Klick auf eine Spalte sortiert, ein Klick auf eine
  Meldung springt zu ihrer Stelle (F4 zeigt das Fenster).
- Ein Rechtsklick auf eine Meldung akzeptiert sie als geprüft, ändert die Stufe ihrer
  Art oder schaltet ihre Prüfung ab; *Projektdaten › Prüfeinstellungen* listet sie und
  tauscht Prüfschemata mit anderen Projekten aus.

## 9. Berichte

- *Projektdaten › Berichte* zeigt jeden Bericht (Betriebsmittelliste, Stückliste,
  Klemmenplan, Kabelplan, Steckerplan, SPS-Belegung, Verbindungsliste, …) als Tabelle
  mit Export nach CSV und Excel.
- *Als Berichtsseite einfügen* (oder *Start › Neue Seite › Neue Berichtsseite*) fügt
  einen Bericht als Seiten des Projekts ein; sie aktualisieren sich, wenn sich das
  Projekt ändert. *Stammdaten › Formulare* wählt die Spalten eines Berichtsformulars
  oder schaltet Klemmen-, Kabel- und Steckerplan sowie die SPS-Belegung auf ihre
  grafische Form um.
- Die Stückliste summiert alle Artikel der Betriebsmittel, Klemmen, Kabel, Kabelbäume,
  des Materials des Schaltschrankaufbaus, der Aderendhülsen und des
  Beschriftungsmaterials; Baugruppen listen ihre Komponenten.
- *Projektdaten › In Tabelle bearbeiten* ändert Betriebsmittel, Verbindungen, Klemmen,
  Kabel, Steckerpins, SPS-Punkte, Seiten und Artikel in großer Zahl (Filtern, Füllen,
  Suchen und Ersetzen, CSV); die Navigatoren öffnen es für ihre Zeilen.

## 10. Schaltschrankaufbau

- Der Arbeitsbereich *Schaltschrankaufbau* (*Ansicht › Arbeitsbereiche* oder die Auswahl
  in der Schnellzugriffsleiste) zeigt die Registerkarte *Layoutbereich* des Menübands:
  Gehäuse, Montageplatten, Tragschienen, Kabelkanäle, Sammelschienen, Ausschnitte und
  die Betriebsmittel des Schaltplans; die 3D-Ansicht zeigt das Gehäuse mit der Tür, den
  verlegten Drähten und der Kanalfüllung.
- Der Arbeitsbereich *Verdrahtung* ist der Verdrahtungsassistent für die Werkstatt
  (Drähte in Reihenfolge mit Farbe, Querschnitt, Länge und Endenbehandlung); der
  Arbeitsbereich *Montage* ist der Montageassistent (Teile in Montagereihenfolge mit
  Platz, Größe und Artikel). *Verdrahtung › Tablet-Modus* vergrößert Texte und
  Schaltflächen für einen Touchscreen.
- Bohrbilder (*Layoutbereich › NC-Daten*) und der Schaltschrankaufbau in 3D (STEP, glTF,
  JT, IFC, *Datei › Export*) werden für die Fertigung exportiert.

## 11. Ausgabe

- *Datei › Export* listet alle Exporte: PDF (alle Seiten oder eine Seitenauswahl wie
  `1-3, 5`, mit Lesezeichen und anklickbaren Querverweisen), HTML zum Drucken, DXF /
  DWG, SVG, die Fertigungs-Leitungsliste, Beschriftung (mit Beschriftungsschemata für
  Beschriftungsprogramme), Kabelbaum-KBL und Nagelbrett-Zeichnungen, SPS-Tabellen,
  AutomationML und 3D-Formate. Exporte mit Optionen fragen diese zuerst ab.
- *Datei › Drucken* öffnet alle Seiten oder die aktuelle Seite im Browser mit dessen
  Druckdialog.
- *Projektdaten › Als PDF in Sprache exportieren* schreibt das PDF in einer der
  Projektsprachen (*Projektdaten › Projektsprachen*).

## 12. Änderungen und Kommentare

- *Projektdaten › Revisionen* legt Revisionen an; Änderungen seit der letzten
  freigegebenen Revision werden auf den Seiten markiert und mit alten und neuen Werten
  in der Revisionsübersicht aufgeführt. Freigegebene Seiten sind gesperrt.
- Der Arbeitsbereich *Kommentare* und *Einfügen › Kommentar* / *Markierung* enthalten
  Kommentare und Redline- / Greenline-Markierungen; *PDF-Kommentare importieren…*
  übernimmt die Kommentare, die ein Prüfer in einem PDF-Export gemacht hat.

## 13. Automatisierung

Alles oben Genannte steht auch ohne die Oberfläche zur Verfügung: Die Befehlszeile
(`cargo run -p openecraft-cli -- help`) und der MCP-Server für KI-Agenten
(`openecraft-cli mcp`) bieten dieselben Befehle, Prüfungen, Berichte und Exporte, und
Plugins fügen ihre eigenen hinzu (siehe [plugins.md](plugins.md)); im Editor stehen sie
in *Plugins › Plugin-Befehle*, wo der Stern einen Befehl an die Registerkarte *Plugins*
heftet.
