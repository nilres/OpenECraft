# Artikelserver und Artikelverwaltung

Das gemeinsame Data Portal aller OpenECraft-Installationen und die Anwendung, die es pflegt.

`openecraft-parts-server` ist das Data Portal aller OpenECraft-Installationen (Desktop
und Web), im Sinne des EPLAN Data Portals: Herstellerkataloge mit einer Facettensuche,
Verlauf und Lebenszyklus der Artikel, Dateien der Artikel (Datenblätter, Bilder,
Modelle, Makros), Benutzerkonten mit Rollen und eine Webseite zum Durchsuchen. Die
Details eines Artikels zeigen seine 2D-Frontansicht und sein 3D-Modell (durch Ziehen
gedreht), wo er sie hat; die Artikel des Beispielkatalogs haben alle beides, wenn ihre
Abmessungen bekannt sind, erzeugt aus den Abmessungen (Geräte, Klemmen, SPS-Module,
Tragschienen, Kanäle, Sammelschienensysteme, Gehäuse und ihre Komponenten, Zubehör). Er
ist ein eigenes Programm und läuft unabhängig vom Editor.

```bash
cargo run --release -p openecraft-parts-server -- --bind 0.0.0.0:8090 --token <secret>
cargo run --release -p openecraft-parts-server -- --storage json --data parts-db.json
cargo run --release -p openecraft-parts-server -- --data parts-db.sqlite --migrate-from parts-db.json
cargo run --release -p openecraft-parts-server -- --install-catalogs all
cargo run --release -p openecraft-parts-server -- --config parts-server.toml
```

Als Container: `docker run -d -p 127.0.0.1:8090:8090 -e OPENECRAFT_TOKEN=<secret> -v parts:/data
ghcr.io/nilres/openecraft-parts-server` (siehe [Installation](installation.md)).

- Konfigurationsdatei (`--config <file>` oder `OPENECRAFT_PARTS_SERVER_CONFIG`): eine
  TOML-Datei mit den Einstellungen der Kommandozeilenoptionen unter ihren Namen (`bind`,
  `storage`, `data`, `token`, `admin_user`, `install_catalogs`, `access`,
  `registration`, `dashboard`; siehe
  [`parts-server/parts-server.example.toml`](../../parts-server/parts-server.example.toml)).
  Optionen auf der Kommandozeile oder in der Umgebung haben Vorrang; relative Pfade
  beziehen sich auf die Datei; unbekannte Schlüssel beenden den Server mit einem Fehler.
  `--migrate-from` ist nur eine Kommandozeilenoption.
- Speicher (`--storage sqlite|json`): standardmäßig SQLite (`--data parts-db.sqlite`,
  eine Datei mit indizierten Artikeln, Verlauf, Konten und Dateien); JSON hält alles in
  einer Datei, die bei jeder Änderung atomar geschrieben wird, mit den Dateien in
  `<name>.files/` daneben. Eine `.json`-Datendatei wählt den JSON-Speicher. Neue
  Datenbanken werden mit dem Beispielkatalog und den mitgelieferten Herstellerkatalogen
  gefüllt; `--migrate-from` kopiert eine vorhandene Datenbank in eine neue.
