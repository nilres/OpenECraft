# Elektrische Dimensionierung, Berechnungen und Nachweise

Leiterdimensionierung, Kurzschluss und Schutzkoordination, Lastberechnung, EMV, funktionale Sicherheit und Regelwerke – alles Teil der Projektprüfung – sowie das Prüfprotokoll der Anlage.

## Elektrische Dimensionierung

**Elektrische Dimensionierung** (IEC 60204-1): Die Leitungen hinter einer
Schutzeinrichtung – von ihrer Lastseite über Hauptkontakte und Überlastrelais – müssen
ihren Bemessungsstrom führen können (Artikel `Current`, der obere Wert eines
Einstellbereichs; Strombelastbarkeit nach Tabelle 6, Verlegeart B2); Kabel erhalten
ihren Spannungsfall bis zur Last mit dem höchsten Bemessungsstrom an ihren Enden (√3
bei drei Adern, Kupfer bei Betriebstemperatur), Auswertung *Kabeldimensionierung*; die
Prüfung meldet zu dünne Leitungen und Kabel (Fehler) und Spannungsfälle über 5 %. Die
Belastbarkeit wird für Umgebungstemperatur, Häufung von Stromkreisen
(IEC 60364-5-52, B.52.17), Aluminium und VPE-Isolierung korrigiert. Ein
Sicherungslasttrennschalter, der mit Trennmessern bestückt ist (Artikel-Untergruppe
`Solid link`, die Freischaltung vor und hinter den Wandlern einer Messung), trennt nur:
Er schützt nichts, hat keine eigene Selektivität und der Strom fließt durch ihn
hindurch; die Last eines Kabels ist der Bemessungsstrom des Artikels, der die Funktion
an seinem Ende bereitstellt (der Motorantrieb an einem Leistungsschalter, nicht der
Schalter).

## Elektrische Berechnungen und Nachweise

**Elektrische Berechnungen und Nachweise** (*Werkzeuge › Elektrische Berechnungen…*):
- *Versorgung*: Spannung, Netzform (TN-S, TN-C, TN-C-S, TT, IT), größter und kleinster
  Kurzschlussstrom am Einspeisepunkt, Bemessungsstrom;
- *Kurzschluss und Schutzkoordination* (IEC 60909-0 für Strahlennetze,
  IEC 60364-4-41, -4-43): die Quelle aus den Kurzschlussströmen am Einspeisepunkt (mit
  ihrem R/X) oder aus einem Transformator (Bemessungsleistung, uk, Kurzschlussverluste,
  Korrekturfaktor KT) und dem vorgelagerten Netz (S''kQ); die Kabel von der Einspeisung
  zu jeder Schutzeinrichtung und weiter bis zum Ende ihres Stromkreises;
  Asynchronmotoren am Netz tragen ILR / IrM = 5 (oder `Locked rotor current`) bei, wenn
  sie 1 % des Stroms der Einspeisung überschreiten. Die Schutzeinrichtungen des Netzes
  mit ihren Zeit-Strom-Bändern (IEC 60898-1 B / C / D / K / Z, Motorschutz Klasse 10,
  gG- / aM-Sicherungen, elektronische LSI-Auslöser mit `Long-time delay`, `Short-time
  pickup`, `Short-time delay`; technische Daten des Artikels `Characteristic`,
  `Breaking capacity`, `Making capacity`, `Instantaneous trip`, `Let-through energy`,
  `Pre-arcing energy`), ersetzt durch die Kennlinien des Herstellers, wo sie angegeben
  sind (`Trip curve min` / `Trip curve max`: `1.5: 600; 3: 20; 10: 0.01` in Vielfachen
  von In, `Let-through energy: 1 kA: 8000; 6 kA: 25000`). Geprüft werden:
  Ausschaltvermögen gegen den Kurzschlussstrom und Einschaltvermögen gegen dessen
  Stoßwert (κ aus R/X), Abschaltung beim kleinsten Fehlerstrom am Ende jedes
  Stromkreises innerhalb der Zeiten der Tabelle 41.1 nach U0 (TN 0,8 / 0,4 / 0,2 /
  0,1 s, TT 0,3 / 0,2 / 0,07 / 0,04 s, Verteilerstromkreise 5 s / 1 s; TT:
  Fehlerstrom-Schutzeinrichtung erforderlich; IT: der zweite Fehler, bei U oder mit
  verteiltem Neutralleiter bei U0), k²S²-Festigkeit der Kabel, Selektivität zur
  vorgelagerten Einrichtung (voll / teilweise bis zu einem Strom) aus der
  Koordinationstabelle des Herstellers, gG-Sicherungen im Verhältnis 1,6, der
  Schmelzenergie einer vorgelagerten Sicherung oder den Zeit-Strom-Bändern;
  Back-up-Schutz aus der Koordinationstabelle deckt ein zu geringes Ausschaltvermögen
  ab. Die Koordinationstabellen (Selektivitäts- und Back-up-Grenzen in kA nach
  Artikelnummer) werden im Fenster und mit `coordination_table` bearbeitet.
  Auswertungen *Kurzschluss und Abschaltung*, *Selektivität* (mit ihrer Grundlage);
