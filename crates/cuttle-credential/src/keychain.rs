//! macOS Keychain backend for the credential vault.
//!
//! Per session-7 design + the validated "expose primitives to operator
//! first" pattern:
//!
//! - Service name: `dev.cuttle.api-keys` (reverse-DNS, conventional).
//! - Account name: matches the env var name the operator would otherwise
//!   set (default `ANTHROPIC_API_KEY`). Multiple-account use cases (e.g.
//!   separate test vs prod keys) work by setting different account names.
//! - Item type: generic password (kSecClassGenericPassword).
//! - Access policy: macOS default. The first read prompts the operator
//!   to authorize cuttle; subsequent reads either re-prompt or
//!   auto-allow depending on operator's "Always Allow" choice. The
//!   audit log records the always-allow toggle via the existing
//!   `AuditEvent::KeychainAlwaysAllowToggled` variant per D-24.
//!
//! Non-macOS platforms get an `Unsupported` error from every function.
//! v0.1 is macOS-only by PRD scope; Linux Keychain (Secret Service) is
//! v0.2+.

use crate::primitives::api_key::ApiKey;
use thiserror::Error;

/// Service name for cuttle's Keychain entries. Reverse-DNS so it
/// doesn't collide with anything else in Keychain Access UI.
pub const KEYCHAIN_SERVICE: &str = "dev.cuttle.api-keys";

/// Default account name. Matches the default env var name so an
/// operator who sets `ANTHROPIC_API_KEY` can also store the same key
/// under that account name and `ApiKey::resolve` will find it either
/// way.
pub const DEFAULT_ACCOUNT: &str = "ANTHROPIC_API_KEY";

#[derive(Error, Debug)]
pub enum KeychainError {
    /// macOS Security.framework returned an error. The wrapped string is
    /// the system error message; cuttle does not invent error text here
    /// because the OS has the most accurate diagnostic.
    #[error("Keychain operation failed: {0}")]
    System(String),

    /// The requested entry does not exist. Distinguished from `System`
    /// because callers (e.g. `ApiKey::resolve`) treat this as
    /// "credential not found, try the next backend" rather than as a
    /// fatal error.
    #[error("no Keychain entry found for service={KEYCHAIN_SERVICE} account={account}")]
    NotFound { account: String },

    /// Build target is not macOS. v0.1 scope is macOS only; Linux
    /// Secret Service / Windows Credential Manager land in v0.2+.
    #[error("Keychain backend is unavailable on this platform (v0.1 macOS-only)")]
    Unsupported,
}

/// Lightweight metadata about a Keychain entry. Intentionally does NOT
/// include the secret bytes; this is what `cuttle credential show`
/// prints to confirm an entry exists without leaking it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeychainMetadata {
    pub service: String,
    pub account: String,
    /// Length of the stored secret. Useful smoke-check ("yes, my 108-byte
    /// sk-ant- key landed correctly") without revealing the bytes.
    pub length: usize,
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    use security_framework::passwords;

    /// macOS errSecItemNotFound. Stable across OS releases per Apple's
    /// SecBase.h; checking the typed code is more robust than scraping
    /// the human-readable error message (which is locale + OS-dependent).
    const ERR_SEC_ITEM_NOT_FOUND: i32 = -25300;

    fn classify(e: security_framework::base::Error, account: &str) -> KeychainError {
        if e.code() == ERR_SEC_ITEM_NOT_FOUND {
            KeychainError::NotFound {
                account: account.to_string(),
            }
        } else {
            KeychainError::System(format!("{e}"))
        }
    }

    pub fn store(account: &str, secret: &[u8]) -> Result<(), KeychainError> {
        passwords::set_generic_password(KEYCHAIN_SERVICE, account, secret)
            .map_err(|e| classify(e, account))
    }

    pub fn fetch(account: &str) -> Result<ApiKey, KeychainError> {
        passwords::get_generic_password(KEYCHAIN_SERVICE, account)
            .map(ApiKey::from_keychain_bytes)
            .map_err(|e| classify(e, account))
    }

    pub fn delete(account: &str) -> Result<(), KeychainError> {
        passwords::delete_generic_password(KEYCHAIN_SERVICE, account)
            .map_err(|e| classify(e, account))
    }

    pub fn metadata(account: &str) -> Result<KeychainMetadata, KeychainError> {
        // No public API for "get length without fetching the bytes" in
        // security-framework, so we fetch then derive. The bytes are
        // dropped immediately (only the length escapes the function).
        // v0.2 should hold metadata reads through ApiKey so the bytes
        // inherit ZeroizeOnDrop on the way out.
        let bytes = passwords::get_generic_password(KEYCHAIN_SERVICE, account)
            .map_err(|e| classify(e, account))?;
        let length = bytes.len();
        drop(bytes);
        Ok(KeychainMetadata {
            service: KEYCHAIN_SERVICE.to_string(),
            account: account.to_string(),
            length,
        })
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::*;

    pub fn store(_account: &str, _secret: &[u8]) -> Result<(), KeychainError> {
        Err(KeychainError::Unsupported)
    }
    pub fn fetch(_account: &str) -> Result<ApiKey, KeychainError> {
        Err(KeychainError::Unsupported)
    }
    pub fn delete(_account: &str) -> Result<(), KeychainError> {
        Err(KeychainError::Unsupported)
    }
    pub fn metadata(_account: &str) -> Result<KeychainMetadata, KeychainError> {
        Err(KeychainError::Unsupported)
    }
}

