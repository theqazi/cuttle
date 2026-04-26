//! [`CredentialRecord`]: persisted metadata about a credential.
//!
//! Per `docs/TDD.md` §2.3. Extends the `auth_mode_decision.md:21-28` schema
//! with the `helper_hash` field (per T-002 / D-2026-04-26-08).
//!
//! Note: the API key itself is NEVER stored in `CredentialRecord` (which lives
//! in `~/.cuttle/credentials/<id>.json` per D-2026-04-26-16). The key lives in
//! the macOS Keychain or encrypted-file fallback; `CredentialRecord` carries
//! only the reference to retrieve it.

use crate::primitives::HelperHash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct CredentialRecord {
    /// Credential identifier ("default" or operator-chosen).
    pub id: String,

    /// Backend storing the actual key bytes.
    pub backend: CredentialBackend,

    /// Hash of the apiKeyHelper script (when backend is `ApiKeyHelper`).
    /// `Option<HelperHash>` not `Option<[u8; 32]>` per D-2026-04-26-17:
    /// raw bytes are forbidden at trust-boundary surfaces.
    pub helper_hash: Option<HelperHash>,

    pub created_at: DateTime<Utc>,
    pub last_refreshed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum CredentialBackend {
    /// macOS Keychain (default in v0.1).
    Keychain { service: String, account: String },

    /// Encrypted file fallback (opt-in; per-session operator confirmation).
    EncryptedFile { path: PathBuf },

    /// apiKeyHelper-style indirection (opt-in only; `helper_hash` REQUIRED).
    ApiKeyHelper {
        helper_path: PathBuf,
        refresh_ttl_secs: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::HelperHash;

    #[test]
    fn record_round_trip_keychain_backend() {
        let r = CredentialRecord {
            id: "default".to_string(),
            backend: CredentialBackend::Keychain {
                service: "com.cuttle.anthropic".to_string(),
                account: "default".to_string(),
            },
            helper_hash: None,
            created_at: Utc::now(),
            last_refreshed_at: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let restored: CredentialRecord = serde_json::from_str(&json).unwrap();
        match restored.backend {
            CredentialBackend::Keychain { service, .. } => {
                assert_eq!(service, "com.cuttle.anthropic");
            }
            _ => panic!("expected Keychain backend"),
        }
    }

    #[test]
    fn record_round_trip_helper_backend() {
        let r = CredentialRecord {
            id: "ci".to_string(),
            backend: CredentialBackend::ApiKeyHelper {
                helper_path: PathBuf::from("/usr/local/bin/cuttle-key-helper"),
                refresh_ttl_secs: 3600,
            },
            helper_hash: Some(HelperHash::compute_from(b"helper script bytes")),
            created_at: Utc::now(),
            last_refreshed_at: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let _restored: CredentialRecord = serde_json::from_str(&json).unwrap();
    }
}
