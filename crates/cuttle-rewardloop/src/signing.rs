//! Signed-provenance for registry entries per WV-04 / D-2026-04-26-13.
//!
//! Provenance signing key is operator-owned in v0.1 single-operator scope;
//! the registry chain is anti-forgetfulness and anti-drift, NOT anti-Sybil
//! against the operator-as-adversary (symmetric to the audit-log T-003 disclaimer).
//! The signing key zeroizes on drop; in-memory only.

use crate::entry::RegistryEntry;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use zeroize::ZeroizeOnDrop;

type HmacSha256 = Hmac<Sha256>;

/// Operator-owned signing key for registry-entry provenance.
/// In v0.1 single-operator scope: anti-forgetfulness, NOT anti-Sybil.
#[derive(ZeroizeOnDrop)]
pub struct ProvenanceSigningKey {
    bytes: [u8; 32],
}

impl ProvenanceSigningKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

/// Registry entry plus its HMAC signature.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignedEntry {
    pub entry: RegistryEntry,
    #[serde(with = "serde_bytes_32")]
    pub hmac: [u8; 32],
}

impl SignedEntry {
    /// Sign an entry with the provenance signing key.
    pub fn sign(entry: RegistryEntry, key: &ProvenanceSigningKey) -> Self {
        let serialized = serde_json::to_vec(&entry).expect("serializable");
        let mut mac =
            HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC accepts any key length");
        mac.update(&serialized);
        let hmac: [u8; 32] = mac.finalize().into_bytes().into();
        Self { entry, hmac }
    }

    /// Verify the HMAC signature with the provided key. Constant-time compare.
    pub fn verify(&self, key: &ProvenanceSigningKey) -> bool {
        let serialized = match serde_json::to_vec(&self.entry) {
            Ok(b) => b,
            Err(_) => return false,
        };
        let mut mac =
            HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC accepts any key length");
        mac.update(&serialized);
        let recomputed: [u8; 32] = mac.finalize().into_bytes().into();
        recomputed.ct_eq(&self.hmac).into()
    }
}

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
    use crate::entry::{EntryKind, EntryProvenance, RegistryEntry};
    use chrono::Utc;
    use rand::RngCore;

    fn fresh_key() -> ProvenanceSigningKey {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        ProvenanceSigningKey::from_bytes(bytes)
    }

    fn sample_entry() -> RegistryEntry {
        RegistryEntry {
            id: "vp-001".into(),
            kind: EntryKind::ValidatedPattern,
            title: "use TtyInputCap for promotion".into(),
            body: "...".into(),
            created_at: Utc::now(),
            provenance: EntryProvenance {
                session_id: "deadbeef".into(),
                model_output_excerpt: "...".into(),
                score: 0.85,
                operator_confirmation_at: None,
            },
        }
    }

    #[test]
    fn sign_then_verify_with_same_key() {
        let key = fresh_key();
        let signed = SignedEntry::sign(sample_entry(), &key);
        assert!(signed.verify(&key));
    }

    #[test]
    fn verify_fails_with_different_key() {
        let key1 = fresh_key();
        let key2 = fresh_key();
        let signed = SignedEntry::sign(sample_entry(), &key1);
        assert!(!signed.verify(&key2));
    }

    #[test]
    fn tampered_entry_fails_verification() {
        let key = fresh_key();
        let mut signed = SignedEntry::sign(sample_entry(), &key);
        signed.entry.body = "TAMPERED".into();
        assert!(!signed.verify(&key));
    }
}
