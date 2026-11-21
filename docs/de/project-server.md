# Projektserver

Gemeinsame Projekte für ein Team: Mehrere Benutzer arbeiten am selben Projekt, jede Seite wird von einem von ihnen gesperrt.

`openecraft-project-server` speichert Projekte für ein Team. Mehrere Benutzer öffnen
dasselbe Projekt gleichzeitig und arbeiten an verschiedenen Seiten; eine Seite wird
jeweils von einem Benutzer bearbeitet.

```bash
cargo run --release -p openecraft-project-server -- --bind 0.0.0.0:8091 --data projects --token <secret>
cargo run --release -p openecraft-project-server -- --config project-server.toml
```

Als Container: `docker run -d -p 127.0.0.1:8091:8091 -e OPENECRAFT_PROJECT_TOKEN=<secret>
-v projects:/data ghcr.io/nilres/openecraft-project-server` (siehe [Installation](installation.md)).

- Konfigurationsdatei (`--config <file>` oder `OPENECRAFT_PROJECT_SERVER_CONFIG`): die
  Einstellungen der Kommandozeilenoptionen in TOML (`bind`, `data`, `token`,
  `admin_user`, `access`, `web`; siehe
  [`project-server/project-server.example.toml`](../../project-server/project-server.example.toml)),
  wie beim Artikelserver.
