# Kabelbäume

Kabelbäume mit 3D-Topologie, Schutz- und Befestigungselementen, KBL-Export, der
Kabelbaum-Schnittstelle für Maschinen und Nagelbrettzeichnungen (*Verbindungen ›
Kabelbäume…*). Das Beispielprojekt `conveyor` hat einen Türkabelbaum.

## Kabelbäume und ihre Leitungen

Einzelne Punkt-zu-Punkt-Verbindungen (`-S1:13 – -K1:A1`) werden Kabelbäumen zugeordnet
(`-WH1`), unabhängig von den anderen Verbindungen desselben Potenzialabschnitts;
Zuordnungen folgen umbenannten Betriebsmitteln, fehlende Verbindungen werden von der
Prüfung gemeldet. Die Auswertung *Kabelbaumdaten* listet ihre Leitungen mit Länge (im
Schaltschrankaufbau verlegt oder die im Kabelbaum eingetragene Länge) und Endenbehandlung auf, das
Kabelbaumfenster zeigt die Steckverbinder mit ihrer Pinbelegung und den
Bündeldurchmesser.

## 3D-Kabelbaumgeometrie

Knoten und Segmente mit Mittellinien, abgeleitet aus den 3D-Verlegungen des
Schaltschrankaufbaus (Steckverbinder außerhalb davon um einen gemeinsamen Knoten
angeordnet) oder im Kabelbaum gespeichert und bearbeitet (Positionen, Biegeradien,
Nagelbrettlängen, Stützpunkte); jede Leitung wird durch die Segmente geführt, jedes
Segment hat seinen Bündeldurchmesser; Band, Schläuche, Wellrohre, Geflechtschläuche und
Schrumpfschlauch auf (Teilen von) Segmenten, Clips, Kabelbinder, Tüllen und Halter an
Positionen; die Prüfung meldet unterbrochene Topologien und zu kleine Biegeradien;
Export des Bündels in 3D (glTF, JT).

## KBL

VDA 4964, Schema KBL 2.4 SR-1, in den Tests gegen das Schema geprüft: ein Container je
Kabelbaum mit allgemeinen Leitungen und Kabeln (Adern), Steckverbindergehäusen mit
Slots und Kammern, Kontakten, Dichtungen und Blindstopfen, Leitungsschutz,
Befestigungen, Zubehör, Beipackteilen, Baugruppen und Komponenten, den Verbindungen
zwischen Kontaktpunkten, Schaltplanverbindungen (Potenzialen), Verdrahtungsgruppen,
Projektoptionen als Modulen, Revisionen, Freigaben, Einheiten, Standardtoleranzen und
der 3D-Topologie (Exporter `kbl`, `kbl_zip`).

## Kabelbaum-Schnittstelle

Für Maschinen und Werkzeuge ([harness-interface.md](../../harness-interface.md)): die
vollständigen Kabelbaumdaten als JSON (`harness_json`, Befehl `harness_data`) und die
Leitungsfertigungsliste für Schneide-, Abisolier- und Crimpautomaten
(`harness_processing`, CSV in Fertigungsreihenfolge mit Zuschnittlängen,
Abisolierlängen, Kontakten und Dichtungen); das Plugin *Kabelbäume* bearbeitet
Topologie, Schutzelemente, Befestigungen, Endenbehandlung und weitere Teile von der
Kommandozeile und über MCP.

## Nagelbrettzeichnungen

Maßstäbliche Nagelbretter: das Bündel als Äste von einem gemeinsamen Knoten zu den
Steckverbindern, jeder so lang, wie es die Leitungslängen erfordern, und so dick wie
sein Bündel, mit Nägeln, Astlängen und der Pintabelle jedes Steckverbinders, auf A3 im
kleinsten Normmaßstab, der passt (Kabelbaumfenster › *Nagelbrett (SVG)…*, Exporter
`nailboard`, Befehl `harness_nailboard`).
