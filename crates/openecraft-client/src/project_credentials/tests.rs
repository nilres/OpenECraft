use super::*;
use std::{cell::RefCell, collections::HashMap};

#[derive(Default)]
struct MemoryStore {
    values: RefCell<HashMap<String, String>>,
    failure: Option<CredentialError>,
}

impl CredentialStore for MemoryStore {
    fn load(&self, account: &Account) -> Result<Option<String>, CredentialError> {
        if let Some(error) = self.failure { return Err(error); }
        Ok(self.values.borrow().get(&account.target()).cloned())
    }
    fn save(&self, account: &Account, token: &str) -> Result<bool, CredentialError> {
        if let Some(error) = self.failure { return Err(error); }
        if !valid_token(token) { return Ok(false); }
        self.values.borrow_mut().insert(account.target(), token.into());
        Ok(true)
    }
    fn forget(&self, account: &Account) -> Result<bool, CredentialError> {
        if let Some(error) = self.failure { return Err(error); }
        Ok(self.values.borrow_mut().remove(&account.target()).is_some())
    }
}

fn account(server: &str, user: &str) -> Account { Account::new(server, user).unwrap() }

#[test]
fn credentials_are_bound_to_the_complete_server_address_and_account() {
    let store = MemoryStore::default();
    let original = account("https://server.test/projects", "Alice");
    store.save(&original, "dummy-session").unwrap();
    for other in [
        account("https://other.test/projects", "Alice"),
        account("https://server.test:8443/projects", "Alice"),
        account("https://server.test/other", "Alice"),
        account("https://server.test/Projects", "Alice"),
        account("https://server.test/projects", "Bob"),
    ] {
        assert!(store.load(&other).unwrap().is_none());
        assert_ne!(original.target(), other.target());
    }
    let local_http = account("http://127.0.0.1:8091/projects", "Alice");
    let local_https = account("https://127.0.0.1:8091/projects", "Alice");
    assert_ne!(local_http.target(), local_https.target());
    assert!(original.target().starts_with("OpenECraft/ProjectServer/Session/v1/"));
    assert!(!original.target().contains("dummy-session"));
}

#[test]
fn equivalent_addresses_share_a_key_and_url_components_cannot_collide() {
    let original = account("https://SERVER.test/projects", " Alice ");
    assert_eq!(original, account("HTTPS://server.test:443/projects/", "Alice"));
    assert_eq!(original.server(), "https://server.test:443/projects");
    assert_eq!(original.user(), "alice");
    assert_eq!(account("https://server.test", "Alice"), account("https://server.test/", "Alice"));
    assert_eq!(account("http://[::1]", "Alice").server(), "http://[::1]:80");
    assert_ne!(account("https://server.test/a/b", "c").target(), account("https://server.test/a", "b/c").target());
    assert_ne!(account("https://server.test/a%2Fb", "c").target(), account("https://server.test/a/b", "c").target());
}

#[test]
fn account_spelling_matches_the_servers_ascii_case_insensitive_lookup() {
    let store = MemoryStore::default();
    let mixed = account("https://server.test/projects", " Alice ");
    let lower = account("https://server.test/projects", "alice");
    assert_eq!(mixed, lower);
    assert_eq!(mixed.user(), "alice");
    store.save(&mixed, "session-fixture").unwrap();
    assert_eq!(store.load(&lower).unwrap().as_deref(), Some("session-fixture"));
    assert!(store.forget(&account("https://server.test/projects", "ALICE")).unwrap());
    assert!(store.load(&mixed).unwrap().is_none());
}

#[test]
fn http_and_https_servers_are_eligible_for_separate_persistence() {
    for server in [
        "https://server.test", "http://server.test", "http://192.168.1.10:8091",
        "http://localhost:8091", "http://127.0.0.1:8091", "http://127.2.3.4", "http://[::1]:8091",
    ] {
        assert!(Account::new(server, "Alice").is_some(), "{server}");
    }
    let store = MemoryStore::default();
    let http = account("http://server.test:8091/projects", "Alice");
    let https = account("https://server.test:8091/projects", "Alice");
    assert_ne!(http.target(), https.target());
    store.save(&http, "http-session-fixture").unwrap();
    assert_eq!(store.load(&http).unwrap().as_deref(), Some("http-session-fixture"));
    assert!(store.load(&https).unwrap().is_none());
    for server in [
        "https://user:secret@server.test", "https://@server.test", "https://server.test?token=secret",
        "http://user:secret@server.test", "http://server.test?token=secret", "http://server.test#secret",
        "https://server.test#secret", "https://server.test?", "https://server.test#",
        "https://server.test\\other", "https://server.test/a\nb", "file:///server.test", "ftp://server.test", "server.test", "",
    ] {
        assert!(Account::new(server, "Alice").is_none(), "{server}");
        assert!(!SystemCredentialStore::available_for(server, "Alice"));
    }
    for user in ["", " ", "Alice\nBob", "Alice\0"] {
        assert!(Account::new("https://server.test", user).is_none());
    }
}

