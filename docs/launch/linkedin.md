# LinkedIn

DRAFT – not posted. Post on a weekday morning (Central European time), after the HN /
Reddit launch, with the 60–90 s video (`video-script.md`) uploaded natively – LinkedIn
shows uploaded video in the feed, links less prominently. Put the links in the post
text, or the first comment if you prefer; keep it in your own voice.

## English

I've spent the last month building OpenECraft: a free, open source electrical CAE for
control cabinet design, in the style of EPLAN.

It runs in the browser with nothing to install – https://editor.openecraft.org – and
on Windows, macOS and Linux.

What you can do with it today:
- Device-oriented schematics: coils and contacts as one device, cross references,
  structure identifiers to IEC 81346, wire and device numbering
- Generated reports: bill of materials, terminal and cable diagrams, connection lists,
  plus a project check
- PLC tag tables for TIA Portal, Studio 5000, TwinCAT, CODESYS and others
- The panel in 3D, with the wires routed through the cable ducts and their fill level
- A wiring assistant that turns a tablet into a wiring station for the workshop
- A command line and an MCP server, so scripts and AI agents can work on projects

To be clear about where it stands: it is one month old, much of the code was written
with AI coding tools [adjust: under my direction and review], and there is no import of
EPLAN projects. The schematic, reports and exports are the solid part; 3D routing in
buildings and the CAD import are still young.

If you design or build control panels, I would value your feedback: what would you need
before you used it on a real job?

Code (MIT / Apache-2.0): https://github.com/nilres/OpenECraft
Website with live demos: https://openecraft.org

#ElectricalEngineering #ControlPanel #Automation #OpenSource #RustLang

## Deutsch

Seit einem Monat arbeite ich an OpenECraft: einem freien Open-Source-E-CAD für den
Schaltschrankbau, im Stil von EPLAN.

Es läuft ohne Installation im Browser – https://editor.openecraft.org – und unter
Windows, macOS und Linux.

Was heute schon geht:
- Betriebsmittelorientierte Schaltpläne: Spule und Kontakte als ein Betriebsmittel,
  Querverweise, Strukturkennzeichen nach IEC 81346, Leitungs- und
  Betriebsmittelnummerierung
- Auswertungen: Stückliste, Klemmenplan, Kabelpläne, Verbindungslisten und eine
  Projektprüfung
- SPS-Variablentabellen für TIA Portal, Studio 5000, TwinCAT, CODESYS und weitere
- Den Schaltschrankaufbau in 3D, mit Verdrahtung durch die Kabelkanäle und deren
  Füllgrad
- Einen Verdrahtungsassistenten, der ein Tablet zum Verdrahtungsplatz in der Werkstatt
  macht
- Kommandozeile und MCP-Server, damit Skripte und KI-Agenten an Projekten arbeiten
  können

Ehrlich zum Stand: Das Projekt ist einen Monat alt, ein großer Teil des Codes ist mit
KI-Werkzeugen entstanden [anpassen: unter meiner Anleitung und Prüfung], und EPLAN-
Projekte lassen sich nicht importieren. Schaltplan, Auswertungen und Exporte sind der
belastbare Teil; 3D-Routing in Gebäuden und der CAD-Import sind noch jung.

Wer Schaltschränke plant oder baut: Ich freue mich über Rückmeldungen. Was bräuchtet
ihr, bevor ihr es für einen echten Auftrag nehmt?

Quellcode (MIT / Apache-2.0): https://github.com/nilres/OpenECraft
Webseite mit Live-Demos: https://openecraft.org

#Elektrotechnik #Schaltschrankbau #Automatisierung #OpenSource
