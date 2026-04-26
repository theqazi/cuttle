//! [`AuditEvent`]: every event the gate or runtime can record.
//!
//! Per `docs/TDD.md` §5.1. The event carries provenance for everything
//! that matters to F-Cuttle-DISABLE / F-Cuttle-FATIGUE per `docs/falsifiers.md`.
//!
//! `serde` does NOT use `default` or `other` per D-2026-04-26-18: missing or
//! unknown variant fails deserialization.

use cuttle_gate::Provenance;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind")]
pub enum AuditEvent {
    /// Tool dispatch (every gate evaluation; the decision lives in
    /// PolicyDecision below).
    ToolDispatch {
        tool_name: String,
        argument_summary: String,
    },
    /// Policy gate decision recorded for the preceding ToolDispatch.
    PolicyDecision {
        tool_name: String,
        decision: String, // "Allow" | "Warn" | "Deny" | "Prompt"
        reason: Option<String>,
        attestation_provenance: Option<Provenance>,
    },
    /// Tool-result metadata. For `secret_bearing` tools only metadata is
    /// recorded (per D-28). For non-secret-bearing tools, `content_sha256`
    /// carries the digest.
    ToolResult {
        tool_name: String,
        length: usize,
        content_sha256: Option<[u8; 32]>,
        success: bool,
    },
    /// Operator disables a harness-mechanic gate (F-Cuttle-DISABLE evidence).
    GateDisabled {
        rule_id: String,
        operator_reason: String,
    },
    /// Audit-log chain rotation (F-Cuttle-DISABLE evidence per D-25 expanded scope).
    ChainRotated {
        old_chain_head: [u8; 32],
        new_chain_head: [u8; 32],
        operator_reason: String,
    },
    /// Operator invokes `--restored-from-backup` (F-Cuttle-DISABLE per §8 case 9).
    RestoredFromBackup { operator_reason: String },
    /// Operator invokes `--override-snapshot-block` (F-Cuttle-DISABLE evidence).
    SnapshotBlockOverridden { operator_reason: String },
    /// Keychain `always_allow` toggled (per D-24 BP-05 closure).
    KeychainAlwaysAllowToggled { enabled: bool },
}
