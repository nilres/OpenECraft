# Symbole und Bibliotheken

Die mitgelieferten Symbolbibliotheken (IEC, NFPA / JIC, GB, GOST, JIS, Fluidtechnik, R&I, einpolig) und der Symboleditor für eigene Symbole.

## Symbolbibliothek IEC 60617

**Symbolbibliothek IEC 60617** (~330 Symbole): Relaisspulen (AC, Stromstoß, gepolt,
Zeitrelais, Zeitschaltuhren, Zähler, Über- / Unterspannungsrelais),
Schließer-/Öffner-/Wechslerkontakte (voreilend / nacheilend, Wischer, anzugs- /
abfallverzögert), Druck- / Zug- / Schlüssel- / Fuß- / Wahlschalter mit Rückstellung oder
Rastung und Schaltstellungsdiagrammen, Not-Halt mit Dreh- / Zug- / Schlüsselentriegelung,
Nocken- / Rollen- / Druck- / Strömungs- / Füllstand- / Bimetallschalter, induktive,
kapazitive, magnetische und optoelektronische Sensoren, Sicherungen (NH),
Sicherungsschalter und -trennschalter, Trennschalter und Lasttrennschalter,
Leistungsschalter, Fehlerstromschutzschalter, Motorschutzschalter, Überlastrelais, AC- /
DC- / EC-Motoren und Lüfter, Meldeleuchten, Leuchten (L, N, PE), Signalleuchten, Sirenen,
anzeigende, schreibende und zählende Messgeräte, Stromwandler,
1- und 3-phasige Transformatoren (Schaltgruppen) und Spartransformatoren, Gleichrichter
und Umrichter, Klemmen, SPS-Ein-/Ausgänge, Potenziale, Unterbrechungspunkte sowie
Elektronik: Widerstände, Kondensatoren (auch gepolt), Dioden, Z- und Fotodioden, LEDs,
NPN- / PNP-Transistoren, N-Kanal-MOSFET, Thyristor, Triac, NTC-Thermistor,
Fotowiderstand, Optokoppler und Brückengleichrichter.

## Symbole für Installationspläne

**Symbole für Installationspläne** (IEC 60617-11): Aus-, Wechsel-, Kreuz-, Serien- und
Jalousieschalter, Dimmer, Taster, Leuchtenauslass, Wandleuchte, Einbauleuchte, Strahler,
Leuchtstofflampe und Notleuchte, Steckdosen mit PE (einfach, doppelt, dreifach, mit
Schalter, spritzwassergeschützt, CEE-Drehstrom), Abzweigdose, Geräteanschlussdose, Daten-
und Antennendose, Heizelement, Raumthermostat, Motor, Lüfter, Türsprechstelle, Klingel,
Rauchwarnmelder, Bewegungsmelder, Fenster- oder Türkontakt und Verteiler, jeweils mit
einem Anschlusspunkt für das Kabel oben. Installationsplan-Seiten zeichnen ein
Betriebsmittel mit dem Plansymbol seines Artikels (*Plansymbol* in den Montagedaten des
Artikels, Befehl `set_plan_symbol`, Liste `plan_symbols`) oder dem seiner Art.

## Symboleditor und Bibliotheken

Neben den IEC-Symbolen gibt es NFPA- / JIC-Symbole
(30: Spulen, Zeitrelaisspulen, Kontakte, zeitverzögerte und Überlastkontakte,
Hauptkontakte, Taster, Not-Halt, Wahl-, Fuß-, Grenz-, Schwimmer-, Druck-,
Temperatur-, Strömungs- und Näherungsschalter, Leistungsschalter, Trennschalter,
Sicherungen, Meldeleuchten,
Hupen, Magnetspulen, Steuertransformatoren, Motoren), Fluidtechnik-Symbole (ISO 1219: Pumpen, Kompressor, Hydraulikmotor, einfach und
doppelt wirkende Zylinder, 3/2-, 4/2- und 5/2-Wegeventile, Rückschlag-, Druckbegrenzungs-,
Kugel- und Stromregelventile, Druckregler, Wartungseinheit, Filter, Speicher, Kühler,
Manometer, Behälter, Schalldämpfer), einpolige / Übersichtssymbole (Querverweise auf das
Betriebsmittel, nicht Teil
der Netzliste) und Black Boxes / SPS-Boxen. Nationale Sätze für China (GB), Russland und
die GUS (GOST) und Japan (JIS) tragen die Kennbuchstaben nach GB/T 7159 (`KM`, `KA`,
`QF`, `SB`, `HL` …), GOST 2.710 (`KM`, `KK`, `QF`, `SB`, `HL` …) und JIS C 0401
(`MC`, `MCCB`, `PBS`, `PL`, `THR` …), sodass platzierte Betriebsmittel als `-KM1`,
`-MCCB1` nummeriert werden; gezeichnet sind sie wie die IEC-Symbole, die GB/T 4728 und
JIS C 0617 übernehmen
(die ESKD-Proportionen der GOST-Zeichnungen werden nicht nachgebildet). Eigene Symbole werden im Symboleditor gezeichnet
(Linien, Rechtecke, Kreise, Texte, Kennzeichen- / Beschriftungsfelder, Anschlusspunkte im
4-mm-Raster, Funktion wie Hauptfunktion, Kontakt, Klemme, SPS-Punkt) oder von einem
mitgelieferten Symbol kopiert; sie werden im Projekt gespeichert und als
Bibliotheksdateien ausgetauscht.
Ein platziertes Symbol der Bibliothek (ein eigenes oder ein importiertes EPLAN-Symbol)
lässt sich größer oder kleiner zeichnen: **Maßstab** in seinen Eigenschaften (oder
`update_symbol` mit `scale`, 0,1 bis 10)
skaliert seine Grafik und seine Anschlusspunkte, während seine Texte ihre Größe behalten,
sodass importierte Symbole zur Zeichnung passen; die Leitungen an seinen Anschlusspunkten
folgen.
