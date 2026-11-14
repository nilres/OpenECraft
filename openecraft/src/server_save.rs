//! Automatic saving of a shared project to the project server: when a save is due and
//! which state of the save is shown. No UI code and no requests; `App` asks it once a
//! frame and sends the save with `Remote::autosave`.

use crate::remote::Link;
use openecraft_i18n::{tr, tr_fmt};

pub const DEFAULT_SECS: u32 = 30;
pub const MIN_SECS: u32 = 10;
pub const MAX_SECS: u32 = 600;

/// The setting of the automatic save (kept with the shell's settings).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
    pub enabled: bool,
    /// Seconds from a change to its save.
    pub secs: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { enabled: true, secs: DEFAULT_SECS }
    }
}

impl Settings {
    pub fn interval(&self) -> f64 {
        f64::from(self.secs.clamp(MIN_SECS, MAX_SECS))
    }
}

/// What decides about an automatic save in this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts {
    /// The shared project has changes the server has not confirmed (and has not left
    /// out of a save: those wait for the user).
    pub changed: bool,
    /// No drag or text input is under way: the project is the committed state.
    pub committed: bool,
    /// A save is under way.
    pub saving: bool,
    pub link: Link,
}

/// The timer of the automatic save.
#[derive(Debug, Default)]
pub struct Auto {
    /// Since when changes wait for their save.
    since: Option<f64>,
}

impl Auto {
    /// Call once a frame, after the frame's edits were committed. True: save now.
    /// Changes are saved one interval after they came up (after the save before was
    /// sent, when that left changes or failed); a save that had to wait (input under
    /// way, a save not answered, no connection) is sent as soon as it can be.
    pub fn due(&mut self, now: f64, settings: &Settings, facts: &Facts) -> bool {
        if !settings.enabled || !facts.changed {
            self.since = None;
            return false;
        }
        let since = *self.since.get_or_insert(now);
        if now - since < settings.interval() || !facts.committed || facts.saving || facts.link != Link::Online {
            return false;
        }
        // The next interval starts with this save.
        self.since = Some(now);
        true
    }

    /// Seconds until the waiting changes are saved (0: as soon as possible); None
    /// without waiting changes.
    pub fn wait(&self, now: f64, settings: &Settings) -> Option<f64> {
        self.since.map(|since| (since + settings.interval() - now).max(0.0))
    }
}

/// Whether the changes of a shared project are on the server.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Everything is saved on the server.
    Saved,
    /// Changes wait for the next save.
    Pending,
    /// A save is under way, not answered yet.
    Saving,
    /// Changes wait for the connection (or the session, or the sign-in).
    Offline,
    /// The last save failed, the changes are not on the server.
    Failed,
    /// The last save left changes out; their copies wait for the user.
    Partial,
}

impl State {
    /// `failed`: the last save failed; `partial`: the last save left changes out and
    /// nothing was changed since.
    pub fn of(link: Link, saving: bool, failed: bool, partial: bool, dirty: bool) -> State {
        if saving {
            State::Saving
        } else if !dirty {
            State::Saved
        } else if partial {
            State::Partial
        } else if link != Link::Online {
            State::Offline
        } else if failed {
            State::Failed
        } else {
            State::Pending
        }
    }

    /// The state in a few words (status bar).
    pub fn label(self) -> &'static str {
        match self {
            State::Saved => tr("Saved"),
            State::Pending => tr("Not saved yet"),
            State::Saving => tr("Saving…"),
            State::Offline => tr("Waiting for the server"),
            State::Failed => tr("Save failed"),
            State::Partial => tr("Partly saved"),
        }
    }

    /// The state as a sentence. `error`: of the failed save; `auto`: the interval of
    /// the automatic save in seconds (None: it is off).
    pub fn text(self, error: Option<&str>, auto: Option<u32>) -> String {
        match (self, auto) {
            (State::Saved, _) => tr("All changes are saved on the project server").into(),
            (State::Saving, _) => tr("Saving to the project server…").into(),
            (State::Partial, _) => tr("Partly saved: the project server left changes out. Your version of them is kept below; they are not saved on the server.").into(),
            (State::Failed, Some(_)) => tr_fmt!("The last save failed: {}. Your changes are not saved on the project server; the automatic save tries again.", error.unwrap_or_default()),
            (State::Failed, None) => tr_fmt!("The last save failed: {}. Your changes are not saved on the project server.", error.unwrap_or_default()),
            (State::Offline, Some(_)) => tr("Changes not saved on the project server: they are saved as soon as the server answers again.").into(),
            (State::Offline, None) => tr("Changes not saved on the project server: save them when the server answers again.").into(),
            (State::Pending, Some(secs)) => tr_fmt!("Changes not saved on the project server yet: finished changes are saved automatically within {} s", secs),
            (State::Pending, None) => tr("Changes not saved on the project server yet").into(),
        }
    }
}

#[cfg(test)]
mod tests;
