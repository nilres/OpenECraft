# Makros, Konfigurator und Bearbeiten in Tabellen

Wiederverwendung von Schaltungen als Makros mit Varianten und Platzhaltern, vom Konfigurator erzeugte Seiten und Massenänderungen in Tabellen und Tabellenkalkulationen.

## Makros

Eine Auswahl (Fenstermakro) oder eine ganze Seite (Seitenmakro) in einer
Makrobibliothek speichern (mit der Anwendung gespeichert, JSON-Import / -Export).
Fenstermakros werden mit der Maus platziert (Geistervorschau, Betriebsmittel erhalten
neue Kennzeichen), Seitenmakros fügen eine Seite ein.
Kopierte Makros führen eigene Symboldefinitionen, Betriebsmittelzuordnungen und
referenzierte Katalogartikel mit. Vorschauen lösen diese mitgeführten Definitionen vor
dem Einfügen auf. Das Einfügen
in ein anderes Projekt erhält vorhandene Katalogeinträge und lokale Symboländerungen.
Platzhalter wie `{MOTOR}` in Kennzeichen, Texten, Funktionstexten und Bezeichnungen
werden beim Einfügen aus **Wertesätzen** gefüllt. **Varianten**: Ein Makro kann dieselbe
Funktion in mehreren Ausführungen enthalten (A, B, … – z. B. Direktanlauf und
Stern-Dreieck); eine
Auswahl unter dem Namen eines vorhandenen Makros mit einer anderen Variante speichern, die
Variante beim Einfügen wählen. **Makrokästen**: Platzierte Fenstermakros merken sich ihr
Makro, ihre Variante, Platzhalterwerte und Elemente (gestrichelter Rahmen auf dem
Bildschirm, *Ansicht › Makrokästen*);
*Makros › Makrokästen › Geänderte Makros aktualisieren* ersetzt den Inhalt der Kästen,
deren
Makro sich in der Bibliothek geändert hat, ein auf der Seite bearbeiteter Kasten wird in
die
Bibliothek zurückgespeichert (Befehl `list_macro_boxes`, Werkzeug `update_macro_boxes`).

## Konfigurator

**Konfigurator** (*Projektdaten › Konfigurator…*): Optionen mit ihren Auswahlwerten (z. B.
`STARTER` = DOL / SD) und Regeln, die ein Seitenmakro in einer Variante mit
Platzhalterwerten (`NAME=Pump; POWER={POWER}`) einfügen, wenn ihre Bedingung zutrifft
(`STARTER=DOL & LINE!=2`, `LINE=1|2`). *Seiten erzeugen* ersetzt die zuvor erzeugten
Seiten (in den Seiteneigenschaften markiert, abschalten, um eine zu behalten);
Optionswerte füllen
`{OPTION}`-Platzhalter; Probleme (unbekanntes Makro, unbekannte Variante oder Option)
halten es an.

## Bearbeiten in Tabellen

**Bearbeiten in Tabellen** (*Projektdaten › In Tabelle bearbeiten…*): Betriebsmittel
(Kennzeichen, Funktionstext,
Beschreibung, Artikel), Verbindungen (Verbindungsnummer, Farbe, Querschnitt, Kabel, Ader),
Klemmen (Leiste, Nummer, Funktionstext) und das Prüfprotokoll (die
Ergebnisse, eine Zeile je Stromkreis; seine Daten und die Besichtigung, eine Zeile je
Wert) als
Zeilen; filtern, Zellen direkt bearbeiten, eine
Spalte füllen oder in den gefilterten Zeilen suchen und ersetzen; CSV-Export / -Import
für Änderungen in einer
Tabellenkalkulation (Zeilen nach Schlüssel, nur geänderte Zellen werden übernommen);
Zeilen auf freigegebenen Seiten
bleiben unverändert. Das Umbenennen eines Betriebsmittels nimmt seine Leisten-, Stecker-
und SPS-Kartendaten mit.
Der Hin- und Rückweg funktioniert auch mit **Excel**-Dateien (`.xlsx`: exportieren,
ändern, importieren);
Berichtstabellen werden aus dem Fenster Berichte und mit `report
--format xlsx --path …` als Excel-Arbeitsmappen exportiert (fette, fixierte Kopfzeile mit
Filter, Zahlen als Zahlen,
Codes wie `007` als Text erhalten).