- *Lastberechnung*: Motoren, Antriebe, Heizungen, Netzgeräte und Transformatoren des
  Netzes mit Wirk- / Scheinleistung und Strom (`Power`, `Current`, `Power factor`,
  `Efficiency`), die Last jeder Schutzeinrichtung, Bedarf mit dem
  Bemessungsbelastungsfaktor der IEC 61439-1 (Tabelle 101) oder einem eingestellten
  Faktor, Außenleiterströme und Unsymmetrie sowie die Lastbilanz jedes 24-V-Netzgeräts
  (`Coil power`, `Current consumption`); Auswertungen *Lastberechnung*, *Lastbilanz
  der Netzgeräte*;
- *EMV*: Motorkabel von Frequenzumrichtern beidseitig geschirmt, Analogsignale in
  geschirmten Kabeln, keine Leistungs- und Signalleiter in einem Kabel,
  Überspannungsschutz;
- *Funktionale Sicherheit* (ISO 13849-1): Sicherheitsfunktionen aus Teilsystemen mit
  ihren Geräten, Kategorie, DCavg, Betätigungen pro Jahr und CCF; PL nach dem
  vereinfachten Verfahren (Bild 5) aus `MTTFd` / `B10d` der Artikel oder
  zertifiziertem `PL` / `PFHd`, kombiniert nach Tabelle 11 oder, wo jedes Teilsystem
  einen PFHd hat, über deren Summe, mit dem passenden SIL; ein zweiter Kanal aus
  anderen Geräten wird symmetrisiert (D.2), der DCavg ergibt sich aus dem `DC` der
  Geräte, wenn das Teilsystem keinen angibt (Anhang E), Geräte, deren T10d innerhalb
  der Gebrauchsdauer von 20 Jahren liegt, werden zum Austausch genannt; Auswertung
  *Sicherheitsfunktionen*;
- *Regelwerke*: eingebaute Bibliotheken (IEC 60204-1, IEC 61439-1, UL 508A / NFPA 79,
  ISO 13849-1, EMV) und Kundenregelwerke (JSON-Import / -Export), die die Grenzwerte
  verschärfen (Spannungsfall, Abschaltzeiten, Mindestquerschnitte, Lastreserve,
  Unsymmetrie der Außenleiter, volle Selektivität, Schirmung, Überspannungsschutz) und
  Prüfungen der Form „jedes Element eines Geltungsbereichs (Projekt, Seiten,
  Betriebsmittel, Artikel, Kabel, Verbindungen, Potenziale), das diese Bedingungen
  erfüllt, muss diese erfüllen“ hinzufügen; Auswertung *Prüfung gegen Regelwerke*;
  alle Ergebnisse sind Teil der Projektprüfung.

## Prüfprotokoll

