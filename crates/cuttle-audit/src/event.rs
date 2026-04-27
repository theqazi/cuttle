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
    /// Operator turn in a `cuttle session start` REPL. Carries digest +
    /// length only; content lives in the per-session transcript file
    /// (`~/.cuttle/sessions/<id>/transcript.jsonl`). The audit log
    /// proves the fact + length of each turn without holding PII.
    UserPrompt {
        #[serde(with = "serde_bytes_32_pub")]
        content_sha256: [u8; 32],
        length: usize,
    },
    /// Assistant turn in a `cuttle session start` REPL. Carries the
    /// same digest + length shape as UserPrompt plus token-usage
    /// metadata reported by the model so telemetry can aggregate
    /// per-session token spend.
    AssistantResponse {
        #[serde(with = "serde_bytes_32_pub")]
        content_sha256: [u8; 32],
        length: usize,
        input_tokens: u32,
        output_tokens: u32,
    },
}

mod serde_bytes_32_pub {
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
    use serde_json;

    #[test]
    fn user_prompt_round_trips_through_serde() {
        let ev = AuditEvent::UserPrompt {
            content_sha256: [7u8; 32],
            length: 42,
        };
        let s = serde_json::to_string(&ev).unwrap();
        let restored: AuditEvent = serde_json::from_str(&s).unwrap();
        match restored {
            AuditEvent::UserPrompt {
                content_sha256,
                length,
            } => {
                assert_eq!(content_sha256, [7u8; 32]);
                assert_eq!(length, 42);
            }
            other => panic!("expected UserPrompt, got {other:?}"),
        }
    }

    #[test]
    fn assistant_response_round_trips_through_serde() {
        let ev = AuditEvent::AssistantResponse {
            content_sha256: [9u8; 32],
            length: 100,
            input_tokens: 50,
            output_tokens: 75,
        };
        let s = serde_json::to_string(&ev).unwrap();
        let restored: AuditEvent = serde_json::from_str(&s).unwrap();
        match restored {
            AuditEvent::AssistantResponse {
                content_sha256,
                length,
                input_tokens,
                output_tokens,
            } => {
                assert_eq!(content_sha256, [9u8; 32]);
                assert_eq!(length, 100);
                assert_eq!(input_tokens, 50);
                assert_eq!(output_tokens, 75);
            }
            other => panic!("expected AssistantResponse, got {other:?}"),
        }
    }

    #[test]
    fn assistant_response_serializes_with_kind_tag() {
        let ev = AuditEvent::AssistantResponse {
            content_sha256: [0u8; 32],
            length: 1,
            input_tokens: 1,
            output_tokens: 1,
        };
        let s = serde_json::to_string(&ev).unwrap();
        assert!(s.contains("\"kind\":\"AssistantResponse\""), "{s}");
    }

    #[test]
    fn user_prompt_serializes_digest_as_byte_array() {
        let ev = AuditEvent::UserPrompt {
            content_sha256: [255u8; 32],
            length: 0,
        };
        let s = serde_json::to_string(&ev).unwrap();
        // 32 copies of 255 should appear as a JSON array, not as a hex string.
        assert!(s.contains("[255,255"), "expected byte array shape, got {s}");
    }
}