#[test]
fn rotating_or_forgetting_a_token_does_not_affect_other_accounts() {
    let store = MemoryStore::default();
    let alice = account("https://server.test", "Alice");
    let bob = account("https://server.test", "Bob");
    store.save(&alice, "first-fixture").unwrap();
    store.save(&bob, "bob-fixture").unwrap();
    store.save(&alice, "rotated-fixture").unwrap();
    assert_eq!(store.load(&alice).unwrap().as_deref(), Some("rotated-fixture"));
    assert!(store.forget(&alice).unwrap());
    assert!(store.load(&alice).unwrap().is_none());
    assert!(!store.forget(&alice).unwrap());
    assert_eq!(store.load(&bob).unwrap().as_deref(), Some("bob-fixture"));
}

#[test]
fn invalid_tokens_do_not_overwrite_a_saved_sign_in() {
    let store = MemoryStore::default();
    let alice = account("https://server.test", "Alice");
    store.save(&alice, "working-fixture").unwrap();
    for token in ["", " ", "with space", "with\r\nheader", "with\0nul"] {
        assert!(!store.save(&alice, token).unwrap());
        assert!(!SystemCredentialStore.save(&alice, token).unwrap());
    }
    assert_eq!(store.load(&alice).unwrap().as_deref(), Some("working-fixture"));
}

#[test]
fn failures_contain_only_the_operation_and_numeric_code() {
    let alice = account("https://server.test", "Alice");
    for operation in [Operation::Read, Operation::Save, Operation::Forget] {
        let error = CredentialError { operation, code: 1312 };
        let store = MemoryStore { failure: Some(error), ..Default::default() };
        assert_eq!(store.load(&alice), Err(error));
        assert_eq!(store.save(&alice, "do-not-print-this-fixture"), Err(error));
        assert_eq!(store.forget(&alice), Err(error));
        let message = format!("{error}; {error:?}");
        assert!(message.contains("1312"));
        for secret in ["do-not-print-this-fixture", "Alice", "server.test", "EPLAN", "Data Portal"] {
            assert!(!message.contains(secret));
        }
    }
}

#[cfg(not(windows))]
#[test]
fn unsupported_platforms_have_no_persistent_fallback() {
    let alice = account("https://server.test", "Alice");
    assert!(!SystemCredentialStore::available_for(alice.server(), alice.user()));
    assert!(SystemCredentialStore.load(&alice).unwrap().is_none());
    assert!(!SystemCredentialStore.save(&alice, "dummy-fixture").unwrap());
    assert!(!SystemCredentialStore.forget(&alice).unwrap());
}

#[cfg(windows)]
#[test]
fn windows_roundtrip_uses_only_unique_synthetic_entries_and_removes_them() {
    let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let server = format!("https://openecraft-project-credential-test-{}-{unique}.invalid/projects", std::process::id());
    let alice = account(&server, "synthetic-Alice");
    let bob = account(&server, "synthetic-Bob");
    let other_path = account(&server.replace("/projects", "/Projects"), "synthetic-Alice");
    struct Cleanup(Vec<Account>);
    impl Drop for Cleanup {
        fn drop(&mut self) { for account in &self.0 { let _ = SystemCredentialStore.forget(account); } }
    }
    let _cleanup = Cleanup(vec![alice.clone(), bob.clone(), other_path.clone()]);
    assert!(SystemCredentialStore::available_for(alice.server(), alice.user()));
    assert!(SystemCredentialStore.load(&alice).unwrap().is_none());
    assert!(SystemCredentialStore.save(&alice, "dummy-session-fixture").unwrap());
    assert_eq!(SystemCredentialStore.load(&alice).unwrap().as_deref(), Some("dummy-session-fixture"));
    assert!(SystemCredentialStore.load(&bob).unwrap().is_none());
    assert!(SystemCredentialStore.load(&other_path).unwrap().is_none());
    assert!(SystemCredentialStore.save(&other_path, "other-path-fixture").unwrap());
    assert_eq!(SystemCredentialStore.load(&alice).unwrap().as_deref(), Some("dummy-session-fixture"));
    assert_eq!(SystemCredentialStore.load(&other_path).unwrap().as_deref(), Some("other-path-fixture"));
    assert!(SystemCredentialStore.save(&bob, "bob-session-fixture").unwrap());
    assert!(SystemCredentialStore.save(&alice, "rotated-session-fixture").unwrap());
    assert_eq!(SystemCredentialStore.load(&alice).unwrap().as_deref(), Some("rotated-session-fixture"));
    assert!(SystemCredentialStore.forget(&alice).unwrap());
    assert!(SystemCredentialStore.load(&alice).unwrap().is_none());
    assert_eq!(SystemCredentialStore.load(&bob).unwrap().as_deref(), Some("bob-session-fixture"));
    assert!(!SystemCredentialStore.forget(&alice).unwrap());
}
