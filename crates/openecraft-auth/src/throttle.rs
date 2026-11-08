//! Slows down guessing passwords: after a few failed sign-ins of a user name from a
//! client the next attempts of that client wait, longer with every further failure.
//! The failures are counted per user name and client, so that others cannot lock a
//! user out, and per client over all names, so that trying one password with many
//! names is slowed down as well.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// Failed sign-ins of a user name allowed before attempts are delayed.
pub const FREE_ATTEMPTS: u32 = 5;
/// Failed sign-ins of a client (all user names) allowed before attempts are delayed.
pub const FREE_CLIENT_ATTEMPTS: u32 = 30;
/// Longest wait between two attempts.
pub const MAX_WAIT: Duration = Duration::from_secs(300);
/// Failures are forgotten after this long without another one.
pub const FORGET_AFTER: Duration = Duration::from_secs(3600);

/// Whose failures are counted: a user name from a client, or a client over all names.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Name(String, Option<IpAddr>),
    Client(IpAddr),
}

#[derive(Default)]
pub struct Throttle {
    failures: HashMap<Key, (u32, Instant)>,
}

impl Throttle {
    fn name_key(name: &str, client: Option<IpAddr>) -> Key {
        Key::Name(name.trim().to_lowercase(), client)
    }

    fn wait_of(&self, key: &Key, free: u32, now: Instant) -> Option<u64> {
        let (n, last) = self.failures.get(key)?;
        let since = now.saturating_duration_since(*last);
        if *n < free || since >= FORGET_AFTER {
            return None;
        }
        let wait = Duration::from_secs(1 << (n - free).min(9)).min(MAX_WAIT);
        let left = wait.saturating_sub(since);
        (!left.is_zero()).then(|| left.as_secs().max(1))
    }

    /// Seconds `client` has to wait before trying `name` again; None if it may now.
    pub fn wait(&self, name: &str, client: Option<IpAddr>, now: Instant) -> Option<u64> {
        let by_name = self.wait_of(&Self::name_key(name, client), FREE_ATTEMPTS, now);
        let by_client = client.and_then(|c| self.wait_of(&Key::Client(c),FREE_CLIENT_ATTEMPTS, now));
        by_name.max(by_client)
    }

    pub fn failed(&mut self, name: &str, client: Option<IpAddr>, now: Instant) {
        let keys = [Some(Self::name_key(name, client)), client.map(Key::Client)];
        for key in keys.into_iter().flatten() {
            let e = self.failures.entry(key).or_insert((0, now));
            if now.saturating_duration_since(e.1) >= FORGET_AFTER {
                e.0 = 0;
            }
            e.0 += 1;
            e.1 = now;
        }
        // Forget failures that are long past.
        if self.failures.len() > 10_000 {
            self.failures.retain(|_, (_, t)| now.saturating_duration_since(*t) < FORGET_AFTER);
        }
    }

    /// The user name signed in from `client`; the failures of the client stay counted.
    pub fn succeeded(&mut self, name: &str, client: Option<IpAddr>) {
        self.failures.remove(&Self::name_key(name, client));
    }
}

#[cfg(test)]
mod tests;
