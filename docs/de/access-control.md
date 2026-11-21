# Zugriffskontrolle der Server

Der Artikelserver und der Projektserver teilen sich ein Zugriffsmodell: Benutzer melden
sich mit einem lokalen Konto oder mit einem Verzeichnisdienst an (Active Directory,
OpenLDAP oder ein anderer LDAP-Server), und was sie dürfen, ergibt sich aus **Rollen**,
die ihnen oder ihren **Gruppen** durch **Zuweisungen** gegeben werden, je Projekt
(Projektserver) oder je Hersteller (Artikelserver).

- Regeln und ihre Auswertung: `openecraft_parts::access` (gemeinsam mit den Clients)
- Verzeichnisdienste und die Einstellungsdatei: `openecraft-auth` (`directory.rs`,
  `config.rs`)
- Rechte und eingebaute Rollen: `openecraft_core::collab::{ProjectRight, ProjectRole}` und
  `openecraft_parts::portal::{PartsRight, Role}`

## Gespeicherte Anmeldungen am Projektserver

Im Bereich **Projektserver** des Editors ist **Angemeldet bleiben** eine ausdrückliche,
optionale Einstellung für den gewählten Server und das gewählte Konto. Unter Windows
gelangt nur das Sitzungstoken des Kontos in die Windows-Anmeldeinformationsverwaltung.
Passwörter werden nie dauerhaft gespeichert. Die Einstellungen der Anwendung enthalten
nur Metadaten zu Server und Konto sowie die Zustimmung, unabhängig je Konto, sodass ein
anderes Editorfenster die Anmeldung eines anderen Kontos nicht abschalten kann. Die
vollständige Serveridentität umfasst Schema, Host, wirksamen Port und Basispfad;
Anmeldedaten wandern nie zu einem anderen Server. HTTP und HTTPS bleiben verschieden,
und diese Funktion ändert oder verschlüsselt den bestehenden Transport nicht.

Beim Start prüft `GET /api/v1/session` das gespeicherte Kontotoken und liefert
`CurrentAccount { user, access }`, mit den aktuellen Rechten und ohne Token oder
Passwort. Root-Token, anonymer Zugriff, ungültige und abgelaufene Kontotoken erhalten
401. Diese Anfrage erneuert das Token nicht und berührt keine Seitensperren.
Die Kontositzung läuft derzeit 30 Tage nach der Anmeldung ab; die davon getrennte
Projektsitzung mit 90 Sekunden Leerlauf bleibt unberührt. Ältere Server ohne diesen
Endpunkt erfordern eine Anmeldung von Hand; ihr 404 löscht keine möglicherweise gültigen
gespeicherten Anmeldedaten. Auch Verbindungsfehler behalten die Zustimmung für einen
späteren neuen Versuch.

Läuft ein Kontotoken ab, während ein gemeinsames Projekt geöffnet ist, setzt eine
erneute Anmeldung mit demselben Server und Konto dieses Projekt über den Ablauf der
Sitzungswiederherstellung fort. Nicht gespeicherte Änderungen und der
Rückgängig-Verlauf bleiben verfügbar; die Anmeldung markiert sie nicht als gespeichert
und ersetzt das Projekt nicht durch eine frische Kopie.

**Abmelden** entfernt die gespeicherten Anmeldedaten und bittet den Server, das
Kontotoken zu widerrufen, nachdem ausstehende Speicherungen und das Schließen von
Projekten beantwortet wurden. **Benutzer wechseln** gibt die Projektsitzung frei und
leert den gewählten Benutzernamen, behält aber die gespeicherte Anmeldung dieses Kontos
für das Zurückwechseln. Ein geöffnetes gemeinsames Projekt und sein Rückgängig-Verlauf
bleiben als lokale Wiederherstellungskopie erhalten, einschließlich nicht gespeicherter
Änderungen. Das lokale Abmelden funktioniert offline; der Bereich meldet gesondert, wenn
die Serversitzung nicht beendet oder der geschützte Speicher nicht geändert werden
konnte. Vorgänge des Betriebssystems laufen der Reihe nach, einschließlich des Löschens
nach einer bereits laufenden Speicherung; das normale Schließen der Anwendung wartet auf
ihre Ergebnisse. Antworten einer früheren Auswahl von Server und Konto stellen nie deren
Identität wieder her und ersetzen nie das geöffnete Projekt. Eine verzögerte
Speicherbestätigung gehört während der erneuten Anmeldung weiterhin zum selben
beibehaltenen Projekt; sie setzt die gespeicherte Revision weiter, ohne die neue
Anmeldung zu ändern. Der Widerruf im Hintergrund sperrt die nächste Anmeldung nicht,
während auf einen Server gewartet wird.

