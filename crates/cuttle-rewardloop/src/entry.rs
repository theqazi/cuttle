//! Registry entry types per `docs/TDD.md` §3 row L5.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EntryKind {
    AntiPattern,
    ValidatedPattern,
}

/// Provenance for a registry entry (session ID, model output that
/// triggered the proposal, score, operator-confirmation timestamp).
/// `serde` derives MUST NOT use `default` or `other` per D-2026-04-26-18.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntryProvenance {
    pub session_id: String,
    pub model_output_excerpt: String,
    pub score: f64,
    /// Set when the operator promotes the entry from pending to canonical.
    pub operator_confirmation_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegistryEntry {
    pub id: String,
    pub kind: EntryKind,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub provenance: EntryProvenance,
}

#[derive(Error, Debug)]
pub enum RegistryEntryError {
    #[error("registry entry id must be non-empty")]
    EmptyId,
    #[error("registry entry score out of range [0.0, 1.0]: {got}")]
    ScoreOutOfRange { got: f64 },
}

impl RegistryEntry {
    /// Validate basic invariants before committing to disk.
    pub fn validate(&self) -> Result<(), RegistryEntryError> {
        if self.id.is_empty() {
            return Err(RegistryEntryError::EmptyId);
        }
        if !(0.0..=1.0).contains(&self.provenance.score) {
            return Err(RegistryEntryError::ScoreOutOfRange {
                got: self.provenance.score,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry() -> RegistryEntry {
        RegistryEntry {
            id: "ap-001".into(),
            kind: EntryKind::AntiPattern,
            title: "Tool dispatch without rationale".into(),
            body: "Never dispatch a destructive shell command without a WHY.".into(),
            created_at: Utc::now(),
            provenance: EntryProvenance {
                session_id: "abcdef".into(),
                model_output_excerpt: "model proposed: never do X".into(),
                score: 0.92,
                operator_confirmation_at: None,
            },
        }
    }

    #[test]
    fn validate_passes_for_normal_entry() {
        let e = sample_entry();
        assert!(e.validate().is_ok());
    }

    #[test]
    fn validate_rejects_empty_id() {
        let mut e = sample_entry();
        e.id = "".into();
        assert!(matches!(e.validate(), Err(RegistryEntryError::EmptyId)));
    }

    #[test]
    fn validate_rejects_score_above_1() {
        let mut e = sample_entry();
        e.provenance.score = 1.5;
        assert!(matches!(
            e.validate(),
            Err(RegistryEntryError::ScoreOutOfRange { .. })
        ));
    }

    #[test]
    fn round_trip_serde() {
        let e = sample_entry();
        let json = serde_json::to_string(&e).unwrap();
        let restored: RegistryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.kind, EntryKind::AntiPattern);
    }
}
