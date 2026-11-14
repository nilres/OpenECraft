//! HTTP routes of the project server (see `openecraft_core::collab` for the API).

use crate::sessions::{LockError, Sessions};
use crate::store::Store;
use crate::users::{Account, Users};
use crate::views::{Viewed, Views};
use axum::extract::{DefaultBodyLimit, Path, Query, Request, State};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use openecraft_auth::admin::{self, DirectoryEntry, DirectoryTestRequest};
use openecraft_auth::{AccessFile, ClientIp, Directory, SignInError, Throttle};
use openecraft_core::collab::{
    changes, merge, save_plan, valid_name, Access, CreateRequest, CurrentAccount, Grant, Lock, LockRequest, LoginRequest, LoginResponse, OpenRequest, Opened, Owned, Policy, ProjectAccess,
    ProjectRight, ProjectRole, ProjectSummary, ProjectUser, Right, SaveRequest, ServerInfo, Session, SessionList, Settings, Snapshot, Status, Subject,
    Target, UserUpdate, API_PREFIX, SERVER,
};
use openecraft_core::model::Project;
use openecraft_parts::api::{bearer_token, token_matches, ApiError, DirectoryTest};
use openecraft_view::ProjectView;
use openecraft_web::TextQuery;
use serde::Serialize;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};

pub struct Server {
    pub store: Store,
    pub sessions: Sessions,
    pub users: Users,
    pub access: AccessFile<ProjectRight>,
    /// Failed sign-ins per user name.
    pub throttle: Throttle,
    /// The projects viewed on the web page.
    pub views: Views,
}

impl Server {
    /// Without token, accounts and directories everyone may do everything.
    fn open_access(&self, token: &Option<String>) -> bool {
        token.is_none() && self.users.accounts.is_empty() && self.access.config.directories.is_empty()
    }

    /// True if a change (the users `after` under `policy`) would leave no one who
    /// manages users where someone does now (without server token, nobody else could).
    fn loses_admin(&self, token: &Option<String>, policy: &Policy<ProjectRight>, after: &[ProjectUser]) -> bool {
        let admin = |policy: &Policy<ProjectRight>, u: &ProjectUser| policy.access(&u.identity(), &u.grants()).with(policy.anonymous_access()).allows(SERVER, ProjectRight::Administer);
        let now = &self.access.config.policy;
        token.is_none() && self.users.accounts.iter().any(|a| admin(now, &a.user)) && !after.iter().any(|u| admin(policy, u))
    }

    fn user_list(&self) -> Vec<ProjectUser> {
        self.users.accounts.iter().map(|a| a.user.clone()).collect()
    }

    /// What a user may do: their role, the grants to them and their groups and what
    /// anonymous requests may do.
    fn access_of(&self, user: &ProjectUser) -> Access<ProjectRight> {
        let policy = &self.access.config.policy;
        policy.access(&user.identity(), &user.grants()).with(policy.anonymous_access())
    }
}

/// Who makes a request and what they may do.
#[derive(Clone, Debug, PartialEq)]
pub struct Caller {
    /// A signed-in user; `None` for the server token, open servers and anonymous requests.
    user: Option<String>,
    access: Access<ProjectRight>,
    /// How the caller came in.
    kind: CallerKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CallerKind {
    /// The server token.
    Token,
    /// A server without token, accounts and directories.
    Open,
    User,
    /// No token, with the rights of anonymous requests.
    Anonymous,
}

impl Caller {
    fn rights(&self, project: &str) -> BTreeSet<ProjectRight> {
        self.access.rights(project)
    }

    fn may(&self, project: &str, right: ProjectRight) -> bool {
        self.access.allows(project, right)
    }

    /// The user name of a request: a signed-in user's own name, otherwise the one sent;
    /// anonymous callers cannot take the name of an account.
    fn name(&self, sent: &str, users: &Users) -> Result<String> {
        match &self.user {
            Some(u) => Ok(u.clone()),
            None => {
                let name = user(sent)?;
                if self.kind == CallerKind::Anonymous && users.accounts.iter().any(|a| a.user.name.eq_ignore_ascii_case(&name)) {
                    return Err(AppError(StatusCode::FORBIDDEN, format!("sign in to work as '{name}'")));
                }
                Ok(name)
            }
        }
    }

    /// A signed-in user may only use their own sessions.
    fn owns(&self, session: &Session) -> Result<()> {
        match &self.user {
            Some(u) if !u.eq_ignore_ascii_case(&session.user) => Err(AppError(StatusCode::FORBIDDEN, "the session belongs to another user".into())),
            _ => Ok(()),
        }
    }

    /// Projects the caller may not view do not exist for them.
    fn see(&self, project: &str) -> Result<()> {
        if self.may(project, ProjectRight::View) {
            Ok(())
        } else {
            Err(not_found(project))
        }
    }

    fn need(&self, project: &str, right: ProjectRight) -> Result<()> {
        self.see(project)?;
        if self.may(project, right) {
            Ok(())
        } else {
            Err(AppError(StatusCode::FORBIDDEN, format!("you lack the right '{}' on project '{project}'", right.name())))
        }
    }

