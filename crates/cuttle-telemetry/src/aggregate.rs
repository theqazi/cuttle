//! Aggregate summaries computed from an audit-event stream.
//!
//! `summarize()` is a single pass over the event iterator that builds:
//! - `ToolDispatchSummary`: per-tool dispatch counts.
//! - `PolicyDecisionSummary`: counts bucketed by decision class
//!   (`Allow` / `Warn` / `Deny` / `Prompt`); unrecognized strings bucket
//!   into `other` so that a future decision class doesn't silently drop.
//! - `OverrideSummary`: counts of every operator-action variant that
//!   counts as F-Cuttle-DISABLE evidence per D-25.
//! - `AbandonSummary`: per-tool `dispatch − completed` count.
//!   "Completed" = a ToolResult landed for the same tool name (regardless
//!   of `success`). The heuristic is bucket-by-tool because v0.1
//!   audit events do not yet carry per-call IDs (v0.2 will refine).

use cuttle_audit::AuditEvent;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolDispatchSummary {
    /// `BTreeMap` so output is alphabetically stable across runs (matters
    /// for the operator who diffs telemetry between sessions).
    pub by_tool: BTreeMap<String, u64>,
    pub total: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicyDecisionSummary {
    pub allow: u64,
    pub warn: u64,
    pub deny: u64,
    pub prompt: u64,
    pub other: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct OverrideSummary {
    pub gate_disabled: u64,
    pub chain_rotated: u64,
    pub restored_from_backup: u64,
    pub snapshot_block_overridden: u64,
    /// Counts only the `enabled: true` toggle (the loosening direction);
    /// matches F-Cuttle-DISABLE evidence per D-25.
    pub keychain_always_allow_enabled: u64,
    /// Counts the `enabled: false` toggle separately (operator restoring
    /// stricter posture is NOT F-Cuttle-DISABLE evidence but is still
    /// useful for posture-change visibility).
    pub keychain_always_allow_disabled: u64,
}

/// Per-session conversation summary. Counts of UserPrompt / AssistantResponse
/// events plus aggregate token spend (sum across all AssistantResponse events).
/// Aggregate-only: content lives in the per-session transcript file, not in
/// the audit log; this struct never holds prompt or response text.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionSummary {
    pub user_prompts: u64,
    pub assistant_responses: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct AbandonSummary {
    /// `dispatch_count − completed_count` per tool, clamped to ≥0. A
    /// tool with completed > dispatch (impossible if the audit log is
    /// internally consistent) clamps to 0; this is per-D-25 robustness
    /// rather than silently producing a negative count. Sums tracked
    /// internally; this map only carries net abandons.
    pub by_tool: BTreeMap<String, u64>,
    pub total: u64,
}

#[derive(Debug, Default)]
struct SummarizeState {
    dispatch: ToolDispatchSummary,
    decisions: PolicyDecisionSummary,
    overrides: OverrideSummary,
    session: SessionSummary,
    completed_by_tool: BTreeMap<String, u64>,
}

/// Single-pass summary over the event stream.
///
/// Returns `(dispatch, decisions, overrides, abandons, session)`.
/// Single pass so even a 100k-entry audit log doesn't pay for a scan
/// per metric.
pub fn summarize<'a, I>(
    events: I,
) -> (
    ToolDispatchSummary,
    PolicyDecisionSummary,
    OverrideSummary,
    AbandonSummary,
    SessionSummary,
)
where
    I: IntoIterator<Item = &'a AuditEvent>,
{
    let mut state = SummarizeState::default();

    for ev in events {
        match ev {
            AuditEvent::ToolDispatch { tool_name, .. } => {
                *state.dispatch.by_tool.entry(tool_name.clone()).or_insert(0) += 1;
                state.dispatch.total += 1;
            }
            AuditEvent::PolicyDecision { decision, .. } => match decision.as_str() {
                "Allow" => state.decisions.allow += 1,
                "Warn" => state.decisions.warn += 1,
                "Deny" => state.decisions.deny += 1,
                "Prompt" => state.decisions.prompt += 1,
                _ => state.decisions.other += 1,
            },
            AuditEvent::ToolResult { tool_name, .. } => {
                *state
                    .completed_by_tool
                    .entry(tool_name.clone())
                    .or_insert(0) += 1;
            }
            AuditEvent::GateDisabled { .. } => state.overrides.gate_disabled += 1,
            AuditEvent::ChainRotated { .. } => state.overrides.chain_rotated += 1,
            AuditEvent::RestoredFromBackup { .. } => {
                state.overrides.restored_from_backup += 1;
            }
            AuditEvent::SnapshotBlockOverridden { .. } => {
                state.overrides.snapshot_block_overridden += 1;
            }
            AuditEvent::KeychainAlwaysAllowToggled { enabled: true } => {
                state.overrides.keychain_always_allow_enabled += 1;
            }
            AuditEvent::KeychainAlwaysAllowToggled { enabled: false } => {
                state.overrides.keychain_always_allow_disabled += 1;
            }
            AuditEvent::UserPrompt { .. } => state.session.user_prompts += 1,
            AuditEvent::AssistantResponse {
                input_tokens,
                output_tokens,
                ..
            } => {
                state.session.assistant_responses += 1;
                state.session.total_input_tokens += *input_tokens as u64;
                state.session.total_output_tokens += *output_tokens as u64;
            }
        }
    }

    let mut abandons = AbandonSummary::default();
    for (tool, dispatched) in &state.dispatch.by_tool {
        let completed = state.completed_by_tool.get(tool).copied().unwrap_or(0);
        let net = dispatched.saturating_sub(completed);
        if net > 0 {
            abandons.by_tool.insert(tool.clone(), net);
            abandons.total += net;
        }
    }

    (
        state.dispatch,
        state.decisions,
        state.overrides,
        abandons,
        state.session,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttle_audit::AuditEvent;

    fn dispatch(tool: &str) -> AuditEvent {
        AuditEvent::ToolDispatch {
            tool_name: tool.into(),
            argument_summary: "x".into(),
        }
    }

    fn result(tool: &str, success: bool) -> AuditEvent {
        AuditEvent::ToolResult {
            tool_name: tool.into(),
            length: 0,
            content_sha256: None,
            success,
        }
    }

    fn decision(d: &str) -> AuditEvent {
        AuditEvent::PolicyDecision {
            tool_name: "t".into(),
            decision: d.into(),
            reason: None,
            attestation_provenance: None,
        }
    }

    #[test]
    fn dispatch_summary_counts_by_tool_name() {
        let events = [dispatch("bash"), dispatch("bash"), dispatch("read")];
        let (d, _, _, _, _) = summarize(events.iter());
        assert_eq!(d.by_tool.get("bash"), Some(&2));
        assert_eq!(d.by_tool.get("read"), Some(&1));
        assert_eq!(d.total, 3);
    }

    #[test]
    fn decision_summary_buckets_each_class() {
        let events = [
            decision("Allow"),
            decision("Allow"),
            decision("Warn"),
            decision("Deny"),
            decision("Prompt"),
        ];
        let (_, d, _, _, _) = summarize(events.iter());
        assert_eq!(d.allow, 2);
        assert_eq!(d.warn, 1);
        assert_eq!(d.deny, 1);
        assert_eq!(d.prompt, 1);
        assert_eq!(d.other, 0);
    }

    #[test]
    fn decision_summary_buckets_unknown_into_other() {
        let events = [decision("FutureClass")];
        let (_, d, _, _, _) = summarize(events.iter());
        assert_eq!(d.other, 1);
        assert_eq!(d.allow, 0);
    }

    #[test]
    fn override_summary_counts_each_event_type() {
        let events = [
            AuditEvent::GateDisabled {
                rule_id: "r".into(),
                operator_reason: "x".into(),
            },
            AuditEvent::ChainRotated {
                old_chain_head: [0u8; 32],
                new_chain_head: [1u8; 32],
                operator_reason: "x".into(),
            },
            AuditEvent::RestoredFromBackup {
                operator_reason: "x".into(),
            },
            AuditEvent::SnapshotBlockOverridden {
                operator_reason: "x".into(),
            },
            AuditEvent::KeychainAlwaysAllowToggled { enabled: true },
            AuditEvent::KeychainAlwaysAllowToggled { enabled: false },
        ];
        let (_, _, o, _, _) = summarize(events.iter());
        assert_eq!(o.gate_disabled, 1);
        assert_eq!(o.chain_rotated, 1);
        assert_eq!(o.restored_from_backup, 1);
        assert_eq!(o.snapshot_block_overridden, 1);
        assert_eq!(o.keychain_always_allow_enabled, 1);
        assert_eq!(o.keychain_always_allow_disabled, 1);
    }

    #[test]
    fn abandon_summary_detects_dispatch_without_result() {
        let events = [
            dispatch("bash"),
            dispatch("bash"),
            result("bash", true),
            // 2 dispatches − 1 result = 1 abandon.
        ];
        let (_, _, _, a, _) = summarize(events.iter());
        assert_eq!(a.by_tool.get("bash"), Some(&1));
        assert_eq!(a.total, 1);
    }

    #[test]
    fn abandon_summary_clamps_when_more_results_than_dispatches() {
        // Audit log inconsistency (shouldn't happen) clamps to 0
        // rather than producing a negative or nonsense entry.
        let events = [dispatch("bash"), result("bash", true), result("bash", true)];
        let (_, _, _, a, _) = summarize(events.iter());
        assert_eq!(a.by_tool.get("bash"), None);
        assert_eq!(a.total, 0);
    }

    #[test]
    fn abandon_summary_counts_failed_results_as_completed() {
        // success=false is still "completed" — the tool returned, just with
        // an error. abandon counts only "no result at all".
        let events = [dispatch("bash"), result("bash", false)];
        let (_, _, _, a, _) = summarize(events.iter());
        assert_eq!(a.total, 0);
    }

    #[test]
    fn empty_event_stream_produces_all_zero_summaries() {
        let events: Vec<AuditEvent> = vec![];
        let (d, p, o, a, _) = summarize(events.iter());
        assert_eq!(d.total, 0);
        assert_eq!(p, PolicyDecisionSummary::default());
        assert_eq!(o, OverrideSummary::default());
        assert_eq!(a.total, 0);
    }

    fn user_prompt(digest: u8, len: usize) -> AuditEvent {
        AuditEvent::UserPrompt {
            content_sha256: [digest; 32],
            length: len,
        }
    }

    fn assistant_response(digest: u8, len: usize, in_t: u32, out_t: u32) -> AuditEvent {
        AuditEvent::AssistantResponse {
            content_sha256: [digest; 32],
            length: len,
            input_tokens: in_t,
            output_tokens: out_t,
        }
    }

    #[test]
    fn session_summary_counts_user_and_assistant_turns() {
        let events = [
            user_prompt(1, 10),
            assistant_response(2, 20, 5, 15),
            user_prompt(3, 30),
            assistant_response(4, 40, 25, 35),
        ];
        let (_, _, _, _, s) = summarize(events.iter());
        assert_eq!(s.user_prompts, 2);
        assert_eq!(s.assistant_responses, 2);
        assert_eq!(s.total_input_tokens, 30);
        assert_eq!(s.total_output_tokens, 50);
    }

    #[test]
    fn session_summary_zero_when_no_turns() {
        let events = [dispatch("bash")];
        let (_, _, _, _, s) = summarize(events.iter());
        assert_eq!(s, SessionSummary::default());
    }

    #[test]
    fn dispatch_summary_uses_btree_for_stable_ordering() {
        let events = [dispatch("zsh"), dispatch("ash"), dispatch("bash")];
        let (d, _, _, _, _) = summarize(events.iter());
        let keys: Vec<&String> = d.by_tool.keys().collect();
        // BTreeMap orders alphabetically.
        assert_eq!(keys, vec!["ash", "bash", "zsh"]);
    }
}
