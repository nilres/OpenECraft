# Leitungen, Kabel, Potenziale und Nummerierung

Wie Verbindungen zu Kabel, Farbe, Querschnitt und Nummer kommen und wie Betriebsmittel, Seiten und Leitungen nummeriert werden.

## Potentialdefinitionen

**Potentialdefinitionen** (*Verbindungen › Potenziale…*): Art (L, N, PE, DC +/−, Signal),
Spannung, Aderfarbe (Codes nach IEC 60757) und Querschnitt je Potenzial; Voreinstellungen
IEC 60204-1 (Maschinen, neue Projekte) und IEC 60445 (Phasenfarben). Leitungen ohne
eigene Farbe / eigenen Querschnitt übernehmen sie vom Potenzial ihrer Verbindung
(auch hinter Klemmen), sonst aus der Projektvorgabe für Verbindungen ohne
definiertes Potenzial; Kabeladern behalten die Werte des Kabels. Die
Leitungseigenschaften zeigen den automatischen Wert und woher er kommt; die
Leitungslisten, die Schaltschrankverdrahtung, die Kanalfüllung, die Aderendhülsen, die
Kabelbäume und die Beschriftung verwenden ihn. Die Prüfung meldet GNYE außerhalb von
Schutzleitern und PE in anderen Farben, manuelle Werte, die von der Definition abweichen,
und Potenziale ohne Definition. *Ansicht › Aderfarben* (mit dem Projekt gespeichert)
zeichnet die Leitungen im Schaltplan und im PDF- / SVG-Export in ihrer Farbe;
zweifarbige Leitungen wie GNYE werden gestrichelt in ihrer zweiten Farbe gezeichnet.

## Kabel

Kabeldefinitionen (Kennzeichen, Typ, Adern, Querschnitt, Länge, Artikel aus der
Produktgruppe *Kabel*). Leitungen werden Adern zugewiesen (mehrere Leitungen auf einmal
erhalten die freien Adern der Reihe nach, die Leitungen einer Verbindung teilen sich eine
Ader); Adern und das Kabelkennzeichen werden im Schaltplan gezeigt, mit einem Strich quer
über die Leitung, wo eine Kabeldefinitionslinie des Kabels sie kreuzt (sonst bei 30 % der
Leitung). Die Aderbezeichnung und das Kabelkennzeichen einer ausgewählten Leitung lassen
sich an ihren Platz ziehen und behalten ihren Versatz von dort; *Zurücksetzen*
in den Leitungseigenschaften (oder `update_wire` mit leerem `core_offset` /
`cable_offset`)
platziert sie wieder automatisch. Berichte Kabelübersicht und
Kabelplan, Kabel in der Stückliste nach Länge, Prüfungen auf unbekannte Kabel, Adern, die
doppelt verwendet werden oder nicht im Kabel sind.

## Verbindungs- und Kabeldefinitionen

Adertexte von Kabeln halten automatisch Abstand zur Definitionslinie und zu ihrer Leitung.
Das Layout prüft geschätzte Textflächen, andere Kabelbeschriftungen, Leitungen, nahe
Symbolgrafik, Pin-/Eigenschafts-/Kennzeichentexte und freie Texte. Es probiert eine
begrenzte Menge naher Positionen in fester Reihenfolge; es garantiert nicht, dass jede
dichte Zeichnung einen freien Platz hat. Verbleibende Konflikte haben im Editor einen
roten Umriss.
Vorhandene manuelle Versätze behalten ihre ursprüngliche Bedeutung und werden nie
stillschweigend zurückgesetzt.

Diese Änderung der Darstellung gilt auch für automatische Beschriftungen in bestehenden
Projekten, einschließlich freigegebener Seiten und PDF-/SVG-Exporte. Das Freigeben einer
Seite schützt ihre Daten vor Änderungen; es friert die Darstellung nicht ein. Manuelle
Versätze behalten ihre Positionen.
Für einen unveränderten Stand einer früheren Freigabe das bei der Freigabe exportierte
PDF aufbewahren.
Der Editor berechnet ein Seitenlayout je Bild und verwendet die tatsächlich gezeichneten
Textflächen sowohl für die Auswahl als auch für das Ziehen. Mehrseitige Exporte
verwenden ihren Projektindex wieder.