- **Webseite** (optional: `--web`, `--web true`, `OPENECRAFT_PROJECT_WEB=true` oder
  `web = true` in der Konfigurationsdatei; standardmäßig aus): `http://<server>:8091/`
  listet die Projekte, die der Benutzer sehen darf (mit den Seiten, der Revision, wer
  sie angelegt und zuletzt geändert hat, wer daran arbeitet und den Rechten des
  Benutzers), und zeigt jedes davon schreibgeschützt: die Seiten nach Abschnitt oder
  nach Struktur gruppiert, die Betriebsmittel mit den Stellen, an denen sie dargestellt
  sind, und die Seiten gezeichnet wie auf Papier (Zoom, Ziehen zum Verschieben,
  Pfeiltasten oder Bild auf / Bild ab für die nächste Seite). Von anderen Benutzern
  gesperrte Seiten sind mit 🔒 und dem Benutzer markiert; die Seite sieht alle 20 s nach
  dem Projekt und folgt neuen Revisionen. Benutzer melden sich mit ihrem Konto oder dem
  Servertoken an; ohne Anmeldung zeigt die Seite, was anonyme Anfragen sehen dürfen (die
  Zugriffseinstellungen). Projektleiter bearbeiten die Zuweisungen eines Projekts
  (*Berechtigungen*). Administratoren erhalten einen Bereich *Verwaltung*: eine
  Übersicht (Projekte in Verwendung, offene Sitzungen, Benutzer je Rolle,
  Zugriffseinstellungen), die Sitzungen (die offenen mit ihrem Benutzer, Projekt,
  Beginn, letzter Aktivität, Speicherungen und gesperrten Seiten, und den Verlauf der
  beendeten mit ihrer Dauer und wie sie endeten: geschlossen, abgelaufen, abgemeldet
  oder Benutzer gelöscht), die Benutzer (Rolle, Projekte, Gruppen, Passwort), die
  Zugriffsrechte (Zugriff ohne Anmeldung, definierte Rollen, Zuweisungen) und die
  Verzeichnisdienste (Active Directory, LDAP, mit einem Test der Verbindung und einer
  Anmeldung). *Prüfprotokoll* eines Projekts zeigt dessen Protokoll für die Prüfung vor
  Ort, gemacht für ein Tablet oder ein Telefon: Prüfer und Bearbeiter geben dort die
  Daten des Protokolls, die Besichtigung und die Messergebnisse jedes Stromkreises ein,
  ohne das Projekt zu öffnen und ohne Sperre, unterschreiben das Protokoll und laden es
  als Excel-Arbeitsmappe herunter
  ([Prüfprotokoll](features/calculations.md#prüfprotokoll)).
  Die Seite gibt es in allen Sprachen der Anwendung; sie verwendet die unten
  beschriebene API, sodass der Server jede Anfrage wie für den Editor prüft. Ohne
  `--web` beantwortet der Server `/` mit einem kurzen Text und hat keine Seitenrouten.
- **Erstellt von, zuletzt geändert von, zuletzt geändert**: Der Server hält für jedes
  Projekt fest, wer es angelegt und wer die letzte Änderung daran gespeichert hat, und
  wann; die Projektliste des Editors (Bereich *Projektserver*), die Webseite und
  `GET /api/v1/projects` (`created_by`, `created_at`, `changed_by`, `changed_at`;
  Sekunden seit 1970, UTC) zeigen sie jedem, der das Projekt ansehen darf. Der Name ist
  der angemeldete Benutzer; auf einem Server ohne Konten (oder mit dem Servertoken) ist
  es der Benutzername, mit dem die Sitzung geöffnet wurde. Er ist unabhängig von
  *Erstellt von* in den Projekteigenschaften, das die Benutzer bearbeiten. Nur eine
  Speicherung, die das Projekt ändert, zählt: Das Öffnen oder Lesen eines Projekts, die
  Statusanfragen, die eine Sitzung am Leben halten, Sperren und das Speichern
  unveränderten Inhalts lassen es, wie es ist. Projekte früherer Serverversionen zeigen
  *Unbekannt* bis zu ihrer nächsten Änderung (das Anlegen bleibt unbekannt), und ein
  als Kopie eines anderen hochgeladenes Projekt ist ein neues Projekt des Benutzers,
  der es hochgeladen hat.
- Projektnamen, die sich nur in Groß- und Kleinschreibung unterscheiden, sind ein Name
  (Windows und macOS halten ihre Dateien nicht auseinander): Anfragen finden ein Projekt
  unter jeder Schreibweise seines Namens; Windows-Gerätenamen (`CON`, `NUL`, `COM1` …) und Namen, die
  mit einem Punkt enden, werden abgelehnt.
- Jedes Projekt wird als geteiltes Projekt in einem eigenen Ordner gespeichert
  (`<name>/<name>.oecp` mit einer kleinen Datei je Seite, Betriebsmittel und Artikel; es
  lässt sich im Desktop-Editor öffnen und unter Versionsverwaltung halten) neben
  `<name>.state.json` (Revision, ID-Zähler, wer das Projekt angelegt und zuletzt
  geändert hat) im Datenverzeichnis. Eine Speicherung schreibt nur die Dateien, die sich
  geändert haben. Projektdateien früherer Versionen (`<name>.oec`) werden beim ersten
  Start umgewandelt und als `<name>.oec.bak` aufbewahrt.
- Mit `--token` (oder `OPENECRAFT_PROJECT_TOKEN`) braucht jede Anfrage
  `Authorization: Bearer <token>`; ohne Token, Benutzerkonten und Verzeichnisdienste
  kann jeder, der den Server erreicht, dessen Projekte ändern, aber nur Clients auf der
  eigenen Maschine des Servers dürfen Benutzer anlegen und die Zugriffseinstellungen
  ändern (damit niemand sonst den Server übernimmt, indem er den ersten Administrator
  anlegt).
- Der Server spricht einfaches HTTP, sodass Token und Passwörter unverschlüsselt über
  das Netzwerk gehen. Halten Sie ihn auf `127.0.0.1` hinter einem TLS-Reverse-Proxy,
  wenn er von anderen Maschinen erreichbar ist, z. B. mit
  [Caddy](https://caddyserver.com/), das das Zertifikat holt:
  `caddy reverse-proxy --from projects.example.com --to 127.0.0.1:8091`. Der Editor
  verwendet dann `https://projects.example.com`.
- **Benutzerkonten** (`users.json` im Datenverzeichnis, Argon2-Passwort-Hashes):
  Betrachter öffnen Projekte schreibgeschützt, Prüfer speichern außerdem Kommentare,
  Markierungen und Freigaben, Bearbeiter sperren, ändern, speichern und legen Projekte
  an, Projektleiter löschen außerdem Projekte und verwalten, wer sie verwenden darf,
  Administratoren verwalten außerdem die Benutzer; die Rolle eines Benutzers kann auf
  einige Projekte beschränkt werden (Namen mit `*` / `?`). `--admin-user <name>`
  (oder `OPENECRAFT_PROJECT_ADMIN_USER`) legt den ersten Administrator mit dem Passwort
  aus `OPENECRAFT_PROJECT_ADMIN_PASSWORD` an (mindestens 8 Zeichen, wie jedes Passwort).
  Benutzer melden sich über *Werkzeuge › Projektserver…* an (Benutzername, Passwort,
  *Anmelden*); der Server verwendet den Namen des Kontos an den Sperren, was auch immer
  die Anfrage sagt, und für die Kommentare, Antworten, Markierungen und Freigaben, die
  ein angemeldeter Benutzer hinzufügt (Freigaben anderer Benutzer bleiben, wie sie
  sind). Anonyme Aufrufer können den Namen eines Kontos nicht verwenden. Leiter eines
  Projekts dürfen Verweigerungen dafür hinzufügen, aber nur Administratoren heben sie
  auf. Benutzernamen haben höchstens 64 Zeichen und keine Steuerzeichen; ein neuer
  Benutzer braucht eine Rolle, Verzeichnisgruppen oder eine Zuweisung in den
  Zugriffseinstellungen. Das Abmelden schließt die mit dieser Anmeldung geöffneten
  Projekte und gibt ihre Sperren frei; das Löschen eines Benutzers schließt alle
  seine. Eine fehlgeschlagene Anmeldung antwortet `wrong user name or password`,
  welches von beiden auch falsch war.
- **Zugriffskontrolle** (`access.json` im Datenverzeichnis, `--access`): Benutzer melden
  sich mit einem lokalen Konto oder mit Active-Directory- / LDAP-Verzeichnisdiensten
  an; Rollen (eingebaut oder von Administratoren definiert) werden Benutzern und
  Verzeichnisgruppen je Projekt zugewiesen, verweigernde Zuweisungen nehmen Rechte weg.
  Wer ein Projekt anlegt, verwaltet es; Projektleiter bearbeiten seine Zuweisungen im
  Abschnitt *Projektzugriff* des Projektserver-Fensters. Siehe
  [docs/access-control.md](access-control.md).
- Sperren: Jede Seite und die Projektdaten (Seitenliste und -reihenfolge,
  Betriebsmittel, Artikel, Schaltschrankaufbau, Einstellungen) werden getrennt gesperrt.
  Wer etwas bearbeitet, das er nicht gesperrt hat, fordert die Sperre automatisch an;
  hält ein anderer Benutzer sie, wird die Änderung zurückgenommen. Von anderen gesperrte
  Seiten sind schreibgeschützt und in der Seitenliste mit 🔒 und dem Namen des Benutzers
  markiert. Sperren enden, wenn sie freigegeben werden, wenn das Projekt geschlossen
  wird oder 90 s nachdem der Editor aufgehört hat, seinen Status zu senden (alle 15 s).
- Das Speichern führt nur das, was Sie gesperrt haben, in das Projekt des Servers
  zusammen; alles andere kommt vom Server. Änderungen anderer Benutzer werden
  automatisch geladen.
  Eine Speicherung nennt die Revision, von der aus bearbeitet wurde (`base_revision`;
  der Editor tut dies): Der Server übernimmt dann auch geänderte Seiten und
  Projektdaten, die niemand gesperrt hat (er sperrt sie zuerst für Sie), und antwortet
  mit den Änderungen, die er ausgelassen hat (`ignored`): von einem anderen Benutzer
  gesperrt, von einem anderen Benutzer seit dieser Revision geändert oder nicht erlaubt.
  Mit `strict` wird in diesem Fall nichts gespeichert (`409`). Der Server hält die
  letzten 16 Revisionen je Projekt im Speicher; eine Speicherung von einer älteren
  Revision aus (oder ohne `base_revision`) speichert nur, was gesperrt ist.
- Jede Sitzung erhält ihren eigenen Bereich von Objekt-IDs, sodass Benutzer nie dieselbe
  ID erzeugen.
- **Sitzungsverlauf**: Jede beendete Sitzung (Benutzer, Projekt, Beginn, letzte
  Aktivität, Ende, Speicherungen, wie sie endete) wird an `sessions.jsonl` im
  Datenverzeichnis angehängt, je eine JSON-Zeile; der Server behält die letzten 5000 und
  schreibt die Datei neu, wenn sie auf doppelt so viele Zeilen angewachsen ist. Die
  Schlüssel der Sitzungen werden nie gespeichert oder gezeigt, nur ihre Nummern.
- Editoren bevorzugen das sitzungsgebundene `POST /projects/{name}/snapshot`. Nur bei
  404 oder 405 verwenden sie den älteren GET-Snapshot, gefolgt vom Status der aktuellen
  Sitzung. Beide müssen dieselbe Revision nennen, bevor der Snapshot angewendet wird;
  der Status liefert die Identitäten der eigenen Sperren. Authentifizierungsfehler
  lösen diesen Rückfall nicht aus. Ein während des Ladens geändertes Projekt wird erneut
  geladen.
- Im Editor: *Werkzeuge › Projektserver…* – die Server-URL, Ihren Benutzernamen und das
  Token eingeben, die Projekte auflisten und eines öffnen oder das aktuelle Projekt
  hochladen. Während ein gemeinsames Projekt geöffnet ist, speichert *Speichern*
  (`Strg+S`) auf dem Server; das Fenster und das Kontextmenü der Seitenliste sperren
  Seiten und geben sie frei. Die Statusleiste zeigt *Projektserver: … / Projekt: …*
  (*Verbindung verloren*, solange der Server seinen Status nicht beantwortet); ein aus
  der letzten Sitzung wiederhergestelltes oder auf dem Server geschlossenes Projekt
  erscheint als *Lokale Wiederherstellungskopie des Serverprojekts: …* und wird in
  einer lokalen Datei gespeichert.
- **Automatisches Speichern:** Während ein gemeinsames Projekt geöffnet ist, werden
  seine abgeschlossenen Änderungen von selbst auf dem Server gespeichert, 30 Sekunden
  nach einer Änderung und dann alle 30 Sekunden, solange die Arbeit weitergeht
  (*Datei › Einstellungen › Server* oder der Bereich *Projektserver*: ein / aus und das
  Intervall, 10 bis 600 s). Ein laufendes Ziehen oder eine laufende Texteingabe wird
  nicht gesendet; sie wird gespeichert, sobald sie abgeschlossen ist. Es läuft immer
  nur eine Speicherung, eine automatische Speicherung gibt keine Sperre frei und ohne
  Änderungen wird nichts gesendet. *Speichern* (`Strg+S`) speichert weiterhin sofort.
  Die Statusleiste und der Bereich unterscheiden die Zustände: *Gespeichert*, *Noch
  nicht gespeichert*, *Wird gespeichert…*, *Warten auf den Server* (keine Antwort,
  Sitzung beendet, abgemeldet: Die Änderungen werden gespeichert, sobald die Sitzung
  weitergeht), *Speichern fehlgeschlagen* (mit dem nächsten Intervall erneut versucht)
  und *Teilweise gespeichert* (der Server hat ausgelassen, was ein anderer Benutzer
  gesperrt oder geändert hat; Ihre Fassung bleibt im Bereich erhalten und wird nicht
  von selbst erneut gesendet), mit der Zeit der letzten vom Server bestätigten
  Speicherung. Ein Projekt gilt erst nach der Antwort des Servers als gespeichert. Die
  lokale Wiederherstellungskopie ist keine Speicherung auf dem Server: Während eines
  Ausfalls wird sie sofort geschrieben, und der Bereich sagt dies erst, nachdem sie
  geschrieben wurde.
- Die Projektliste wählt ein Projekt aus (ein Doppelklick öffnet es). *Eigenschaften…*
  lädt seine Eigenschaften vom Server (nie aus der Liste) und zeigt, wer es wann auf dem
  Server angelegt hat (*Unbekannt* für Projekte, die angelegt wurden, bevor der Server
  dies festhielt; dies ist nicht das *Erstellt von* der Zeichnung, das die Benutzer
  bearbeiten). Wer die Projektdaten bearbeiten darf, ändert sie dort, ohne das Projekt
  zu öffnen, sofern keine Sitzung die Projektdaten gesperrt hat; der Server ersetzt dann
  nur die Projektinformationen und lehnt ab, wenn sie seit dem Laden geändert wurden
  (*Neu laden* zeigt die neueren). Für das geöffnete gemeinsame Projekt ist
  *Eigenschaften…* der übliche Dialog, der die Projektdaten sperrt.
- *Projekt kopieren…* (für das ausgewählte oder das geöffnete gemeinsame Projekt) zeigt
  sofort die Eigenschaften der Kopie: einen freien Namen auf dem Server (*… (Kopie)*)
  und Titel, Beschreibung, Firma, Ersteller, Datum, Revision und Projekteigenschaften
  des Originals, alles änderbar. Bis *Anlegen und öffnen* existiert nichts auf dem
  Server; *Abbrechen* hinterlässt nichts, und eine abgelehnte Kopie (Name vergeben, kein
  Recht, sie anzulegen) behält die Eingaben zum Korrigieren. Die Kopie wird auf dem
  Server aus dem gespeicherten Stand erstellt: Änderungen, die noch niemand gespeichert
  hat, auch die anderer Benutzer, sind nicht Teil davon. Bei nicht gespeicherten
  Änderungen im geöffneten Projekt fragt der Editor zuerst, ob sie gespeichert werden
  sollen. Wurde das Original auf dem Server geändert, nachdem der Dialog es geladen hat,
  wird die Kopie erst nach erneuter Bestätigung erstellt. Sitzungen, Seitensperren und
  Zugriffsrechte werden nicht kopiert: Die Kopie wird in einer neuen Sitzung geöffnet
  und ein angemeldeter Benutzer verwaltet sie wie jedes Projekt, das er anlegt.
  Kommentare, Markierungen, Freigaben (mit ihren Unterschriften), Änderungsanträge,
  veröffentlichte Revisionen und Baselines werden ausgelassen, sofern nicht *Auch
  Revisionen, Baselines und Prüfvermerke kopieren* angehakt ist; die Freigebenden des
  Freigabeablaufs bleiben wie alle Einstellungen. Der Server hält fest, wer die Kopie
  wann und von welchem Projekt und welcher Revision angelegt hat.
- *Datei › Einstellungen › Beim Start* legt fest, was der Editor beim Start öffnet: die
  Startseite mit lokalen Dateien und Serverprojekten, das letzte Projekt (ein
  Serverprojekt wird wieder auf dem Server geöffnet), die Projektliste des Servers oder
  eine gewählte Datei oder ein gewähltes Serverprojekt. Kann der Server das Projekt
  nicht öffnen, bietet ein Dialog an, es erneut zu versuchen, sich anzumelden, ein
  anderes Projekt zu wählen oder mit der lokalen Kopie weiterzuarbeiten; nicht
  gespeicherte Änderungen der Kopie werden zuerst gespeichert oder verworfen, nie
  stillschweigend überschrieben.
- REST-API (`/api/v1`): `GET /info` (Name, Version und die Anzahl der Projekte, die der
  Aufrufer sehen darf; wird auch ohne Token beantwortet), `GET|POST /projects` (`POST`
  braucht das `project`), `GET|PUT|DELETE /projects/{name}`,
  `POST /projects/{name}/open|status|lock|release|close`, `GET|PUT /projects/{name}/access`,
  `GET|PUT /projects/{name}/properties` (die Projektinformationen mit der Revision, wer
  die Projektdaten gesperrt hat, den Rechten des Aufrufers, wer das Projekt angelegt und zuletzt geändert hat und wovon es kopiert wurde; `PUT`
  nimmt `user`, `revision` und `info`, zählt als letzte Änderung, braucht das Recht, die Projektdaten zu bearbeiten, antwortet `409`,
  solange die Projektdaten gesperrt sind, und `412`, wenn sich die Informationen seit `revision` geändert haben),
  `POST /projects/{name}/copy` (`name` und `user` der Kopie, optional die
  `revision`, die die Quelle noch haben muss – sonst `412` –, ihre `info` und `records`;
  braucht das Recht, die Quelle anzusehen und den neuen Namen anzulegen, antwortet wie
  das Öffnen eines Projekts),
  `GET|PUT /projects/{name}/inspection?lang=de` (das Prüfprotokoll wie gespeichert: die
  Daten des Protokolls, die Besichtigung, die Stromkreise mit den Daten des Schaltplans,
  ihren Grenzwerten, Ergebnissen und Bewertungen, und die Bewertung des Protokolls,
  `InspectionSheet`; `PUT` nimmt `user`, `circuit` oder `source` (beide leer: die Daten
  des Protokolls und die Besichtigung als `check.<item>`), die `rev` seines Datensatzes wie geladen und die
  `values` nach Feld, braucht das Recht, Prüfvermerke zu speichern oder zu bearbeiten,
  antwortet `409`, wenn sich der Datensatz seit `rev` geändert hat, und zählt als
  Änderung des Projekts),
  `GET /projects/{name}/inspection.xlsx?lang=de` (das Protokoll als Excel-Arbeitsmappe:
  eine Zusammenfassung, die Stromkreise und die Quellen),
  `POST /login` (`user` und `password`; `name` wird für `user` angenommen, wie auf dem
  Artikelserver), `POST /logout`, `GET /users`, `PUT|DELETE /users/{name}`, `GET|PUT /access`,
  `GET /directories`, `PUT|DELETE /directories/{id}`, `POST /directories/{id}/test`
  (Administratoren; die Passwörter der Dienstkonten werden nie zurückgesendet),
  `GET /sessions?limit=` (Administratoren: die offenen Sitzungen und die zuletzt
  beendeten, `SessionList`) (siehe
  `openecraft_core::collab`). `GET /info` nennt auch den angemeldeten `user` (das
  Servertoken meldet sich als `admin` an), was der Aufrufer darf (`access`) und ob die
  Webseite eingeschaltet ist (`web`). Mit `--web` außerdem `GET /texts?lang=de` (die
  Texte der Seite), `GET /projects/{name}/view?lang=de` (Seiten, Abschnitte, Strukturen,
  Betriebsmittel, Sperren und die Rechte des Aufrufers, aus `openecraft_view`) und
  `GET /projects/{name}/pages/{id}/svg` (eine Seite als SVG); beide brauchen das Recht,
  das Projekt anzusehen, und werden aus einer Analyse je Projekt und Revision gezeichnet
  (die letzten 8 Projekte werden behalten).

## Öffentlicher Projektserver

Das Projekt betreibt einen öffentlichen Projektserver unter
[project-server.openecraft.org](https://project-server.openecraft.org) mit
eingeschalteter Webseite. Die Arbeit darauf braucht ein Konto: Melden Sie sich auf der
Webseite an, um die Projekte zu sehen, die Sie öffnen dürfen, geben Sie die Server-URL
`https://project-server.openecraft.org` im Bereich *Projektserver* des Editors ein und
melden Sie sich dort mit demselben Konto an. Er zeigt, wie mehrere Benutzer an einem
Projekt arbeiten; Projekte einer Firma gehören auf einen eigenen Projektserver (oben),
hinter eigenen Zugriffseinstellungen.

## Impressum, Haftungsausschluss und Datenschutzhinweis

Unter der Webseite (`--web`) stehen die Links *Datenschutz* und, wenn der Betreiber
einen Namen hat, *Impressum* (auf Englisch *Privacy* und *Legal notice*), die jeweils
einen Dialog öffnen, sowie die Haftungsausschlüsse von OpenECraft (keine Gewährleistung,
die genannten Produktnamen gehören ihren Inhabern). Sie sind ohne Anmeldung vorhanden,
wie bei der [Website](../website.md#legal-notice), deren Felder und Variablen dieselben
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
fehlgeschlagenen Anmeldungen; wer wann an welchem Projekt arbeitet (Sitzungen, Sperren, die zuletzt beendeten Sitzungen) und wer ein Projekt zuletzt geändert hat. Was nur der Betreiber weiß (wer den
Server hostet, was ein Reverse-Proxy protokolliert und wie lange, an wen man sich
wendet, wie es Artikel 13 DSGVO verlangt), wird in `privacy` gesetzt und darunter
gezeigt. Halten Sie die Texte wahr: Eine Änderung, die etwas anderes speichert oder
sendet, ändert auch sie (`renderLegal` von `common.js`, `privacy()` des Skripts der
Seite). Ob ein Server ein Impressum braucht und was darin stehen muss, hängt vom
Betreiber und vom Land ab: OpenECraft zeigt nur, was gesetzt ist.
