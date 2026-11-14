//! The properties of a project of the server shown from the project list: those of an
//! existing project to change, or of a copy, which exists only here until it is
//! created. They are loaded fresh from the server, never taken from the project list.

use super::{Remote, Reply};
use openecraft_client::{decode, ehttp};
use openecraft_core::collab::{copy_name, name_key, valid_name, CopyRequest, CopySource, ProjectMeta, ProjectProperties, ProjectRight, ProjectSummary, CHANGED_STATUS};
use openecraft_core::model::ProjectInfo;
use openecraft_i18n::tr_fmt;

/// Why the server did not take a change of properties or a copy.
#[derive(Clone, Debug, PartialEq)]
pub struct Refusal {
    /// The project was changed since the properties were loaded.
    pub changed: bool,
    pub error: String,
}

impl Refusal {
    fn of<T: serde::de::DeserializeOwned>(result: ehttp::Result<ehttp::Response>) -> Result<T, Refusal> {
        let changed = result.as_ref().is_ok_and(|r| r.status == CHANGED_STATUS);
        decode(result).map_err(|error| Refusal { changed, error })
    }
}

/// Why the properties of an existing project cannot be changed now.
#[derive(Clone, Debug, PartialEq)]
pub enum Blocked {
    /// The user may not change the project data.
    Rights,
    /// The user whose session has locked the project data.
    Locked(String),
}

