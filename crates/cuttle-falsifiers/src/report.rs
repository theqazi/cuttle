//! [`FalsifierReport`]: per-predicate evaluation result.

use serde::{Deserialize, Serialize};

/// Status of a falsifier predicate after evaluation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FalsifierStatus {
    /// No evidence the predicate has fired during the evaluation window.
    Clean,
    /// Predicate fired; the named claim is partially or fully refuted.
    Fired,
    /// Insufficient data to evaluate (e.g., evaluation window too short).
    InsufficientData,
}

/// Per-predicate evaluation result.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FalsifierReport {
    pub predicate_id: String,
    pub status: FalsifierStatus,
    pub evidence_count: u64,
    pub threshold: u64,
    pub notes: String,
}