- Mitgelieferte Kataloge: `crates/openecraft-parts/catalogs/<id>.json` (ein `catalog`
  und seine `parts`, aufgelistet in `openecraft_parts::bundled`, die auch die
  Beispielprojekte verwenden) sind kuratierte Startsätze echter
  Herstellerartikel: `abb` (Leitungsschutzschalter, FI-Schutzschalter, Schütze,
  Motorschutzschalter), `beckhoff` (EtherCAT-Koppler und -Klemmen), `cosmo`
  (Edelstahl-Heizkreisverteiler für 2 - 14 Kreise und Einzelraumregelboxen RTL / TH
  für Fußbodenheizung), `dehn` (Überspannungsschutzgeräte Typ 1 - 3, für PV-Anlagen und für Datenleitungen), `gira` (System-55-Steckdosen,
  Schalter- und Tastereinsätze, Netzwerkdose Cat.6A mit Wippen, Abdeckungen
  und Rahmen), `hager` (Leitungsschutzschalter,
  FI-Schutzschalter, Installationsschütze, Lasttrennschalter, NH-Sicherungslasttrennschalter
  für 60-mm-Sammelschienen und Sicherungslasttrennleisten für 185 mm mit einrastbaren
  Stromwandlern, Leistungsschalter h3+ mit Motorantrieben und Auslösern, das
  NA-Schutzrelais EU400, univers-Schränke mit ihren Bausteinen,
  Energiezähler), `janitza` (Netzanalysator UMG 604-PRO), `ledvance` (LED-Deckenleuchten SURFACE
  CIRCULAR, Downlight DOWNLIGHT SLIM, PANEL 600, Lichtleiste DAMP PROOF, Rundleuchten und
  Notlicht-Rundleuchten), `jinko` (PV-Module Tiger Neo
  54HL4-B All Black 405 - 415 Wp), `longi` (PV-Module Hi-MO X6 Explorer LR5-54HTH
  425 - 435 Wp), `loxone` (Energiezähler für
  Stromwandler 1 / 5 A am Tree-Bus), `mdt` (KNX-Aktoren,
  IP-Schnittstelle / -Router, Spannungsversorgung, KNX-Taster, Glastaster und
  Raumtemperaturregler für Unterputzmontage), `mennekes` (CEE-Wandsteckdosen 16 A und 32 A,
  5-polig), `merten` (Herdanschlussdose für Unterputzmontage), `phoenix_contact` (Reihenklemmen,
  Relaismodule, Stromversorgungen), `pylontech` (19-Zoll-LiFePO4-Batteriemodule), `siemens` (SIRIUS-Schütze, Überlastrelais,
  Motorstarter, SENTRON-Leitungsschutzschalter), `signify` (Philips CoreLine Downlight
  DN140B, Panel RC132V, Feuchtraumleuchte WT120C, Wandleuchte WL120V), `spelsberg` (Abzweigdosen
  Abox 025-L bis 350-L, als Gehäuse des Raums platziert), `victron` (MultiPlus-II Wechselrichter /
  Ladegerät, Inverter RS Smart Solar, SmartSolar MPPT, Orion-Tr, Cerbo GX, SmartShunt, BMV-712, Lynx
  Distributor; überwiegend Artikel für die direkte Wandmontage), `wago` (Verbindungsklemmen 221 und
  2273 für Abzweigdosen) und `weidmueller` (Klemmen,
  Relaismodul, Stromversorgungen, CD-Verdrahtungskanäle). `--install-catalogs all` (oder IDs wie
  `siemens`) installiert oder aktualisiert sie in einer vorhandenen Datenbank: Neue
  Artikel werden hinzugefügt, Artikel, die niemand geändert hat, werden aktualisiert,
  von Benutzern geänderte oder gelöschte Artikel bleiben erhalten. Artikel ohne Modelle
  erhalten aus ihren Montagemaßen erzeugte. Fügen Sie nur Daten hinzu, die
  weitergegeben werden dürfen (Fakten aus öffentlichen Datenblättern, keine kopierten
  Texte, Bilder oder Modelle). Jeder mitgelieferte Katalog hat ein Logo,
  `catalogs/logos/<id>.svg`, das er bei der Installation erhält, solange er kein Logo
  hat: das Logo der Firma von Wikimedia Commons (gemeinfrei oder CC BY-SA 4.0, Quellen
  und Lizenzen in `catalogs/logos/README.md`; die Logos sind Marken ihrer Inhaber) oder
  ein neutrales Schild mit dem Namen, wo Commons kein frei lizenziertes hat (COSMO,
  Janitza, MDT, Pylontech).
- Logos: Jeder Katalog kann das Logo seiner Firma haben (PNG, JPEG, GIF, WebP oder SVG,
  höchstens 512 kB), gezeigt auf den Katalogkarten, in der Katalogliste der Verwaltung
  und an den Artikeln des Katalogs. Wer einen Katalog ändern darf, ändert sein Logo in
  *Verwaltung › Kataloge › Bearbeiten* (`PUT` / `DELETE
  /api/v1/catalogs/{id}/logo?name=`, `GET` liefert es mit einer abschottenden Content
  Security Policy aus, sodass ein SVG keine Skripte ausführt). Das Speichern eines
  Katalogs behält sein Logo; die Logodatei wird mit den Dateien der Artikel gespeichert
  und ist Teil der Sicherungen.
- Bibliotheken: JSON-Dokumente, die alle Projekte gemeinsam nutzen (`GET` / `PUT /api/v1/libraries/{name}`),
  bisher `forms`, die Schriftfelder, Auswertungsschemata und Plotrahmen der Firma, die
  der Editor (*Formulare › Schriftfelder › Firmenbibliothek*) und die Kommandozeile
  veröffentlichen und laden. Jeder, der Artikel lesen darf, liest sie; sie zu ändern
  braucht das Recht, Kataloge für den ganzen Server zu bearbeiten (Bearbeiter,
  Administratoren).
