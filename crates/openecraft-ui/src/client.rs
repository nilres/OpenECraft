//! Client for the global parts server (`openecraft-parts-server`, the data portal) or
//! the EPLAN Data Portal.
//! Requests run in the background (thread on desktop, fetch in the browser); results
//! are collected with [`ServerClient::poll`]. The requests themselves are built by
//! `openecraft_client`.

use openecraft_i18n::{tr, tr_fmt};
use openecraft_client::{check, decode, decode_found, ehttp, Call, Connection, Endpoint, PortalKind};
use openecraft_parts::api::{Accessories, ImportResult, PartList, PartQuery, ServerInfo, Session, UserUpdate};
use openecraft_parts::portal::{AttachmentKind, Catalog, HistoryEntry, SearchResult, User};
use openecraft_parts::Part;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Result of a finished request.
#[derive(Debug)]
pub enum Reply {
    Info(Result<ServerInfo, String>),
    Parts(Result<PartList, String>),
    Imported(Result<ImportResult, String>),
    /// A part was saved, or a file attached to / removed from it.
    Saved(Result<Box<Part>, String>),
    /// Part number and result.
    Deleted(String, Result<(), String>),
    Search(Result<SearchResult, String>),
    /// Server versions of local parts (update check).
    Versions(Result<SearchResult, String>),
    /// Part number and its changes.
    History(String, Result<Vec<HistoryEntry>, String>),
    /// Part number and the whole part (with its models) for the details of the data
    /// portal; `None` if the server does not know it.
    Detail(String, Result<Option<Box<Part>>, String>),
    /// Part number, its accessories and the parts it fits.
    Accessories(String, Result<Accessories, String>),
    Catalogs(Result<Vec<Catalog>, String>),
    CatalogSaved(Result<Catalog, String>),
    CatalogDeleted(String, Result<(), String>),
    SignedIn(Result<Session, String>),
    SignedOut,
    Favorites(Result<Vec<String>, String>),
    Users(Result<Vec<User>, String>),
    UserSaved(Result<User, String>),
    UserDeleted(String, Result<(), String>),
    /// File name, purpose given with the request and the content.
    File(String, FileUse, Result<Vec<u8>, String>),
    /// Name and JSON text of a library; `None` if the server has none of that name.
    Library(String, Result<Option<String>, String>),
    LibrarySaved(String, Result<(), String>),
}

/// What a downloaded file is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileUse {
    /// Save it as a file.
    Save,
    /// Merge it into the macro library.
    Macros,
}

impl Reply {
    pub fn error(&self) -> Option<&str> {
        match self {
            Reply::Info(Err(e))
            | Reply::Parts(Err(e))
            | Reply::Imported(Err(e))
            | Reply::Saved(Err(e))
            | Reply::Deleted(_, Err(e))
            | Reply::Search(Err(e))
            | Reply::Versions(Err(e))
            | Reply::History(_, Err(e))
            | Reply::Detail(_, Err(e))
            | Reply::Accessories(_, Err(e))
            | Reply::Catalogs(Err(e))
            | Reply::CatalogSaved(Err(e))
            | Reply::CatalogDeleted(_, Err(e))
            | Reply::SignedIn(Err(e))
            | Reply::Favorites(Err(e))
            | Reply::Users(Err(e))
            | Reply::UserSaved(Err(e))
            | Reply::UserDeleted(_, Err(e))
            | Reply::File(_, _, Err(e))
            | Reply::Library(_, Err(e))
            | Reply::LibrarySaved(_, Err(e)) => Some(e),
            _ => None,
        }
    }

