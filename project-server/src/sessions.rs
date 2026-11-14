//! Sessions and their locks. Kept in memory: a session ends when it is closed, when its
//! user signs out or is deleted, when the server stops or after `SESSION_TIMEOUT_SECS`
//! without a request. Ended sessions go to the history (`history`).

use crate::history::History;
use openecraft_auth::unix_now;
use openecraft_core::collab::{Lock, Session, SessionEnd, SessionRecord, Target, SESSION_TIMEOUT_SECS};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

struct Entry {
    number: u64,
    user: String,
    project: String,
    /// Seconds since 1970.
    started: u64,
    seen: Instant,
    saves: u32,
    locks: BTreeSet<Target>,
    /// Hash of the sign-in token the session was opened with: signing out ends it.
    owner: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum LockError {
    /// Unknown or expired session.
    Expired,
    /// Held by another session.
    Held(Lock),
}

#[derive(Default)]
pub struct Sessions {
    sessions: BTreeMap<String, Entry>,
    counter: u64,
    pub history: History,
}

impl Entry {
    /// The session as administrators see it; `now` is the time of `now_unix`.
    fn record(&self, now: Instant, now_unix: u64) -> SessionRecord {
        let idle = now.saturating_duration_since(self.seen).as_secs();
        SessionRecord {
            number: self.number,
            user: self.user.clone(),
            project: self.project.clone(),
            started: self.started,
            last_seen: now_unix.saturating_sub(idle).max(self.started),
            saves: self.saves,
            locks: self.locks.iter().copied().collect(),
            ..Default::default()
        }
    }
}

impl Sessions {
    pub fn with_history(history: History) -> Sessions {
        Sessions { history, ..Default::default() }
    }

    /// Removes the sessions `ended` says and puts them into the history; returns how many.
    fn end(&mut self, how: SessionEnd, now: Instant, ended: impl Fn(&str, &Entry) -> bool) -> usize {
        let ids: Vec<String> = self.sessions.iter().filter(|(id, e)| ended(id, e)).map(|(id, _)| id.clone()).collect();
        let now_unix = unix_now();
        for id in &ids {
            if let Some(e) = self.sessions.remove(id) {
                let record = SessionRecord { locks: Vec::new(), ended: Some(now_unix), end: Some(how), ..e.record(now, now_unix) };
                self.history.add(record);
            }
        }
        ids.len()
    }

    /// The open sessions, newest first.
    pub fn active(&self, now: Instant) -> Vec<SessionRecord> {
        let now_unix = unix_now();
        let mut list: Vec<SessionRecord> = self.sessions.values().map(|e| e.record(now, now_unix)).collect();
        list.sort_by_key(|r| std::cmp::Reverse(r.number));
        list
    }

    /// Counts a save of the session.
    pub fn saved(&mut self, session: &str) {
        if let Some(e) = self.sessions.get_mut(session) {
            e.saves += 1;
        }
    }

    /// Starts a session of `user` on `project`.
    pub fn open(&mut self, project: &str, user: &str, now: Instant) -> Session {
        self.counter += 1;
        // The number is shown to others, the random part is the key to the session.
        let id = format!("{}-{}", self.counter, openecraft_auth::new_token());
        let entry = Entry { number: self.counter, user: user.to_string(), project: project.to_string(), started: unix_now(), seen: now, saves: 0, locks: BTreeSet::new(), owner: None };
        self.sessions.insert(id.clone(), entry);
        Session { user: user.to_string(), session: id }
    }

    /// Marks a session as opened with the sign-in token whose hash is `owner`.
    pub fn set_owner(&mut self, session: &str, owner: String) {
        if let Some(e) = self.sessions.get_mut(session) {
            e.owner = Some(owner);
        }
    }

    /// Ends the sessions opened with the sign-in token whose hash is `owner` (the
    /// user signed out); returns how many.
    pub fn close_owned(&mut self, owner: &str) -> usize {
        self.end(SessionEnd::SignedOut, Instant::now(), |_, s| s.owner.as_deref() == Some(owner))
    }

    /// Ends all sessions of `user` (the account was deleted); returns how many.
    pub fn close_user(&mut self, user: &str) -> usize {
        self.end(SessionEnd::UserDeleted, Instant::now(), |_, s| s.user.eq_ignore_ascii_case(user.trim()))
    }

    /// Ends the sessions without request for `SESSION_TIMEOUT_SECS`.
    pub fn expire(&mut self, now: Instant) {
        let timeout = Duration::from_secs(SESSION_TIMEOUT_SECS);
        self.end(SessionEnd::Expired, now, |_, s| {
            let expired = now.saturating_duration_since(s.seen) >= timeout;
            if expired {
                log::info!("session {} of {} on {} expired", s.number, s.user, s.project);
            }
            expired
        });
    }

    fn entry(&mut self, project: &str, session: &Session) -> Result<&mut Entry, LockError> {
        match self.sessions.get_mut(&session.session) {
            Some(e) if e.project == project && e.user == session.user => Ok(e),
            _ => Err(LockError::Expired),
        }
    }

    /// Keeps the session alive.
    pub fn touch(&mut self, project: &str, session: &Session, now: Instant) -> Result<(), LockError> {
        self.entry(project, session)?.seen = now;
        Ok(())
    }

    /// All locks on `project`.
    pub fn locks(&self, project: &str) -> Vec<Lock> {
        let mut out: Vec<Lock> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.project == project)
            .flat_map(|(id, s)| s.locks.iter().map(|t| Lock { target: *t, user: s.user.clone(), session: id.clone() }))
            .collect();
        out.sort_by_key(|l| l.target);
        out
    }

    /// Users with a session on `project`.
    pub fn users(&self, project: &str) -> Vec<String> {
        let users: BTreeSet<&String> = self.sessions.values().filter(|s| s.project == project).map(|s| &s.user).collect();
        users.into_iter().cloned().collect()
    }

    /// Locks all `targets` or none of them.
    pub fn lock(&mut self, project: &str, session: &Session, targets: &[Target], now: Instant) -> Result<(), LockError> {
        self.touch(project, session, now)?;
        if let Some(held) = self.locks(project).into_iter().find(|l| l.session != session.session && targets.contains(&l.target)) {
            return Err(LockError::Held(held));
        }
        self.entry(project, session)?.locks.extend(targets.iter().copied());
        Ok(())
    }

    pub fn release(&mut self, project: &str, session: &Session, targets: &[Target], now: Instant) -> Result<(), LockError> {
        let entry = self.entry(project, session)?;
        entry.seen = now;
        for t in targets {
            entry.locks.remove(t);
        }
        Ok(())
    }

    /// Ends a session and releases its locks.
    pub fn close(&mut self, project: &str, session: &Session) -> Result<(), LockError> {
        self.entry(project, session)?;
        self.end(SessionEnd::Closed, Instant::now(), |id, _| id == session.session);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
