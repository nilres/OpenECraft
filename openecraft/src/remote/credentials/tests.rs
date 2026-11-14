use super::*;
use openecraft_client::project_credentials::{CredentialError, Operation};
use std::{cell::RefCell, collections::HashMap};

#[derive(Default)]
struct MemoryStore {
    values: RefCell<HashMap<Account, String>>,
    calls: RefCell<Vec<Kind>>,
    failure: Option<CredentialError>,
    forget_failure: Option<CredentialError>,
    decline_save: bool,
}

impl CredentialStore for MemoryStore {
    fn load(&self, account: &Account) -> Result<Option<String>, CredentialError> {
        self.calls.borrow_mut().push(Kind::Load);
        if let Some(error) = self.failure { return Err(error); }
        Ok(self.values.borrow().get(account).cloned())
    }
    fn save(&self, account: &Account, token: &str) -> Result<bool, CredentialError> {
        self.calls.borrow_mut().push(Kind::Save);
        if let Some(error) = self.failure { return Err(error); }
        if self.decline_save { return Ok(false); }
        self.values.borrow_mut().insert(account.clone(), token.into());
        Ok(true)
    }
    fn forget(&self, account: &Account) -> Result<bool, CredentialError> {
        self.calls.borrow_mut().push(Kind::Forget);
        if let Some(error) = self.failure { return Err(error); }
        if let Some(error) = self.forget_failure { return Err(error); }
        Ok(self.values.borrow_mut().remove(account).is_some())
    }
}

fn account() -> Account { Account::new("https://server.test/projects", "synthetic-Alice").unwrap() }
fn pending(generation: u64, job: Job) -> Pending { Pending { generation, job, ctx: None } }

#[test]
fn a_forget_waits_for_the_running_save_and_cannot_be_overtaken_by_a_load() {
    let mut queue = Queue::default();
    let store = MemoryStore::default();
    let account = account();
    assert!(queue.enqueue(pending(1, Job::Save(account.clone(), "dummy-session".into()))));
    let saving = queue.next().unwrap();
    // The first enqueue owns the worker until it drains the queue. New jobs do
    // not start a second worker while the first OS operation is in progress.
    assert!(!queue.enqueue(pending(2, Job::Forget(account.clone()))));
    assert!(!queue.enqueue(pending(3, Job::Load(account.clone()))));
    assert!(queue.busy());
    let saved = execute(saving.generation, saving.job, &store, true);
    assert_eq!(saved.kind, Kind::Save);
    assert_eq!(saved.generation, 1);
    assert_eq!(saved.account, account);
    assert!(saved.result.unwrap().is_none());
    // The worker advances without waiting for the UI to poll any answer.
    let forgetting = queue.next().unwrap();
    let forgotten = execute(forgetting.generation, forgetting.job, &store, true);
    assert_eq!(forgotten.kind, Kind::Forget);
    assert_eq!(forgotten.generation, 2);
    assert!(forgotten.result.unwrap().is_none());
    let loading = queue.next().unwrap();
    let loaded = execute(loading.generation, loading.job, &store, true);
    assert_eq!(loaded.kind, Kind::Load);
    assert_eq!(loaded.generation, 3);
    assert!(loaded.result.unwrap().is_none());
    assert!(queue.busy());
    assert!(queue.next().is_none());
    assert!(!queue.busy());
    assert_eq!(*store.calls.borrow(), [Kind::Save, Kind::Forget, Kind::Load]);
}

#[test]
fn results_keep_the_original_generation_and_account() {
    let store = MemoryStore::default();
    let alice = account();
    let bob = Account::new(alice.server(), "synthetic-Bob").unwrap();
    execute(11, Job::Save(alice.clone(), "alice-fixture".into()), &store, true).result.unwrap();
    execute(12, Job::Save(bob.clone(), "bob-fixture".into()), &store, true).result.unwrap();
    let loaded = execute(13, Job::Load(alice.clone()), &store, true);
    assert_eq!(loaded.generation, 13);
    assert_eq!(loaded.account, alice);
    assert_eq!(loaded.result.unwrap().as_deref(), Some("alice-fixture"));
    execute(14, Job::Forget(alice.clone()), &store, true).result.unwrap();
    assert!(store.load(&alice).unwrap().is_none());
    assert_eq!(store.load(&bob).unwrap().as_deref(), Some("bob-fixture"));
}

#[test]
fn forgetting_an_already_absent_entry_succeeds_on_a_supported_backend() {
    let store = MemoryStore::default();
    let finished = execute(1, Job::Forget(account()), &store, true);
    assert!(finished.result.unwrap().is_none());
    assert_eq!(*store.calls.borrow(), [Kind::Forget]);
}

#[test]
fn unsupported_forget_is_explicit_and_does_not_touch_a_store() {
    let store = MemoryStore::default();
    let finished = execute(1, Job::Forget(account()), &store, false);
    assert_eq!(finished.result.unwrap_err(), unavailable());
    assert!(store.calls.borrow().is_empty());
}

#[test]
fn a_declined_save_is_reported_instead_of_claiming_the_token_was_remembered() {
    let store = MemoryStore { decline_save: true, ..Default::default() };
    let finished = execute(1, Job::Save(account(), "fixture".into()), &store, false);
    assert_eq!(finished.result.unwrap_err(), unavailable());
    assert!(store.values.borrow().is_empty());
}

