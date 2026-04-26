//! Registry filesystem layout + operator-review queue per T-010 / D-22.
//!
//! Layout per `docs/TDD.md` §3 + reuses the cuttle-memory canonical/quarantine
//! pattern so the operator-review queue is a known shape:
//!
//! ```text
//! ~/.cuttle/rewardloop/
//! ├── canonical/                 # operator-promoted AP/VP entries
//! │   └── <id>.json              # SignedEntry
//! └── pending/                   # LEARN-proposed; awaiting operator review
//!     └── <id>.json              # SignedEntry
//! ```
//!
//! Promotion requires a `&TtyInputCap` capability witness (per D-17).

use crate::entry::{RegistryEntry, RegistryEntryError};
use crate::signing::{ProvenanceSigningKey, SignedEntry};
use cuttle_gate::TtyInputCap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub struct Registry {
    root: PathBuf,
}

impl Registry {
    /// Initialize a registry rooted at `root` (typically `~/.cuttle/rewardloop/`).
    /// Creates `canonical/` and `pending/` if they do not exist.
    pub fn ensure(root: PathBuf) -> Result<Self, RegistryError> {
        std::fs::create_dir_all(root.join("canonical"))?;
        std::fs::create_dir_all(root.join("pending"))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn canonical_dir(&self) -> PathBuf {
        self.root.join("canonical")
    }

    pub fn pending_dir(&self) -> PathBuf {
        self.root.join("pending")
    }

    /// LEARN proposes an entry: write SignedEntry to pending/. Entry is NOT
    /// auto-promoted; promotion requires `&TtyInputCap` (per D-17).
    pub fn propose(
        &self,
        entry: RegistryEntry,
        key: &ProvenanceSigningKey,
    ) -> Result<PathBuf, RegistryError> {
        entry.validate()?;
        let signed = SignedEntry::sign(entry, key);
        let path = self.pending_dir().join(format!("{}.json", signed.entry.id));
        if path.exists() {
            return Err(RegistryError::PendingExists { path });
        }
        let json = serde_json::to_string_pretty(&signed)?;
        std::fs::write(&path, json)?;
        Ok(path)
    }

    /// Operator promotes a pending entry to canonical. Requires `&TtyInputCap`
    /// witness (per D-17). Verifies the HMAC signature with the same key
    /// (round-trip integrity check).
    pub fn promote(
        &self,
        entry_id: &str,
        key: &ProvenanceSigningKey,
        _cap: &TtyInputCap,
    ) -> Result<PathBuf, RegistryError> {
        let pending_path = self.pending_dir().join(format!("{}.json", entry_id));
        if !pending_path.exists() {
            return Err(RegistryError::PendingMissing { path: pending_path });
        }
        let raw = std::fs::read_to_string(&pending_path)?;
        let mut signed: SignedEntry = serde_json::from_str(&raw)?;
        if !signed.verify(key) {
            return Err(RegistryError::SignatureInvalid {
                entry_id: entry_id.to_string(),
            });
        }
        // Stamp the operator-confirmation timestamp + re-sign.
        signed.entry.provenance.operator_confirmation_at = Some(chrono::Utc::now());
        let resigned = SignedEntry::sign(signed.entry, key);
        let canonical_path = self
            .canonical_dir()
            .join(format!("{}.json", resigned.entry.id));
        if canonical_path.exists() {
            return Err(RegistryError::CanonicalExists {
                path: canonical_path,
            });
        }
        std::fs::write(&canonical_path, serde_json::to_string_pretty(&resigned)?)?;
        std::fs::remove_file(&pending_path)?;
        Ok(canonical_path)
    }

    /// Count pending entries (used for F-Cuttle-DISABLE / D-25 backlog signal).
    pub fn pending_count(&self) -> Result<usize, RegistryError> {
        let dir = self.pending_dir();
        if !dir.exists() {
            return Ok(0);
        }
        let count = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "json")
                    .unwrap_or(false)
            })
            .count();
        Ok(count)
    }
}

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("entry validation: {0}")]
    Validation(#[from] RegistryEntryError),
    #[error("pending entry already exists at {path:?}")]
    PendingExists { path: PathBuf },
    #[error("pending entry not found at {path:?}")]
    PendingMissing { path: PathBuf },
    #[error("canonical entry already exists at {path:?}")]
    CanonicalExists { path: PathBuf },
    #[error("HMAC signature invalid for entry {entry_id}")]
    SignatureInvalid { entry_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{EntryKind, EntryProvenance, RegistryEntry};
    use chrono::Utc;
    use cuttle_gate::capabilities::__internal_input_cap_factory;
    use rand::RngCore;

    fn fresh_key() -> ProvenanceSigningKey {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        ProvenanceSigningKey::from_bytes(bytes)
    }

    fn sample_entry(id: &str) -> RegistryEntry {
        RegistryEntry {
            id: id.into(),
            kind: EntryKind::AntiPattern,
            title: "test".into(),
            body: "body".into(),
            created_at: Utc::now(),
            provenance: EntryProvenance {
                session_id: "test-session".into(),
                model_output_excerpt: "test excerpt".into(),
                score: 0.9,
                operator_confirmation_at: None,
            },
        }
    }

    #[test]
    fn propose_writes_to_pending() {
        let tmp = tempfile::tempdir().unwrap();
        let r = Registry::ensure(tmp.path().to_path_buf()).unwrap();
        let key = fresh_key();
        let path = r.propose(sample_entry("ap-1"), &key).unwrap();
        assert!(path.exists());
        assert!(path.starts_with(r.pending_dir()));
        assert_eq!(r.pending_count().unwrap(), 1);
    }

    #[test]
    fn promote_moves_to_canonical_with_cap() {
        let tmp = tempfile::tempdir().unwrap();
        let r = Registry::ensure(tmp.path().to_path_buf()).unwrap();
        let key = fresh_key();
        let _ = r.propose(sample_entry("ap-1"), &key).unwrap();
        let cap = __internal_input_cap_factory::issue();
        let canonical = r.promote("ap-1", &key, &cap).unwrap();
        assert!(canonical.exists());
        assert!(canonical.starts_with(r.canonical_dir()));
        // Verify operator_confirmation_at is set.
        let restored: SignedEntry =
            serde_json::from_str(&std::fs::read_to_string(&canonical).unwrap()).unwrap();
        assert!(restored.entry.provenance.operator_confirmation_at.is_some());
        assert_eq!(r.pending_count().unwrap(), 0);
    }

    #[test]
    fn promote_with_wrong_key_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let r = Registry::ensure(tmp.path().to_path_buf()).unwrap();
        let key1 = fresh_key();
        let key2 = fresh_key();
        let _ = r.propose(sample_entry("ap-1"), &key1).unwrap();
        let cap = __internal_input_cap_factory::issue();
        let res = r.promote("ap-1", &key2, &cap);
        assert!(matches!(res, Err(RegistryError::SignatureInvalid { .. })));
    }

    #[test]
    fn propose_duplicate_id_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let r = Registry::ensure(tmp.path().to_path_buf()).unwrap();
        let key = fresh_key();
        r.propose(sample_entry("ap-1"), &key).unwrap();
        let res = r.propose(sample_entry("ap-1"), &key);
        assert!(matches!(res, Err(RegistryError::PendingExists { .. })));
    }

    #[test]
    fn promote_missing_id_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let r = Registry::ensure(tmp.path().to_path_buf()).unwrap();
        let key = fresh_key();
        let cap = __internal_input_cap_factory::issue();
        let res = r.promote("nonexistent", &key, &cap);
        assert!(matches!(res, Err(RegistryError::PendingMissing { .. })));
    }
}
