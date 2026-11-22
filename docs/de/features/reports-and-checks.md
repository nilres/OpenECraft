# Berichte, Formulare und die Projektprüfung

Berichte als Fenster, CSV / Excel oder mitlaufende Berichtsseiten, Schriftfelder und Berichtsformulare (auch grafische und eigene gezeichnete Formulare) und die Projektprüfung mit ihrer Meldungsverwaltung.

## Berichte

Inhaltsverzeichnis, Betriebsmittelliste, Stückliste (mit Artikeln und Preisen),
Klemmenplan, Verbindungsliste, SPS-Zuordnungsliste, Potenzialübersicht, Kabelübersicht,
Kabelplan — in einem Fenster,
als CSV-Export oder als mitlaufende Berichtsseiten eingefügt.

Der Filter des Fensters *Berichte* findet einen Bericht über seinen Titel, seinen Namen
oder die Normen, denen er folgt, ohne Rücksicht auf Groß-/Kleinschreibung, Leerzeichen
und Bindestriche: `VDE 0100 600`, `0100-600` oder
`IEC 60364-6` finden die drei Prüfprotokolle, `ISO 13849` die Sicherheitsfunktionen,
`60909` den Kurzschlussbericht (`list_reports` mit `search` auf der
Befehlszeile).

Kabel, das als Meterware verkauft wird, braucht für seine Materialmenge eine positive
Länge. Eine fehlende
Länge wird als `?` gezeigt; haben andere Kabel dieses Artikels Längen, zeigt die Zeile
die bekannte Menge plus `?`. Auch Preise und die Berichtssumme weisen den
unbekannten Anteil aus. Die Kabelprüfung nennt jedes Meterware-Kabel, das eine Länge
braucht.
Kabel, das stückweise gezählt wird, behält seine Menge von einem Stück, wenn keine Länge
angegeben ist.

Kabelartikel können in ihren technischen Daten `Outer diameter: 6.3 mm` angeben. Das
Auswählen
des Artikels übernimmt dieses Maß für die Verlegung; ein Artikel ohne gültiges Maß
kehrt zur Schätzung aus seinen Adern und dem Querschnitt zurück. `apply_cable_part` stellt
dieselbe Katalogoperation über das gemeinsame Befehlsregister bereit.
Die Kabelauswahl enthält projektlokale Artikel ebenso wie den eingerichteten Katalog;
lokale Metadaten und Klassifikationen haben bei wiederholten Artikelnummern Vorrang.
`list_cable_parts` stellt die Auswahl des Projekts und die mitgelieferte für Plugins und
CLI/MCP bereit.

## Formulare