    /// Users and access are managed by administrators; on an open server only from the
    /// machine it runs on (`client`), so that nobody else takes it over by creating the
    /// first administrator.
    fn admin(&self, client: Option<std::net::IpAddr>) -> Result<()> {
        if self.kind == CallerKind::Open && client.is_some_and(|c| !c.is_loopback()) {
            return Err(AppError(StatusCode::FORBIDDEN, "a server without access token or user accounts manages its users on its own machine only (or start it with --admin-user)".into()));
        }
        if self.may(SERVER, ProjectRight::Administer) {
            Ok(())
        } else {
            Err(AppError(StatusCode::FORBIDDEN, "only administrators manage users and access".into()))
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub server: Arc<Mutex<Server>>,
    /// Bearer token required for every request. `None` = open server.
    pub token: Option<String>,
    /// The web page is served (`--web`).
    pub web: bool,
}

pub struct AppError(StatusCode, String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, Json(ApiError { error: self.1 })).into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        log::error!("storage error: {e}");
        AppError(StatusCode::INTERNAL_SERVER_ERROR, format!("storage error: {e}"))
    }
}

impl From<LockError> for AppError {
    fn from(e: LockError) -> Self {
        match e {
            LockError::Expired => AppError(StatusCode::GONE, "the session has expired, open the project again".into()),
            LockError::Held(l) => AppError(StatusCode::CONFLICT, format!("{} is locked by {}", describe(l.target, None), l.user)),
        }
    }
}

/// A lock error naming the page as the users see it.
fn lock_error(e: LockError, project: Option<&Project>) -> AppError {
    match e {
        LockError::Held(l) => AppError(StatusCode::CONFLICT, format!("{} is locked by {}", describe(l.target, project), l.user)),
        e => e.into(),
    }
}

type Result<T> = std::result::Result<T, AppError>;

/// The page by its name (its id if the project is not at hand or has no such page).
fn describe(target: Target, project: Option<&Project>) -> String {
    match target {
        Target::Project => "the project data".into(),
        Target::Page(id) => match project.and_then(|p| p.pages.iter().find(|p| p.id == id)) {
            Some(page) => format!("page {}", page.name),
            None => format!("page {id}"),
        },
    }
}

/// Locks as other users may see them: the session id of a lock is the key to it, so
/// only the caller's own sessions keep theirs (the others keep just their number).
fn redact(mut locks: Vec<Lock>, own: Option<&str>) -> Vec<Lock> {
    for l in &mut locks {
        if Some(l.session.as_str()) != own {
            l.session = l.session.split('-').next().unwrap_or_default().to_string();
        }
    }
    locks
}

fn bad(msg: impl Into<String>) -> AppError {
    AppError(StatusCode::BAD_REQUEST, msg.into())
}

pub fn router(state: AppState) -> Router {
    let project = format!("{API_PREFIX}/projects/{{name}}");
    let mut app = Router::new()
        .route("/", if state.web { get(crate::web::index) } else { get(index) })
        .route(&format!("{API_PREFIX}/info"), get(info))
        .route(&format!("{API_PREFIX}/projects"), get(list).post(create))
        .route(&project, get(snapshot).put(save).delete(delete_project))
        .route(&format!("{project}/open"), post(open))
        .route(&format!("{project}/snapshot"), post(session_snapshot))
        .route(&format!("{project}/status"), post(status))
        .route(&format!("{project}/lock"), post(lock))
        .route(&format!("{project}/release"), post(release))
        .route(&format!("{project}/close"), post(close))
        .route(&format!("{project}/access"), get(project_access).put(save_project_access))
        .route(&format!("{API_PREFIX}/login"), post(login).layer(DefaultBodyLimit::max(SMALL_BODY)))
        .route(&format!("{API_PREFIX}/logout"), post(logout))
        .route(&format!("{API_PREFIX}/session"), get(current_account))
        .route(&format!("{API_PREFIX}/users"), get(list_users))
        .route(&format!("{API_PREFIX}/users/{{name}}"), put(save_user).delete(delete_user))
        .route(&format!("{API_PREFIX}/access"), get(access).put(save_access))
        .route(&format!("{API_PREFIX}/sessions"), get(sessions))
        .route(&format!("{API_PREFIX}/directories"), get(directories))
        .route(&format!("{API_PREFIX}/directories/{{id}}"), put(put_directory).delete(delete_directory))
        .route(&format!("{API_PREFIX}/directories/{{id}}/test"), post(test_directory));
    if state.web {
        app = app
            .merge(crate::web::routes())
            .route(&format!("{API_PREFIX}/texts"), get(crate::web::texts))
            .route(&format!("{project}/view"), get(view))
            .route(&format!("{project}/pages/{{id}}/svg"), get(page_svg));
    }
    app.layer(DefaultBodyLimit::max(256 * 1024 * 1024))
        .layer(axum::middleware::from_fn_with_state(state.clone(), known_callers))
        .layer(axum::middleware::map_response(openecraft_auth::json_errors))
        .layer(axum::middleware::map_response(openecraft_auth::security_headers))
        .layer(cors())
        .with_state(state)
}

/// Largest body of a sign-in.
const SMALL_BODY: usize = 64 * 1024;

/// Requests with a body need a known caller, checked before the body is read: nobody
/// else makes the server read and parse large bodies.
async fn known_callers(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let open = [Method::GET, Method::HEAD, Method::OPTIONS].contains(req.method())
        || ["/login", "/logout"].iter().any(|p| req.uri().path() == format!("{API_PREFIX}{p}"));
    if !open {
        if let Err(e) = server(&state, req.headers()) {
            return e.into_response();
        }
    }
    next.run(req).await
}

/// Any origin may call the API (e.g. the web app): authentication uses bearer
/// tokens, not cookies.
fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::PUT, Method::POST, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

fn presented(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(bearer_token).filter(|t| !t.is_empty())
}

fn lock_server(state: &AppState) -> MutexGuard<'_, Server> {
    state.server.lock().unwrap_or_else(|e| e.into_inner())
}