- Benutzer: Leser führen Favoriten, Hersteller ändern die Artikel und Kataloge ihres
  eigenen Herstellers, Bearbeiter alle Artikel, Administratoren verwalten außerdem die
  Benutzer. Das `--token` (oder `OPENECRAFT_TOKEN`) ist das Administratortoken;
  `--admin-user <name>` (oder `OPENECRAFT_ADMIN_USER`) legt ein Administratorkonto mit
  dem Passwort aus `OPENECRAFT_ADMIN_PASSWORD` an.
  Benutzer melden sich mit `POST /login` an (`name` und `password`; `user` wird für
  `name` angenommen, wie auf dem Projektserver) und senden das Sitzungstoken als
  `Authorization: Bearer <token>`. Ein Token, das der Server nicht kennt (eine
  abgelaufene Sitzung), wird mit `401` abgelehnt, auch beim Lesen, damit Clients es
  bemerken, statt anonym zu lesen. Passwörter
  lokaler Konten (auch das von `--admin-user`) haben mindestens 8 Zeichen; nach 5 fehlgeschlagenen Anmeldungen eines Benutzernamens wartet jeder weitere Versuch
  (1 s, verdoppelt bis zu 5 Minuten, Antwort `429`). Ohne Token, Konten und Verzeichnisdienste ist der Server
  schreibgeschützt.
- Registrierung: Wenn sie offen ist, legt jeder auf der Webseite ein lokales Leserkonto
  an (*Registrieren* neben der Anmeldung, `POST /register` mit `name` und `password`);
  Administratoren geben weitere Rechte. Standardmäßig wartet das neue Konto auf die
  Freigabe durch einen Administrator (`202`, keine Sitzung): Es kann sich nicht
  anmelden, bis ein Administrator es in *Verwaltung › Benutzer* freigibt
  (`POST /users/{name}/approval`) oder ablehnt (löscht); wartende Konten werden zuerst
  aufgelistet und in der Übersicht gezählt. Ohne Freigabe (abgeschaltet in
  *Verwaltung › Zugriffsrechte*, gespeichert als `"registration_approval": false`) ist
  der neue Benutzer sofort angemeldet. Das Freigeben bleibt die Entscheidung eines
  Administrators, auch wenn die Startoptionen die Registrierung festlegen. Konten, die
  warten, wenn die Freigabe abgeschaltet wird, warten weiter, bis sie freigegeben sind.
  Sie ist geschlossen, sofern sie nicht eingeschaltet wird: von Administratoren in
  *Verwaltung › Zugriffsrechte* (gespeichert als `"registration": true` in der Datei der
  Zugriffseinstellungen) oder festgelegt durch die Startoptionen
  `--registration true|false`, `OPENECRAFT_REGISTRATION` oder `registration` in der
  Konfigurationsdatei, die Vorrang vor dem Schalter haben. Registrierte Benutzer lesen
  die Artikel auch dort, wo das Lesen eine Anmeldung braucht. Ein Client darf 5 Konten
  registrieren, dann wartet jeder weitere Versuch (1 s, verdoppelt bis zu 5 Minuten);
  Namen, auf die sich Zuweisungen beziehen (auch mit Platzhaltern), und `admin` sind
  nicht verfügbar, damit sich niemand in die Rechte einer Zuweisung hineinregistriert.
  Ein registrierter Name steht auch vor einem Verzeichnisbenutzer desselben Namens, der
  sich noch nicht angemeldet hat: Weisen Sie auf Servern mit Verzeichnisdiensten nach
  Gruppen zu.
- Zugriffskontrolle (`<data>.access.json`, `--access`): Anmeldung mit Active-Directory-
  oder LDAP-Verzeichnisdiensten neben den lokalen Konten, Benutzergruppen, von
  Administratoren definierte Rollen und Zuweisungen von Rollen an Benutzer und Gruppen
  je Hersteller oder je Katalog, mit verweigernden Zuweisungen; das anonyme Lesen kann
  abgeschaltet werden. Siehe
  [docs/access-control.md](access-control.md).
- Dashboard: Das *Dashboard* der Webseite zeigt die Anzahl der Artikel, Kataloge,
  Hersteller und Dateien und Diagramme der Artikel je Hersteller, Produktgruppe,
  Katalog, Hauptgruppe und Lebenszyklus, die letzten Änderungen je Monat, wie viele
  Artikel eine Beschreibung, einen Preis, Abmessungen, Modelle, Dateien und Datenblätter
  haben, die Benutzer je Rolle (Administratoren) und die zuletzt geänderten Artikel;
  seine Balken öffnen die Artikel, die sie zählen (`GET /api/v1/dashboard`).
  Administratoren blenden es in *Verwaltung › Übersicht* aus (`"dashboard": false` in
  den Zugriffseinstellungen, `PUT /api/v1/dashboard/settings`); die Startoption
  `--dashboard true|false`, `OPENECRAFT_DASHBOARD` oder `dashboard` in der
  Konfigurationsdatei legen es fest.
