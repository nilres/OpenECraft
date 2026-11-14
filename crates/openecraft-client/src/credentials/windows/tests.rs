use super::*;
use std::{sync::{atomic::{AtomicU64, Ordering}, mpsc, Arc, Barrier}, time::{Duration, SystemTime, UNIX_EPOCH}};

struct Fixture(String);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        Self(format!("OpenECraft/Test/Credentials/v1/{}-{unique}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) { let _ = forget(&self.0); }
}

#[test]
fn all_store_operations_wait_for_the_same_target_lock() {
    let fixture = Fixture::new();
    assert!(load(&fixture.0).unwrap().is_none());
    for operation in [Operation::Read, Operation::Save, Operation::Forget] {
        save(&fixture.0, "synthetic-old-token", "synthetic-user").unwrap();
        let held = TargetLock::acquire(&fixture.0, Operation::Save).unwrap();
        let target = fixture.0.to_ascii_uppercase();
        let (started, ready) = mpsc::channel();
        let (completed, done) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            started.send(()).unwrap();
            let result = match operation {
                Operation::Read => load(&target).map(|_| ()),
                Operation::Save => save(&target, "synthetic-new-token", "synthetic-user").map(|_| ()),
                Operation::Forget => forget(&target).map(|_| ()),
            };
            completed.send(result).unwrap();
        });
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(done.recv_timeout(Duration::from_millis(50)), Err(mpsc::RecvTimeoutError::Timeout)));
        drop(held);
        done.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
        worker.join().unwrap();
    }
}

#[test]
fn concurrent_conditional_delete_cannot_remove_a_newer_token() {
    let fixture = Fixture::new();
    assert!(load(&fixture.0).unwrap().is_none());
    for _ in 0..16 {
        save(&fixture.0, "synthetic-old-token", "synthetic-user").unwrap();
        let barrier = Arc::new(Barrier::new(3));
        let save_target = fixture.0.clone();
        let save_barrier = barrier.clone();
        let writer = std::thread::spawn(move || {
            save_barrier.wait();
            save(&save_target, "synthetic-new-token", "synthetic-user")
        });
        let forget_target = fixture.0.clone();
        let forget_barrier = barrier.clone();
        let deleting = std::thread::spawn(move || {
            forget_barrier.wait();
            forget_if(&forget_target, "synthetic-old-token")
        });
        barrier.wait();
        assert!(writer.join().unwrap().unwrap());
        deleting.join().unwrap().unwrap();
        assert_eq!(load(&fixture.0).unwrap().as_deref(), Some("synthetic-new-token"));
    }
}

#[test]
fn independent_credential_entries_survive_concurrent_os_access() {
    let barrier = Arc::new(Barrier::new(8));
    let workers: Vec<_> = (0..8).map(|_| {
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            let fixture = Fixture::new();
            barrier.wait();
            for _ in 0..32 {
                assert!(save(&fixture.0, "synthetic-parallel-token", "synthetic-user").unwrap());
                assert_eq!(load(&fixture.0).unwrap().as_deref(), Some("synthetic-parallel-token"));
                assert!(!forget_if(&fixture.0, "different-fixture").unwrap());
                assert_eq!(load(&fixture.0).unwrap().as_deref(), Some("synthetic-parallel-token"));
            }
        })
    }).collect();
    for worker in workers { worker.join().unwrap(); }
}