/// Checks the token (the server token or the session of a user) and returns the
/// server with expired sessions removed and who is calling.
fn server<'a>(state: &'a AppState, headers: &HeaderMap) -> Result<(MutexGuard<'a, Server>, Caller)> {
    let mut server = lock_server(state);
    server.sessions.expire(Instant::now());
    let token = presented(headers);
    let caller = if token.is_some_and(|p| state.token.as_ref().is_some_and(|t| token_matches(p, t))) {
        Caller { user: None, access: Access::full(), kind: CallerKind::Token }
    } else if server.open_access(&state.token) {
        Caller { user: None, access: Access::full(), kind: CallerKind::Open }
    } else if let Some(a) = token.and_then(|t| server.users.by_token(t)) {
        Caller { user: Some(a.user.name.clone()), access: server.access_of(&a.user), kind: CallerKind::User }
    } else if token.is_none() && !server.access.config.policy.anonymous_access().is_empty() {
        Caller { user: None, access: server.access.config.policy.anonymous_access(), kind: CallerKind::Anonymous }
    } else {
        return Err(AppError(StatusCode::UNAUTHORIZED, "missing or wrong access token".into()));
    };
    Ok((server, caller))
}

/// An administrator by the built-in role.
fn not_found(name: &str) -> AppError {
    AppError(StatusCode::NOT_FOUND, format!("project '{name}' not found"))
}

/// Longest user name the server accepts.
const MAX_USER_LEN: usize = 64;

fn user(name: &str) -> Result<String> {
    let user = name.trim();
    if user.is_empty() {
        return Err(bad("a user name is required"));
    }
    if user.chars().any(char::is_control) || user.chars().count() > MAX_USER_LEN {
        return Err(bad(format!("'{}' cannot be used as a user name", user.escape_debug())));
    }
    Ok(user.to_string())
}

/// The stored name of a project whose name differs from `name` at most in case
/// (`name` itself if there is none).
fn canonical(server: &Server, name: String) -> String {
    server.store.taken(&name).map(String::from).unwrap_or(name)
}

/// Hash of the sign-in token of a request: project sessions opened with it end
/// when the user signs out.
fn owner(headers: &HeaderMap, caller: &Caller) -> Option<String> {
    caller.user.as_ref().and(presented(headers)).map(|t| openecraft_auth::sha256_hex(t.as_bytes()))
}

impl Server {
    fn snapshot(&self, name: &str, own: Option<&str>) -> Result<Snapshot> {
        let entry = self.store.get(name).ok_or_else(|| not_found(name))?;
        Ok(Snapshot { revision: entry.revision, project: entry.project.clone(), locks: redact(self.sessions.locks(name), own), ignored: Vec::new() })
    }

    fn status(&self, name: &str, own: Option<&str>) -> Result<Status> {
        let entry = self.store.get(name).ok_or_else(|| not_found(name))?;
        Ok(Status { revision: entry.revision, locks: redact(self.sessions.locks(name), own) })
    }

    fn exists(&self, name: &str) -> Result<()> {
        self.store.get(name).map(|_| ()).ok_or_else(|| not_found(name))
    }

    fn open(&mut self, name: &str, user: &str, rights: BTreeSet<ProjectRight>, owner: Option<String>) -> Result<Opened> {
        if self.store.get(name).is_none() {
            return Err(not_found(name));
        }
        let first_id = self.store.reserve_ids(name)?;
        let session = self.sessions.open(name, user, Instant::now());
        if let Some(owner) = owner {
            self.sessions.set_owner(&session.session, owner);
        }
        log::info!("{user} opened {name}");
        let snapshot = self.snapshot(name, Some(&session.session))?;
        Ok(Opened { session, first_id, snapshot, rights: rights.into_iter().collect() })
    }

    /// The grants on a project and the roles `caller` may give.
    fn project_access(&self, name: &str, caller: &Caller) -> ProjectAccess {
        let policy = &self.access.config.policy;
        let roles = policy.role_names().into_iter().filter(|r| policy.rights_of(r).is_some_and(|rights| rights.iter().all(|x| caller.may(name, *x)))).collect();
        ProjectAccess { grants: policy.grants_on(name), roles }
    }
}

async fn index(State(state): State<AppState>) -> String {
    let server = lock_server(&state);
    format!(
        "OpenECraft project server {}\n\n{} projects, {}\n\nAPI: {API_PREFIX}/info, {API_PREFIX}/projects, {API_PREFIX}/projects/{{name}}\nWeb page: start the server with --web\n",
        env!("CARGO_PKG_VERSION"),
        server.store.projects.len(),
        if server.open_access(&state.token) { "open access" } else { "access with token or user account" }
    )
}

/// Name and version for everybody (like the parts server); the projects the caller
/// may see, who they are and what they may do.
async fn info(State(state): State<AppState>, headers: HeaderMap) -> Json<ServerInfo> {
    let mut info = ServerInfo { name: "OpenECraft project server".into(), version: env!("CARGO_PKG_VERSION").into(), web: state.web, ..Default::default() };
    if let Ok((server, caller)) = server(&state, &headers) {
        info.projects = server.store.projects.keys().filter(|name| caller.may(name, ProjectRight::View)).count();
        info.user = match (&caller.user, caller.kind) {
            (Some(name), _) => server.users.accounts.iter().find(|a| &a.user.name == name).map(|a| a.user.clone()),
            (None, CallerKind::Token) => Some(ProjectUser { name: TOKEN_USER.into(), role: Some(ProjectRole::Admin), ..Default::default() }),
            _ => None,
        };
        info.access = caller.access;
    }
    Json(info)
}

