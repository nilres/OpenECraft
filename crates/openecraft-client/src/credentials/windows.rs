use super::{CredentialError, Operation};
use std::{ptr, slice};
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, ERROR_INVALID_DATA, ERROR_INVALID_PARAMETER, ERROR_NOT_FOUND, ERROR_TIMEOUT, WAIT_ABANDONED, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::Security::Credentials::{CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_MAX_CREDENTIAL_BLOB_SIZE, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC};
use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};
use zeroize::{Zeroize, Zeroizing};

fn wide(value: &str) -> Vec<u16> { value.encode_utf16().chain(Some(0)).collect() }
fn error(operation: Operation) -> CredentialError { CredentialError { operation, code: unsafe { GetLastError() } } }

struct NativeLock { handle: HANDLE, owned: bool }

struct TargetLock { _target: NativeLock, _store: NativeLock }

impl TargetLock {
    fn acquire(target: &str, operation: Operation) -> Result<Self, CredentialError> {
        // Credential Manager compares target names without ASCII case.
        let name = wide(&format!("Local\\OpenECraft.Credentials.{}", crate::update::sha256(target.to_ascii_lowercase().as_bytes())));
        // Serialize changes to the shared credential set, retaining the target
        // lock for cooperation with clients that only lock individual entries.
        let store = NativeLock::named(&wide("Local\\OpenECraft.Credentials.Store"), operation)?;
        let target = NativeLock::named(&name, operation)?;
        Ok(Self { _target: target, _store: store })
    }
}

impl NativeLock {
    fn named(name: &[u16], operation: Operation) -> Result<Self, CredentialError> {
        let handle = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() { return Err(error(operation)); }
        let mut lock = Self { handle, owned: false };
        let code = match unsafe { WaitForSingleObject(handle, 30_000) } {
            WAIT_OBJECT_0 | WAIT_ABANDONED => { lock.owned = true; return Ok(lock); }
            WAIT_TIMEOUT => ERROR_TIMEOUT,
            WAIT_FAILED => unsafe { GetLastError() },
            _ => ERROR_INVALID_DATA,
        };
        Err(CredentialError { operation, code })
    }
}

impl Drop for NativeLock {
    fn drop(&mut self) {
        unsafe {
            if self.owned { ReleaseMutex(self.handle); }
            CloseHandle(self.handle);
        }
    }
}

struct Credential(*mut CREDENTIALW);
impl Drop for Credential {
    fn drop(&mut self) {
        // CredRead owns one writable allocation. Clear its secret before freeing it.
        unsafe {
            if !self.0.is_null() {
                let c = &mut *self.0;
                if c.CredentialBlobSize > 0 && !c.CredentialBlob.is_null() {
                    slice::from_raw_parts_mut(c.CredentialBlob, c.CredentialBlobSize as usize).zeroize();
                }
                CredFree(self.0.cast());
            }
        }
    }
}

pub fn load(target: &str) -> Result<Option<String>, CredentialError> {
    let _lock = TargetLock::acquire(target, Operation::Read)?;
    load_unlocked(target)
}

fn load_unlocked(target: &str) -> Result<Option<String>, CredentialError> {
    let target = wide(target);
    let mut pointer = ptr::null_mut();
    if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut pointer) } == 0 {
        let error = error(Operation::Read);
        return if error.code == ERROR_NOT_FOUND { Ok(None) } else { Err(error) };
    }
    let credential = Credential(pointer);
    let invalid = || CredentialError { operation: Operation::Read, code: ERROR_INVALID_DATA };
    if credential.0.is_null() { return Err(invalid()); }
    let c = unsafe { &*credential.0 };
    if c.CredentialBlobSize == 0 { return Ok(None); }
    if c.CredentialBlob.is_null() || c.CredentialBlobSize > CRED_MAX_CREDENTIAL_BLOB_SIZE { return Err(invalid()); }
    let bytes = unsafe { slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize) };
    let token = std::str::from_utf8(bytes).map_err(|_| invalid())?.to_owned();
    Ok((!token.trim().is_empty()).then_some(token))
}

pub fn save(target: &str, token: &str, user: &str) -> Result<bool, CredentialError> {
    if token.len() > CRED_MAX_CREDENTIAL_BLOB_SIZE as usize {
        return Err(CredentialError { operation: Operation::Save, code: ERROR_INVALID_PARAMETER });
    }
    let _lock = TargetLock::acquire(target, Operation::Save)?;
    let mut target = wide(target);
    let mut user = wide(user);
    let mut secret = Zeroizing::new(token.as_bytes().to_vec());
    let credential = CREDENTIALW {
        Type: CRED_TYPE_GENERIC, TargetName: target.as_mut_ptr(),
        CredentialBlobSize: secret.len() as u32, CredentialBlob: secret.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE, UserName: user.as_mut_ptr(), ..Default::default()
    };
    if unsafe { CredWriteW(&credential, 0) } == 0 { Err(error(Operation::Save)) } else { Ok(true) }
}

pub fn forget(target: &str) -> Result<bool, CredentialError> {
    let _lock = TargetLock::acquire(target, Operation::Forget)?;
    forget_unlocked(target)
}

fn forget_unlocked(target: &str) -> Result<bool, CredentialError> {
    let target = wide(target);
    if unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) } != 0 { return Ok(true); }
    let error = error(Operation::Forget);
    if error.code == ERROR_NOT_FOUND { Ok(false) } else { Err(error) }
}

/// Compare and delete under the same cross-process lock used by all writes.
pub fn forget_if(target: &str, expected: &str) -> Result<bool, CredentialError> {
    let _lock = TargetLock::acquire(target, Operation::Forget)?;
    let current = load_unlocked(target)?.map(Zeroizing::new);
    if current.as_ref().map(|token| token.as_str()) != Some(expected) { return Ok(false); }
    forget_unlocked(target)
}

#[cfg(test)]
mod tests;