- Exporte tragen die Version des Servers, damit eine spätere Version sie beim Import
  umwandeln kann: `GET /export.json` ist ein Objekt mit `export` (`format`,
  `format_version`, `server`, `server_version`, `revision`, `exported`) und `parts`
  (`/import` und die Editoren lesen es wie bisher), `GET /export.csv` beginnt mit einer
  Zeile `# format=openecraft-parts;format_version=1;server_version=…` (der CSV-Import
  überspringt solche Zeilen vor der Kopfzeile), Sicherungen halten die Felder als
  `export.<name>` in ihrer Tabelle `meta`, die auf der Webseite exportierten Kataloge
  und Zugriffseinstellungen haben ein Objekt `export` und die Benutzer-CSV dieselbe
  erste Zeile. Die Antworten tragen außerdem den Header `X-OpenECraft-Server-Version`.
- Artikel werden geprüft, bevor sie gespeichert werden: Artikelnummern haben höchstens
  128 Zeichen, die anderen Felder höchstens 256 (Beschreibung, Hinweis, technische
  Daten, Zertifikate und Datenblatt 20000), der Katalog, den ein Artikel nennt, muss
  existieren, und ein Import lehnt eine Datei ab, die eine Artikelnummer doppelt
  enthält.
- Jede Änderung ist eine neue Revision; jeder Artikel behält seine Revision, den
  Zeitpunkt der Änderung und die letzten 20 Fassungen (Verlauf). Artikel haben einen
  Lebenszyklus (aktiv, auslaufend, abgekündigt) und einen Nachfolger. Dateien werden
  einmal je Inhalt gespeichert; eine Datei wird gelöscht, sobald weder ein Artikel noch
  eine Fassung im Verlauf sie verwendet (und beim Start).
- Produktgruppen: Die EPLAN-Produktobergruppen, -Produktgruppen und -Untergruppen mit
  ihren Austauschnummern (`openecraft_parts::productgroups`) werden im Artikeleditor
  neben den eigenen Gruppen von OpenECraft angeboten.
- Suche (`POST /search`): Text (jedes Wort muss vorkommen), Hauptgruppe, Gruppe,
  Untergruppe, Hersteller, Katalog, Lebenszyklus und technische Daten (Spannung, Strom,
  Leistung und die Zeilen `Name: value` der technischen Daten), sortiert und
  seitenweise, mit Facetten, die die passenden Artikel je Wert zählen.
- Webseite: `http://<server>:8090/` – Dashboard, Suche mit Facetten, Artikeldetails mit
  Dateien und Verlauf, Kataloge, Anmeldung und Favoriten, auf Englisch, Deutsch,
  Französisch und Spanisch. Das Logo und der Titel oben links führen zurück zur
  Startseite (alle Artikel, keine Filter). Die Liste zeigt 10, 25, 50, 100 oder 200
  Artikel je Seite (*Pro Seite*, vom Browser behalten). Die Adresse hält die Suche und
  den geöffneten Artikel fest (Lesezeichen, die Zurück-Schaltfläche des Browsers);
  `?lang=de` wählt die Sprache.
- Neue Artikel auf der Webseite: Benutzer, die Artikel eines Herstellers oder Katalogs
  ändern dürfen, erhalten *Neuer Artikel* über der Liste und *Bearbeiten* / *Löschen* an
  den Artikeln, die sie ändern dürfen; das Formular enthält die allgemeinen,
  technischen, kaufmännischen und Montagedaten, bietet die Kataloge an, denen der
  Benutzer Artikel hinzufügen darf, und ersetzt nie einen vorhandenen Artikel
  (`PUT /parts/{number}` mit `If-None-Match: *` antwortet `409`).
- Verwaltung auf der Webseite (*Verwaltung*, Administratoren und Benutzern gezeigt, die
  Kataloge oder Artikel ändern dürfen; Anmeldung mit einem Benutzer oder, auf einem
  neuen Server, mit dem Administratortoken): eine Übersicht des Servers (Speicher,
  Revision, Benutzer je Rolle, Zugriffseinstellungen, die Rechte des Aufrufers);
  *Benutzer* – lokale Benutzer anlegen, ändern und löschen (Rolle, Hersteller, Gruppen,
  neues Passwort), ihre letzte Anmeldung und offenen Sitzungen sehen und sie überall
  abmelden; *Zugriffsrechte* – die Rolle von Anfragen ohne Anmeldung, definierte Rollen
  mit ihren Rechten und die Zuweisungen an Benutzer und Gruppen je Hersteller oder
  Katalog; *Gruppen* – Benutzergruppen mit ihren Mitgliedern; *Verzeichnisdienste* – die
  Active-Directory- und LDAP-Verbindungen mit einem Test der Verbindung und einer
  Anmeldung; *Kataloge* – Kataloge anlegen, ändern und löschen, die Berechtigungen von
  Benutzern und Gruppen an jedem Katalog, und die mitgelieferten Kataloge installieren
  oder aktualisieren; *Artikel importieren* (CSV,
  JSON, BMEcat, EPLAN-XML); *Export und Sicherung* – die Artikel als CSV oder JSON, die
  Kataloge, die Benutzer (CSV, ohne Passwörter) und die Zugriffseinstellungen (JSON, die
  sich wiederherstellen lassen) und eine Sicherung der ganzen Datenbank. Die Seite
  bietet nur an, was der Benutzer darf; der Server prüft jede Änderung.
