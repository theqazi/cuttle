//! [`SessionId`]: opaque session identifier.
//!
//! Per `docs/TDD.md` §2.4. New sessions get a fresh random identifier; the
//! identifier appears in the lockfile path and the audit-log session field.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Session identifier. Random 16-byte nonce hex-encoded.
///
/// `pub(crate)` constructor; only the runtime crate can mint via
/// [`SessionId::new_random`]. Other crates receive `&SessionId` from the
/// runtime via `Session::id()`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    /// Generate a new random session identifier (32-character hex string).
    /// `pub(crate)`: only the runtime crate can mint.
    /// `#[allow(dead_code)]` until `cuttle-cli` wires the session orchestrator.
    #[allow(dead_code)]
    pub(crate) fn new_random() -> Self {
        let mut rng = rand::rng();
        let bytes: [u8; 16] = rng.random();
        Self(bytes.iter().map(|b| format!("{:02x}", b)).collect())
    }

    /// Parse a SessionId from a string. Used at session-resume time when
    /// reading `state-coherence.json`. Validates length and hex characters.
    pub fn from_str_validated(s: &str) -> Result<Self, SessionIdError> {
        if s.len() != 32 {
            return Err(SessionIdError::WrongLength { actual: s.len() });
        }
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SessionIdError::NotHex);
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SessionIdError {
    #[error("session id must be 32 hex characters; got {actual}")]
    WrongLength { actual: usize },
    #[error("session id contains non-hex characters")]
    NotHex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_random_produces_32_hex_chars() {
        let id = SessionId::new_random();
        assert_eq!(id.as_str().len(), 32);
        assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn parse_validates_length() {
        let r = SessionId::from_str_validated("abc");
        assert!(matches!(r, Err(SessionIdError::WrongLength { actual: 3 })));
    }

    #[test]
    fn parse_validates_hex() {
        let bad = "z".repeat(32);
        let r = SessionId::from_str_validated(&bad);
        assert!(matches!(r, Err(SessionIdError::NotHex)));
    }

    #[test]
    fn parse_accepts_valid() {
        let original = SessionId::new_random();
        let parsed = SessionId::from_str_validated(original.as_str()).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn round_trip_serde() {
        let id = SessionId::new_random();
        let json = serde_json::to_string(&id).unwrap();
        let restored: SessionId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, restored);
    }
}