Beim Überfahren eines Adertextes oder des Kabelkennzeichens neben einer Leitung wird
dessen Ziehfläche sichtbar. Direkt ziehen, um nur den Text zu verschieben, ohne zuerst
die Leitung auszuwählen; das geht auch bei einer vorherigen Mehrfachauswahl.
Leitungsenden bleiben eigene Geometriegriffe.
Freigegebene Seiten und schreibgeschützt geöffnete Projekte lassen sich so nicht ändern.

Eine Kabeldefinition auswählen und **Adertexte ausrichten…** in ihrem Kontextmenü, im
Menüband Verbindungen oder in ihren Eigenschaften wählen. Oberhalb/links oder
unterhalb/rechts und einen Mindestabstand wählen. Die Seite zeigt eine Vorschau; bis
**Übernehmen** ändert sich nichts.
Die gekreuzten Adertexte dieses Kabels auf dieser Seite werden gemeinsam ausgerichtet,
mit einem gemeinsamen Abstand, der bei Bedarf vergrößert wird, um Hindernissen
auszuweichen. Das Übernehmen ersetzt ihre manuellen Positionen bewusst in einem
Rückgängig-Schritt. Positionen der Kabelkennzeichen, Leitungsgeometrie,
Kabel-/Aderzuordnungen und andere Seiten bleiben unverändert. Wird keine freie
Platzierung gefunden, die Optionen ändern oder die Texte einzeln platzieren.
Gespeicherte Positionen überstehen Speichern/erneutes Öffnen und verwenden denselben
Darstellungsweg für den SVG-/PDF-Export.

Um ein Kabel umzubenennen, auf seine Definitionslinie oder Beschriftung rechtsklicken und
**Kabel umbenennen…** wählen. Dieselbe Aktion steht im Kabelnavigator und in den
Eigenschaften, die mit F2 oder einem Doppelklick geöffnet werden. Der Dialog markiert das
vollständige Kennzeichen: das neue Kennzeichen eingeben und die Eingabetaste drücken oder
mit Esc abbrechen. Verweise auf allen Seiten folgen der Umbenennung, die sich in einem
Schritt rückgängig machen lässt. Doppelte Kennzeichen und Änderungen an freigegebenen
Seiten werden abgelehnt. Gemeinsame Projekte holen die Projektsperre und die Sperren der
betroffenen Seiten, bevor die Änderung angewendet wird. Das Abbrechen gibt nur die vom
Dialog neu angeforderten Sperren frei, nachdem ihre Anforderung abgeschlossen ist;
Sperren, die andere lokale Änderungen schützen, bleiben.
Die Aktion steht auch in *Verbindungen › Kabel*. Die Automatisierung kann `rename_cable`
mit `cable` (aktuelles vollständiges Kennzeichen) und `tag` (neues vollständiges
Kennzeichen) verwenden.
Ältere Projektserver werden unterstützt: Fehlt der Endpunkt für den Sitzungs-Snapshot,
kombiniert der Editor den älteren Snapshot mit dem Sperrstatus der Sitzung bei derselben
Revision. Bei einer neueren Revision während des Ladens wird erneut versucht, statt
Stände zu mischen.

Das Zeichnen einer Kabeldefinition zeigt eine Vorschau der gekreuzten Leitungen und der
freien Adern. Ihre Eigenschaften zeigen die Zahl der zugeordneten/nicht zugeordneten und
die Zuordnungsreihenfolge, beginnend am ersten Punkt der Linie. Das Umkehren der Linie
kehrt die Reihenfolge für freie Adern um;
vorhandene Zuordnungen behalten ihre Ader. Adern, die bereits anderswo im Projekt
verwendet werden, stehen nicht zur Verfügung. Die Kabelbeschriftung (zum Beispiel
`4x1.5`) beschreibt das eingetragene Kabel,
nicht die Zahl der Kreuzungen.