Schriftfelder sind Listen aus Linien, Rechtecken, Texten mit Platzhaltern
(`{project}`, `{page}`, `{sheet}`, `{page.previous}` / `{page.next}`, `{rev.date}`,
Revisionstabellen mit `{rev.1.name}`, `{rev.2.name}` …, Benutzereigenschaften
`{prop:Name}` …)
und Bildern, im Schriftfeldeditor bearbeitet und als Dateien weitergegeben; eines je
Projekt mit einer
Abweichung je Seite (mitgeliefert: Standard, Compact, None). Texte können fett, kursiv und
gedreht sein (z. B. 90° für aufwärts laufende Texte). Ein Bildelement behält sein eigenes
Bild
(PNG, JPEG, z. B. das Firmenlogo, das mit der Schriftfelddatei mitwandert) oder zeigt das
Projektlogo aus *Projekteigenschaften › Logo* (z. B. das des Kunden, in jedem
Projekt anders), in sein Rechteck eingepasst mit seinem Seitenverhältnis; das Deckblatt
zeigt das
Projektlogo über dem Projektnamen oder an einem eigenen Platz (*Projekteigenschaften ›
Logo auf dem Deckblatt*, `set_cover_logo`). Gezeichnete Berichtsformulare nehmen dieselben
Elemente.
Jedes Element lässt sich auf Seiten bestimmter Typen beschränken (z. B. ein Feld nur auf
Berichtsseiten) oder auf einen ausgefüllten Platzhalter (z. B. die zweite Zeile einer
Revisionstabelle nur mit
`{rev.2.name}`); Linien, Rechtecke und Texte erhalten ihre eigene Farbe, Linien und
Rechtecke
Strichelung. **QR-Codes** kodieren einen Text mit Platzhaltern (z. B. einen Link zum
Projekt in der
Dokumentenverwaltung mit `{project}` und `{page}`). Das Schriftfeld einer Seite ist ihr
eigenes, sonst das ihres Seitentyps (*Schriftfeld nach Seitentyp*), ihres Plotrahmens
(z. B. ein schmaleres für A4 hoch), die Projektvorgabe. `{rev.status}`,
`{rev.approved_by}` und `{rev.approved_date}` zeigen den Stand des Revisionsablaufs
und wer die letzte Revision genehmigt hat.
**Stempel**: ein großer Text quer über jede Seite, der dem Revisionsablauf folgt, z. B.
ENTWURF, während eine Revision vorbereitet wird, ZUR FREIGABE in der Prüfung, nichts
(oder WIE GEBAUT) nach der
Freigabe (*Formulare › Schriftfelder › Stempel*, `set_stamp`).
**Firmenbibliothek**: Die eigenen Schriftfelder, Berichtsschemata und Plotrahmen des
Projekts werden
auf dem Artikelserver veröffentlicht und in andere Projekte geladen (*Schriftfelder ›
Firmenbibliothek*, `parts_company_forms` (`publish`); das Veröffentlichen braucht das
Recht, Kataloge zu bearbeiten). Berichtsformulare wählen die Spalten, ihre Reihenfolge,
die Überschrift und
die Zeilen je Berichtsseite, die Ausrichtung jeder Spalte, Spalten, die in einer
Summenzeile
unter der letzten Zeile summiert werden (mit der Einheit ihrer Werte), und die Schattierung
jeder zweiten Zeile.
**Grafische Formulare**: Ein Berichtsformular des Klemmen-, Kabel- oder Steckerplans oder
der SPS-Zuordnung
kann auf seine grafische Form umschalten – die Leiste, gezeichnet mit einem Kasten je Klemme (Etagen markiert), Endhaltern
und Abschlussdeckeln, Brücken und den externen und internen Zielen mit
Potenzial, Funktionstext und verknüpftem Ort; oder jedes Kabel als Band mit seinen Adern
von Ende zu Ende (Potenzial, Verbindungsnummer, Reserveadern gestrichelt); der
Steckerplan als
Gehäuse mit Buchsen- und Stifthälfte je Pin; die SPS-Zuordnung als Modul je Karte,
bei dem die Eingänge von links hereinkommen und die Ausgänge nach rechts hinausgehen.
**Gezeichnete Formulare**: Jeder Bericht (auch die von Plugins) lässt sich mit einem
eigenen Formular zeichnen –
Grafik, die einmal je Seite gezeichnet wird, und eine Zeilenzeichnung, die für jede Zeile
wiederholt wird, nach unten für
Listen oder seitwärts (z. B. eine Klemme neben der anderen); seine Texte nehmen die
Spalten als
Platzhalter (`{Device tag}`, `{Part number}` …), `{report}` und die Platzhalter der
Schriftfelder. Der Formulareditor (*Berichtsformulare › Eigenes gezeichnetes Formular*)
beginnt mit der
Tabelle des Berichts und zeigt die Seite mit ihren echten Zeilen, während Linien,
Rechtecke und
Texte hinzugefügt und gezogen werden.

![Die grafische Form des Klemmenplans mit Endhaltern, Abschlussdeckeln und den Zielen jeder Klemme](../../images/terminal-diagram.png)

## Projektprüfung

Hilfsfunktionen ohne Hauptfunktion, doppelte Hauptfunktionen,
offene Anschlusspunkte, Unterbrechungspunkte ohne Gegenstück, Kurzschlüsse zwischen
Potenzialen, doppelte Klemmennummern / SPS-Adressen, Betriebsmittel ohne Artikel, mehr
Leiter
an einem Anschlusspunkt, als dessen Klemmstelle aufnimmt, oder Leiter, die dafür zu dünn
oder zu dick sind (Prüfung
*Leiter an Anschlüssen*, nach den Anschlusspunktdaten der Artikel, z. B. aus
EDZ-Dateien).
**Meldungsverwaltung** (im Sinne der Prüfschemata von EPLAN): Jede Meldung hat
einen Code je Art (`wires.3f9a1c`, in jeder Sprache derselbe) und einen Fingerabdruck je
Vorkommen. Ein Rechtsklick auf eine Meldung im Fenster Meldungen akzeptiert sie als
geprüft (ausgeblendet
und nicht gezählt, bis sie sich ändert), ändert die Stufe ihrer Art (Fehler, Warnung,
Hinweis, aus) oder schaltet ihre Prüfung ab; die Einstellungen (⚙) listen die Prüfungen,
geänderten
Stufen und akzeptierten Meldungen (mit Notiz, Benutzer, Datum und ob sie noch
gemeldet werden). Die Registerkarte filtert nach Stufe, Seite und Text und exportiert die
Meldungen als CSV.
Die Einstellungen werden mit dem Projekt gespeichert; die abgeschalteten Prüfungen und die
geänderten
Stufen werden als **Prüfschema**-Datei (JSON) zwischen Projekten ausgetauscht. Befehle:
`list_checks`, `check_settings`, `set_check`, `set_message_level`, `accept_message`;
Werkzeuge `export_check_scheme`, `import_check_scheme`; `check` zeigt Codes und
Fingerabdrücke.