- Sicherung (`GET /backup`, Administratoren): eine SQLite-Datei mit den Artikeln, ihrem
  Verlauf und ihren Dateien, den Katalogen, den Benutzern mit ihren Passwort-Hashes und
  den Favoriten, gleich welchen Speicher der Server hat. Stellen Sie sie wieder her,
  indem Sie einen Server mit `--data <backup>.sqlite` starten, oder kopieren Sie sie mit
  `--migrate-from` in eine neue Datenbank. Die Datei der Zugriffseinstellungen
  (Verzeichnisdienste, Rollen, Zuweisungen) ist nicht Teil davon: Bewahren Sie auch
  eine Kopie von `<data>.access.json` auf.
- REST-API (`/api/v1`, siehe `openecraft_parts::api`): `GET /info`, `GET /parts`,
  `POST /search`, `GET|PUT|DELETE /parts/{number}`, `GET /parts/{number}/accessories` (sein Zubehör
  und, für ein Zubehör, die Artikel, zu denen es passt), `GET /parts/{number}/history[/{revision}]`,
  `POST|DELETE /parts/{number}/attachments`, `GET /files/{id}` (PDF, Rasterbilder und
  Text werden im Browser gezeigt, andere Dateien wie SVG und XML werden
  heruntergeladen, sodass keine Datei ein Skript auf der Adresse des Servers ausführt), `POST /import` (CSV,
  JSON oder BMEcat), `GET /export.csv` (alle Artikel oder die der Abfrage in der Adresse:
  `q`, Gruppen, Hersteller, Katalog, Lebenszyklus, Sortierung), `GET /export.json` (dasselbe als
  JSON-Artikelliste, die `/import` liest), `GET /backup`, `GET|PUT|DELETE /catalogs/{id}`,
  `GET /bundled`, `POST /bundled/{id}` (einen mitgelieferten Katalog installieren), `POST /login`,
  `POST /register`, `GET|PUT /registration`, `POST /users/{name}/approval`,
  `POST /logout`, `GET|PUT /favorites`, `GET /users` (mit ihren offenen Sitzungen und
  der letzten Anmeldung), `PUT|DELETE /users/{name}`, `DELETE /users/{name}/sessions`
  (überall abmelden), `GET|PUT /access`, `GET|PUT|DELETE /catalogs/{id}/logo`, `GET|PUT /catalogs/{id}/grants`, `GET /groups`,
  `PUT|DELETE /groups/{name}`, `GET /directories`, `PUT|DELETE /directories/{id}`,
  `POST /directories/{id}/test`, `GET /dashboard`, `GET|PUT /dashboard/settings`.
- Im Editor: *Stammdaten › Data Portal* – den Server durchsuchen, die Details mit der
  Frontansicht und dem 3D-Modell ansehen, Artikel in die lokale Datenbank übernehmen, ihre Dateien speichern, ihre Makros der Makrobibliothek hinzufügen und die
  Datenbank auf neuere Fassungen, auslaufende und abgekündigte Artikel prüfen. Die
  Details zeigen den Katalog des Artikels mit seinem Logo (ausgeblendet mit
  *Einstellungen › Erscheinungsbild › Data Portal*). Logos werden auf einer weißen
  Platte gezeichnet; SVG-Logos werden mit `resvg` und der eigenen Schrift des Editors
  gerendert, sodass sie auf dem Desktop und im Browser gleich aussehen. *Betriebsmittel
  › Artikelverwaltung › 🌐 Artikelserver* lädt alle Artikel herunter oder lädt Artikel
  hoch. Die Projektprüfung warnt vor Betriebsmitteln mit abgekündigten Artikeln;
  `use_successor_parts` stellt sie um.