Gibt es zu wenige freie Adern oder hat eine gekreuzte Leitung bereits eine
widersprechende Zuordnung, wird **keine der anstehenden Leitungen zugeordnet**. Die Linie
bleibt mit einer Prüfwarnung und einem Korrekturhinweis in ihren Eigenschaften im
Plan. Die Aderliste bearbeiten oder ein anderes Kabel wählen, dann **Gekreuzte
Verbindungen zuordnen** wählen. Vorhandene
Zuordnungen müssen ausdrücklich in den Leitungseigenschaften geändert werden; ein anderes
Kabel wird nie stillschweigend überschrieben. Die physische Kabel-/Aderliste wird nie
automatisch vergrößert, um eine Abweichung verschwinden zu lassen.

Wird das Anlegen eines neuen Kabels durch eine widersprechende Zuordnung blockiert, bleibt
nur die Definitionslinie erhalten: Es wird kein unbenutztes Kabel angelegt. Nach dem
Beheben des Konflikts ein Kabel in ihren Eigenschaften wählen. Eine importierte
Aderbezeichnung ohne
Kabel ist ebenfalls ein Konflikt; sie ausdrücklich in den Leitungseigenschaften löschen
oder ändern.
Die deaktivierte Schaltfläche **Gekreuzte Verbindungen zuordnen** erklärt beim Überfahren
Engpässe und Konflikte. Interaktive Vorschauen verwenden den Zuordnungsindex der Sitzung
wieder; das Anwenden
prüft immer das aktuelle Projekt erneut.

Die Automatisierung verwendet `preview_cable_definition` mit `page`, zwei `points` und
einem optionalen `cable` für eine reine Lesevorschau. `apply_cable_definition` nimmt
`page` und die `id` der Definition und meldet `changed` sowie die resultierende
`assignment`.
`add_cable_definition` behält seine bisherigen Antwortfelder und ergänzt `assignment`,
einschließlich Zählwerten, freier Adern, Konflikten und geordneter
Leitungs-/Ader-Kandidaten.

Ein Verbindungsdefinitionspunkt auf einer Leitung legt
Nummer, Farbe und Querschnitt der ganzen Verbindung und die Ader seiner Leitung fest; eine
Kabeldefinitionslinie, die über Leitungen gezeichnet wird, macht sie zu Adern eines Kabels
(neues Kabel mit nummerierten Adern oder ein gewähltes). Die Prüfung meldet Punkte neben
Leitungen, widersprüchliche oder nicht angewendete Punkte und Linien unbekannter Kabel.

## Verbindungssymbole: T-Stücke, Ecken, Kreuzungen und Brücken

Drei Betriebsmittel auf einem Potenzial können als A–B und A–C oder als A–B und B–C
verdrahtet sein; die Netzliste allein kann das nicht unterscheiden. Ein **T-Stück**
(*Einfügen › Verbindungen ›
T-Stück*, platziert, wo drei Leitungen zusammentreffen) speichert diese Wahl mit dem
Projekt, im Sinne der Verbindungssymbole von EPLAN: Seine drei Anschlüsse sind der Stamm
(die Richtung des T-Stücks: links, rechts, oben oder unten) und die beiden Arme des
Balkens quer dazu. Vier Varianten je Richtung benennen den Anschluss der *Quelle* und die
Reihenfolge der beiden *Ziele*:

| Variante | Quelle | Erstes Ziel | Zweites Ziel |
| --- | --- | --- | --- |
| 1 | linker / oberer Arm | der andere Arm (geradeaus) | der Stamm |
| 2 | rechter / unterer Arm | der andere Arm (geradeaus) | der Stamm |
| 3 | Stamm | linker / oberer Arm | rechter / unterer Arm |
| 4 | Stamm | rechter / unterer Arm | linker / oberer Arm |

Die beiden physischen Drähte der Verbindung laufen vom Betriebsmittel an der Quelle zum
Betriebsmittel am ersten Ziel und zum Betriebsmittel am zweiten Ziel. Das Betriebsmittel
eines Anschlusses ist der erste Anschlusspunkt entlang seiner Leitung, auch über
Unterbrechungspunkte hinweg; wo die Leitung ein anderes T-Stück erreicht, gibt dieses
seine Ziele weiter (wenn die Leitung an seiner Quelle ankommt) oder seine Quelle. Die
Verbindungsliste, die Verbindungspläne, die
Schaltschrankverdrahtung, die Kabelbäume und jeder andere Nutzer der Verbindungen
verwenden dieses Ergebnis. Verbindungen ohne T-Stück behalten wie bisher die
Schaltplanreihenfolge, und ein
T-Stück, das nicht zu seinen Leitungen passt (nicht drei Leitungen, keine in seiner
Richtung, ein Anschlusspunkt an der Stelle), wird ignoriert und von der Prüfung
*Verbindungssymbole* gemeldet, ebenso zwei Symbole auf einem Punkt und eine Verbindung,
deren Symbole sie in einer
Schleife verdrahten (mehr Drähte, als ihre Betriebsmittel brauchen, z. B. die Quellen
zweier T-Stücke, die sich über eine
Leitung hinweg gegenüberstehen).

