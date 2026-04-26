//! [`HelperHash`]: SHA-256 digest of an apiKeyHelper script with constant-time comparison.
//!
//! Per `docs/PRD.md` §6.1.1 (T-002 apiKeyHelper hash-pinning) and `docs/TDD.md` §2.4.
//! Constructor is `pub(crate)`: only computed by hashing actual helper-script bytes.
//! Raw `[u8; 32]` cannot be coerced into `HelperHash` from outside this crate.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// SHA-256 digest of an apiKeyHelper script's bytes.
///
/// Constant-time comparison prevents timing side-channels per `docs/TDD.md`
/// §2.4. `serde` round-trips per D-2026-04-26-18; no `default` / `other`
/// attribute is allowed on this type.
#[derive(Serialize, Deserialize, Clone, Eq)]
pub struct HelperHash(#[serde(with = "serde_bytes_array")] [u8; 32]);

impl HelperHash {
    /// Compute a `HelperHash` from helper-script bytes. `pub(crate)`: only
    /// the credential-vault crate can mint by hashing actual script content.
    pub(crate) fn compute_from(helper_script_bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(helper_script_bytes);
        Self(hasher.finalize().into())
    }

    /// Constant-time comparison: returns true iff the SHA-256 of
    /// `helper_script_bytes` equals this `HelperHash`.
    pub fn matches(&self, helper_script_bytes: &[u8]) -> bool {
        let computed = Self::compute_from(helper_script_bytes);
        self.0.ct_eq(&computed.0).into()
    }

    /// Diagnostic-only access to the digest bytes. Avoid in production logic
    /// (use `matches` for comparisons to prevent timing side-channels).
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl PartialEq for HelperHash {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

/// serde adapter for fixed-size byte arrays as JSON arrays of u8.
mod serde_bytes_array {
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

    #[test]
    fn matches_same_bytes() {
        let h = HelperHash::compute_from(b"#!/bin/sh\nexit 0\n");
        assert!(h.matches(b"#!/bin/sh\nexit 0\n"));
    }

    #[test]
    fn rejects_different_bytes() {
        let h = HelperHash::compute_from(b"#!/bin/sh\nexit 0\n");
        assert!(!h.matches(b"#!/bin/sh\nexit 1\n"));
    }

    #[test]
    fn round_trip_serde() {
        let original = HelperHash::compute_from(b"helper script bytes");
        let json = serde_json::to_string(&original).unwrap();
        let restored: HelperHash = serde_json::from_str(&json).unwrap();
        // PartialEq uses constant-time comparison; HelperHash deliberately
        // does not derive Debug to avoid leaking digest bytes in panic
        // messages or log output, so we cannot use assert_eq!.
        assert!(original == restored, "round-trip should preserve digest");
    }

    #[test]
    fn rejects_wrong_byte_length() {
        let bad = serde_json::json!([1, 2, 3]).to_string();
        let r: Result<HelperHash, _> = serde_json::from_str(&bad);
        assert!(r.is_err());
    }
}
