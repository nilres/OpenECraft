# Fluidtechnik, Verfahrenstechnik und Anlagenplanung

Schläuche und Rohre in eigenen Netzen, Elektro-Fluid-Verknüpfungen, PLT-Stellen, die Vorplanung der Anlage und die einpolige / allpolige Projektierung.

## Fluidtechnik

Leitungen zwischen den Anschlüssen von Fluidtechnik- und R&I-Symbolen sind Schläuche /
Rohre in eigenen Netzen (nicht mit elektrischen Potenzialen vermischt; ein
Fluidanschluss an einem elektrischen Netz ist ein Fehler), getrennt von den Drähten
nummeriert (`H1`, `H2` …), mit Medium, Nennweite, Farbe und Länge, aufgelistet in der
*Schlauchliste*.

- *Fluid-Netzliste*: Jeder Anschluss erhält seine Funktion aus seiner Bezeichnung nach
  ISO 1219 / ISO 5599 (Versorgung, Druck P/1, Arbeitsanschluss A/B/2/4, Entlüftung
  R/S/3/5, Rücklauf T, Steueranschluss X/Y/Z/12/14, Leckage L, durchgehend) und jede
  Schlauchleitung ihre Art (Druck-, Arbeits-, Rücklauf- / Abluft-, Steuerleitung …) mit
  ihren Abzweigen (T-Stücke).
- *Schlauchkonfiguration*: je Schlauch oder Rohr der Leitungstyp, der Artikel
  (Meterware), die Endverschraubungen, der Höchstdruck und der Biegeradius; die
  Zuschnittlänge enthält zusätzlich die Schnittzugabe des Projekts je Ende. Schläuche
  zwischen Betriebsmitteln, die auf derselben Montageplatte platziert sind, werden wie
  Drähte durch deren Kanäle verlegt, was die Länge ergibt, wenn keine eingestellt ist.
  Die *Rohr- / Schlauchmaterialliste* summiert Stücke, Zuschnittlängen, Bögen und
  Verschraubungen je Typ, Nennweite und Artikel.
- *Fluid-Gerätegruppen*: Ventilinseln, Wartungseinheiten, Verteilerblöcke und Aggregate
  mit ihren Geräten in der Reihenfolge der Steckplätze und der Anzahl der Plätze.
- *Elektro-Fluid*: Fluidgeräte werden mit den elektrischen Betriebsmitteln verknüpft,
  die sie betätigen oder überwachen (Magnetspulen, Positions- und Druckschalter); beide
  Schaltpläne zeigen Querverweise aufeinander, und *Abgleichen* verknüpft Magnetventile
  mit der Spule derselben Nummer (`-QM3` → `-MB3`) und kopiert Funktionstexte.
- Prüfungen: Druckleitungen an einem Rücklauf / einer Entlüftung, unterschiedliche
  Medien oder Nennweiten an einem Schlauch oder einem Gerät, Schläuche, die für weniger
  als den Betriebsdruck ausgelegt oder kürzer als ihr Weg sind, Magnetventile ohne
  Spule, Spulen ohne Ventil, falsche oder fehlende Verknüpfungen, volle oder doppelte
  Gruppen; offene Entlüftungen pneumatischer Geräte entlüften in die Atmosphäre und
  sind keine offenen Anschlusspunkte.
- Auswertungen: *Fluid-Netzliste*, *Schlauchkonfiguration*, *Rohr- /
  Schlauchmaterialliste*, *Fluid-Gerätegruppen*, *Elektro-Fluid-Querverweise*.

![Pneumatikplan des Beispiels pneumatic_press mit Zylindern, 5/2-Wegeventilen, Drosselrückschlagventilen und Querverweisen auf die Magnetspulen](../../images/pneumatics.png)

## Verfahrenstechnik (R&I)

R&I-Symbole (ISO 10628: Behälter, Pumpen, Armaturen, Wärmetauscher, Messgeräte)
unterstützen die Vorplanung: Messgeräte tragen Kennbuchstaben und Stellennummer
(`TIC 101`), die PLT-Stellen der Vorplanung (*Werkzeuge › Fluid und
Verfahrenstechnik…*) listen ihre Geräte auf, und die *PLT-Stellenübersicht* zeigt, ob
jede Stelle geplant, in Bearbeitung oder im Schaltplan projektiert ist.

## Vorplanung

**Vorplanung** (*Betriebsmittel › Vorplanung…*): die Anlage als Baum von Segmenten
(Anlage, Bereich, Einheit, …) mit Funktion und Ort, die im Baum nach unten vererbt
werden, und den darin geplanten Geräten (Kennzeichen, Art, Beschreibung, Leistung,
Spannung, Artikel, Seitenmakro); die geplante Leistung je Segment, Segmentvorlagen, der
Zustand jedes geplanten Geräts gegenüber dem Schaltplan (geplant, projektiert, Artikel
abweichend; auch eine Prüfung und die Auswertung *Vorplanung*), Schaltplanseiten, die
aus den Seitenmakros der noch nicht projektierten Geräte erzeugt werden, und der
Austausch als CSV / Excel (Befehle `preplanning`, `add_segment`, `plan_device`, …;
Werkzeuge `import_preplanning`, `generate_preplanning_pages`).

## Einpolige / allpolige Projektierung

**Einpolige / allpolige Projektierung** (*Betriebsmittel › Einpolig / allpolig…*):
Geräte werden mit einpoligen Symbolen (Leistungsschalter, Lasttrennschalter,
Motorschutzschalter, Sicherung, Überlastrelais, Schütz, Motor, Transformator) auf
Übersichtsseiten geplant und im allpoligen Schaltplan projektiert; beide teilen sich
das Betriebsmittelkennzeichen. Das Fenster listet jedes Gerät als geplant, projektiert,
nicht im Übersichtsschaltplan oder nicht passend auf und erzeugt den Schaltplan einer
einpoligen Seite (3-polige Leistungssymbole, Leiter, Schützspulen) oder einen
Übersichtsschaltplan der Leistungsgeräte des Schaltplans (angeordnet von der
Einspeisung zu den Verbrauchern). Die Prüfung meldet geplante Geräte, nicht passende
Symbole, Verbindungen des Übersichtsschaltplans, die im Schaltplan fehlen, und
abweichende Funktionstexte; *Funktionstexte abgleichen* füllt leere aus der anderen
Darstellung.