- Der Server spricht einfaches HTTP, sodass Token und Passwörter unverschlüsselt über
  das Netzwerk gehen. Halten Sie ihn auf `127.0.0.1` hinter einem TLS-Reverse-Proxy,
  wenn er von anderen Maschinen erreichbar ist, z. B. mit
  [Caddy](https://caddyserver.com/), das das Zertifikat holt:
  `caddy reverse-proxy --from parts.example.com --to 127.0.0.1:8090`.

## Öffentlicher Artikelserver

Das Projekt betreibt einen öffentlichen Artikelserver unter
[parts-server.openecraft.org](https://parts-server.openecraft.org) mit dem
Beispielkatalog und den mitgelieferten Herstellerkatalogen. Jeder liest ihn ohne Konto:
auf seiner Webseite (Suche, Kataloge, Dashboard, die Details eines Artikels mit seiner
Frontansicht und seinem 3D-Modell), im Editor (*Stammdaten › Data Portal*), in der
Artikelverwaltung und auf der Kommandozeile oder durch KI-Agenten. Er ist der
voreingestellte Server von allen (`openecraft_parts::api::PUBLIC_URL`): Der Editor und
die Artikelverwaltung starten mit seiner Adresse, die Kommandozeile verwendet ihn ohne
`--server` / `OPENECRAFT_SERVER`. Ein eigener Server wird stattdessen als Server-URL
eingegeben (`--server` oder `parts_connect` mit seiner `url`); die zuletzt eingegebene
Adresse wird behalten. Er ist ein Ort, um das Data Portal auszuprobieren und Artikel in
die eigene Datenbank zu übernehmen; eine Firma hält ihre eigenen Artikel auf einem
eigenen Server (oben), wo sie entscheidet, wer sie ändern darf. Die Daten der Kataloge
sind aus öffentlichen Datenblättern zusammengestellt: Prüfen Sie die Artikel vor der
Bestellung gegen die Daten des Herstellers.

## Impressum, Haftungsausschluss und Datenschutzhinweis

Unter der Webseite stehen die Links *Datenschutz* und, wenn der Betreiber einen Namen
hat, *Impressum* (auf Englisch *Privacy* und *Legal notice*), die jeweils einen Dialog
öffnen, sowie die Haftungsausschlüsse von OpenECraft (keine Gewährleistung, die
genannten Produktnamen gehören ihren Inhabern). Sie sind ohne Anmeldung vorhanden, wie
bei der [Website](../website.md#legal-notice), deren Felder und Variablen dieselben
sind:

| Schlüssel von `[legal]` | Variable | Zeigt |
| --- | --- | --- |
| `name` | `OPENECRAFT_LEGAL_NAME` | Die Person oder Firma, die den Server betreibt; leer blendet das Impressum aus |
| `address` | `OPENECRAFT_LEGAL_ADDRESS` | Ihre Postanschrift |
| `email` | `OPENECRAFT_LEGAL_EMAIL` | E-Mail-Adresse, als Link |
| `phone` | `OPENECRAFT_LEGAL_PHONE` | Telefonnummer |
| `representative` | `OPENECRAFT_LEGAL_REPRESENTATIVE` | Wer eine Firma vertritt |
| `register` | `OPENECRAFT_LEGAL_REGISTER` | Registergericht und -nummer |
| `vat` | `OPENECRAFT_LEGAL_VAT_ID` | Umsatzsteuer-Identifikationsnummer |
| `responsible` | `OPENECRAFT_LEGAL_RESPONSIBLE` | Wer für den Inhalt verantwortlich ist, mit Anschrift |
| `text` | `OPENECRAFT_LEGAL_TEXT` | Weiterer Text: Haftungsausschlüsse, Streitbeilegung |
| `privacy` | `OPENECRAFT_LEGAL_PRIVACY` | Was der Betreiber dem Datenschutzhinweis hinzufügt: das Hosting, die Protokolle, an wen man sich wendet |

Setzen Sie sie in der Tabelle `[legal]` der Konfigurationsdatei oder mit den Variablen,
die Vorrang haben; Zeilen der Anschrift und der Texte werden durch `|` getrennt (oder
durch Zeilenumbrüche in der Datei). `GET /api/v1/legal` liefert sie als JSON. Die
Beschriftungen folgen der Sprache der Seite; die Werte werden gezeigt, wie sie gesetzt
sind.

Der Datenschutzhinweis sagt in der Sprache der Seite, was die Seite und der Server tun:
keine Cookies, kein Tracking, nichts von anderen Servern geladen; die Sprache, die
Einstellungen der Seite und der Schlüssel der Sitzung im Speicher des Browsers; die
Konten (Name, Rolle, Gruppen, das Passwort als Hash); die für kurze Zeit gemerkten
fehlgeschlagenen Anmeldungen; wer einen Artikel wann geändert hat (sein Verlauf) und die Favoriten eines Benutzers. Was nur der Betreiber weiß (wer den
Server hostet, was ein Reverse-Proxy protokolliert und wie lange, an wen man sich
wendet, wie es Artikel 13 DSGVO verlangt), wird in `privacy` gesetzt und darunter
gezeigt. Halten Sie die Texte wahr: Eine Änderung, die etwas anderes speichert oder
sendet, ändert auch sie (`renderLegal` von `common.js`, `privacy()` des Skripts der
Seite). Ob ein Server ein Impressum braucht und was darin stehen muss, hängt vom
Betreiber und vom Land ab: OpenECraft zeigt nur, was gesetzt ist.

## EPLAN Data Portal

Die Artikelverwaltung verbindet sich auch mit dem EPLAN Data Portal: Wählen Sie *EPLAN
Data Portal* als Servertyp im Fenster des Data Portals oder in *Artikelverwaltung › 🌐
Artikelserver*, behalten Sie die Adresse `https://dataportal.eplan.com/api` und geben
Sie ein persönliches Zugriffstoken ein (für das Data Portal im Profil Ihres
EPLAN-Kontos angelegt, ePulse). Das Portal arbeitet dann wie ein Artikelserver: Suche
nach Text, Hersteller und Produktgruppe, Artikeldetails mit den kaufmännischen Daten
(CSV), DXF-Zeichnungen und Bildern, Übernahme von Artikeln in die lokale Datenbank und
die Aktualisierungsprüfung der Datenbank.

Das PAT wird als `Authorization: Bearer PAT:<token>` gesendet (eingefügte Präfixe `PAT:`
und `Bearer ` werden ebenfalls angenommen). Der Verbindungstest liest den öffentlichen
Katalog und lädt dann die kaufmännischen Daten seines ersten gelieferten Artikels
herunter, um den authentifizierten Download-Zugriff zu prüfen. Eine erfolgreiche
öffentliche Suche allein beweist keine Authentifizierung;
`/authorization` wird nicht verwendet, weil es PATs ablehnt, die das Download-Gateway annimmt. Downloads von DXF und
kaufmännischen Daten senden die Artikel-ID per POST an `/download/{type}/part`, wie der
Webclient des Portals; vorhandene Anhang-IDs in gespeicherten Projekten werden weiter unterstützt. Die Nutzung
des Portals und der daraus übernommenen Artikeldaten unterliegt den Nutzungsbedingungen
von EPLAN und der Lizenz des Kontos: Prüfen Sie sie, bevor Sie Portalartikel auf einen
gemeinsamen Artikelserver stellen. OpenECraft steht in keiner Verbindung zu EPLAN.

- `openecraft_client::eplan` kapselt die REST-API des Portals (JSON:API): Es baut die
  Anfragen des Portals aus den Abfragen des Artikelservers und wandelt die Antworten in
  `ServerInfo`, `SearchResult` und `Part` um. `openecraft_client::Connection` bietet die
  Lesefunktionen beider Serverarten; der Editor und die Kommandozeile verwenden es.
- Die Verbindung ist schreibgeschützt: keine Uploads, Kataloge, Benutzer, Anmeldung,
  Favoriten, kein Verlauf und kein Download der ganzen Datenbank. Die Facetten zählen
  die Artikel der gezeigten Seite, da das Portal sie nicht zählt; nach Katalogen,
  Lebenszyklus und technischen Daten kann nicht gefiltert werden.
- Das Portal findet Artikel über seine eigenen IDs, nicht über die Nummer: Ein daraus
  geladener Artikel behält die ID (`portal_id`), sodass die Übernahme markierter Artikel
  und die Aktualisierungsprüfung sie in einer Anfrage laden. Die Aktualisierungsprüfung
  vergleicht nur die Artikel, die aus dem Portal kamen; eine einzelne Artikelnummer wird
  durch eine exakte Suche gefunden.
- Hersteller werden nach ihrem Kurznamen gefiltert, Produktgruppen nach der ID des
  EPLAN-Katalogeintrags: (Produktobergruppe + 1) · 10⁸ + (Produktgruppe + 1) · 10⁴ +
  (Untergruppe + 1), z. B. `200050000` für Elektrotechnik / Stecker.
- Artikel im EPLAN-Format (Makros, Anschlusspunkte) kommen als EDZ-Dateien von der
  Webseite des Portals; importieren Sie sie wie andere Artikeldateien (siehe [Artikelverwaltung](features/parts.md)).
- Nur die Desktop-Version und die Kommandozeile erreichen das Portal: Es erlaubt keine
  Cross-Origin-Anfragen von Browsern, sodass die Web-Version einen Proxy bräuchte.

### Gemeinsames Data-Portal-PAT unter Windows

Der Editor und CLI/MCP können dasselbe PAT über die Windows-Anmeldeinformationsverwaltung
verwenden. Geben Sie das PAT im Data Portal des Editors ein, wählen Sie **Token auf
diesem Computer speichern** und verwenden Sie **Verbindung testen**. Das Speichern ist
standardmäßig aus: Ohne diese Zustimmung bleibt das PAT in der Sitzung. Nur eine
erfolgreiche Prüfung des geschützten Dienstes speichert einen Ersatz.
Ein Speicherfehler wird gesondert gezeigt und macht die Verbindung nicht ungültig. Das PAT wird für
den aktuellen Windows-Benutzer auf diesem Computer gespeichert, nicht in einem Projekt,
in JSON-Einstellungen, einer Umgebungsdatei oder einer MCP-Antwort.

Lassen Sie das PAT-Feld leer, um den gespeicherten Wert automatisch zu verwenden. Bei
der ersten Anfrage lädt die Oberfläche ihn im Hintergrund, bevor sie diese Anfrage
aufbaut; gleichzeitige Anfragen teilen sich das Nachschlagen. Der UI-Thread wartet nie
auf das Lesen der Anmeldedaten. Das Ergebnis, auch ein fehlendes Token oder ein
Lesefehler, wird für die Sitzung zwischengespeichert. **Verbindung testen** erzwingt ein
frisches Nachschlagen nach einem Ersetzen oder Entfernen von außen oder einem
Lesefehler. **Gespeichertes Token entfernen** leert auch den Zwischenspeicher. Ein
anderer Endpunkt oder ein ausdrückliches Token verwendet einen eigenen
Zwischenspeicher, sodass ein älteres Nachschlagen ihn nicht ersetzen kann.

MCP verwendet den gespeicherten Wert, wenn
`parts_connect` `Eplan` ohne ausdrückliches Token wählt. Jede Anfrage löst die
Anmeldedaten erneut auf, sodass Ersetzungen und Entfernungen ohne Neustart von MCP
wirken. Ein für die aktuelle EPLAN-Verbindung angegebenes ausdrückliches Token hat
Vorrang; MCP speichert ausdrückliche Sitzungstoken nicht im gemeinsamen Speicher.

Anmeldedaten sind an HTTPS-Ursprung, Port und API-Pfad gebunden. Ein Wechsel des Servers
verwendet nicht das gespeicherte PAT eines anderen Endpunkts. **Gespeichertes Token
entfernen** entfernt nur den Eintrag des aktuellen Portals und leert sein Feld in der
Oberfläche. Andere Plattformen behalten Anmeldedaten nur für die Sitzung; eine
Speicherung im Klartext wird nicht als Rückfall verwendet.

Beim Umstieg von einer Version, die PATs nur im Speicher hielt, geben Sie das PAT einmal
im aktualisierten Editor ein, wählen **Token auf diesem Computer speichern** und testen
die Verbindung. Es muss nicht in MCP eingegeben werden. Verbindungstests fragen den
geschützten Token-Dienst des Dateianbieters und öffentliche Katalog-Metadaten ab; sie
laden keine Datei mit kaufmännischen Daten herunter und hängen nicht davon ab, dass ein
bestimmter Katalogartikel eine hat.

## Artikelverwaltung

`openecraft-parts-manager` ist eine eigenständige Anwendung, die die Artikel eines
Artikelservers pflegt: neue Artikel anlegen, vorhandene bearbeiten (einschließlich 2D- /
3D-Modellen), Artikel umbenennen und löschen, CSV- / JSON-Artikeldateien hochladen und
alle Artikel exportieren. Sie verwendet denselben Artikeleditor wie die
Artikelverwaltung des Editors und läuft auf dem Desktop und im Browser.

```bash
cargo run --release -p openecraft-parts-manager        # desktop
cd parts-manager && trunk serve                        # web, http://127.0.0.1:8082
cd parts-manager && trunk build --release              # static files in parts-manager/dist/
```

*Rechtliches und Datenschutz* der oberen Leiste zeigt den Haftungsausschluss und den
Datenschutzhinweis der Artikelverwaltung und, in einem Web-Build, dessen `config.js`
einen Betreiber nennt, dessen Impressum
([installation.md](installation.md#impressum-eines-web-editors)).

Geben Sie die Server-URL und das Zugriffstoken ein (nur im Speicher gehalten) und
klicken Sie auf *Verbinden*, oder melden Sie sich mit Benutzername und Passwort an.
Änderungen werden mit *Speichern* (`Strg+S`) auf dem Server gespeichert; der Editor
fragt, bevor nicht gespeicherte Änderungen verworfen werden und bevor ein Artikel
gelöscht wird. *Dateien und Verlauf* über dem Artikeleditor lädt die Dateien eines
Artikels hoch, entfernt sie und zeigt seine Änderungen; *Kataloge* pflegt die
Herstellerkataloge (mit ihren Logos: *Logo wählen…*, *Logo entfernen*, für einen Katalog
ohne nicht gespeicherte Änderungen) und *Benutzer* (Administratoren) die Konten und
Rollen. *Logos* neben dem Sprachmenü zeigt die Logos in der Katalogliste an oder blendet
sie aus (wird gemerkt).