/// Store a secret under the cuttle Keychain service. Overwrites any
/// existing entry with the same account name (security-framework's
/// `set_generic_password` is upsert by design).
pub fn store_in_keychain(account: &str, secret: &[u8]) -> Result<(), KeychainError> {
    imp::store(account, secret)
}

/// Fetch a secret as an `ApiKey`. The bytes inherit ApiKey's
/// `ZeroizeOnDrop` so they're zeroed when the returned ApiKey drops.
pub fn fetch_from_keychain(account: &str) -> Result<ApiKey, KeychainError> {
    imp::fetch(account)
}

/// Delete the entry. `NotFound` is surfaced rather than swallowed so
/// `cuttle credential delete` can report "nothing to delete" cleanly.
pub fn delete_from_keychain(account: &str) -> Result<(), KeychainError> {
    imp::delete(account)
}

/// Return non-secret metadata about a stored entry. Used by
/// `cuttle credential show` to confirm an entry exists + its size
/// without printing the secret.
pub fn keychain_metadata(account: &str) -> Result<KeychainMetadata, KeychainError> {
    imp::metadata(account)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a unique-per-test account name so concurrent test runs
    /// + repeated test runs don't collide in the user's Keychain.
    fn unique_account(suffix: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("CUTTLE_TEST_{suffix}_{nanos}")
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn store_returns_unsupported_off_macos() {
        let r = store_in_keychain("x", b"y");
        assert!(matches!(r, Err(KeychainError::Unsupported)));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn round_trip_store_fetch_delete() {
        let account = unique_account("round_trip");
        let secret = b"sk-test-keychain-round-trip-1234567890";

        // Store.
        store_in_keychain(&account, secret).expect("store should succeed");

        // Fetch returns the same bytes.
        let fetched = fetch_from_keychain(&account).expect("fetch should succeed");
        assert_eq!(fetched.consume(), secret);

        // Metadata reports the right length.
        let meta = keychain_metadata(&account).expect("metadata should succeed");
        assert_eq!(meta.account, account);
        assert_eq!(meta.length, secret.len());
        assert_eq!(meta.service, KEYCHAIN_SERVICE);

        // Delete.
        delete_from_keychain(&account).expect("delete should succeed");

        // Fetch after delete returns NotFound. Manual match instead of
        // matches! + {r:?} formatter: ApiKey deliberately has no Debug
        // derive (don't leak secret bytes via panic output) so the
        // Result's Debug bound is unsatisfiable.
        match fetch_from_keychain(&account) {
            Err(KeychainError::NotFound { .. }) => {}
            Err(other) => panic!("expected NotFound after delete, got {other:?}"),
            Ok(_) => panic!("expected NotFound after delete, got Ok"),
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn store_overwrites_existing_entry() {
        let account = unique_account("overwrite");

        store_in_keychain(&account, b"first-value").unwrap();
        store_in_keychain(&account, b"second-value").unwrap();
        let fetched = fetch_from_keychain(&account).unwrap();
        assert_eq!(fetched.consume(), b"second-value");

        // Cleanup.
        let _ = delete_from_keychain(&account);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn fetch_missing_returns_not_found() {
        let account = unique_account("missing");
        // Don't store anything; fetch must report NotFound.
        let r = fetch_from_keychain(&account);
        assert!(matches!(r, Err(KeychainError::NotFound { .. })));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn delete_missing_returns_not_found() {
        let account = unique_account("delete_missing");
        let r = delete_from_keychain(&account);
        assert!(matches!(r, Err(KeychainError::NotFound { .. })));
    }

    #[test]
    fn metadata_does_not_expose_secret_bytes() {
        // Compile-time discipline: KeychainMetadata MUST NOT have a
        // field that exposes the secret. If a future contributor adds
        // one, this test forces them to also remove this assertion +
        // confirms they thought about it.
        // The struct is exhaustive (no #[non_exhaustive]) so we can
        // pattern-match every field here.
        let m = KeychainMetadata {
            service: "s".to_string(),
            account: "a".to_string(),
            length: 0,
        };
        let KeychainMetadata {
            service: _,
            account: _,
            length: _,
        } = m;
    }
}
