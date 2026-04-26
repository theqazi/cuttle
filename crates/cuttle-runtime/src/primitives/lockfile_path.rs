//! [`LockfilePath`]: validated path under `~/.cuttle/run/<session-id>.lock`.
//!
//! Per `docs/PRD.md` §8 case 6 + `docs/TDD.md` §2.4 + §3.7. Constructor is
//! `pub(crate)`; only the runtime crate can mint. Path is canonical to the
//! per-session location; arbitrary paths are rejected.

use crate::primitives::SessionId;
use std::path::{Path, PathBuf};

/// Canonical lockfile path for the given session.
pub struct LockfilePath(PathBuf);

impl LockfilePath {
    /// Compute the canonical lockfile path for a session.
    /// `pub(crate)`: only the runtime crate constructs.
    /// `#[allow(dead_code)]` until `cuttle-cli` wires the session orchestrator.
    #[allow(dead_code)]
    pub(crate) fn for_session(session_id: &SessionId) -> Result<Self, LockfilePathError> {
        let home = dirs::home_dir().ok_or(LockfilePathError::NoHomeDir)?;
        let path = home
            .join(".cuttle")
            .join("run")
            .join(format!("{}.lock", session_id));
        Ok(Self(path))
    }

    /// Construct from a known path (used at session-resume time when reading
    /// state-coherence.json). Validates the path matches the canonical shape.
    /// `#[allow(dead_code)]` until `cuttle-cli` wires the session orchestrator.
    #[allow(dead_code)]
    pub(crate) fn from_existing(
        path: PathBuf,
        session_id: &SessionId,
    ) -> Result<Self, LockfilePathError> {
        let expected = Self::for_session(session_id)?;
        if path != expected.0 {
            return Err(LockfilePathError::NotCanonical {
                expected: expected.0,
                got: path,
            });
        }
        Ok(expected)
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LockfilePathError {
    #[error("could not determine home directory")]
    NoHomeDir,
    #[error("path is not canonical for the session: expected {expected:?}, got {got:?}")]
    NotCanonical { expected: PathBuf, got: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_session_includes_session_id() {
        let id = SessionId::new_random();
        let p = LockfilePath::for_session(&id).unwrap();
        let s = p.as_path().to_string_lossy().into_owned();
        assert!(s.contains(id.as_str()));
        assert!(s.ends_with(".lock"));
        assert!(s.contains("/.cuttle/run/"));
    }

    #[test]
    fn from_existing_accepts_canonical() {
        let id = SessionId::new_random();
        let canonical = LockfilePath::for_session(&id).unwrap();
        let copy = canonical.as_path().to_path_buf();
        let recreated = LockfilePath::from_existing(copy, &id).unwrap();
        assert_eq!(canonical.as_path(), recreated.as_path());
    }

    #[test]
    fn from_existing_rejects_arbitrary() {
        let id = SessionId::new_random();
        let bad = PathBuf::from("/tmp/attacker.lock");
        let r = LockfilePath::from_existing(bad, &id);
        assert!(matches!(r, Err(LockfilePathError::NotCanonical { .. })));
    }
}
