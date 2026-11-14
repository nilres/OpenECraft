use crate::endpoint::with_content_type;
use openecraft_i18n::tr;
use openecraft_core::collab::{
    access_path, project_path, user_path, CreateRequest, Failed, Failure, Grant, LockRequest, LoginRequest, OpenRequest, Policy, ProjectRight, SaveRequest,
    Session, Snapshot, Status, Target, UserUpdate, API_PREFIX, DEFAULT_PORT,
};
use openecraft_core::model::Project;
use openecraft_parts::api;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Like [`crate::decode`], with why the request failed: no answer, the session ended,
/// not signed in, or another error of the server ([`Failure`]).
pub fn decode_failure<T: DeserializeOwned>(result: ehttp::Result<ehttp::Response>) -> Result<T, Failed> {
    let why = Failure::of(result.as_ref().ok().map(|r| r.status));
    crate::decode(result).map_err(|error| Failed { why, error })
}

/// Address and access token of a project server; builds the requests of its API
/// (`openecraft_core::collab`). Every request carries the token, if one is set.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectEndpoint {
    pub url: String,
    pub token: String,
}

impl Default for ProjectEndpoint {
    fn default() -> Self {
        ProjectEndpoint { url: format!("http://127.0.0.1:{DEFAULT_PORT}"), token: String::new() }
    }
}

impl ProjectEndpoint {
    /// Loads a session snapshot, also from servers predating the session endpoint.
    /// A legacy snapshot is paired with the session's status at the same revision,
    /// so redacted lock identities never replace the client's own locks.
    pub fn fetch_session_snapshot(&self, name: &str, session: &Session, done: impl FnOnce(Result<Snapshot, Failed>) + Send + 'static) {
        let endpoint = self.clone();
        let name = name.to_string();
        let session = session.clone();
        ehttp::fetch(self.session_snapshot(&name, &session), move |result| {
            if !result.as_ref().is_ok_and(|r| matches!(r.status, 404 | 405)) {
                done(decode_failure(result));
                return;
            }
            ehttp::fetch(endpoint.snapshot(&name), move |result| {
                let mut snapshot: Snapshot = match decode_failure(result) {
                    Ok(snapshot) => snapshot,
                    Err(error) => { done(Err(error)); return; }
                };
                ehttp::fetch(endpoint.status(&name, &session), move |result| {
                    done(decode_failure::<Status>(result).and_then(|status| {
                        if status.revision != snapshot.revision {
                            return Err(Failed { why: Failure::Refused, error: tr("The project changed while loading. Its changes will be loaded again.").into() });
                        }
                        snapshot.locks = status.locks;
                        Ok(snapshot)
                    }));
                });
            });
        });
    }

    fn request(&self, mut req: ehttp::Request) -> ehttp::Request {
        if !self.token.trim().is_empty() {
            req.headers.insert("Authorization", format!("Bearer {}", self.token.trim()));
        }
        req
    }

    fn get(&self, path: &str) -> ehttp::Request {
        self.request(ehttp::Request::get(api::url(&self.url, path)))
    }

    fn json(&self, method: &str, path: &str, body: &impl Serialize) -> ehttp::Request {
        let body = serde_json::to_vec(body).unwrap_or_default();
        let url = api::url(&self.url, path);
        let req = if method == "PUT" { ehttp::Request::put(url, body) } else { ehttp::Request::post(url, body) };
        self.request(with_content_type(req, "application/json"))
    }

    /// GET /api/v1/info
    pub fn info(&self) -> ehttp::Request {
        self.get(&format!("{API_PREFIX}/info"))
    }

    /// GET /api/v1/session: validates a user sign-in and reads its current identity.
    pub fn current_account(&self) -> ehttp::Request {
        self.get(&format!("{API_PREFIX}/session"))
    }

    /// GET /api/v1/projects
    pub fn list(&self) -> ehttp::Request {
        self.get(&format!("{API_PREFIX}/projects"))
    }

    /// POST /api/v1/projects: stores a new project and opens it.
    pub fn create(&self, name: &str, user: &str, project: &Project) -> ehttp::Request {
        let req = CreateRequest { name: name.into(), user: user.into(), project: project.clone() };
        self.json("POST", &format!("{API_PREFIX}/projects"), &req)
    }