Das Symbol ist eine kurze Diagonale von der Quelle zum zweiten Ziel (das erste Ziel
liegt geradeaus); *Darstellung › Punkt* zeichnet stattdessen den üblichen
Verbindungspunkt und behält die Verlegung. Das Fenster Eigenschaften zeigt die vier
Varianten als Vorschauen mit nummerierten Zielen, die Richtung und die Drähte der
Verbindung. Das Drehen oder Spiegeln der Zeichnung dreht das T-Stück mit seinen
Leitungen. Verbindungssymbole folgen ihren Leitungen:
Bewegt sich ein Betriebsmittel oder ein Abgang entlang der Schiene, gleitet das T-Stück
mit dem Abgang; bewegt sich die Schiene, dehnen sich ihre Abgänge und das T-Stück bleibt
auf ihr; bewegt sich der ganze Knoten,
bewegt sich das T-Stück mit.

Solange ein Werkzeug für Verbindungssymbole aktiv ist, enthält das Fenster Eigenschaften
(vom Werkzeug wieder eingeblendet,
wenn es geschlossen war) die Verlegungen des nächsten Symbols zur Auswahl, Tab schaltet sie durch und R (Drehen) dreht das Symbol. Ein gerade
platziertes Symbol ist nicht ausgewählt, diese Auswahl wirkt also nur auf die noch zu
platzierenden Symbole; ein
platziertes wird geändert, indem man es auswählt. Wo bereits drei Leitungen
zusammentreffen, übernimmt ein T-Stück
seine Richtung von ihnen und der gewählten Verlegung.

Eine **Ecke** (*Einfügen › Verbindungen › Ecke*) ist das zweite Verbindungssymbol: zwei
Leitungen, die sich im rechten Winkel treffen, mit vier Ausrichtungen (Tab oder R wählt
beim Platzieren die nächste).
Sie
ändert keine Verlegung; ihr Nutzen ist das automatische Verdrahten: Ist *Autoverbinden*
eingeschaltet, zeichnet das Platzieren einer
Ecke (oder eines T-Stücks) Leitungen von Anschlusspunkten, die ihren freien Armen
geradeaus zugewandt sind (auch von einem T-Stück auf einer durchlaufenden Leitung), und
ein Symbol, das so platziert
oder verschoben wird, dass ein Anschlusspunkt einem freien Arm zugewandt ist, wird mit ihm verdrahtet. So verdrahtet eine Ecke am Treffpunkt zweier Verbindungen beide
mit. Eine Ecke, die zu ihren Leitungen passt, wird als nichts weiter als der Knick
gezeichnet; eine ohne ihre beiden
Leitungen zeigt ihre Arme in der Fehlerfarbe und wird von der Prüfung gemeldet.

Eine **Kreuzung** (*Einfügen › Verbindungen › Kreuzung*) verbindet die beiden Leitungen,
die sich an ihrem Punkt kreuzen und ohne sie unverbunden bleiben. Sie besteht aus zwei
verbundenen T-Stücken: Jedes Ende
ihrer geraden Linie (senkrecht oder waagerecht) ist zuerst geradeaus mit dem anderen Ende
und als Zweites quer mit einem Arm verdrahtet; die beiden Varianten entscheiden, welches
Ende den linken / oberen
Arm nimmt. Im Beispiel von EPLAN führt eine senkrechte Kreuzung der Variante 2 eine
Verbindung von oben
zuerst nach unten und als Zweites nach links, und eine von unten zuerst nach oben und als
Zweites nach rechts. Eine **Brücke** (*Einfügen › Verbindungen › Brücke*) hat ebenfalls
vier Leitungen
und ist ein Stern von der Seite, von der die Verbindung kommt, zu den drei anderen: zuerst
die gegenüberliegende
Seite, dann die linke oder obere, dann die rechte oder untere. Beide lassen sich als
Punkt zeichnen, behalten dann ihre Verlegung und drehen sich mit der Zeichnung. Jedes
Symbol ist eine Liste von
Verknüpfungen zwischen seinen Armen; wo eine Leitung ein anderes Symbol erreicht, gibt
dieses Symbol die
Betriebsmittel der Arme weiter, die mit dem verknüpft sind, an dem die Leitung ankommt.