Das **Prüfprotokoll** (Erstprüfung, IEC 60364-6 / DIN VDE 0100-600) hat vier Teile: die
Daten des Protokolls mit der Besichtigung, die Quellen von SELV-, PELV- und
schutzgetrennten Stromkreisen, die Stromkreise mit ihren Ergebnissen und das
Gesamtergebnis mit der Unterschrift. Es wird in den Auswertungen *Prüfprotokoll* (die
Stromkreise), *Prüfprotokoll: SELV, PELV und Schutztrennung* und
*Prüfprotokoll: Zusammenfassung* angezeigt, im Bereich *Werkzeuge › Prüfprotokoll* des Editors bearbeitet (dieselben
Abschnitte wie auf der Webseite: Protokolldaten und Besichtigung, Grenzwerte,
Unterschrift, ein Abschnitt je Quelle und je Stromkreis, *Excel exportieren*), in
*Projektdaten › In Tabelle bearbeiten* (*Prüfprotokoll*, eine Zeile je Stromkreis, und
*Prüfprotokoll: Daten*, eine Zeile je Wert des Protokolls und der Quellen), mit dem
Befehl `inspection_report` und auf der Webseite des Projektservers.

**Die Stromkreise** sind die aus dem Netz gespeisten Schutzeinrichtungen in der
Reihenfolge des Schaltplans, ausgefüllt aus dem Schaltplan:

- Stromkreis (Betriebsmittelkennzeichen), Beschreibung (des Betriebsmittels, sonst
  sein Funktionstext), Außenleiter, die Kabel hinter dem Betriebsmittel mit ihren
  Typen, der Querschnitt (der Kabel, sonst der Leitungen);
- Schutzeinrichtung: Auslösecharakteristik und Bemessungsstrom (Artikeldaten, wie für
  die Kurzschlussberechnung);
- Fehlerstrom-Schutzeinrichtung, die den Stromkreis schützt (seine eigene oder eine
  vorgelagerte), mit ihrem Typ (`Residual current type` des Betriebsmittels oder
  seines Artikels; ein `S`, wie in `A S`, kennzeichnet eine selektive) und ihrem
  Bemessungsdifferenzstrom;
- die größte Schleifenimpedanz, die besteht: zwei Drittel von U0 / Ia (Ia: der Strom,
  der innerhalb der Zeit der Tabelle 41.1 abschaltet; IEC 60364-6, Anhang C; auch in
  TT-Systemen, 411.5.4; IT: der zweite Fehler, U / 2 Ia oder mit verteiltem
  Neutralleiter U0 / 2 Ia, 411.6.4), oder, wo eine Fehlerstrom-Schutzeinrichtung den
  Stromkreis schützt, U0 / IΔn (TN, IT) und 50 V / IΔn (TT: RA × IΔn ≤ UL, 411.5.3; Zs
  enthält RA).

Was das Projekt nicht hergibt, wird als *fehlt* angezeigt, niemals angenommen: keine
Charakteristik C, keine 30 mA und kein Grenzwert, der darauf beruht – anders als bei
den Berechnungen, die sie annehmen.

Die Ergebnisse eines Stromkreises, so behalten, wie sie eingegeben wurden (`0,35`,
`> 500`), und gegen ihre Grenzwerte geprüft:

| Ergebnis | Grenzwert |
| --- | --- |
| Durchgängigkeit des Schutzleiters (Ω) | kein Grenzwert in der Norm; daneben der aus den Kabeln des Stromkreises berechnete Widerstand (Länge, Querschnitt, Leiterwerkstoff bei 20 °C), ein Hinweis oberhalb des 1,5-Fachen davon |
| Messspannung Isolation (V) | mindestens 500 V bis 500 V Nennspannung, 1000 V darüber (Tabelle 6.1) |
| Isolation L–PE (MΩ; die Außenleiter oder alle miteinander verbundenen aktiven Leiter gegen PE) | mindestens 1 MΩ |
| Isolation N–PE und L–N (MΩ, wo getrennt gemessen) | mindestens 1 MΩ |
| Schleifenimpedanz Zs (Ω) | die größte Schleifenimpedanz oben |
| Spannungsfall (%) | der Grenzwert der Regelwerke (5 % ohne eines) |
| Auslösestrom RCD (mA) | ½ IΔn … IΔn |
| Auslösezeit RCD bei IΔn (ms) | höchstens 300 ms; Typ S 130 … 500 ms |
| Auslösezeit RCD bei 5 × IΔn (ms) | höchstens 40 ms; Typ S 50 … 150 ms |
| Berührungsspannung UB bei IΔn (V) | höchstens 50 V |
| Polarität | OK / Nicht OK |
| Drehfeld (Drehstromkreise) | OK / Nicht OK |
| Funktionsprüfung (Schaltgeräte, Verriegelungen, Steuerungen) | OK / Nicht OK |
| Geprüft von, Datum, Messgerät, Bemerkung | aufgezeichnet |