    /// Text for a status line.
    pub fn describe(&self) -> String {
        if let Some(e) = self.error() {
            return tr_fmt!("Parts server: {}", e);
        }
        match self {
            Reply::Info(Ok(info)) => {
                let access = match &info.user {
                    Some(u) => tr_fmt!("signed in as {} ({})", u.name, tr(u.role.title())),
                    None if info.writable => tr("changes with token").to_string(),
                    None => tr("read-only").to_string(),
                };
                tr_fmt!("Connected: {} {} – {} parts, revision {}, {}", info.name, info.version, info.parts, info.revision, access)
            }
            Reply::Parts(Ok(list)) => tr_fmt!("Loaded {} parts (revision {})", list.parts.len(), list.revision),
            Reply::Imported(Ok(r)) => tr_fmt!("Uploaded {} parts, server revision {}", r.imported, r.revision),
            Reply::Saved(Ok(p)) => tr_fmt!("Saved part {}", p.number),
            Reply::Deleted(number, Ok(())) => tr_fmt!("Deleted part {}", number),
            Reply::Search(Ok(r)) => tr_fmt!("{} parts found", r.total),
            Reply::Versions(Ok(r)) => tr_fmt!("Compared with {} parts of the server", r.total),
            Reply::History(number, Ok(h)) => tr_fmt!("{} changes of part {}", h.len(), number),
            Reply::Catalogs(Ok(c)) => tr_fmt!("{} catalogs", c.len()),
            Reply::CatalogSaved(Ok(c)) => tr_fmt!("Saved catalog {}", c.name),
            Reply::CatalogDeleted(id, Ok(())) => tr_fmt!("Deleted catalog {}", id),
            Reply::SignedIn(Ok(s)) => tr_fmt!("Signed in as {} ({})", s.user.name, tr(s.user.role.title())),
            Reply::SignedOut => tr("Signed out").to_string(),
            Reply::Favorites(Ok(f)) => tr_fmt!("{} favorites", f.len()),
            Reply::Users(Ok(u)) => tr_fmt!("{} users", u.len()),
            Reply::UserSaved(Ok(u)) => tr_fmt!("Saved user {}", u.name),
            Reply::UserDeleted(name, Ok(())) => tr_fmt!("Deleted user {}", name),
            Reply::File(name, _, Ok(data)) => tr_fmt!("Downloaded {} ({} bytes)", name, data.len()),
            Reply::Library(name, Ok(Some(_))) => tr_fmt!("Loaded the library {}", name),
            Reply::Library(name, Ok(None)) => tr_fmt!("The parts server has no library {} yet", name),
            Reply::LibrarySaved(name, Ok(())) => tr_fmt!("Saved the library {}", name),
            _ => String::new(),
        }
    }
}

pub struct ServerClient {
    /// Kind of the server: the OpenECraft parts server or the EPLAN Data Portal (read
    /// through `openecraft_client::eplan`, so the part handling works the same).
    pub kind: PortalKind,
    pub url: String,
    /// Access token for changes: the administrator token or the token of a session;
    /// for the EPLAN Data Portal the personal access token (kept in memory only).
    pub token: String,
    pub info: Option<ServerInfo>,
    pub status: String,
    /// Address and token of the other kind of server.
    other: (String, String),
    /// Portal ids of the parts the EPLAN Data Portal sent, by number (it loads
    /// several parts only by id).
    portal_ids: HashMap<String, String>,
    /// The last finished request failed (`status` shows why).
    failed: bool,
    pending: usize,
    inbox: Arc<Mutex<Vec<Reply>>>,
}

impl Default for ServerClient {
    fn default() -> Self {
        let Endpoint { url, token } = Endpoint::default();
        ServerClient {
            kind: PortalKind::OpenECraft,
            url,
            token,
            info: None,
            status: String::new(),
            other: (PortalKind::Eplan.default_url(), String::new()),
            portal_ids: HashMap::new(),
            failed: false,
            pending: 0,
            inbox: Arc::default(),
        }
    }
}

impl ServerClient {
    pub fn busy(&self) -> bool {
        self.pending > 0
    }

    /// True if the last connection check found a server that accepts changes.
    pub fn writable(&self) -> bool {
        self.info.as_ref().is_some_and(|i| i.writable)
    }

    /// The server only offers searching and reading parts (EPLAN Data Portal).
    pub fn read_only(&self) -> bool {
        self.kind.read_only()
    }

    /// The signed in user (as of the last connection check or sign-in).
    pub fn user(&self) -> Option<&User> {
        self.info.as_ref().and_then(|i| i.user.as_ref())
    }

    pub fn endpoint(&self) -> Endpoint {
        Endpoint { url: self.url.clone(), token: self.token.clone() }
    }

    pub fn connection(&self) -> Connection {
        Connection::new(self.kind, &self.url, &self.token)
    }