Befehle: `add_t_junction` (Richtung aus den Leitungen dort, wenn nicht angegeben),
`add_corner`
(Arme aus den Leitungen dort, z. B. `down left`), `add_crossing` (`line` vertical oder
horizontal, `variant` 1 oder 2), `add_bridge` (`direction`, aus der die Verbindung
kommt), alle
mit `autoconnect`, um die Leitungen zu zugewandten Anschlusspunkten zu zeichnen, `update_connection_symbol`,
`list_connection_symbols` (Art, Verlegung, Verknüpfungen und die Drähte von jedem).
`autoconnect`
nimmt auch die IDs von Verbindungssymbolen.

## Verbindungsnummern

Eine Nummer je Verbindung (alle ihre Leitungsstücke; die beiden Seiten einer
Klemme sind getrennte Verbindungen). Automatische Nummerierung: fortlaufend, Seite/Spalte
(`3.4.1`) oder nach Potenzialname (`L1`, `+24V`), mit Präfix und Startwert; manuell
gesetzte Nummern bleiben erhalten. Gezeigt im Schaltplan, in der Verbindungsliste und
der Leitungsliste des Schaltschranks; Prüfungen auf widersprüchliche und doppelte
Nummern.

## Nummerierung

**Nummerierung** (im Sinne der Offline-Nummerierung von EPLAN): *Betriebsmittel ›
Betriebsmittel nummerieren…* nummeriert Betriebsmittelkennzeichen je Struktur und
Kennbuchstabe in Schaltplanreihenfolge
(Seite, Spalte, von oben nach unten) nach einem Format neu: `{letter}{n}` (`-K1`),
`{letter}{page}.{n}` (`-K3.1`), `{letter}{page}{column}` (`-K34`) oder im NFPA-Stil
`{page}{column}{letter}` (`-34K`), mit Startwert, Schrittweite, Buchstaben und
Geltungsbereich sowie einer
Vorschau der neuen Kennzeichen; alles, was auf ein Kennzeichen verweist, folgt (Artikel,
Leisten,
Stecker, SPS-Daten, Schaltschrankaufbau, Kabelbäume); Kennzeichen von Betriebsmitteln,
die nicht platziert sind
(Betriebsmittelliste, Schaltschrankaufbau), werden nicht an andere vergeben. *Start ›
Seiten neu nummerieren…* nummeriert Seiten
mit Startwert und Schrittweite, je Struktur neu beginnend und unter Beibehaltung der
Unterseiten (`3.a`; *Start › Neue Seite › Neue
Unterseite* fügt `3.1`, `3.2`, … nach der aktuellen Seite ein). Seitenabschnitte
enthalten Seiten
und Unterabschnitte wie Verzeichnisse (*Start › Neuer Abschnitt*, Seiten im Navigator
*Seiten* darauf ziehen); die Seitennamen tragen die Abschnittsnummern (`2.3.1`), folgen,
wenn ein
Abschnitt neu nummeriert oder verschoben wird, und das Neunummerieren zählt je Abschnitt.
Die Leitungsnummerierung
erhält ein Schema *Format* mit Platzhaltern (`{page}`, `{column}`, `{row}`,
`{potential}`, `{kind}`, `{color}`, `{cross_section}`, Zähler `{n}`, gezählt je
Wert des Rests), Zählerstellen und Schrittweite sowie Potenzialarten, die keine Nummer
erhalten
(z. B. PE). Befehle: `renumber_devices`, `renumber_pages`, `set_wire_numbering`.