/// The name the server token signs in with on the web page.
const TOKEN_USER: &str = "admin";

async fn list(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Vec<ProjectSummary>>> {
    let (server, caller) = server(&state, &headers)?;
    let list = server
        .store
        .projects
        .iter()
        .filter(|(name, _)| caller.may(name, ProjectRight::View))
        .map(|(name, e)| ProjectSummary {
            name: name.clone(),
            title: e.project.info.name.clone(),
            pages: e.project.pages.len(),
            revision: e.revision,
            users: server.sessions.users(name),
            rights: caller.rights(name).into_iter().collect(),
        })
        .collect();
    Ok(Json(list))
}

async fn create(State(state): State<AppState>, headers: HeaderMap, Json(body): Json<serde_json::Value>) -> Result<(StatusCode, Json<Opened>)> {
    if body.get("project").is_none_or(serde_json::Value::is_null) {
        return Err(bad("the request needs the project"));
    }
    let req: CreateRequest = serde_json::from_value(body).map_err(|e| AppError(StatusCode::UNPROCESSABLE_ENTITY, format!("invalid request: {e}")))?;
    let (mut server, mut caller) = server(&state, &headers)?;
    let user = caller.name(&req.user, &server.users)?;
    if !caller.may(&req.name, ProjectRight::Create) {
        return Err(AppError(StatusCode::FORBIDDEN, format!("you may not create project '{}'", req.name)));
    }
    if !valid_name(&req.name) {
        return Err(bad(format!("'{}' is not a valid project name", req.name)));
    }
    if let Some(existing) = server.store.taken(&req.name) {
        return Err(AppError(StatusCode::CONFLICT, format!("project '{existing}' already exists")));
    }
    server.store.create(&req.name, req.project)?;
    log::info!("{user} created {}", req.name);
    // A signed-in user manages the projects they create.
    if let Some(owner) = caller.user.clone().filter(|_| !caller.may(&req.name, ProjectRight::ManageAccess)) {
        let mut policy = server.access.config.policy.clone();
        let owner_grant = Grant::new(Subject::User(owner), ProjectRole::Manager.name(), &req.name);
        policy.grants.push(owner_grant.clone());
        server.access.set_policy(policy).map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
        caller.access = caller.access.with(Policy::default().access(&Default::default(), &[owner_grant]));
    }
    let rights = caller.rights(&req.name);
    let owner = owner(&headers, &caller);
    Ok((StatusCode::CREATED, Json(server.open(&req.name, &user, rights, owner)?)))
}

async fn delete_project(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>) -> Result<StatusCode> {
    let (mut server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.need(&name, ProjectRight::Delete)?;
    server.exists(&name)?;
    let users = server.sessions.users(&name);
    if !users.is_empty() {
        return Err(AppError(StatusCode::CONFLICT, format!("project '{name}' is open ({})", users.join(", "))));
    }
    server.store.delete(&name)?;
    server.views.remove(&name);
    let mut policy = server.access.config.policy.clone();
    if !policy.grants_on(&name).is_empty() {
        policy.set_grants_on(&name, Vec::new());
        server.access.set_policy(policy).map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }
    log::info!("{} deleted {name}", caller.user.as_deref().unwrap_or("token"));
    Ok(StatusCode::NO_CONTENT)
}

async fn snapshot(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>) -> Result<Json<Snapshot>> {
    let (server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.see(&name)?;
    Ok(Json(server.snapshot(&name, None)?))
}

async fn open(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>, Json(req): Json<OpenRequest>) -> Result<Json<Opened>> {
    let (mut server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.see(&name)?;
    let user = caller.name(&req.user, &server.users)?;
    let owner = owner(&headers, &caller);
    Ok(Json(server.open(&name, &user, caller.rights(&name), owner)?))
}

/// Merges what the session has locked (and the review records) into the stored project.
/// With the base revision of the request, free targets it changed are locked for it
/// first; the changes left out are listed in the answer (`ignored`).
async fn save(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>, Json(mut req): Json<SaveRequest>) -> Result<Json<Snapshot>> {
    let (mut server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.see(&name)?;
    caller.owns(&req.session)?;
    let rights = caller.rights(&name);
    if ![ProjectRight::Review, ProjectRight::EditPages, ProjectRight::EditProject].iter().any(|r| rights.contains(r)) {
        return Err(AppError(StatusCode::FORBIDDEN, format!("you may only view project '{name}'")));
    }
    server.sessions.touch(&name, &req.session, Instant::now())?;
    let stored = server.store.get(&name).ok_or_else(|| not_found(&name))?.project.clone();
    if let Some(user) = &caller.user {
        openecraft_core::review::sign(&stored.review, &mut req.project.review, user);
    }
    let mut ignored = Vec::new();
    match server.store.at(&name, req.base_revision).filter(|_| req.base_revision > 0).cloned() {
        Some(base) => {
            let (mine, theirs) = (changes(&base, &req.project), changes(&base, &stored));
            let may = |t: Target| caller.may(&name, if t == Target::Project { ProjectRight::EditProject } else { ProjectRight::EditPages });
            let plan = save_plan(&mine, &theirs, &server.sessions.locks(&name), &req.session.session, may);
            if req.strict && !plan.ignored.is_empty() {
                let what: Vec<String> = plan.ignored.iter().map(|t| describe(*t, Some(&stored))).collect();
                return Err(AppError(StatusCode::CONFLICT, format!("not saved: {} cannot be taken over (locked or changed by another user)", what.join(", "))));
            }
            if !plan.lock.is_empty() {
                server.sessions.lock(&name, &req.session, &plan.lock, Instant::now()).map_err(|e| lock_error(e, Some(&stored)))?;
            }
            ignored = plan.ignored;
        }
        None if req.strict && req.base_revision > 0 => {
            return Err(AppError(StatusCode::CONFLICT, format!("revision {} is no longer kept: load the project again", req.base_revision)));
        }
        None => {}
    }
    let locks = server.sessions.locks(&name);
    let owned = Owned::of(&locks, &req.session.session);
    if owned.project {
        caller.need(&name, ProjectRight::EditProject)?;
        let removed = locks.iter().find(|l| match l.target {
            Target::Page(id) => l.session != req.session.session && !req.project.pages.iter().any(|p| p.id == id),
            Target::Project => false,
        });
        if let Some(l) = removed {
            return Err(AppError(StatusCode::CONFLICT, format!("{} cannot be deleted, it is locked by {}", describe(l.target, Some(&stored)), l.user)));
        }
    }
    if !owned.pages.is_empty() {
        caller.need(&name, ProjectRight::EditPages)?;
    }
    let merged = merge(&stored, &req.project, &owned);
    let revision = server.store.save(&name, merged)?;
    server.sessions.saved(&req.session.session);
    log::info!("{} saved {name} (revision {revision})", req.session.user);
    server.sessions.release(&name, &req.session, &req.release, Instant::now())?;
    if !ignored.is_empty() {
        log::info!("{} saved {name} without {} changed targets", req.session.user, ignored.len());
    }
    let mut snapshot = server.snapshot(&name, Some(&req.session.session))?;
    snapshot.ignored = ignored;
    Ok(Json(snapshot))
}

async fn status(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>, Json(session): Json<Session>) -> Result<Json<Status>> {
    let (mut server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.see(&name)?;
    caller.owns(&session)?;
    server.exists(&name)?;
    server.sessions.touch(&name, &session, Instant::now())?;
    Ok(Json(server.status(&name, Some(&session.session))?))
}

async fn session_snapshot(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>, Json(session): Json<Session>) -> Result<Json<Snapshot>> {
    let (mut server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.see(&name)?;
    caller.owns(&session)?;
    server.exists(&name)?;
    server.sessions.touch(&name, &session, Instant::now())?;
    Ok(Json(server.snapshot(&name, Some(&session.session))?))
}

async fn lock(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>, Json(req): Json<LockRequest>) -> Result<Json<Status>> {
    let (mut server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.see(&name)?;
    caller.owns(&req.session)?;
    for t in &req.targets {
        caller.need(&name, if *t == Target::Project { ProjectRight::EditProject } else { ProjectRight::EditPages })?;
    }
    let entry = server.store.get(&name).ok_or_else(|| not_found(&name))?;
    if let Some(missing) = req.targets.iter().find(|t| match t {
        Target::Page(id) => !entry.project.pages.iter().any(|p| p.id == *id),
        Target::Project => false,
    }) {
        return Err(AppError(StatusCode::NOT_FOUND, format!("{} does not exist on the server", describe(*missing, None))));
    }
    let project = entry.project.clone();
    server.sessions.lock(&name, &req.session, &req.targets, Instant::now()).map_err(|e| lock_error(e, Some(&project)))?;
    Ok(Json(server.status(&name, Some(&req.session.session))?))
}

async fn release(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>, Json(req): Json<LockRequest>) -> Result<Json<Status>> {
    let (mut server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.see(&name)?;
    caller.owns(&req.session)?;
    server.exists(&name)?;
    server.sessions.release(&name, &req.session, &req.targets, Instant::now())?;
    Ok(Json(server.status(&name, Some(&req.session.session))?))
}

async fn close(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>, Json(session): Json<Session>) -> Result<Json<Status>> {
    let (mut server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.see(&name)?;
    caller.owns(&session)?;
    server.exists(&name)?;
    server.sessions.close(&name, &session)?;
    log::info!("{} closed {name}", session.user);
    Ok(Json(server.status(&name, None)?))
}

/// A project as the web page shows it: the view with the locks and the users working on
/// it, and what the caller may do.
#[derive(Debug, Serialize)]
struct ViewAnswer {
    name: String,
    /// Revision on the server.
    server_revision: u64,
    locks: Vec<Lock>,
    users: Vec<String>,
    rights: Vec<ProjectRight>,
    #[serde(flatten)]
    view: ProjectView,
}

/// The view of the project `name` (kept per revision) with what the answer of the web page
/// shows besides the view.
async fn viewed(state: &AppState, headers: &HeaderMap, name: String) -> Result<(Arc<Viewed>, ViewAnswer)> {
    let (project, answer) = {
        let (mut server, caller) = server(state, headers)?;
        let name = canonical(&server, name);
        caller.see(&name)?;
        let revision = server.store.get(&name).ok_or_else(|| not_found(&name))?.revision;
        let answer = ViewAnswer {
            server_revision: revision,
            locks: redact(server.sessions.locks(&name), None),
            users: server.sessions.users(&name),
            rights: caller.rights(&name).into_iter().collect(),
            name,
            view: ProjectView::default(),
        };
        if let Some(v) = server.views.get(&answer.name, revision) {
            return Ok((v, answer));
        }
        (server.store.get(&answer.name).ok_or_else(|| not_found(&answer.name))?.project.clone(), answer)
    };
    let revision = answer.server_revision;
    let viewed = tokio::task::spawn_blocking(move || Arc::new(Viewed::new(project, revision))).await.map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    lock_server(state).views.put(&answer.name, viewed.clone());
    Ok((viewed, answer))
}

/// `GET /api/v1/projects/{name}/view?lang=de`: pages, sections, structures and devices
/// for the web page (read only).
async fn view(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>, Query(q): Query<TextQuery>) -> Result<Json<ViewAnswer>> {
    let (viewed, mut answer) = viewed(&state, &headers, name).await?;
    answer.view = viewed.view(q.lang());
    Ok(Json(answer))
}

/// `GET /api/v1/projects/{name}/pages/{id}/svg`: a page drawn as on paper.
async fn page_svg(State(state): State<AppState>, headers: HeaderMap, Path((name, id)): Path<(String, u64)>) -> Result<Response> {
    let (viewed, _) = viewed(&state, &headers, name).await?;
    let svg = tokio::task::spawn_blocking(move || viewed.page_svg(id)).await.map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let svg = svg.ok_or_else(|| AppError(StatusCode::NOT_FOUND, format!("page {id} not found")))?;
    Ok(([(header::CONTENT_TYPE, "image/svg+xml"), (header::CACHE_CONTROL, "private, no-cache")], svg.to_string()).into_response())
}

async fn project_access(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>) -> Result<Json<ProjectAccess>> {
    let (server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.need(&name, ProjectRight::ManageAccess)?;
    server.exists(&name)?;
    Ok(Json(server.project_access(&name, &caller)))
}

/// Replaces the grants on the project. Nobody gives rights they do not have.
async fn save_project_access(State(state): State<AppState>, headers: HeaderMap, Path(name): Path<String>, Json(grants): Json<Vec<Grant>>) -> Result<Json<ProjectAccess>> {
    let (mut server, caller) = server(&state, &headers)?;
    let name = canonical(&server, name);
    caller.need(&name, ProjectRight::ManageAccess)?;
    server.exists(&name)?;
    let allowed = server.project_access(&name, &caller).roles;
    let old = server.access.config.policy.grants_on(&name);
    for g in grants.iter().chain(&old) {
        if !allowed.iter().any(|r| r.eq_ignore_ascii_case(g.role.trim())) {
            return Err(AppError(StatusCode::FORBIDDEN, format!("you may not give or take the role '{}'", g.role.trim())));
        }
    }
    // Managers of the project may deny more, but only administrators lift denials.
    if !caller.may(SERVER, ProjectRight::Administer) && !denials(&old, &name).is_subset(&denials(&grants, &name)) {
        return Err(AppError(StatusCode::FORBIDDEN, "only administrators lift denials".into()));
    }
    let mut policy = server.access.config.policy.clone();
    policy.set_grants_on(&name, grants);
    server.access.set_policy(policy).map_err(bad)?;
    log::info!("access to {name} changed by {}", caller.user.as_deref().unwrap_or("token"));
    Ok(Json(server.project_access(&name, &caller)))
}

/// The denials among grants on the project `name`, comparable.
fn denials(grants: &[Grant], name: &str) -> BTreeSet<(Subject, String)> {
    grants.iter().filter(|g| g.deny).map(|g| (g.subject.clone(), format!("{}\n{}", g.role.trim().to_lowercase(), Grant { scope: name.into(), ..g.clone() }.scope()))).collect()
}

/// Checks a password (Argon2) without holding the server or blocking other requests.
async fn password_matches(password: &str, hash: Option<&str>) -> bool {
    let (password, hash) = (password.to_string(), hash.map(String::from));
    tokio::task::spawn_blocking(move || openecraft_auth::verify_account_password(&password, hash.as_deref())).await.unwrap_or(false)
}

/// Signs in with a local account or, if the server has none of that name, with its
/// directories; a directory user gets an account (role, groups, sessions) on the
/// first sign-in.
async fn login(State(state): State<AppState>, ClientIp(client): ClientIp, Json(req): Json<LoginRequest>) -> Result<Json<LoginResponse>> {
    if let Some(wait) = lock_server(&state).throttle.wait(&req.user, client, Instant::now()) {
        return Err(AppError(StatusCode::TOO_MANY_REQUESTS, format!("too many failed sign-ins, try again in {wait} s")));
    }
    let refused = || {
        lock_server(&state).throttle.failed(&req.user, client, Instant::now());
        log::warn!("failed sign-in of '{}'", req.user.trim());
        AppError(StatusCode::UNAUTHORIZED, "wrong user name or password".into())
    };
    let (local, directories) = {
        let mut server = lock_server(&state);
        let local = server.users.find_mut(&req.user).filter(|a| req.provider.trim().is_empty() && a.user.provider.is_empty()).map(|a| a.password.clone());
        (local, server.access.config.directories.clone())
    };
    if local.is_some() || directories.is_empty() {
        // Without an account a password is checked all the same, so the answer does not
        // tell which names exist.
        if !password_matches(&req.password, local.as_deref()).await {
            return Err(refused());
        }
        let mut server = lock_server(&state);
        // The password may have changed meanwhile.
        let account = server.users.find_mut(&req.user).filter(|a| Some(&a.password) == local.as_ref()).ok_or_else(refused)?;
        let token = openecraft_auth::start_session(&mut account.sessions, openecraft_auth::unix_now());
        let user = account.user.clone();
        server.users.save()?;
        server.throttle.succeeded(&req.user, client);
        log::info!("{} signed in", user.name);
        let access = server.access_of(&user);
        return Ok(Json(LoginResponse { token, user, access }));
    }
    let identity = match openecraft_auth::sign_in(&directories, &req.provider, &req.user, &req.password).await {
        Ok(identity) => identity,
        Err(SignInError::Unavailable(e)) => return Err(AppError(StatusCode::SERVICE_UNAVAILABLE, e)),
        Err(_) => return Err(refused()),
    };
    let mut server = lock_server(&state);
    let account = match server.users.find_mut(&identity.name) {
        Some(a) if a.user.provider.eq_ignore_ascii_case(&identity.provider) => a,
        Some(_) => return Err(AppError(StatusCode::CONFLICT, format!("the name '{}' belongs to another account", identity.name))),
        None => {
            let user = ProjectUser { name: identity.name.clone(), provider: identity.provider.clone(), ..Default::default() };
            server.users.accounts.push(Account { user, password: String::new(), sessions: Vec::new() });
            server.users.accounts.last_mut().expect("just added")
        }
    };
    account.user.groups = identity.groups;
    let token = openecraft_auth::start_session(&mut account.sessions, openecraft_auth::unix_now());
    let user = account.user.clone();
    server.users.save()?;
    server.throttle.succeeded(&req.user, client);
    log::info!("{} signed in with {}", user.name, user.provider);
    let access = server.access_of(&user);
    Ok(Json(LoginResponse { token, user, access }))
}

/// Restores only a user's sign-in, never the server token or anonymous access.
/// Reading it neither creates a token nor extends its absolute lifetime.
async fn current_account(State(state): State<AppState>, headers: HeaderMap) -> Result<Response> {
    let server = lock_server(&state);
    let token = presented(&headers).filter(|t| !state.token.as_ref().is_some_and(|root| token_matches(t, root)));
    let user = token.and_then(|t| server.users.by_token(t)).map(|a| a.user.clone())
        .ok_or_else(|| AppError(StatusCode::UNAUTHORIZED, "a signed-in user session is required".into()))?;
    let access = server.access_of(&user);
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(CurrentAccount { user, access })).into_response())
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Result<StatusCode> {
    let Some(token) = presented(&headers).map(str::to_string) else { return Ok(StatusCode::NO_CONTENT) };
    let mut server = lock_server(&state);
    let ended = server.users.accounts.iter_mut().any(|a| openecraft_auth::end_session(&mut a.sessions, &token));
    if ended {
        server.users.save()?;
        // The projects opened with this sign-in are closed and their locks released.
        let closed = server.sessions.close_owned(&openecraft_auth::sha256_hex(token.as_bytes()));
        if closed > 0 {
            log::info!("{closed} project sessions closed on signing out");
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn list_users(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap) -> Result<Json<Vec<ProjectUser>>> {
    let (server, caller) = server(&state, &headers)?;
    caller.admin(client)?;
    Ok(Json(server.users.accounts.iter().map(|a| a.user.clone()).collect()))
}

async fn save_user(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap, Path(name): Path<String>, Json(req): Json<UserUpdate>) -> Result<Json<ProjectUser>> {
    let (mut server, caller) = server(&state, &headers)?;
    caller.admin(client)?;
    let name = user(&name)?;
    if name.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(bad(format!("'{name}' cannot be used as a user name")));
    }
    let hash = match req.password.as_deref().filter(|p| !p.is_empty()) {
        Some(p) => {
            openecraft_auth::check_password(p).map_err(bad)?;
            Some(openecraft_auth::hash_password(p).map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e))?)
        }
        None => None,
    };
    let groups = req.groups.map(|g| g.into_iter().map(|g| g.trim().to_string()).filter(|g| !g.is_empty()).collect::<Vec<_>>());
    // The account as it will be: administrators by role, grant or group count.
    let mut after = server.user_list();
    for u in after.iter_mut().filter(|u| u.name.eq_ignore_ascii_case(&name)) {
        u.role = req.role;
        if let Some(g) = groups.clone().filter(|_| u.provider.is_empty()) {
            u.groups = g;
        }
    }
    if server.loses_admin(&state.token, &server.access.config.policy, &after) {
        return Err(AppError(StatusCode::CONFLICT, "the last administrator cannot be demoted".into()));
    }
    let user = match server.users.find_mut(&name) {
        Some(a) => {
            if !a.user.provider.is_empty() && hash.is_some() {
                return Err(bad(format!("'{name}' signs in with directory {}, which keeps the password", a.user.provider)));
            }
            a.user.role = req.role;
            a.user.projects = req.projects;
            if let Some(g) = groups.filter(|_| a.user.provider.is_empty()) {
                a.user.groups = g;
            }
            if let Some(h) = hash {
                a.password = h;
                a.sessions.clear();
            }
            a.user.clone()
        }
        None => {
            let password = hash.ok_or_else(|| bad("a new user needs a password"))?;
            let granted = server.access.config.policy.grants.iter().any(|g| matches!(&g.subject, Subject::User(u) if u.eq_ignore_ascii_case(&name)));
            // Without role, groups or grants of their own the account could do nothing.
            if req.role.is_none() && groups.as_ref().is_none_or(Vec::is_empty) && !granted {
                return Err(bad("a new user needs a role (viewer, reviewer, editor, manager or admin), groups or grants in the access settings"));
            }
            let user = ProjectUser { name: name.clone(), role: req.role, projects: req.projects, groups: groups.unwrap_or_default(), ..Default::default() };
            server.users.accounts.push(Account { user: user.clone(), password, sessions: Vec::new() });
            user
        }
    };
    server.users.save()?;
    log::info!("user {name} saved");
    Ok(Json(user))
}

async fn delete_user(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap, Path(name): Path<String>) -> Result<StatusCode> {
    let (mut server, caller) = server(&state, &headers)?;
    caller.admin(client)?;
    let after: Vec<ProjectUser> = server.user_list().into_iter().filter(|u| !u.name.eq_ignore_ascii_case(name.trim())).collect();
    if server.loses_admin(&state.token, &server.access.config.policy, &after) {
        return Err(AppError(StatusCode::CONFLICT, "the last administrator cannot be deleted".into()));
    }
    let before = server.users.accounts.len();
    server.users.accounts.retain(|a| !a.user.name.eq_ignore_ascii_case(name.trim()));
    if server.users.accounts.len() == before {
        return Err(AppError(StatusCode::NOT_FOUND, format!("user '{name}' not found")));
    }
    server.users.save()?;
    // A deleted user keeps no project open and no page locked.
    let closed = server.sessions.close_user(&name);
    log::info!("user {} deleted ({closed} project sessions closed)", name.trim());
    Ok(StatusCode::NO_CONTENT)
}

async fn access(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap) -> Result<Json<Settings<ProjectRight>>> {
    let (server, caller) = server(&state, &headers)?;
    caller.admin(client)?;
    Ok(Json(server.access.settings()))
}

async fn save_access(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap, Json(policy): Json<Policy<ProjectRight>>) -> Result<Json<Settings<ProjectRight>>> {
    let (mut server, caller) = server(&state, &headers)?;
    caller.admin(client)?;
    if server.loses_admin(&state.token, &policy, &server.user_list()) {
        return Err(AppError(StatusCode::CONFLICT, "the access settings would leave nobody who manages users".into()));
    }
    server.access.set_policy(policy).map_err(bad)?;
    log::info!("access settings changed by {}", caller.user.as_deref().unwrap_or("token"));
    Ok(Json(server.access.settings()))
}

/// `?limit=`: ended sessions to answer with (default [`SESSIONS_SHOWN`]).
#[derive(serde::Deserialize)]
struct SessionsQuery {
    limit: Option<usize>,
}

/// Ended sessions answered without `limit`.
const SESSIONS_SHOWN: usize = 500;

/// The open sessions with the names of their locked pages and the last ended ones
/// (administrators).
async fn sessions(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap, Query(q): Query<SessionsQuery>) -> Result<Json<SessionList>> {
    let (server, caller) = server(&state, &headers)?;
    caller.admin(client)?;
    let mut active = server.sessions.active(Instant::now());
    for r in &mut active {
        let Some(entry) = server.store.get(&r.project) else { continue };
        r.pages = r
            .locks
            .iter()
            .filter_map(|t| match t {
                Target::Page(id) => entry.project.pages.iter().find(|p| p.id == *id).map(openecraft_core::structure::page_full_name),
                Target::Project => None,
            })
            .collect();
    }
    let history = server.sessions.history.recent(q.limit.unwrap_or(SESSIONS_SHOWN));
    Ok(Json(SessionList { active, history, kept: crate::history::KEPT }))
}

async fn directories(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap) -> Result<Json<Vec<DirectoryEntry>>> {
    let (server, caller) = server(&state, &headers)?;
    caller.admin(client)?;
    Ok(Json(server.access.config.directories.iter().map(DirectoryEntry::of).collect()))
}

/// Creates or changes a directory users sign in with.
async fn put_directory(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap, Path(id): Path<String>, Json(d): Json<Directory>) -> Result<Json<DirectoryEntry>> {
    let (mut server, caller) = server(&state, &headers)?;
    caller.admin(client)?;
    let d = admin::directory_of(&server.access.config.directories, &id, d).map_err(bad)?;
    let list = admin::with_directory(&server.access.config.directories, d.clone());
    server.access.set_directories(list).map_err(bad)?;
    log::info!("saved directory {} ({})", d.id, caller.user.as_deref().unwrap_or("token"));
    Ok(Json(DirectoryEntry::of(&d)))
}

/// Removes a directory; its users keep their accounts but cannot sign in.
async fn delete_directory(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap, Path(id): Path<String>) -> Result<StatusCode> {
    let (mut server, caller) = server(&state, &headers)?;
    caller.admin(client)?;
    let list = admin::without_directory(&server.access.config.directories, &id).ok_or_else(|| AppError(StatusCode::NOT_FOUND, format!("directory '{id}' not found")))?;
    server.access.set_directories(list).map_err(bad)?;
    log::info!("deleted directory {id} ({})", caller.user.as_deref().unwrap_or("token"));
    Ok(StatusCode::NO_CONTENT)
}

/// Tests the settings of a directory (saved or not).
async fn test_directory(State(state): State<AppState>, ClientIp(client): ClientIp, headers: HeaderMap, Path(id): Path<String>, Json(req): Json<DirectoryTestRequest>) -> Result<Json<DirectoryTest>> {
    let d = {
        let (server, caller) = server(&state, &headers)?;
        caller.admin(client)?;
        admin::directory_of(&server.access.config.directories, &id, req.directory).map_err(bad)?
    };
    Ok(Json(admin::test_directory(&d, &req.name, &req.password).await))
}

#[cfg(test)]
mod tests;
