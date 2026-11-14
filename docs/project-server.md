# Project server

Shared projects for a team: several users work on the same project, each page is locked by one of them.

`openecraft-project-server` stores projects for a team. Several users open the same
project at the same time and work on different pages; a page is edited by one user at a
time.

```bash
cargo run --release -p openecraft-project-server -- --bind 0.0.0.0:8091 --data projects --token <secret>
cargo run --release -p openecraft-project-server -- --config project-server.toml
```

As a container: `docker run -d -p 127.0.0.1:8091:8091 -e OPENECRAFT_PROJECT_TOKEN=<secret>
-v projects:/data ghcr.io/nilres/openecraft-project-server` (see [installation](installation.md)).

- Configuration file (`--config <file>` or `OPENECRAFT_PROJECT_SERVER_CONFIG`): the
  settings of the command line options in TOML (`bind`, `data`, `token`, `admin_user`,
  `access`, `web`; see
  [`project-server/project-server.example.toml`](../project-server/project-server.example.toml)),
  as for the parts server.
- **Web page** (optional: `--web`, `--web true`, `OPENECRAFT_PROJECT_WEB=true` or
  `web = true` in the configuration file; off by default): `http://<server>:8091/`
  lists the projects the user may see (with the pages, the revision, who created and
  last changed them, who works on them and the user's rights) and shows each of them read only: the pages by section or
  grouped by structure, the devices with the places they are shown on, and the pages
  drawn as on paper (zoom, drag to move, arrow keys or Page up / Page down for the
  next page). Pages locked by other users are marked with 🔒 and the user; the page
  looks at the project every 20 s and follows new revisions. Users sign in with their
  account or the server token; without sign-in the page shows what anonymous requests
  may see (the access settings). Project managers edit the grants of a project
  (*Permissions*). Administrators get an *Administration* area: an overview (projects
  in use, open sessions, users per role, access settings), the sessions (the open ones
  with their user, project, start, last activity, saves and locked pages, and the
  history of the ended ones with their duration and how they ended: closed, expired,
  signed out or user deleted), the users (role, projects, groups,
  password), the access rights (access without sign-in, defined roles, grants) and the
  directories (Active Directory, LDAP, with a test of the connection and of a
  sign-in). The page is in all languages of the app; it uses the API below, so the
  server checks every request as for the editor. Without `--web` the server answers
  `/` with a short text and has no page routes.
- **Created by, last changed by, last changed**: the server keeps for each project
  who created it and who saved the last change to it, and when; the project list of
  the editor (*Project server* pane), the web page and `GET /api/v1/projects`
  (`created_by`, `created_at`, `changed_by`, `changed_at`; seconds since 1970, UTC)
  show them to everyone who may view the project. The name is the signed-in user; on
  a server without accounts (or with the server token) it is the user name the
  session was opened with. It is independent of *Created by* in the project
  properties, which the users edit. Only a save that changes the project counts:
  opening or reading a project, the status requests that keep a session alive, locks
  and saving unchanged content leave it as it is. Projects of earlier server versions
  show *Unknown* until their next change (the creation stays unknown), and a project
  uploaded as a copy of another one is a new project of the user who uploaded it.
- Project names differing only in case are one name (Windows and macOS do not tell
  their files apart): requests find a project under any case of its name; Windows device names (`CON`, `NUL`, `COM1` …) and names ending
  with a dot are refused.
- Each project is stored as a split project in a folder of its own
  (`<name>/<name>.oecp` with one small file per page, device and part; it opens in the
  desktop editor and can be kept under version control) next to `<name>.state.json`
  (revision, id counter, who created and last changed the project) in the data
  directory. A save writes only the files that changed. Project files of earlier versions (`<name>.oec`) are converted at the
  first start and kept as `<name>.oec.bak`.
- With `--token` (or `OPENECRAFT_PROJECT_TOKEN`) every request needs
  `Authorization: Bearer <token>`; without it, user accounts and directories everyone
  who reaches the server can change its projects, but only clients on the server's own
  machine may create users and change the access settings (so nobody else takes the
  server over by creating the first administrator).
- The server speaks plain HTTP, so tokens and passwords cross the network unencrypted.
  Keep it on `127.0.0.1` behind a TLS reverse proxy when it is reachable from other
  machines, e.g. with [Caddy](https://caddyserver.com/), which fetches the certificate:
  `caddy reverse-proxy --from projects.example.com --to 127.0.0.1:8091`. The editor
  then uses `https://projects.example.com`.
- **User accounts** (`users.json` in the data directory, Argon2 password hashes):
  viewers open projects read-only, reviewers also save comments, markup and approvals,
  editors lock, change, save and create projects, project managers also delete
  projects and manage who may use them, administrators also manage the users; a user's
  role can be limited to some projects (names with `*` / `?`). `--admin-user <name>`
  (or `OPENECRAFT_PROJECT_ADMIN_USER`) creates the first administrator with the password from
  `OPENECRAFT_PROJECT_ADMIN_PASSWORD` (at least 8 characters, as every password). Users
  sign in from *Tools › Project server…* (user name, password, *Sign in*); the server uses
  the account's name on the locks, whatever the request says, and for the comments,
  replies, markup and approvals a signed-in user adds (approvals of other users stay as
  they are). Anonymous callers cannot use the name of an account. Managers of a project
  may add denials on it but only administrators lift them. User names have at most 64
  characters and no control characters; a new user needs a role, directory groups or a
  grant in the access settings. Signing out closes the projects opened with that
  sign-in and releases their locks; deleting a user closes all of theirs. A failed
  sign-in answers `wrong user name or password` whichever of the two was wrong.
- **Access control** (`access.json` in the data directory, `--access`): users sign in
  with a local account or with Active Directory / LDAP directories; roles (built-in or
  defined by administrators) are granted to users and directory groups per project,
  deny grants take rights away. Whoever creates a project manages it; project managers
  edit its grants in the *Project access* section of the project server window. See
  [docs/access-control.md](access-control.md).
- Locks: every page and the project data (page list and order, devices, parts, panel
  layout, settings) are locked separately. Editing something you have not locked
  requests the lock automatically; if another user holds it, the change is taken back.
  Pages locked by others are read-only and marked with 🔒 and the user's name in the
  page list. Locks end when they are released, when the project is closed, or 90 s after
  the editor stopped sending its status (every 15 s).
- Saving merges only what you have locked into the server's project; everything else
  comes from the server. Changes of other users are loaded automatically.
  A save names the revision it was edited from (`base_revision`; the editor does):
  the server then also takes over changed pages and project data nobody has locked
  (it locks them for you first) and answers with the changes it left out
  (`ignored`): locked by another user, changed by another user since that revision or
  not allowed. With `strict` nothing is saved in that case (`409`). The server keeps
  the last 16 revisions per project in memory; a save from an older revision (or
  without `base_revision`) saves only what is locked.
- Each session gets its own range of object ids, so users never create the same id.
- **Session history**: every ended session (user, project, start, last activity, end,
  saves, how it ended) is appended to `sessions.jsonl` in the data directory, one JSON
  line each; the server keeps the last 5000 and writes the file anew when it has grown
  to twice as many lines. The keys of the sessions are never stored or shown, only
  their numbers.
- Editors prefer the session-bound `POST /projects/{name}/snapshot`. On 404 or
  405 only, they use the older GET snapshot followed by the current session's
  status. Both must name the same revision before the snapshot is applied; the
  status supplies own lock identities. Authentication failures do not trigger
  this fallback. A project changed during loading is loaded again.
- In the editor: *Tools › Project server…* – enter the server URL, your user name and the
  token, list the projects and open one, or upload the current project. While a shared
  project is open, *Save* (`Ctrl+S`) saves to the server; the window and the page list's
  context menu lock and release pages. The status bar shows *Project server: … /
  Project: …* (*connection lost* while the server does not answer its status); a
  project restored from the last session or closed on the server shows as *Local
  recovery copy of server project: …* and is saved to a local file.
- **Automatic saving:** while a shared project is open, its finished changes are
  saved to the server by themselves, 30 seconds after a change and then every 30
  seconds while the work goes on (*File › Settings › Servers* or the *Project server*
  pane: on / off and the interval, 10 to 600 s). A drag or a text input under way is
  not sent; it is saved as soon as it is finished. Only one save runs at a time, an
  automatic save releases no lock and nothing is sent without changes. *Save*
  (`Ctrl+S`) still saves at once. The status bar and the pane tell the states apart:
  *Saved*, *Not saved yet*, *Saving…*, *Waiting for the server* (no answer, session
  ended, signed out: the changes are saved as soon as the session goes on), *Save
  failed* (tried again with the next interval) and *Partly saved* (the server left
  out what another user has locked or changed; your version is kept in the pane and
  is not sent again by itself), with the time of the last save the server confirmed.
  A project counts as saved only after the server's answer. The local recovery copy
  is no save on the server: during an outage it is written at once, and the pane
  says so only after it was written.
- The project list selects a project (a double click opens it). *Properties…* loads its
  properties from the server (never from the list) and shows who created it on the
  server and when (*Unknown* for projects created before the server kept that;
  this is not the *Created by* of the drawing, which users edit). Who may edit the
  project data changes them there without opening the project, unless a session has
  locked the project data; the server then replaces only the project info, and refuses
  if it was changed since it was loaded (*Reload* shows the newer one). For the open
  shared project, *Properties…* is the usual dialog, which locks the project data.
- *Copy project…* (for the selected or the open shared project) shows the properties
  of the copy at once: a free name on the server (*… copy*) and the title,
  description, company, author, date, revision and project properties of the original,
  all to be changed. Nothing exists on the server until *Create and open*; *Cancel*
  leaves nothing behind, and a refused copy (name taken, no right to create it) keeps
  the entries to correct. The copy is made on the server from the saved state:
  changes nobody has saved yet, also those of other users, are not part of it. With
  unsaved changes in the open project the editor asks first whether to save them. If
  the original was changed on the server after the dialog loaded it, the copy is not
  made until confirmed again. Sessions, page locks and access rights are not copied:
  the copy is opened in a new session and a signed-in user manages it like any project
  they create. Comments, markup, approvals (with their signatures), change requests,
  released revisions and baselines are left out unless *Also copy revisions, baselines
  and review records* is ticked; the approvers of the release workflow stay like all
  settings. The server records who created the copy, when, and from which project and
  revision.
- *File › Settings › On startup* decides what the editor opens when it starts: the
  start page with local files and server projects, the last project (a server project
  is opened on the server again), the project list of the server or a chosen file or
  server project. If the server cannot open the project, a dialog offers to retry, to
  sign in, to choose another project or to go on with the local copy; unsaved changes
  of the copy are saved or discarded first, never overwritten silently.
- REST API (`/api/v1`): `GET /info` (name, version and the number of projects the
  caller may see; answered without token too), `GET|POST /projects` (`POST` needs the
  `project`), `GET|PUT|DELETE /projects/{name}`,
  `POST /projects/{name}/open|status|lock|release|close`, `GET|PUT /projects/{name}/access`,
  `GET|PUT /projects/{name}/properties` (the project info with the revision, who has
  locked the project data, the caller's rights, who created and last changed the project and what it was copied from; `PUT`
  takes `user`, `revision` and `info`, counts as the last change, needs the right to edit the project data, answers `409`
  while the project data is locked and `412` if the info changed since `revision`),
  `POST /projects/{name}/copy` (`name` and `user` of the copy, optionally the
  `revision` the source must still have – `412` otherwise –, its `info` and `records`;
  needs the right to view the source and to create the new name, answers like opening
  a project),
  `POST /login` (`user` and `password`; `name` is taken for `user`, as on the parts
  server), `POST /logout`, `GET /users`, `PUT|DELETE /users/{name}`, `GET|PUT /access`,
  `GET /directories`, `PUT|DELETE /directories/{id}`, `POST /directories/{id}/test`
  (administrators; the passwords of the service accounts are never sent back),
  `GET /sessions?limit=` (administrators: the open sessions and the last ended ones,
  `SessionList`) (see
  `openecraft_core::collab`). `GET /info` also tells the signed-in `user` (the server
  token signs in as `admin`), what the caller may do (`access`) and whether the web
  page is on (`web`). With `--web` also `GET /texts?lang=de` (the page's texts),
  `GET /projects/{name}/view?lang=de` (pages, sections, structures, devices, locks and
  the caller's rights, from `openecraft_view`) and `GET /projects/{name}/pages/{id}/svg`
  (a page as SVG); both need the right to view the project and are drawn from one
  analysis per project and revision (the last 8 projects are kept).