    /// GET /api/v1/projects/{name}
    pub fn snapshot(&self, name: &str) -> ehttp::Request {
        self.get(&project_path(name))
    }

    /// Loads a snapshot with this session's own locks left identifiable.
    pub fn session_snapshot(&self, name: &str, session: &Session) -> ehttp::Request {
        self.json("POST", &format!("{}/snapshot", project_path(name)), session)
    }

    /// POST /api/v1/projects/{name}/open
    pub fn open(&self, name: &str, user: &str) -> ehttp::Request {
        self.json("POST", &format!("{}/open", project_path(name)), &OpenRequest { user: user.into() })
    }

    /// PUT /api/v1/projects/{name}: saves what the session changed since `base_revision`
    /// (the revision last loaded): what it has locked and what is free.
    pub fn save(&self, name: &str, session: &Session, project: &Project, release: Vec<Target>, base_revision: u64) -> ehttp::Request {
        let req = SaveRequest { session: session.clone(), project: project.clone(), release, base_revision, strict: false };
        self.json("PUT", &project_path(name), &req)
    }

    /// POST /api/v1/projects/{name}/status
    pub fn status(&self, name: &str, session: &Session) -> ehttp::Request {
        self.json("POST", &format!("{}/status", project_path(name)), session)
    }

    /// POST /api/v1/projects/{name}/lock
    pub fn lock(&self, name: &str, session: &Session, targets: Vec<Target>) -> ehttp::Request {
        self.json("POST", &format!("{}/lock", project_path(name)), &LockRequest { session: session.clone(), targets })
    }

    /// POST /api/v1/projects/{name}/release
    pub fn release(&self, name: &str, session: &Session, targets: Vec<Target>) -> ehttp::Request {
        self.json("POST", &format!("{}/release", project_path(name)), &LockRequest { session: session.clone(), targets })
    }

    /// POST /api/v1/projects/{name}/close
    pub fn close(&self, name: &str, session: &Session) -> ehttp::Request {
        self.json("POST", &format!("{}/close", project_path(name)), session)
    }

    /// POST /api/v1/login: a session token for the user.
    pub fn login(&self, user: &str, password: &str) -> ehttp::Request {
        self.json("POST", &format!("{API_PREFIX}/login"), &LoginRequest { user: user.into(), password: password.into(), ..Default::default() })
    }

    /// POST /api/v1/logout: ends the session of the token.
    pub fn logout(&self) -> ehttp::Request {
        self.json("POST", &format!("{API_PREFIX}/logout"), &())
    }

    /// GET /api/v1/users (administrators)
    pub fn users(&self) -> ehttp::Request {
        self.get(&format!("{API_PREFIX}/users"))
    }

    /// PUT /api/v1/users/{name} (administrators)
    pub fn save_user(&self, name: &str, update: &UserUpdate) -> ehttp::Request {
        self.json("PUT", &user_path(name), update)
    }

    /// DELETE /api/v1/users/{name} (administrators)
    pub fn delete_user(&self, name: &str) -> ehttp::Request {
        self.request(ehttp::Request::delete(&api::url(&self.url, &user_path(name))))
    }

    /// DELETE /api/v1/projects/{name}: deletes a project nobody has open.
    pub fn delete(&self, name: &str) -> ehttp::Request {
        self.request(ehttp::Request::delete(&api::url(&self.url, &project_path(name))))
    }

    /// GET /api/v1/projects/{name}/access: the grants on the project.
    pub fn project_access(&self, name: &str) -> ehttp::Request {
        self.get(&access_path(name))
    }

    /// PUT /api/v1/projects/{name}/access: replaces the grants on the project.
    pub fn save_project_access(&self, name: &str, grants: &[Grant]) -> ehttp::Request {
        self.json("PUT", &access_path(name), &grants)
    }

    /// GET /api/v1/access: roles, grants and directories (administrators)
    pub fn access(&self) -> ehttp::Request {
        self.get(&format!("{API_PREFIX}/access"))
    }

    /// PUT /api/v1/access: replaces roles and grants (administrators)
    pub fn save_access(&self, policy: &Policy<ProjectRight>) -> ehttp::Request {
        self.json("PUT", &format!("{API_PREFIX}/access"), policy)
    }
}

#[cfg(test)]
mod tests;