    /// Switches the kind of server; the address and token of each kind are kept.
    pub fn set_kind(&mut self, kind: PortalKind) {
        if kind == self.kind {
            return;
        }
        let current = (std::mem::take(&mut self.url), std::mem::take(&mut self.token));
        (self.url, self.token) = std::mem::replace(&mut self.other, current);
        self.kind = kind;
        self.info = None;
        self.status.clear();
        self.failed = false;
    }

    /// The address of a kind of server (stored with the settings).
    pub fn url_of(&self, kind: PortalKind) -> &str {
        if kind == self.kind {
            &self.url
        } else {
            &self.other.0
        }
    }

    pub fn set_url_of(&mut self, kind: PortalKind, url: String) {
        if kind == self.kind {
            self.url = url;
        } else {
            self.other.0 = url;
        }
    }

    /// Address of a file for the web browser (`None`: it needs the access token).
    pub fn file_link(&self, id: &str) -> Option<String> {
        self.connection().file_link(id)
    }

    /// Sends the call; a refused call (`Err`) is answered at once.
    fn send<T: 'static>(
        &mut self,
        ctx: &egui::Context,
        call: Result<Call<T>, String>,
        status: String,
        wrap: impl 'static + Send + FnOnce(Result<T, String>) -> Reply,
    ) {
        self.status = status;
        self.pending += 1;
        let inbox = self.inbox.clone();
        let ctx = ctx.clone();
        let deliver = move |reply| {
            if let Ok(mut inbox) = inbox.lock() {
                inbox.push(reply);
            }
            ctx.request_repaint();
        };
        match call {
            Ok(Call { request, decode }) => ehttp::fetch(request, move |result| deliver(wrap(decode(result)))),
            Err(e) => deliver(wrap(Err(e))),
        }
    }

    /// A function only the OpenECraft parts server offers.
    fn server<T: 'static>(
        &self,
        request: impl FnOnce(&Endpoint) -> ehttp::Request,
        decode: impl FnOnce(ehttp::Result<ehttp::Response>) -> Result<T, String> + Send + 'static,
    ) -> Result<Call<T>, String> {
        let connection = self.connection();
        Ok(Call::new(request(connection.parts_server()?), decode))
    }

    /// Checks the connection (server info).
    pub fn check(&mut self, ctx: &egui::Context) {
        let call = self.connection().info();
        self.send(ctx, Ok(call), tr("Connecting…").into(), Reply::Info);
    }

    /// Loads all parts.
    pub fn download(&mut self, ctx: &egui::Context) {
        let call = self.server(|e| e.list(&PartQuery::default()), decode);
        self.send(ctx, call, tr("Downloading parts…").into(), Reply::Parts);
    }

    /// Loads the parts with these numbers (e.g. to add them to the project).
    pub fn fetch(&mut self, ctx: &egui::Context, numbers: &[String]) {
        let ids: Vec<String> = numbers.iter().filter_map(|n| self.portal_ids.get(n)).cloned().collect();
        let call = self.connection().find(numbers, &ids);
        self.send(ctx, Ok(call), tr("Downloading parts…").into(), |r| {
            Reply::Parts(r.map(|s| PartList { revision: s.revision, total: s.total, offset: s.offset, parts: s.parts }))
        });
    }

    /// Faceted search (part lists without models).
    pub fn search(&mut self, ctx: &egui::Context, query: &PartQuery) {
        let call = self.connection().search(&PartQuery { brief: true, ..query.clone() });
        self.send(ctx, Ok(call), tr("Searching…").into(), Reply::Search);
    }

    /// Loads the server versions of local parts for an update check; on the EPLAN Data
    /// Portal of the parts loaded from there ([`PortalKind::update_candidates`]).
    pub fn versions(&mut self, ctx: &egui::Context, local: &[Part]) {
        let local = self.kind.update_candidates(local);
        let numbers: Vec<String> = local.iter().map(|p| p.number.clone()).collect();
        let ids: Vec<String> = local.iter().map(|p| p.portal_id.clone()).filter(|id| !id.is_empty()).collect();
        let call = if self.read_only() && ids.is_empty() {
            Err(tr("No part of the database comes from the EPLAN Data Portal").to_string())
        } else {
            Ok(self.connection().find(&numbers, &ids))
        };
        self.send(ctx, call, tr("Checking for part updates…").into(), Reply::Versions);
    }

    /// Uploads parts as JSON (needs the access token).
    pub fn upload(&mut self, ctx: &egui::Context, parts: &[Part]) {
        let call = self.server(|e| e.import(serde_json::to_vec(parts).unwrap_or_default(), "application/json"), decode);
        self.send(ctx, call, tr_fmt!("Uploading {} parts…", parts.len()), Reply::Imported);
    }

    /// Uploads the content of a CSV or JSON parts file.
    pub fn upload_file(&mut self, ctx: &egui::Context, content: Vec<u8>) {
        let call = self.server(|e| e.import(content, "text/plain; charset=utf-8"), decode);
        self.send(ctx, call, tr("Uploading parts file…").into(), Reply::Imported);
    }

    /// Creates or replaces a part.
    pub fn save(&mut self, ctx: &egui::Context, part: &Part) {
        let call = self.server(|e| e.save(part), decode);
        self.send(ctx, call, tr_fmt!("Saving {}…", part.number), |r| Reply::Saved(r.map(Box::new)));
    }

    pub fn delete(&mut self, ctx: &egui::Context, number: &str) {
        let call = self.server(|e| e.delete(number), |r| check(r).map(|_| ()));
        let n = number.to_string();
        self.send(ctx, call, tr_fmt!("Deleting {}…", number), move |r| Reply::Deleted(n, r));
    }

    pub fn history(&mut self, ctx: &egui::Context, number: &str) {
        let call = self.server(|e| e.history(number), decode);
        let n = number.to_string();
        self.send(ctx, call, tr("Loading the history…").into(), move |r| Reply::History(n, r));
    }

    /// The whole part with its models, for the details (the search sends parts without
    /// them); the status line stays.
    pub fn detail(&mut self, ctx: &egui::Context, number: &str) {
        let call = self.connection().part(number);
        let n = number.to_string();
        let status = self.status.clone();
        self.send(ctx, Ok(call), status, move |r| Reply::Detail(n, r.map(|p| p.map(Box::new))));
    }

    /// The accessories of a part and the parts it fits, for the details; the status
    /// line stays.
    pub fn accessories(&mut self, ctx: &egui::Context, number: &str) {
        let call = self.connection().accessories(number);
        let n = number.to_string();
        let status = self.status.clone();
        self.send(ctx, Ok(call), status, move |r| Reply::Accessories(n, r));
    }

    /// Stores a file with a part.
    pub fn attach(&mut self, ctx: &egui::Context, number: &str, name: &str, kind: Option<AttachmentKind>, data: Vec<u8>) {
        let call = self.server(|e| e.attach(number, name, kind, data), decode);
        self.send(ctx, call, tr_fmt!("Uploading {}…", name), |r| Reply::Saved(r.map(Box::new)));
    }

    pub fn detach(&mut self, ctx: &egui::Context, number: &str, id: &str) {
        let call = self.server(|e| e.detach(number, id), decode);
        self.send(ctx, call, tr("Removing the file…").into(), |r| Reply::Saved(r.map(Box::new)));
    }

    pub fn download_file(&mut self, ctx: &egui::Context, id: &str, name: &str, purpose: FileUse) {
        let call = self.connection().file(id);
        let n = name.to_string();
        self.send(ctx, Ok(call), tr_fmt!("Downloading {}…", name), move |r| Reply::File(n, purpose, r));
    }

    pub fn catalogs(&mut self, ctx: &egui::Context) {
        let call = self.server(Endpoint::catalogs, decode);
        self.send(ctx, call, tr("Loading catalogs…").into(), Reply::Catalogs);
    }

    pub fn save_catalog(&mut self, ctx: &egui::Context, catalog: &Catalog) {
        let call = self.server(|e| e.save_catalog(catalog), decode);
        self.send(ctx, call, tr_fmt!("Saving {}…", catalog.name), Reply::CatalogSaved);
    }

    pub fn delete_catalog(&mut self, ctx: &egui::Context, id: &str) {
        let call = self.server(|e| e.delete_catalog(id), |r| check(r).map(|_| ()));
        let id = id.to_string();
        self.send(ctx, call, tr_fmt!("Deleting {}…", id), move |r| Reply::CatalogDeleted(id, r));
    }

    /// Signs in; the session's token replaces [`ServerClient::token`].
    pub fn sign_in(&mut self, ctx: &egui::Context, name: &str, password: &str) {
        let call = self.server(|e| e.login(name, password), decode);
        self.send(ctx, call, tr("Signing in…").into(), Reply::SignedIn);
    }

    pub fn sign_out(&mut self, ctx: &egui::Context) {
        let call = self.server(Endpoint::logout, |_| Ok(()));
        self.send(ctx, call, tr("Signing out…").into(), |_| Reply::SignedOut);
    }

    /// Loads a library (JSON) shared by all projects, e.g. the company's forms.
    pub fn library(&mut self, ctx: &egui::Context, name: &str) {
        let call = self.server(|e| e.library(name), |r| decode_found::<serde_json::Value>(r).map(|v| v.map(|v| v.to_string())));
        let n = name.to_string();
        self.send(ctx, call, tr_fmt!("Loading {}…", name), move |r| Reply::Library(n, r));
    }

    pub fn save_library(&mut self, ctx: &egui::Context, name: &str, json: &str) {
        let call = self.server(|e| e.save_library(name, json), |r| check(r).map(|_| ()));
        let n = name.to_string();
        self.send(ctx, call, tr_fmt!("Saving {}…", name), move |r| Reply::LibrarySaved(n, r));
    }

    pub fn favorites(&mut self, ctx: &egui::Context) {
        let call = self.server(Endpoint::favorites, decode);
        self.send(ctx, call, tr("Loading favorites…").into(), Reply::Favorites);
    }

    pub fn set_favorites(&mut self, ctx: &egui::Context, numbers: &[String]) {
        let call = self.server(|e| e.set_favorites(numbers), decode);
        self.send(ctx, call, tr("Saving favorites…").into(), Reply::Favorites);
    }

    pub fn users(&mut self, ctx: &egui::Context) {
        let call = self.server(Endpoint::users, decode);
        self.send(ctx, call, tr("Loading users…").into(), Reply::Users);
    }

    pub fn save_user(&mut self, ctx: &egui::Context, name: &str, update: &UserUpdate) {
        let call = self.server(|e| e.save_user(name, update), decode);
        self.send(ctx, call, tr_fmt!("Saving {}…", name), Reply::UserSaved);
    }

    pub fn delete_user(&mut self, ctx: &egui::Context, name: &str) {
        let call = self.server(|e| e.delete_user(name), |r| check(r).map(|_| ()));
        let n = name.to_string();
        self.send(ctx, call, tr_fmt!("Deleting {}…", name), move |r| Reply::UserDeleted(n, r));
    }

    /// Takes the finished requests and updates `status`, `info` and the token.
    pub fn poll(&mut self) -> Vec<Reply> {
        let replies: Vec<Reply> = self.inbox.lock().map(|mut i| std::mem::take(&mut *i)).unwrap_or_default();
        for reply in &replies {
            self.pending = self.pending.saturating_sub(1);
            let found = match reply {
                Reply::Search(Ok(r)) | Reply::Versions(Ok(r)) => r.parts.as_slice(),
                Reply::Parts(Ok(list)) => list.parts.as_slice(),
                _ => &[],
            };
            for p in found.iter().filter(|p| !p.portal_id.is_empty()) {
                self.portal_ids.insert(p.number.clone(), p.portal_id.clone());
            }
            match reply {
                Reply::Info(Ok(info)) => self.info = Some(info.clone()),
                Reply::Info(Err(_)) => self.info = None,
                Reply::SignedIn(Ok(session)) => {
                    self.token = session.token.clone();
                    if let Some(info) = &mut self.info {
                        info.user = Some(session.user.clone());
                    }
                }
                Reply::SignedOut => {
                    self.token.clear();
                    if let Some(info) = &mut self.info {
                        info.user = None;
                    }
                }
                _ => {}
            }
            if !matches!(reply, Reply::Detail(_, Ok(_))) {
                self.status = reply.describe();
                self.failed = reply.error().is_some();
            }
        }
        replies
    }

    /// True if the status line shows an error.
    pub fn failed(&self) -> bool {
        self.failed
    }
}

#[cfg(test)]
mod tests;