Das Aufräumen ungültiger Token vergleicht das abgelehnte Token, bevor es gelöscht wird.
Es schreibt nie eine neue Ablehnung: Ein anderes Fenster kann bereits eine neuere
Anmeldung gespeichert haben. Anmeldungen über Verzeichnisdienste behalten ihre
Zustimmung unter dem vom Server bestätigten Kontonamen, auch wenn der eingegebene Name
mit Domäne qualifiziert oder ein Benutzerprinzipalname war.

Der Browser, macOS und Linux bieten derzeit nur die Anmeldung für die Sitzung. Ihr
Kontrollkästchen ist mit einer Erklärung deaktiviert; es gibt keinen Rückfall auf Token
oder Passwort im lokalen Speicher, in IndexedDB, in gewöhnlichen Einstellungen oder in
Projektdateien. Die Windows-Unterstützung verwendet das vorhandene
Anmeldedaten-Backend des Betriebssystems, das auch für EPLAN-PATs genutzt wird, mit
einem eigenen Namensraum. Diese Sitzungs- und Plattformfunktionen ändern weder
Projektbefehle noch die Plugin-API.

Die isolierte Live-Prüfung ist `scripts/test-project-sign-in.ps1` (PowerShell 7,
Windows). Sie verwendet einen Wegwerf-Projektserver auf der Loopback-Adresse,
synthetische Konten, einen eigenen Metadatenordner und eindeutige Schlüssel für
Anmeldedaten, die der Test anschließend entfernt.

## Rechte und eingebaute Rollen

Projektserver (der Geltungsbereich einer Zuweisung ist ein Projektname):

| Recht | Erlaubt |
| --- | --- |
| `view` | das Projekt in der Liste sehen, es schreibgeschützt öffnen |
| `review` | Prüfvermerke (Kommentare, Markierungen, Freigaben) ohne Sperren speichern |
| `edit_pages` | Seiten sperren und ändern |
| `edit_project` | die Projektdaten sperren und ändern (Seitenliste, Betriebsmittel, Artikel, Einstellungen) |
| `create` | Projekte anlegen (der Geltungsbereich ist der neue Name) |
| `delete` | ein Projekt löschen, das niemand geöffnet hat |
| `manage_access` | die Zuweisungen für das Projekt ändern |
| `administer` | Benutzer, Rollen und alle Zuweisungen (nur aus Zuweisungen auf `*`) |

| Rolle | Rechte |
| --- | --- |
| `viewer` | view |
| `reviewer` | view, review |
| `editor` | view, review, edit_pages, edit_project, create |
| `manager` | editor + delete, manage_access |
| `admin` | alles |

Ein angemeldeter Benutzer, der ein Projekt anlegt, wird dessen `manager` (eine Zuweisung
auf das neue Projekt), sodass Projekteigentümer verwalten, wer an ihren Projekten
arbeitet.

Artikelserver (der Geltungsbereich einer Zuweisung ist ein Hersteller oder ein Katalog
als `catalog:<id>`):

| Recht | Erlaubt |
| --- | --- |
| `read` | durchsuchen, suchen, Dateien herunterladen, Favoriten (ganzer Server) |
| `edit_parts` | Artikel und ihre Dateien anlegen und ändern, importieren |
| `delete_parts` | Artikel löschen |
| `edit_catalogs` | Kataloge anlegen, ändern und löschen |
| `administer` | Benutzer, Rollen und Zuweisungen (ganzer Server) |

Eingebaute Rollen: `reader` (read), `manufacturer` und `editor` (read, edit_parts,
delete_parts, edit_catalogs), `admin` (alles). Die eigene Rolle eines Kontos wirkt
weiterhin wie bisher: Ein Herstellerbenutzer hat die Herstellerrolle nur für seinen
Hersteller.

