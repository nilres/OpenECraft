# Revisionen, Kommentierung und Änderungsmanagement

Revisionen mit gesperrten Seiten und markierten Änderungen, der Genehmigungsablauf, Baselines mit Vergleichen und Zusammenführen, Kommentare und Redlining sowie Änderungsanträge.

## Revisionen

Eine Revision speichert einen Schnappschuss aller Seiten und kann sie freigeben
(sperren); gesperrte Seiten sind schreibgeschützt. Änderungen seit der letzten
freigegebenen Revision (hinzugefügte / geänderte /
entfernte Symbole, Drähte, Texte, Grafiken, Bemaßungen, Bilder, Verbindungs- und
Kabeldefinitionen, Seiten und ihre Eigenschaften) werden aufgelistet, auf den Seiten
mit einem Revisionsdreieck markiert und in der *Revisionsübersicht* ausgewertet;
geänderte Elemente nennen die geänderten Eigenschaften mit altem und neuem Wert
(`Device tag: -K1 › -K2; Position`).

## Freigabeablauf

Sind Genehmiger festgelegt (*Projektdaten › Revisionen… ›
Genehmigungsablauf*: Benutzernamen und die Anzahl der nötigen Genehmigungen), beginnt
eine neue Revision als Entwurf, wird zur Prüfung eingereicht, von den Genehmigern
genehmigt oder abgelehnt (festgehalten mit Benutzer, Datum und Notiz) und nach der
Genehmigung freigegeben. Seiten, die nach dem Einreichen in irgendeiner Weise geändert
wurden, blockieren die Freigabe, bis die Revision erneut eingereicht wird (frühere
Genehmigungen werden ungültig).

## Baselines und Vergleichen / Zusammenführen

Eine Baseline speichert das gesamte Projekt (Seiten, Betriebsmittel,
Artikel, Kabel, Schaltschrankaufbau, Einstellungen); die Freigabe einer Revision
speichert automatisch eine. *Projektdaten › Vergleichen und Baselines…* vergleicht das Projekt mit einer Baseline, den Seiten einer
Revision oder einer anderen Projektdatei: hinzugefügte / entfernte / geänderte Seiten
(mit den geänderten Elementen), Betriebsmittel, Artikel, Potenziale, Kabel, Kabelbäume,
Gehäuse, Klemmenleisten, Stecker, SPS-Karten und Abschnitte der Einstellungen.
Ausgewählte Unterschiede werden übernommen.

## Kommentierung: Kommentare und Redlining

Kommentare an einer Seitenposition mit Status (offen, in
Arbeit, erledigt, abgelehnt), Autor, Datum und Antworten; Redline- (zu ändern) und
Greenline-Markierungen (geändert wie gebaut) als Linie, Rechteck, Revisionswolke oder
Text. Sie werden auf einer eigenen Ebene gezeichnet (auf dem Bildschirm sichtbar, nicht
in Exporten), in der Registerkarte *Kommentare* des Navigators aufgelistet und können
auch auf freigegebenen Seiten und auf Seiten angebracht werden, die ein anderer
Benutzer gesperrt hat. Auf dem Projektserver werden sie Datensatz für Datensatz ohne
Sperren zusammengeführt.
**PDF-Kommentare**, die ein Prüfer in einem PDF-Export angebracht hat (Acrobat oder ein
anderer Viewer: Notizen, Textfelder, Hervorhebungen, Formen, Antworten), werden in die
Kommentarebene importiert (*Kommentare › PDF-Kommentare importieren…*, Befehl
`import_pdf_comments`); Rechtecke und Linien werden zusätzlich zu Redline-Markierungen,
bereits importierte Kommentare werden übersprungen.

## Änderungsanträge

Anträge von innerhalb oder außerhalb des Projekts (Nummer, Titel,
Beschreibung, Antragsteller, Herkunft, Priorität, Status, betroffene Seiten und
Betriebsmittel, umsetzende Revision) mit verknüpften Kommentaren (*Projektdaten ›
Änderungsanträge…*). Austausch mit anderen Systemen als CSV
(`Number;Title;Description;Requester;Origin;Date;Priority;
Status;Pages;Devices;Revision`) oder JSON; der Import aktualisiert Anträge mit
bekannter Nummer.