Ein Stromkreis ist *Nicht bestanden*, wenn ein Wert außerhalb seines Grenzwerts liegt
oder eine Prüfung fehlgeschlagen ist, *Bestanden*, wenn die Durchgängigkeit, die
Isolation (mit ihrer Messspannung), die Schleifenimpedanz, die Polarität, die
Funktionsprüfung, (bei Drehstrom) das Drehfeld und (mit einem RCD) die Auslösezeit bei
IΔn vorliegen, *Unvollständig* dazwischen und *Nicht geprüft* ohne Werte.

**SELV, PELV und Schutztrennung** (6.4.3.4): Jedes Netzgerät und jeder Transformator
des Schaltplans ist eine Quelle mit ihrer Ausgangsspannung (die `Voltage` des Artikels
nach dem `/`, wie in `230 V / 24 V DC`, oder `Output voltage`; DC bei Netzgeräten, AC
bei Transformatoren). Der Prüfer wählt ihre Schutzmaßnahme – SELV, PELV,
Schutztrennung oder keine eigene – und trägt die Isolation ihrer Stromkreise gegen
andere Stromkreise und Erde mit der Messspannung ein: SELV und PELV mindestens 250 V
und 0,5 MΩ (Tabelle 6.1), Schutztrennung wie das Netz. Eine SELV- oder PELV-Quelle über
50 V AC oder 120 V DC besteht nicht.

**Der am Speisepunkt gemessene Fehlerstrom** wird gegen das Ausschaltvermögen der
Schutzeinrichtungen geprüft (Artikeldaten, `Breaking capacity`; 434.5.1): Eine direkt
vom Speisepunkt gespeiste Einrichtung, deren Ausschaltvermögen geringer ist, besteht
nicht, eine weiter hinten (wo der Fehlerstrom kleiner ist) wird zum Prüfen markiert;
Back-up-Schutz durch die vorgelagerte Einrichtung (Koordinationstabellen) deckt das
ab.

**Die Daten des Protokolls**: Auftraggeber, Ort der Anlage, Errichter, Grund der
Prüfung (Neuanlage, Erweiterung, Änderung, Instandsetzung), Prüfer, Messgeräte
(IEC 61557), die am Speisepunkt gemessenen Werte (äußere Schleifenimpedanz Ze,
unbeeinflusster Fehlerstrom, Erdungswiderstand RA, Schutzpotenzialausgleich über die
Haupterdungsschiene, der Widerstand isolierender Fußböden und Wände nichtleitender
Räume: mindestens 50 kΩ bis 500 V, 100 kΩ darüber, IEC 60364-4-41, C.1) und das Datum
der nächsten Prüfung. In einem TT-System darf RA nicht größer sein als 50 V / der
größte Bemessungsdifferenzstrom der Anlage. In einem IT-System kommen die
Funktionsprüfung der Isolationsüberwachungseinrichtung und der Strom eines ersten
Fehlers Id hinzu, der 50 V / RA nicht überschreiten darf (411.6.2). **Die
Besichtigung** (6.4.2)
beantwortet 16 Punkte mit OK, Nicht OK oder Entfällt: Schutz gegen elektrischen Schlag,
Brandschottungen, Leiter, Schutzeinrichtungen, Trenn- und Schaltgeräte, äußere
Einflüsse, Kennzeichnung von N und PE, einpolige Schaltgeräte in den Außenleitern,
Schaltpläne und Warnhinweise, Beschriftung, Anschlüsse, Schutz- und
Potenzialausgleichsleiter, Zugänglichkeit, zusätzlicher Schutz durch RCDs mit höchstens
30 mA, Überspannungsschutz, Maßnahmen gegen elektromagnetische Störungen. Eigene Punkte
werden für die Anlage ergänzt (die Punkte der Ausgabe, gegen die die Anlage geprüft
wird, die Checkliste eines Kunden); sie gehen wie die anderen in das Gesamtergebnis
ein.