Die Rechte an einem Artikel oder einem Katalog sind die an seinem Hersteller zusammen
mit denen an seinem Katalog (`catalog:siemens`): Eine Zuweisung auf einen Katalog lässt
Benutzer und Gruppen den Katalog und seine Artikel ändern, gleich welchen Herstellers,
und eine verweigernde Zuweisung auf eines von beiden nimmt das Recht weg.
Administratoren legen die Zuweisungen eines Katalogs in *Verwaltung › Kataloge ›
Berechtigungen* fest (`GET` / `PUT /api/v1/catalogs/{id}/grants`); das Lesen bleibt ein
Recht des ganzen Servers.

## Wie Rechte aufgelöst werden

1. Die eingebaute Rolle des Kontos (Projektserver: für seine Projekte; keine für
   Benutzer, die aus einem Verzeichnisdienst kamen).
2. Jede Zuweisung der Richtlinie, deren Subjekt auf den Benutzer passt
   (`{"user": "anna"}`) oder auf eine seiner Gruppen (`{"group": "corp:Engineering"}`
   eines Verzeichnisdienstes, `{"group": "Wiring"}`, auf dem Artikelserver definiert).
   Namen und Geltungsbereiche nehmen die Platzhalter `*` und `?` an und unterscheiden
   keine Groß- und Kleinschreibung; `{"user": "*"}` ist jeder angemeldete Benutzer.
3. Die Rolle anonymer Anfragen (`anonymous`), die auch angemeldete Benutzer haben.
4. **Verweigernde Zuweisungen gewinnen:** Rechte einer Zuweisung mit `"deny": true`
   werden in ihrem Geltungsbereich weggenommen, was auch immer die anderen Zuweisungen
   geben.

Serverweite Rechte (`administer` und `read` auf dem Artikelserver) kommen nur aus
Zuweisungen auf `*`. Das Servertoken (`--token`) behält den vollen Zugriff.

## Die Einstellungsdatei

`access.json` im Datenverzeichnis des Projektservers, `<data>.access.json` neben der
Artikeldatenbank (beide mit `--access <file>` änderbar). Eine fehlende Datei bedeutet:
keine Verzeichnisdienste, keine definierten Rollen oder Zuweisungen; auf dem
Artikelserver dürfen anonyme Anfragen lesen (`"anonymous": "reader"`). Administratoren
ändern Rollen, Zuweisungen und `anonymous` über die API (`GET|PUT /api/v1/access`), die
die Datei neu schreibt, und die Verzeichnisdienste über `/api/v1/directories` (`GET`,
`PUT|DELETE /{id}`, `POST /{id}/test`). Die Webseiten tun dies ebenfalls: die des
Artikelservers (*Verwaltung › Zugriffsrechte*, die sie auch exportiert und
wiederherstellt) und die des Projektservers, wenn er mit `--web` läuft
(*Verwaltung › Zugriffsrechte* und *Verzeichnisdienste*).

```json
{
  "directories": [
    {
      "id": "corp",
      "title": "Corporate AD",
      "kind": "active_directory",
      "url": "ldaps://dc1.corp.example",
      "base_dn": "DC=corp,DC=example",
      "bind_dn": "CN=svc-openecraft,OU=Service,DC=corp,DC=example",
      "bind_password_env": "OPENECRAFT_LDAP_PASSWORD",
      "nested_groups": true,
      "allowed_groups": ["OpenECraft *"]
    },
    {
      "id": "lab",
      "kind": "ldap",
      "url": "ldap://ldap.lab.example",
      "starttls": true,
      "user_bind": "uid={user},ou=people,dc=lab,dc=example",
      "base_dn": "ou=people,dc=lab,dc=example",
      "group_filter": "(&(objectClass=groupOfNames)(member={dn}))",
      "group_base_dn": "ou=groups,dc=lab,dc=example"
    }
  ],
  "anonymous": "",
  "roles": [
    { "name": "checker", "description": "Checks drawings", "rights": ["view", "review"] }
  ],
  "grants": [
    { "subject": { "group": "corp:Engineering" }, "role": "editor", "scope": "Plant*" },
    { "subject": { "group": "corp:Quality" }, "role": "checker", "scope": "*" },
    { "subject": { "group": "corp:Contractors" }, "role": "viewer", "scope": "Secret*", "deny": true },
    { "subject": { "group": "corp:OpenECraft Admins" }, "role": "admin", "scope": "*" }
  ]
}
```