/// Why the name of a copy cannot be used.
#[derive(Clone, Debug, PartialEq)]
pub enum NameError {
    Invalid,
    /// The project of that name on the server.
    Taken(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Draft {
    /// The project on the server: the one to change, or the source of the copy.
    pub source: String,
    pub copy: bool,
    /// Name of the copy on the server.
    pub name: String,
    pub info: ProjectInfo,
    /// The info as loaded.
    original: ProjectInfo,
    /// The properties are loaded.
    pub loaded: bool,
    /// Server revision the properties were loaded at; 0 for a copy of whatever is
    /// saved when it is created.
    pub revision: u64,
    pub locked_by: Option<String>,
    pub rights: Vec<ProjectRight>,
    /// Who created and last changed the project on the server.
    pub meta: ProjectMeta,
    /// The project it was copied from.
    pub copied_from: Option<CopySource>,
    /// The copy also gets the review and release records of the source.
    pub records: bool,
    /// A request is under way.
    pub busy: bool,
    /// The copy is being created: the dialog stays until the server answers.
    pub creating: bool,
    /// The project was changed on the server since the properties were loaded.
    pub changed: bool,
    pub error: String,
}

impl Draft {
    /// The properties of `source`, still to be loaded; a copy gets a free name
    /// (`projects`: the last list of the server).
    pub fn new(source: &str, copy: bool, projects: &[ProjectSummary]) -> Draft {
        let taken: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
        let name = if copy { copy_name(source, &taken) } else { String::new() };
        Draft { source: source.into(), copy, name, ..Default::default() }
    }

    /// The properties as the server has them now.
    pub fn take(&mut self, p: ProjectProperties) {
        self.info = p.info.clone();
        self.original = p.info;
        self.revision = p.revision;
        self.locked_by = p.locked_by;
        self.rights = p.rights;
        self.meta = p.meta;
        self.copied_from = p.copied_from;
        self.loaded = true;
        self.busy = false;
        self.changed = false;
        self.error.clear();
    }

    /// The server refused. A copy whose source changed meanwhile keeps the entries and
    /// takes the current state the next time; changed properties are loaded again.
    pub fn refused(&mut self, refusal: Refusal) {
        self.busy = false;
        self.creating = false;
        self.changed = refusal.changed;
        if refusal.changed && self.copy {
            self.revision = 0;
            self.error.clear();
        } else {
            self.error = refusal.error;
        }
    }

    /// Why the properties of the existing project cannot be changed (a copy always can).
    pub fn blocked(&self) -> Option<Blocked> {
        if self.copy {
            return None;
        }
        if !self.rights.contains(&ProjectRight::EditProject) {
            return Some(Blocked::Rights);
        }
        self.locked_by.clone().map(Blocked::Locked)
    }

    pub fn edited(&self) -> bool {
        self.info != self.original
    }

    /// Why the name of the copy cannot be used (`projects`: the last list of the server).
    pub fn name_error(&self, projects: &[ProjectSummary]) -> Option<NameError> {
        if !valid_name(&self.name) {
            return Some(NameError::Invalid);
        }
        projects.iter().find(|p| name_key(&p.name) == name_key(&self.name)).map(|p| NameError::Taken(p.name.clone()))
    }

    /// The copy can be created, or the changed properties saved.
    pub fn ready(&self, projects: &[ProjectSummary]) -> bool {
        if !self.loaded || self.busy {
            return false;
        }
        if self.copy {
            self.name_error(projects).is_none()
        } else {
            self.blocked().is_none() && self.edited() && !self.changed
        }
    }

    fn request(&self, user: &str) -> CopyRequest {
        CopyRequest { name: self.name.clone(), user: user.into(), revision: self.revision, info: Some(self.info.clone()), records: self.records }
    }
}

impl Remote {
    /// Shows the properties of the project `name`, loaded from the server: to change
    /// them, or (`copy`) as those of a copy to create.
    pub fn show_properties(&mut self, name: &str, copy: bool) {
        self.draft = Some(Draft::new(name, copy, &self.projects));
        self.load_properties();
    }

    /// Loads the properties of the draft again; entries made are replaced.
    pub fn load_properties(&mut self) {
        let Some(d) = &mut self.draft else { return };
        d.busy = true;
        d.error.clear();
        let name = d.source.clone();
        let req = self.endpoint().properties(&name);
        self.send(req, tr_fmt!("Loading the properties of {}…", name), move |r| Reply::Properties(name, decode(r)));
    }

    /// Saves the changed properties of the draft's project.
    pub fn save_properties(&mut self) {
        let endpoint = self.endpoint();
        let user = self.user.trim().to_string();
        let Some(d) = self.draft.as_mut().filter(|d| !d.copy && d.loaded && !d.busy) else { return };
        d.busy = true;
        d.error.clear();
        let name = d.source.clone();
        let req = endpoint.save_properties(&name, &user, d.revision, &d.info);
        self.send(req, tr_fmt!("Saving the properties of {}…", name), move |r| Reply::PropertiesSaved(name, Refusal::of(r)));
    }

    /// Creates the copy of the draft on the server and opens it.
    pub fn copy(&mut self) {
        let endpoint = self.endpoint();
        let user = self.user.trim().to_string();
        let Some(d) = self.draft.as_mut().filter(|d| d.copy && d.loaded && !d.busy) else { return };
        d.busy = true;
        d.creating = true;
        d.error.clear();
        let (source, name) = (d.source.clone(), d.name.clone());
        let req = endpoint.copy(&source, &d.request(&user));
        self.send(req, tr_fmt!("Creating {} as a copy of {}…", name, source), move |r| Reply::Copied(name, Refusal::of(r).map(Box::new)));
    }

    /// Takes the answers about the draft. A created copy goes on as an opened project.
    pub(super) fn take_draft(&mut self, reply: Reply) -> Option<Reply> {
        match reply {
            Reply::Properties(name, result) => {
                self.status.clear();
                if let Some(d) = self.draft.as_mut().filter(|d| d.source == name) {
                    match result {
                        Ok(p) => d.take(p),
                        Err(error) => d.refused(Refusal { changed: false, error }),
                    }
                }
                None
            }
            Reply::PropertiesSaved(name, Ok(p)) => {
                self.status = tr_fmt!("Saved the properties of {} (revision {})", name, p.revision);
                if let Some(listed) = self.projects.iter_mut().find(|l| l.name == name) {
                    listed.title = p.info.name.clone();
                    listed.revision = p.revision;
                    listed.meta = p.meta.clone();
                }
                if self.draft.as_ref().is_some_and(|d| !d.copy && d.source == name) {
                    self.draft = None;
                }
                None
            }
            Reply::PropertiesSaved(name, Err(refusal)) | Reply::Copied(name, Err(refusal)) => {
                self.status = tr_fmt!("Project server: {}", refusal.error);
                if let Some(d) = self.draft.as_mut().filter(|d| if d.copy { d.name == name } else { d.source == name }) {
                    d.refused(refusal);
                }
                None
            }
            Reply::Copied(name, Ok(opened)) => {
                let (source, revision) = self.draft.take().map(|d| (d.source, d.revision)).unwrap_or_default();
                let project = &opened.snapshot.project;
                self.projects.push(ProjectSummary {
                    name: name.clone(),
                    title: project.info.name.clone(),
                    pages: project.pages.len(),
                    revision: opened.snapshot.revision,
                    users: vec![opened.session.user.clone()],
                    rights: opened.rights.clone(),
                    // The server's times come with the next list.
                    meta: ProjectMeta { created_by: opened.session.user.clone(), changed_by: opened.session.user.clone(), ..Default::default() },
                    copied_from: Some(CopySource { name: source.clone(), revision }),
                });
                self.copied = Some(source);
                Some(Reply::Opened(name, Ok(opened)))
            }
            other => Some(other),
        }
    }
}

#[cfg(test)]
mod tests;
