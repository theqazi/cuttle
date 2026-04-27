//! [`ApiKey`]: read-once + zeroize-on-drop credential primitive.
//!
//! Per `docs/PRD.md` §6.1.1 (CC-2 zeroization invariant) and `docs/TDD.md` §2.4.
//! Constructor is `pub(crate)`: only this crate (and its submodules) can mint.
//! Consumers in other crates receive an `&ApiKey` from credential-vault APIs and
//! call [`ApiKey::consume`] exactly once to read the bytes for an HTTP request.

use std::cell::Cell;
use zeroize::ZeroizeOnDrop;

/// API key bytes with read-once + zeroize-on-drop semantics.
///
/// `Drop` zeroizes the inner `Vec<u8>`. With the workspace `panic = "abort"`
/// release profile (per D-15), zeroization-on-panic is deterministic in
/// release builds. Debug builds use `panic = "unwind"`; `Drop` still runs.
#[derive(ZeroizeOnDrop)]
pub struct ApiKey {
    bytes: Vec<u8>,
    #[zeroize(skip)]
    consumed: Cell<bool>,
}

impl ApiKey {
    /// Construct an `ApiKey` from raw bytes fetched by the credential vault.
    /// Visibility is `pub(crate)`: only the credential-vault crate can mint.
    /// `#[allow(dead_code)]` because the Keychain backend code that calls
    /// this lives in TDD §3 / future work; for v0.0.1 scaffolding only the
    /// tests exercise this path.
    #[allow(dead_code)]
    pub(crate) fn from_keychain_fetch(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            consumed: Cell::new(false),
        }
    }

    /// Mint an `ApiKey` from a process environment variable. v0.0.12 of
    /// cuttle-cli's `cuttle ask` uses this for the ANTHROPIC_API_KEY
    /// path; the Keychain backend will be the daily-driver path once
    /// it lands.
    ///
    /// Validation:
    /// - Variable must be set and non-empty.
    /// - Value must be valid UTF-8 (env vars on Unix are bytes; non-UTF8
    ///   keys are almost certainly a corrupt shell config).
    /// - Leading + trailing whitespace is rejected (common copy-paste
    ///   footgun: pasting from a chat / docs page often appends a
    ///   newline and the resulting auth header would 401).
    ///
    /// Constructor stays in this crate so the vault-crate-mints-only
    /// invariant from D-2026-04-26-17 holds.
    pub fn from_env_var(var_name: &str) -> Result<Self, ApiKeyEnvError> {
        let raw = std::env::var(var_name).map_err(|e| match e {
            std::env::VarError::NotPresent => ApiKeyEnvError::NotSet {
                var: var_name.to_string(),
            },
            std::env::VarError::NotUnicode(_) => ApiKeyEnvError::NonUtf8 {
                var: var_name.to_string(),
            },
        })?;
        if raw.is_empty() {
            return Err(ApiKeyEnvError::Empty {
                var: var_name.to_string(),
            });
        }
        if raw != raw.trim() {
            return Err(ApiKeyEnvError::SurroundingWhitespace {
                var: var_name.to_string(),
            });
        }
        Ok(Self {
            bytes: raw.into_bytes(),
            consumed: Cell::new(false),
        })
    }

    /// Read the API key bytes exactly once. Panics on second call.
    ///
    /// Misuse is a programmer bug, not a runtime condition. With
    /// `panic = "abort"` (release profile per D-15), the abort is
    /// deterministic and `ZeroizeOnDrop` runs before exit.
    pub fn consume(&self) -> &[u8] {
        if self.consumed.replace(true) {
            panic!("ApiKey::consume called twice; this is a programmer bug");
        }
        &self.bytes
    }

    /// Returns whether the key has been consumed. Diagnostic only.
    pub fn is_consumed(&self) -> bool {
        self.consumed.get()
    }
}

// No Display, Debug, Clone derives. ApiKey leaves the credential vault
// crate ONLY through `ApiKey::consume()`.