**Die Grenzwerte** sind Einstellungen des Protokolls: Jeder hat einen Standardwert und
seine Quelle, und ein Projekt setzt andere Werte, wo die Ausgabe der Norm, gegen die
geprüft wird, abweicht (*Grenzwerte* auf der Protokollkarte der Webseite, die Zeilen
`limit.<name>` von *Prüfprotokoll: Daten*, `limit.<name>` im `header` von
`inspection_report`). Die Standardwerte wurden im November 2026 mit öffentlichen
Sekundärquellen (Hersteller- und Schulungsunterlagen) verglichen, nicht mit dem Text
der Normen:

| Grenzwert | Standardwert | Quelle |
| --- | --- | --- |
| Kleinster Isolationswiderstand des Netzes | 1 MΩ | IEC 60364-6, Tabelle 6.1 (6A in der Ausgabe 2006) |
| Kleinster Isolationswiderstand und Messspannung von SELV und PELV | 0,5 MΩ bei 250 V | IEC 60364-6, Tabelle 6.1 |
| Anteil von U0 / Ia, den die gemessene Schleifenimpedanz erreichen darf | ⅔ | IEC 60364-6, informativer Anhang (C.61.3.6.2 der Ausgabe 2006); BS 7671 verwendet 0,8 |
| Vereinbarte Grenze der Berührungsspannung UL | 50 V | IEC 60364-4-41, 411.5.3 (TT), 411.6.2 (IT) |
| Auslösezeit RCD bei IΔn / 5 × IΔn | 300 ms / 40 ms | IEC 61008-1, IEC 61009-1 |
| Selektive RCDs bei IΔn / 5 × IΔn | 130 – 500 ms / 50 – 150 ms | IEC 61008-1, IEC 61009-1 |
| Höchste Spannung von SELV und PELV | 50 V AC, 120 V DC | IEC 60364-4-41, 414.1.1 |
| Isolierende Fußböden und Wände | 50 kΩ bis 500 V, 100 kΩ darüber | IEC 60364-4-41, C.1 (britische Leitfäden verwenden 0,5 MΩ) |
| Schutzleiter: Hinweis oberhalb des berechneten Widerstands | 1,5-fach | OpenECraft; die Norm legt keinen Grenzwert fest |

**Das Gesamtergebnis** des Protokolls ist *Nicht bestanden*, wenn ein Stromkreis, eine
Quelle, ein Punkt der Besichtigung oder ein Wert des Protokolls nicht besteht,
*Bestanden*, wenn jeder Stromkreis und jede Quelle bestanden hat, jeder Punkt
beantwortet, der Prüfer genannt, (TT) RA gemessen und (IT) die
Isolationsüberwachungseinrichtung geprüft ist. **Die Unterschrift** hält fest, wer
wann unterschrieben hat; jede spätere Änderung der Daten oder eines Ergebnisses nimmt
die Unterschrift wieder weg. Das Protokoll wird als Excel-Arbeitsmappe exportiert (auf
der Webseite mit einem Blatt für die Zusammenfassung, den Stromkreisen und den
Quellen).

**Vor Ort, mit Tablet oder Smartphone:** Die Webseite des Projektservers
([Projektserver](../project-server.md)) zeigt das Protokoll jedes Projekts unter
*Prüfprotokoll*: eine Karte mit den Daten des Protokolls, der Besichtigung, dem
Gesamtergebnis und *Protokoll unterschreiben*, eine Karte je Quelle von SELV-, PELV-
oder schutzgetrennten Stromkreisen, dann eine Karte je Stromkreis mit den Daten
des Schaltplans und den Feldern der Ergebnisse (Zifferntastatur, der Grenzwert neben
jedem Feld, Werte außerhalb der Grenzwerte sofort markiert; die Messspannung der
Isolation, der Prüfer, das heutige Datum und das letzte Messgerät werden für einen
neuen Stromkreis vorbelegt). Wer Kommentare speichern oder das Projekt bearbeiten
darf, trägt sie dort ein und speichert jede Karte für sich, ohne das Projekt zu öffnen
und ohne Sperre; noch nicht gespeicherte Werte bleiben im Browser (auch über eine
unterbrochene Verbindung hinweg). Die Editoren, die das Projekt geöffnet haben,
übernehmen sie mit der nächsten Revision; ihre eigenen Speicherungen erhalten sie (die
Daten und die Ergebnisse werden Karte für Karte zusammengeführt, wie die
Kommentardatensätze). *Excel* lädt das Protokoll herunter.
