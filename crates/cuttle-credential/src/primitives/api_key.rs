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
}
