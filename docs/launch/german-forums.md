# sps-forum.de und mikrocontroller.net

ENTWURF – nicht gepostet. Ein paar Tage nach den Entwickler-Kanälen posten
(posting-plan.md), in eigenen Worten nachbearbeiten.

## Vorher prüfen

- **sps-forum.de:** Die Forenregeln und Nutzungsbedingungen (Link im Fuß der Seite)
  am Tag lesen – hier nicht geprüft. Passendes Unterforum ist voraussichtlich
  *E-CAD* (dort laufen die EPLAN-Themen); falls es einen Bereich für Software- oder
  Projektvorstellungen gibt, dorthin. Eigenes Projekt klar als solches kennzeichnen;
  im Zweifel vorher kurz einen Moderator anschreiben. Das Forum ist ein Fachforum von
  Praktikern: kein Werbeton, konkrete Fragen stellen.
- **mikrocontroller.net:** Das Forum *Projekte & Code* ist für Vorstellungen eigener
  Projekte, laut Forenbeschreibung „nur mit Code oder Schaltplan“ und ohne Fragen; in
  der Community gilt es als Ort für (zumindest in einer ersten Version) funktionierende
  Projekte. Der Link aufs Repository erfüllt das. Mit kritischen bis rauen Antworten
  rechnen, auch zur KI-Nutzung – ruhig und sachlich bleiben. Wer eher Diskussion will,
  nimmt ein Diskussionsforum wie *PC-Programmierung* – Forenbeschreibung lesen.
- Beide Foren: Antworten selbst schreiben, nicht generieren lassen.
- Der Name „EPLAN“ ist eine Marke; nur zur Einordnung nennen („im Stil von EPLAN“),
  nicht als Vergleich in der Überschrift zuspitzen.

## sps-forum.de – Unterforum E-CAD

**Titel**

> OpenECraft: freies E-CAD im Stil von EPLAN, läuft im Browser – Feedback aus der Praxis gesucht

**Text**

Hallo zusammen,

ich bin [Name], [eine Zeile zu dir: z. B. Elektrokonstrukteur / Entwickler bei …], und
arbeite an **OpenECraft**, einem freien Open-Source-E-CAD für den Schaltschrankbau
(Lizenz MIT / Apache-2.0). Es läuft ohne Installation im Browser:
https://editor.openecraft.org – und als Desktop-Programm unter Windows, Linux und macOS.

Gleich vorweg, damit es keine Missverständnisse gibt: Das ist mein eigenes Projekt, es
ist einen Monat alt, und ein großer Teil des Codes ist mit KI-Werkzeugen entstanden
[anpassen: unter meiner Anleitung, von mir geprüft]. Genau deshalb suche ich Leute, die
das beruflich machen und mir sagen, wo es fachlich hakt.

**Was es kann (Stand heute):**

- Betriebsmittelorientierte Schaltpläne: Spule und Kontakte eines Schützes sind ein
  Betriebsmittel mit Querverweisen und Kontaktspiegel, Betriebsmittelkennzeichen mit
  Strukturkennzeichen nach IEC 81346 (`=A1+CAB1-K1`), Unterbrechungspunkte,
  Potentiale mit Farben und Querschnitten, Leitungs-, Betriebsmittel- und
  Seitennummerierung.
- Etwa 330 Symbole nach IEC 60617, dazu NFPA / JIC, Fluid (ISO 1219), R&I und
  Installationspläne; eigene Symbole im Symboleditor.
- Klemmenleisten (mehrstöckig, mit Zubehör), Stecker mit Steckerbild, Kabel mit Adern.
- Auswertungen als Fenster, CSV / Excel oder als Berichtsseiten: Stückliste,
  Betriebsmittelliste, Klemmenplan, Kabelübersicht und Kabelplan, Verbindungsliste,
  SPS-Belegung; dazu eine Projektprüfung mit Meldungsverwaltung.
- SPS: Karten mit Ein- und Ausgängen, Variablentabellen für TIA Portal (Excel
  *PLC Tags*), STEP 7 klassisch (SDF), Studio 5000, TwinCAT 3, Control Expert,
  GX Works und CODESYS; Hardware-Import aus AutomationML.
- Schaltschrankaufbau in 3D: Geräte in echter Größe auf Tragschienen, Kabelkanäle mit
  Füllgrad, virtuelle Verdrahtung durch die Kanäle, Bohrbilder / NC-Daten, Klimatisierung
  nach IEC/TR 60890 mit Wärmebild, Anreihschränke.
- Verdrahtungsassistent für die Werkstatt: Checkliste aller Leitungen mit Farbe,
  Querschnitt, Länge, Abisolierlänge und Aderendhülse, die aktuelle Leitung in 3D
  hervorgehoben. Auf dem Tablet `https://editor.openecraft.org/#wiring` öffnen.
- PDF mit klickbaren Querverweisen, DXF / DWG, Drahtlisten für Verdrahtungsautomaten,
  Import von EDZ-Dateien aus dem EPLAN Data Portal für Artikel, ein gemeinsamer
  Artikelserver.