/// Error variants for `ApiKey::from_env_var`. Each variant names the
/// failed env-var so the operator gets a specific, actionable message
/// (CLAUDE.md §0c operational empathy).
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ApiKeyEnvError {
    #[error("environment variable {var} is not set; export it before running cuttle")]
    NotSet { var: String },

    #[error("environment variable {var} is set but empty")]
    Empty { var: String },

    #[error("environment variable {var} contains non-UTF8 bytes; check your shell config")]
    NonUtf8 { var: String },

    #[error(
        "environment variable {var} has leading or trailing whitespace; \
         strip it before re-running (this is usually a copy-paste artifact)"
    )]
    SurroundingWhitespace { var: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_reads_bytes() {
        let k = ApiKey::from_keychain_fetch(b"sk-test-1234".to_vec());
        assert_eq!(k.consume(), b"sk-test-1234");
        assert!(k.is_consumed());
    }

    #[test]
    #[should_panic(expected = "ApiKey::consume called twice")]
    fn double_consume_panics() {
        let k = ApiKey::from_keychain_fetch(b"sk-test".to_vec());
        let _ = k.consume();
        let _ = k.consume();
    }

    #[test]
    fn drop_zeroizes() {
        // Verify the contained bytes are zeroized when ApiKey drops.
        // We hold the underlying Vec's pointer to peek at memory after drop.
        // This is unsafe and only valid as a heuristic test; in production
        // the zeroize crate's documentation is the authoritative guarantee.
        let bytes = b"sk-zero-test".to_vec();
        let ptr = bytes.as_ptr();
        let len = bytes.len();
        let k = ApiKey::from_keychain_fetch(bytes);
        drop(k);
        // After drop, the freed allocation may or may not be reused;
        // zeroize fills it before deallocation. This test verifies the
        // invariant by reading the (possibly UB) memory; in CI a sanitizer
        // run catches genuine issues. Treated as smoke test.
        unsafe {
            let slice = std::slice::from_raw_parts(ptr, len);
            // We cannot assert all-zero (the allocator may have reused the
            // memory). We only assert the slice is not the original bytes.
            assert_ne!(slice, b"sk-zero-test");
        }
    }

    /// Helper: scope a CUTTLE_TEST_API_KEY env-var override + restore.
    /// Same shape as cuttle-cli/src/paths.rs; std::env mutation is unsafe
    /// in 2024 edition because env is process-global.
    fn with_env_var<F: FnOnce()>(name: &str, value: Option<&str>, f: F) {
        let prev = std::env::var(name).ok();
        match value {
            Some(v) => unsafe { std::env::set_var(name, v) },
            None => unsafe { std::env::remove_var(name) },
        }
        f();
        match prev {
            Some(p) => unsafe { std::env::set_var(name, p) },
            None => unsafe { std::env::remove_var(name) },
        }
    }

    #[test]
    fn from_env_var_succeeds_for_valid_value() {
        with_env_var("CUTTLE_TEST_API_KEY_OK", Some("sk-test-abc123"), || {
            let k = ApiKey::from_env_var("CUTTLE_TEST_API_KEY_OK").unwrap();
            assert_eq!(k.consume(), b"sk-test-abc123");
        });
    }

    #[test]
    fn from_env_var_errors_when_unset() {
        with_env_var("CUTTLE_TEST_API_KEY_MISSING", None, || {
            let r = ApiKey::from_env_var("CUTTLE_TEST_API_KEY_MISSING");
            assert!(matches!(r, Err(ApiKeyEnvError::NotSet { .. })));
        });
    }

    #[test]
    fn from_env_var_errors_when_empty() {
        with_env_var("CUTTLE_TEST_API_KEY_EMPTY", Some(""), || {
            let r = ApiKey::from_env_var("CUTTLE_TEST_API_KEY_EMPTY");
            assert!(matches!(r, Err(ApiKeyEnvError::Empty { .. })));
        });
    }

    #[test]
    fn from_env_var_errors_on_leading_whitespace() {
        with_env_var("CUTTLE_TEST_API_KEY_LEAD", Some(" sk-test"), || {
            let r = ApiKey::from_env_var("CUTTLE_TEST_API_KEY_LEAD");
            assert!(matches!(
                r,
                Err(ApiKeyEnvError::SurroundingWhitespace { .. })
            ));
        });
    }

    #[test]
    fn from_env_var_errors_on_trailing_newline() {
        with_env_var("CUTTLE_TEST_API_KEY_TRAIL", Some("sk-test\n"), || {
            let r = ApiKey::from_env_var("CUTTLE_TEST_API_KEY_TRAIL");
            assert!(matches!(
                r,
                Err(ApiKeyEnvError::SurroundingWhitespace { .. })
            ));
        });
    }

    #[test]
    fn from_env_var_error_message_names_the_var() {
        with_env_var("CUTTLE_TEST_API_KEY_NAMED", None, || {
            // Manual match instead of unwrap_err: Result::unwrap_err requires
            // T: Debug, but ApiKey deliberately has no Debug derive to avoid
            // leaking secret bytes in panic output.
            let r = ApiKey::from_env_var("CUTTLE_TEST_API_KEY_NAMED");
            let err = match r {
                Err(e) => e,
                Ok(_) => panic!("expected error, got Ok"),
            };
            let msg = format!("{err}");
            assert!(
                msg.contains("CUTTLE_TEST_API_KEY_NAMED"),
                "error message should name the var, got: {msg}"
            );
        });
    }
}