#[test]
fn a_failed_os_operation_keeps_its_generation_without_stopping_the_queue() {
    let mut queue = Queue::default();
    let store = MemoryStore { failure: Some(CredentialError { operation: Operation::Save, code: 1312 }), ..Default::default() };
    assert!(queue.enqueue(pending(21, Job::Save(account(), "secret-fixture".into()))));
    assert!(!queue.enqueue(pending(22, Job::Forget(account()))));
    let saving = queue.next().unwrap();
    let finished = execute(saving.generation, saving.job, &store, true);
    assert_eq!(finished.generation, 21);
    let error = finished.result.unwrap_err();
    assert!(error.contains("1312"));
    assert!(!error.contains("secret-fixture"));
    assert_eq!(queue.next().unwrap().generation, 22);
}

#[test]
fn an_enqueue_after_the_worker_becomes_idle_starts_a_new_worker() {
    let mut queue = Queue::default();
    assert!(queue.enqueue(pending(1, Job::Load(account()))));
    assert!(queue.next().is_some());
    assert!(queue.next().is_none());
    assert!(!queue.busy());
    assert!(queue.enqueue(pending(2, Job::Forget(account()))));
    assert_eq!(queue.next().unwrap().generation, 2);
}

#[test]
fn an_empty_credentials_queue_is_idle_and_polling_never_starts_os_access() {
    let mut credentials = Credentials::default();
    assert!(!credentials.busy());
    assert!(credentials.poll().is_empty());
    assert!(!credentials.busy());
}

#[test]
fn a_finished_save_keeps_closing_blocked_until_the_ui_has_persisted_its_preference() {
    let mut credentials = Credentials::default();
    // The closing UI has just polled. The worker can finish before its following
    // busy check; the queued answer still has to update the opt-in metadata.
    assert!(credentials.poll().is_empty());
    credentials.inbox.lock().unwrap().push(Finished {
        generation: 1, account: account(), kind: Kind::Save, result: Ok(None),
    });
    assert!(!credentials.queue.lock().unwrap().busy());
    assert!(credentials.busy());
    let finished = credentials.poll();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].kind, Kind::Save);
    assert!(!credentials.busy());
}

#[test]
fn conditional_forget_removes_only_the_matching_account_token() {
    let store = MemoryStore::default();
    let alice = account();
    let bob = Account::new(alice.server(), "synthetic-Bob").unwrap();
    store.values.borrow_mut().insert(alice.clone(), "expected-fixture".into());
    store.values.borrow_mut().insert(bob.clone(), "other-account-fixture".into());
    let finished = execute(7, Job::ForgetIf(alice.clone(), "expected-fixture".into()), &store, true);
    assert_eq!(finished.kind, Kind::ForgetIf);
    assert_eq!(finished.generation, 7);
    assert!(finished.result.unwrap().is_none());
    assert!(!store.values.borrow().contains_key(&alice));
    assert_eq!(store.values.borrow().get(&bob).map(String::as_str), Some("other-account-fixture"));
    assert_eq!(*store.calls.borrow(), [Kind::Load, Kind::Forget]);
}

#[test]
fn conditional_forget_preserves_a_newer_token_and_tolerates_an_absent_entry() {
    let store = MemoryStore::default();
    let alice = account();
    store.values.borrow_mut().insert(alice.clone(), "newer-fixture".into());
    let finished = execute(1, Job::ForgetIf(alice.clone(), "older-fixture".into()), &store, true);
    assert!(finished.result.unwrap().is_none());
    assert_eq!(store.values.borrow().get(&alice).map(String::as_str), Some("newer-fixture"));
    store.values.borrow_mut().clear();
    assert!(execute(2, Job::ForgetIf(alice, "older-fixture".into()), &store, true).result.unwrap().is_none());
    assert_eq!(*store.calls.borrow(), [Kind::Load, Kind::Load]);
}

#[test]
fn conditional_forget_reports_read_and_delete_failures_without_tokens() {
    let alice = account();
    let mut store = MemoryStore { failure: Some(CredentialError { operation: Operation::Read, code: 1312 }), ..Default::default() };
    let failed = execute(1, Job::ForgetIf(alice.clone(), "secret-fixture".into()), &store, true);
    let error = failed.result.unwrap_err();
    assert!(error.contains("1312") && !error.contains("secret-fixture"));
    assert_eq!(*store.calls.borrow(), [Kind::Load]);
    store.failure = None;
    store.forget_failure = Some(CredentialError { operation: Operation::Forget, code: 5 });
    store.calls.borrow_mut().clear();
    store.values.borrow_mut().insert(alice.clone(), "secret-fixture".into());
    let failed = execute(2, Job::ForgetIf(alice.clone(), "secret-fixture".into()), &store, true);
    let error = failed.result.unwrap_err();
    assert!(error.contains('5') && !error.contains("secret-fixture"));
    assert!(store.values.borrow().contains_key(&alice));
    assert_eq!(*store.calls.borrow(), [Kind::Load, Kind::Forget]);
}