**In fünf Minuten ausprobieren:** *Datei › Neu › Beispielprojekte* →
*3 · Stern-Dreieck-Anlauf* (Haupt- und Steuerstromkreis, drei verriegelte Schütze,
Zeitrelais, Motorkabel). Dann *Projektdaten › Berichte* → *Stückliste*, *Klemmenplan*,
*Kabelübersicht*; F4 zeigt die Meldungen der Projektprüfung. Für 3D und Verdrahtung:
*Datei › Neu › Demoprojekt öffnen*, Arbeitsbereich *Schaltschrankaufbau* →
*Layoutbereich › 3D-Ansicht*, danach Arbeitsbereich *Verdrahtung*. Die Oberfläche
steht unter *Ansicht › Sprache* auf Deutsch.

**Was es (noch) nicht kann:**

- Kein Import von EPLAN-Projekten (.elk / .zw1). Artikel kommen über EDZ, EPLAN-XML,
  BMEcat oder CSV.
- Junges Projekt: Fehler sind zu erwarten, besonders beim 3D-Routing in Räumen und
  Gebäuden und beim STEP-Import. Das Dateiformat ist versioniert und ältere Dateien
  werden beim Öffnen umgewandelt – trotzdem PDFs aufheben.
- Die mitgelieferten Herstellerkataloge sind aus öffentlichen Datenblättern
  zusammengestellt: Artikelnummern vor der Bestellung prüfen. Die Berechnungen
  (Leitungsdimensionierung, Kurzschluss, Selektivität, Spannungsfall) sind Hilfsmittel,
  kein zertifiziertes Werkzeug.

**Meine Fragen an euch:**

1. Was würde euch daran hindern, einen echten Schrank damit zu zeichnen?
2. Welche Auswertung oder welcher Export fehlt für euren Ablauf oder ist falsch?
3. Passt der Verdrahtungsassistent zu dem, wie bei euch tatsächlich verdrahtet wird?

Fehler gern als Issue auf GitHub (https://github.com/nilres/OpenECraft/issues) oder
über *Fehler melden* im Editor, der das GitHub-Formular vorausfüllt – es wird nichts
automatisch gesendet. Quellcode: https://github.com/nilres/OpenECraft

Danke fürs Ausprobieren!
[Name]

## mikrocontroller.net – Forum „Projekte & Code“

**Titel**

> OpenECraft – Open-Source-E-CAD für Schaltschränke in Rust (Desktop und Browser)

**Text**

Hallo,

ich stelle hier **OpenECraft** vor, ein freies E-CAD für Schaltpläne und
Schaltschrankaufbau, geschrieben in Rust. Eine Codebasis läuft nativ unter Windows,
Linux und macOS und als WebAssembly im Browser.

- Quellcode (MIT / Apache-2.0): https://github.com/nilres/OpenECraft
- Im Browser ausprobieren: https://editor.openecraft.org
- Webseite mit Live-Demos: https://openecraft.org

Das ist kein Platinen-CAD wie KiCad, sondern das Werkzeug für die Ebene darüber: Schütze,
Klemmen, SPS-Karten, Kabel, Tragschienen und Kabelkanäle. Der Schaltplan ist
betriebsmittelorientiert (Spule und Kontakte eines Schützes sind ein Betriebsmittel mit
Querverweisen), daraus entstehen Stückliste, Klemmenplan, Kabelpläne und
SPS-Variablentabellen, und der Schaltschrank wird in 3D aufgebaut und verdrahtet.

Technisch, für die, die es interessiert:

- 24 Crates, ein UI-freier Kern (~120.000 Zeilen) mit Projektmodell, Netzliste,
  Prüfungen, Berichten und den Exporten SVG / PDF / DXF / DWG; die Oberfläche ist
  egui / eframe.
- Bildschirm, SVG, PDF und DXF zeichnen über dieselbe Schnittstelle (`Draw`-Trait).
- Alle Funktionen am Projekt (~300 Befehle) stehen auch auf der Kommandozeile und als
  MCP-Server für KI-Agenten bereit (`openecraft-cli mcp`).
- STEP / IGES / Parasolid-Import für 3D-Modelle der Artikel über den CAD-Kern
  `monstertruck`.
- Grundrisse auf Papier (Scan oder PDF) lassen sich unter eine Gebäudeebene legen;
  Wände, Türen, Fenster und Räume werden darin gesucht (bisher nur rechtwinklige Wände).

Offen gesagt: Das Projekt ist einen Monat alt, und der Großteil des Codes ist mit
KI-Werkzeugen (Claude Code) entstanden [anpassen: deine Rolle]; etwa 2.800 Tests und
Konventionstests halten es zusammen. Es gibt keinen Import von EPLAN-Projekten, und
gerade 3D-Routing und STEP-Import sind noch jung.

Rückmeldungen, Kritik und Fehlerberichte gern hier oder als Issue auf GitHub.

[Name]
