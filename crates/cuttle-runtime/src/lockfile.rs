//! Lockfile read/write with HMAC integrity per D-2026-04-26-23 (WV-02 closure).
//!
//! Per `docs/TDD.md` §3.7. Lockfile contents = (parent_pid, 32-byte session
//! token, HMAC of contents using per-session signing key). The signing key
//! lives in process memory only (see [`crate::SigningKey`]); a child process
//! inheriting the file descriptor can read the lockfile but cannot regenerate
//! a valid HMAC.
//!
//! Atomic create via `O_CREAT | O_EXCL` (`std::fs::OpenOptions::create_new`)
//! closes the create-then-fsync TOCTOU window. Constant-time HMAC compare via
//! `subtle` crate closes the read-then-trust window. Signing-key-in-memory
//! closes the inherit-then-forge window.

use crate::primitives::LockfilePath;
use crate::signing_key::SigningKey;
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::io::{Read, Write};
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Lockfile contents schema. The HMAC field covers `(parent_pid, session_token)`
/// using the per-session signing key.
#[derive(Serialize, Deserialize, Debug)]
pub struct LockfileContents {
    pub parent_pid: u32,
    #[serde(with = "serde_bytes_32")]
    pub session_token: [u8; 32],
    #[serde(with = "serde_bytes_32")]
    pub hmac: [u8; 32],
}

#[derive(thiserror::Error, Debug)]
pub enum LockfileError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("HMAC verification failed; lockfile may be forged or signing key wrong")]
    HmacMismatch,
    #[error("lockfile already exists at {path:?}; another Cuttle session may be running")]
    AlreadyExists { path: std::path::PathBuf },
}

/// Atomically create the lockfile at `path` with HMAC-protected contents.
/// Fails with `LockfileError::AlreadyExists` if the path already exists.
pub fn write_lockfile(path: &LockfilePath, signing_key: &SigningKey) -> Result<(), LockfileError> {
    let parent_pid = std::process::id();
    let mut session_token = [0u8; 32];
    rand::rng().fill_bytes(&mut session_token);
    let mut mac =
        HmacSha256::new_from_slice(signing_key.as_bytes()).expect("HMAC accepts any key length");
    mac.update(&parent_pid.to_le_bytes());
    mac.update(&session_token);
    let hmac: [u8; 32] = mac.finalize().into_bytes().into();
    let contents = LockfileContents {
        parent_pid,
        session_token,
        hmac,
    };

    if let Some(parent) = path.as_path().parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.as_path())
    {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(LockfileError::AlreadyExists {
                path: path.as_path().to_path_buf(),
            });
        }
        Err(e) => return Err(e.into()),
    };

    let json = serde_json::to_string(&contents)?;
    file.write_all(json.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

/// Read and verify the lockfile at `path` using the expected signing key.
/// Returns `LockfileError::HmacMismatch` if the HMAC does not verify
/// (lockfile may be attacker-crafted, or the signing key is wrong because
/// the operator has rotated keys, etc.).
pub fn read_lockfile(
    path: &LockfilePath,
    expected_signing_key: &SigningKey,
) -> Result<LockfileContents, LockfileError> {
    let mut file = std::fs::File::open(path.as_path())?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let contents: LockfileContents = serde_json::from_str(&buf)?;

    let mut mac = HmacSha256::new_from_slice(expected_signing_key.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(&contents.parent_pid.to_le_bytes());
    mac.update(&contents.session_token);
    let recomputed: [u8; 32] = mac.finalize().into_bytes().into();

    let valid: bool = recomputed.ct_eq(&contents.hmac).into();
    if !valid {
        return Err(LockfileError::HmacMismatch);
    }
    Ok(contents)
}

/// serde adapter for fixed-size byte arrays as JSON arrays of u8.
mod serde_bytes_32 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        bytes.as_slice().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        if v.len() != 32 {
            return Err(serde::de::Error::custom(format!(
                "expected 32 bytes, got {}",
                v.len()
            )));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&v);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::SessionId;

    /// Build a LockfilePath inside a tempdir so tests don't touch ~/.cuttle/.
    fn lockfile_path_in_tmp(tmp: &tempfile::TempDir, session_id: &SessionId) -> std::path::PathBuf {
        tmp.path()
            .join(".cuttle")
            .join("run")
            .join(format!("{}.lock", session_id))
    }

    /// Write the lockfile by hand at a tempdir path, bypassing the canonical
    /// `~/.cuttle/run/` rule. Used by tests to verify HMAC behavior in
    /// isolation; production code uses LockfilePath::for_session.
    fn write_at(path: &std::path::Path, signing_key: &SigningKey) -> Result<(), LockfileError> {
        let parent_pid = std::process::id();
        let mut session_token = [0u8; 32];
        rand::rng().fill_bytes(&mut session_token);
        let mut mac = HmacSha256::new_from_slice(signing_key.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(&parent_pid.to_le_bytes());
        mac.update(&session_token);
        let hmac: [u8; 32] = mac.finalize().into_bytes().into();
        let contents = LockfileContents {
            parent_pid,
            session_token,
            hmac,
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        let json = serde_json::to_string(&contents)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }

    fn read_at(
        path: &std::path::Path,
        expected_signing_key: &SigningKey,
    ) -> Result<LockfileContents, LockfileError> {
        let mut file = std::fs::File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let contents: LockfileContents = serde_json::from_str(&buf)?;
        let mut mac = HmacSha256::new_from_slice(expected_signing_key.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(&contents.parent_pid.to_le_bytes());
        mac.update(&contents.session_token);
        let recomputed: [u8; 32] = mac.finalize().into_bytes().into();
        let valid: bool = recomputed.ct_eq(&contents.hmac).into();
        if !valid {
            return Err(LockfileError::HmacMismatch);
        }
        Ok(contents)
    }

    #[test]
    fn write_then_read_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let id = SessionId::new_random();
        let path = lockfile_path_in_tmp(&tmp, &id);
        let key = SigningKey::new_random();
        write_at(&path, &key).unwrap();
        let contents = read_at(&path, &key).unwrap();
        assert_eq!(contents.parent_pid, std::process::id());
    }

    #[test]
    fn read_with_wrong_key_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let id = SessionId::new_random();
        let path = lockfile_path_in_tmp(&tmp, &id);
        let key1 = SigningKey::new_random();
        let key2 = SigningKey::new_random();
        write_at(&path, &key1).unwrap();
        let r = read_at(&path, &key2);
        assert!(matches!(r, Err(LockfileError::HmacMismatch)));
    }

    #[test]
    fn write_twice_at_same_path_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let id = SessionId::new_random();
        let path = lockfile_path_in_tmp(&tmp, &id);
        let key = SigningKey::new_random();
        write_at(&path, &key).unwrap();
        let r = write_at(&path, &key);
        assert!(matches!(r, Err(LockfileError::Io(_))));
    }

    #[test]
    fn tampered_token_fails_hmac() {
        let tmp = tempfile::tempdir().unwrap();
        let id = SessionId::new_random();
        let path = lockfile_path_in_tmp(&tmp, &id);
        let key = SigningKey::new_random();
        write_at(&path, &key).unwrap();

        // Read, mutate session_token, rewrite (without recomputing HMAC).
        let raw = std::fs::read_to_string(&path).unwrap();
        let mut contents: LockfileContents = serde_json::from_str(&raw).unwrap();
        contents.session_token[0] ^= 0x01;
        std::fs::write(&path, serde_json::to_string(&contents).unwrap()).unwrap();

        let r = read_at(&path, &key);
        assert!(matches!(r, Err(LockfileError::HmacMismatch)));
    }
}