Auf dem Artikelserver lässt `"registration": true` Benutzer sich selbst als Leser
registrieren (geschaltet in *Verwaltung › Zugriffsrechte*; die Startoption
`--registration` hat Vorrang, siehe [docs/parts-server.md](parts-server.md)); ihre
Konten warten auf die Freigabe durch einen Administrator, sofern nicht
`"registration_approval": false` gesetzt ist.
`"dashboard": false` blendet das Dashboard der Webseite aus (*Verwaltung ›
Übersicht*; die Startoption `--dashboard` hat Vorrang).

### Gruppen

Der Artikelserver führt Benutzergruppen in `groups` (*Verwaltung › Gruppen*,
`GET /api/v1/groups`, `PUT` / `DELETE /api/v1/groups/{name}`):

```json
"groups": [{ "name": "Wiring", "description": "Panel shop", "members": ["anna", "bob"] }]
```

Mitglieder sind Benutzernamen, lokale Benutzer ebenso wie Benutzer eines
Verzeichnisdienstes (mit dem Namen, mit dem sie sich anmelden). Zuweisungen an
`{"group": "Wiring"}` gelten für die Mitglieder neben den Gruppen ihres
Verzeichnisdienstes; Gruppennamen können kein `:` enthalten, das die Gruppen der
Verzeichnisdienste abtrennt. Die Gruppen eines Benutzers (*Benutzer*, `groups` von
`PUT /api/v1/users/{name}`) sind Mitgliedschaften: Ihr Speichern fügt den Benutzer den
genannten Gruppen hinzu (neue Namen werden zu Gruppen) und entfernt ihn aus den anderen.
Gruppen, die lokale Konten früher nannten, werden beim Start in Gruppen des Servers
umgewandelt. Das Löschen einer Gruppe löscht die Zuweisungen an sie, das Löschen eines
Benutzers seine Mitgliedschaften; Namen von Mitgliedern können nicht registriert
werden, und eine Änderung, nach der niemand mehr Benutzer verwalten würde, wird
abgelehnt.

### Verzeichnisdienste

Administratoren richten die Verzeichnisdienste auch in *Verwaltung ›
Verzeichnisdienste* ein (`GET /api/v1/directories`, `PUT` /
`DELETE /api/v1/directories/{id}`), das kein Passwort zeigt (ein leeres Passwort behält
das gespeicherte) und die Einstellungen testet: *Testen* verbindet sich und bindet sich
mit dem Dienstkonto, mit Benutzername und Passwort meldet es den Benutzer an und zeigt
seine Gruppen (`POST /api/v1/directories/{id}/test`).

| Schlüssel | Bedeutung |
| --- | --- |
| `id` | Kurzname; Gruppen erscheinen als `id:group`, Benutzer des Verzeichnisdienstes zeigen ihn |
| `kind` | `active_directory` (Standard) oder `ldap` |
| `url` | `ldap://…` oder `ldaps://…`; `starttls` stuft `ldap://` hoch |
| `insecure_tls` | jedes Zertifikat annehmen (nur für Tests); sonst wird der CA-Speicher des Systems verwendet |
| `bind_dn`, `bind_password` / `bind_password_env` | Dienstkonto, um Benutzer nachzuschlagen |
| `user_bind` | ohne Dienstkonto: Bind-Name des Benutzers, `{user}@corp.example` oder `uid={user},ou=…` |
| `base_dn` | wo Benutzer gesucht werden |
| `user_filter` | Standard `(&(objectClass=user)(sAMAccountName={user}))` / `(&(objectClass=person)(uid={user}))` |
| `name_attribute` | Kontoname, Standard `sAMAccountName` / `uid` |
| `nested_groups` | Active Directory: auch Gruppen von Gruppen (`LDAP_MATCHING_RULE_IN_CHAIN`) |
| `group_filter`, `group_base_dn` | die Gruppen suchen (`{dn}`, `{user}`), statt `memberOf` zu lesen |
| `allowed_groups` | nur Mitglieder dieser Gruppen dürfen sich anmelden (leer: alle) |
| `timeout_secs` | Zeitlimit für Verbindung und Anfrage (Standard 10) |

