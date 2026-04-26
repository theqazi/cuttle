//! [`SigningKey`]: per-session HMAC signing key.
//!
//! Per `docs/TDD.md` §3.7 (lockfile HMAC) and D-2026-04-26-23 (WV-02 closure).
//! The signing key lives in process memory only; it is NEVER written to disk.
//! Child processes inheriting file descriptors cannot regenerate a valid HMAC
//! without the signing key.

use rand::RngCore;
use zeroize::ZeroizeOnDrop;

/// Per-session HMAC signing key. 32 bytes of cryptographically secure randomness.
///
/// `ZeroizeOnDrop`: the key bytes are zeroized when the SigningKey drops.
/// Combined with `panic = "abort"` (per D-15), zeroization-on-panic is
/// deterministic in release builds.
///
/// Constructor is `pub(crate)`: only the runtime crate can mint at session
/// start. The key never leaves the runtime crate; consumers receive an
/// HMAC computation result, not the key itself.
#[derive(ZeroizeOnDrop)]
pub struct SigningKey {
    bytes: [u8; 32],
}

impl SigningKey {
    /// Mint a new random signing key. `pub(crate)`: only the runtime crate
    /// at session-start can call.
    /// `#[allow(dead_code)]` until `cuttle-cli` wires the session orchestrator.
    #[allow(dead_code)]
    pub(crate) fn new_random() -> Self {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        Self { bytes }
    }

    /// Borrow the key bytes for HMAC computation. `pub(crate)`: only the
    /// runtime crate's lockfile module needs this; outside callers use
    /// the `compute_hmac` / `verify_hmac` helpers in the lockfile module.
    pub(crate) fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

// No Display, Debug, Clone, Serialize derives. SigningKey never leaves
// the runtime crate's lockfile module.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_random_produces_32_bytes() {
        let k = SigningKey::new_random();
        assert_eq!(k.as_bytes().len(), 32);
    }

    #[test]
    fn two_keys_are_different() {
        let k1 = SigningKey::new_random();
        let k2 = SigningKey::new_random();
        assert_ne!(k1.as_bytes(), k2.as_bytes());
    }
}
