use openecraft_client::project_credentials::{Account, CredentialStore, SystemCredentialStore};
use openecraft_i18n::tr;
use std::{collections::VecDeque, sync::{Arc, Mutex}};

// These types intentionally have no Debug implementation: jobs and replies may
// contain a session token.
pub enum Job {
    Load(Account),
    Save(Account, String),
    Forget(Account),
    ForgetIf(Account, String),
}

impl Job {
    fn account(&self) -> &Account {
        match self { Self::Load(account) | Self::Save(account, _) | Self::Forget(account) | Self::ForgetIf(account, _) => account }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind { Load, Save, Forget, ForgetIf }

pub struct Finished {
    pub generation: u64,
    pub account: Account,
    pub result: Result<Option<String>, String>,
    pub kind: Kind,
}

struct Pending {
    generation: u64,
    job: Job,
    ctx: Option<egui::Context>,
}

#[derive(Default)]
struct Queue {
    pending: VecDeque<Pending>,
    running: bool,
}

impl Queue {
    fn enqueue(&mut self, pending: Pending) -> bool {
        self.pending.push_back(pending);
        if self.running { return false; }
        self.running = true;
        true
    }

    fn next(&mut self) -> Option<Pending> {
        let next = self.pending.pop_front();
        if next.is_none() { self.running = false; }
        next
    }

    fn busy(&self) -> bool { self.running || !self.pending.is_empty() }
}

/// One OS access at a time. A forget requested after a save always runs after it;
/// the caller still checks generations before applying a completed result.
#[derive(Default)]
pub struct Credentials {
    queue: Arc<Mutex<Queue>>,
    inbox: Arc<Mutex<Vec<Finished>>>,
}

impl Credentials {
    pub fn enqueue(&mut self, generation: u64, job: Job, ctx: Option<egui::Context>) {
        let start = self.queue.lock().unwrap_or_else(|error| error.into_inner())
            .enqueue(Pending { generation, job, ctx });
        if start { self.start_worker(); }
    }

    pub fn poll(&mut self) -> Vec<Finished> {
        std::mem::take(&mut *self.inbox.lock().unwrap_or_else(|error| error.into_inner()))
    }

    pub fn busy(&self) -> bool {
        self.queue.lock().unwrap_or_else(|error| error.into_inner()).busy()
            || !self.inbox.lock().unwrap_or_else(|error| error.into_inner()).is_empty()
    }

    fn start_worker(&self) {
        let queue = self.queue.clone();
        let inbox = self.inbox.clone();
        openecraft_ui::io::background(move || {
            let mut next = queue.lock().unwrap_or_else(|error| error.into_inner()).next();
            while let Some(pending) = next {
                let account = pending.job.account();
                let available = SystemCredentialStore::available_for(account.server(), account.user());
                let finished = execute(pending.generation, pending.job, &SystemCredentialStore, available);
                inbox.lock().unwrap_or_else(|error| error.into_inner()).push(finished);
                // Mark the queue idle before waking the UI for the final answer.
                next = queue.lock().unwrap_or_else(|error| error.into_inner()).next();
                if let Some(ctx) = pending.ctx { ctx.request_repaint(); }
            }
        });
    }
}

fn unavailable() -> String { tr("Remembering this sign-in is not available on this device.").into() }

fn execute(generation: u64, job: Job, store: &impl CredentialStore, available: bool) -> Finished {
    let account = job.account().clone();
    let (kind, result) = match job {
        Job::Load(_) => (Kind::Load, store.load(&account).map_err(|error| error.to_string())),
        Job::Save(_, token) => (Kind::Save, store.save(&account, &token).map_err(|error| error.to_string())
            .and_then(|saved| if saved { Ok(None) } else { Err(unavailable()) })),
        Job::Forget(_) => (Kind::Forget, if available {
            // Windows also returns false when the entry is already absent.
            store.forget(&account).map(|_| None).map_err(|error| error.to_string())
        } else { Err(unavailable()) }),
        Job::ForgetIf(_, expected) => (Kind::ForgetIf, if available {
            store.forget_if(&account, &expected).map(|_| None).map_err(|error| error.to_string())
        } else { Err(unavailable()) }),
    };
    Finished { generation, account, result, kind }
}

#[cfg(test)]
mod tests;