Die Anmeldung versucht zuerst die lokalen Konten; ein Name ohne lokales Konto geht der
Reihe nach an die Verzeichnisdienste (oder nur an den im `provider` der Anfrage
genannten). Der erste Verzeichnisdienst, der den Benutzer kennt, prüft das Passwort (ein
Bind als der Benutzer; leere Passwörter werden abgelehnt, bevor der Verzeichnisdienst
gefragt wird) und liefert die Gruppen. Der Server führt ein Konto für den
Verzeichnisbenutzer (kein Passwort; Gruppen bei jeder Anmeldung aufgefrischt), sodass
Administratoren ihn in der Benutzerliste sehen und Sitzungstoken wie für lokale Benutzer
funktionieren. Ein Name, der bereits zu einem lokalen Konto gehört, kann sich nicht über
einen Verzeichnisdienst anmelden.

Passwörter lokaler Konten brauchen mindestens 8 Zeichen. Nach 5 fehlgeschlagenen
Anmeldungen eines Benutzernamens (lokal oder Verzeichnisdienst) von einer Clientadresse
wartet jeder weitere Versuch dieses Namens von dort, zunächst 1 s und nach jedem
weiteren Fehlschlag doppelt so lange, bis zu 5 Minuten; nach 30 fehlgeschlagenen
Anmeldungen eines Clients mit beliebigen Namen warten seine Versuche auf dieselbe Weise.
Die Server antworten währenddessen mit `429 Too Many Requests`. Andere können einen
Benutzer auf diese Weise nicht aussperren; eine erfolgreiche Anmeldung vergisst die
Fehlschläge des Namens (nicht die des Clients), und Fehlschläge werden nach einer Stunde
vergessen. Die Ablehnung eines Namens ohne Konto dauert so lange wie die eines falschen
Passworts, sodass die Antworten nicht verraten, welche Namen existieren. Fehler beider
Server kommen als `{"error": "…"}`, auch für unbekannte Routen und Anfragen, die sich
nicht parsen lassen.

Eine Sitzung (das Token einer Anmeldung) endet bei der Abmeldung, wenn das Passwort
geändert wird und 30 Tage nach der Anmeldung; ein Benutzer behält höchstens 20. Von
früheren Versionen gespeicherte Sitzungen haben keinen Beginn und gelten als beendet:
Ihre Benutzer melden sich einmal neu an. Anfragen, die etwas ändern, werden geprüft,
bevor ihr Rumpf gelesen wird, sodass unbekannte Aufrufer einen Server nicht dazu bringen
können, große Rümpfe zu lesen; die Anmeldung (und die Suche des Artikelservers) nehmen
höchstens 64 kB an.

Beide Server senden mit jeder Antwort `X-Content-Type-Options: nosniff`,
`X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` und eine Content Security
Policy. Die Benutzer- und Zugriffsdateien (`users.json`, `access.json`, eine
JSON-Artikeldatenbank) werden nur für ihren Eigentümer lesbar geschrieben (Unix). Ziehen
Sie `bind_password_env` dem `bind_password` vor (die Server warnen vor einem Passwort in
der Datei) und `OPENECRAFT_TOKEN` / `OPENECRAFT_PROJECT_TOKEN` oder die
Konfigurationsdatei dem `--token`, das andere Benutzer der Maschine in der Prozessliste
sehen. Keiner der Server spricht TLS: Stellen Sie einen Reverse-Proxy mit HTTPS vor
einen Server, der über das Netzwerk erreicht wird.

Werte werden für LDAP-Filter und DNs maskiert. Halten Sie das Dienstkonto
schreibgeschützt.

## Clients

- Editor, *Werkzeuge › Projektserver…*: zeigt die Rechte am geöffneten Projekt;
  Betrachter und Prüfer können es nicht ändern, Projektleiter bearbeiten die
  Zuweisungen dafür im Abschnitt *Projektzugriff*.
- Artikelverwaltung: Der Benutzereditor zeigt den Verzeichnisdienst und die Gruppen
  eines Benutzers und bearbeitet die Gruppen des Servers, in denen der Benutzer Mitglied
  ist.
- Kommandozeile / MCP: `parts_access` (setzt auch `anonymous`), `parts_grant` (mit
  `deny`, oder `revoke`, um Zuweisungen zu entfernen), `parts_role` (definieren, ändern,
  `delete`), `parts_registration` (`enabled`, `approval`), `parts_user_approve` und
  `parts_user_save` mit `groups`.
