//! [`Decision`]: Allow / Warn / Deny / Prompt graduation per D-2026-04-26-21.
//!
//! Per `docs/TDD.md` §3.4. Resolves PRD v3 §10 OQ-9 with Carlos's "configurable
//! risk dial" lens: graduated decisions let the operator opt specific tools into
//! Warn-not-Deny while preserving deny-by-default for unmatched dispatches.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Policy-gate decision returned to the runtime.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Decision {
    /// Tool executes; audit-log records the call.
    Allow,
    /// Tool executes; audit-log records the call AND the warning reason;
    /// `cuttle telemetry` counts warns separately per PRD v3 §6.1.6.
    Warn { reason: String },
    /// Tool does NOT execute; audit-log records the deny reason + the optional
    /// `suggested_exception` so the deny path surfaces the operator option per
    /// PRD v3 §8 case 8.
    Deny {
        reason: String,
        suggested_exception: Option<ExceptionSuggestion>,
    },
    /// Ask operator at TTY; non-interactive mode collapses to `Deny` per
    /// PRD v3 §8 case 3.
    Prompt { question: String },
}

/// Exception suggestion attached to a `Deny` decision. Operator can fill in
/// the required evidence fields and resubmit through Option C escape-hatch
/// per PRD v3 §6.1.3.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExceptionSuggestion {
    pub rule_id: String,
    pub required_fields: Vec<String>,
}

/// Errors returned from `dispatch()`. The `Denied` variant carries the
/// human-readable reason and the optional exception suggestion.
#[derive(Error, Debug)]
pub enum GateError {
    #[error("policy denied tool: {reason}")]
    Denied {
        reason: String,
        suggested_exception: Option<ExceptionSuggestion>,
    },
    #[error("audit-log write failed: {0}")]
    Audit(String),
    #[error("policy evaluation failed: {0}")]
    Policy(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_round_trip() {
        let d = Decision::Allow;
        let json = serde_json::to_string(&d).unwrap();
        let r: Decision = serde_json::from_str(&json).unwrap();
        matches!(r, Decision::Allow);
    }

    #[test]
    fn deny_with_suggestion_round_trip() {
        let d = Decision::Deny {
            reason: "Destructive shell".to_string(),
            suggested_exception: Some(ExceptionSuggestion {
                rule_id: "bash-destructive-shell".to_string(),
                required_fields: vec![
                    "target_enumeration".to_string(),
                    "system_path_allowlist".to_string(),
                    "why".to_string(),
                ],
            }),
        };
        let json = serde_json::to_string(&d).unwrap();
        let r: Decision = serde_json::from_str(&json).unwrap();
        match r {
            Decision::Deny {
                reason,
                suggested_exception: Some(s),
            } => {
                assert_eq!(reason, "Destructive shell");
                assert_eq!(s.rule_id, "bash-destructive-shell");
                assert_eq!(s.required_fields.len(), 3);
            }
            _ => panic!("expected Deny with suggestion"),
        }
    }
}
