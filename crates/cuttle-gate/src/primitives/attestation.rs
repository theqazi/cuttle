//! [`AttestationBody`]: gate-bypass evidence with operator-vs-model provenance.
//!
//! Per `docs/PRD.md` §6.1.5 (T-001 attestation-provenance separation invariant)
//! and `docs/TDD.md` §2.4. The constructor for `Provenance::Tty` requires a
//! [`crate::TtyInputCap`] witness; the constructor for `Provenance::Model`
//! requires no capability (model output is freely producible).
//!
//! **Limitation explicitly disclaimed** per WV-06 / WV-07: the type system
//! enforces bytes-typed-by-operator vs bytes-emitted-by-model separation. It
//! does NOT enforce operator-INTENT vs operator-FATIGUE-KEYPRESS. F-Cuttle-FATIGUE
//! per `docs/falsifiers.md` measures the residual.

use crate::capabilities::TtyInputCap;
use serde::{Deserialize, Serialize};

/// Provenance discriminant. `serde` does NOT use `default` or `other` per
/// D-2026-04-26-18: missing or unknown variant fails deserialization.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Bytes typed by the operator at the TTY (or written to operator-controlled
    /// surfaces like `~/.claude/CLAUDE.md`).
    Tty,
    /// Bytes emitted by the model.
    Model,
}

/// Attestation body submitted to the policy gate as evidence.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttestationBody {
    provenance: Provenance,
    content: String,
}

impl AttestationBody {
    /// Construct an attestation body with TTY provenance.
    ///
    /// Requires a `&TtyInputCap` witness; only the `cuttle-input` crate holds
    /// the capability to call this. Untrusted modules (skills loader, model
    /// client) cannot construct `Provenance::Tty` attestations.
    pub fn from_tty_input(_cap: &TtyInputCap, content: String) -> Self {
        Self {
            provenance: Provenance::Tty,
            content,
        }
    }

    /// Construct an attestation body with Model provenance.
    ///
    /// No capability required; the model client produces these freely. The
    /// gate's policy evaluation rejects `Provenance::Model` attestations as
    /// evidence by matching on the `provenance` field.
    pub fn from_model_output(content: String) -> Self {
        Self {
            provenance: Provenance::Model,
            content,
        }
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::__internal_input_cap_factory;

    #[test]
    fn tty_provenance_requires_cap() {
        let cap = __internal_input_cap_factory::issue();
        let a = AttestationBody::from_tty_input(&cap, "operator reason".to_string());
        assert_eq!(a.provenance(), &Provenance::Tty);
        assert_eq!(a.content(), "operator reason");
    }

    #[test]
    fn model_provenance_no_cap() {
        let a = AttestationBody::from_model_output("model suggested".to_string());
        assert_eq!(a.provenance(), &Provenance::Model);
    }

    #[test]
    fn round_trip_preserves_provenance() {
        let cap = __internal_input_cap_factory::issue();
        let original = AttestationBody::from_tty_input(&cap, "op".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let restored: AttestationBody = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.provenance(), &Provenance::Tty);
        assert_eq!(restored.content(), "op");
    }

    #[test]
    fn rejects_unknown_provenance() {
        let bad = r#"{"provenance":"Spoofed","content":"x"}"#;
        let r: Result<AttestationBody, _> = serde_json::from_str(bad);
        assert!(
            r.is_err(),
            "expected deserialization to reject unknown variant"
        );
    }

    #[test]
    fn rejects_missing_provenance() {
        let bad = r#"{"content":"missing provenance"}"#;
        let r: Result<AttestationBody, _> = serde_json::from_str(bad);
        assert!(
            r.is_err(),
            "expected deserialization to reject missing field"
        );
    }
}
