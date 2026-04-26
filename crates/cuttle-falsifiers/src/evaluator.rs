//! Per-predicate evaluators that scan the audit log for evidence.
//!
//! v0.1 ships the data-collection side per BP-02 / D-25; the automated
//! periodic evaluator is v0.2 scope. v0.1 evaluators run on-demand via
//! `cuttle telemetry --falsifier-eval`.

use crate::report::{FalsifierReport, FalsifierStatus};
use cuttle_audit::AuditEvent;

/// Iterate over an audit-event stream and count F-Cuttle-DISABLE evidence.
/// Per `docs/falsifiers.md` F-Cuttle-DISABLE expanded scope (D-25):
/// - GateDisabled events (any count ≥ 1)
/// - ChainRotated events (any count ≥ 1)
/// - RestoredFromBackup events (any count ≥ 1)
/// - SnapshotBlockOverridden events (count ≥ 3 in window)
/// - KeychainAlwaysAllowToggled { enabled: true } (any count ≥ 1)
pub fn evaluate_disable<'a, I>(events: I) -> FalsifierReport
where
    I: IntoIterator<Item = &'a AuditEvent>,
{
    let mut hard_count = 0u64;
    let mut snapshot_overrides = 0u64;
    for ev in events {
        match ev {
            AuditEvent::GateDisabled { .. }
            | AuditEvent::ChainRotated { .. }
            | AuditEvent::RestoredFromBackup { .. } => {
                hard_count += 1;
            }
            AuditEvent::SnapshotBlockOverridden { .. } => {
                snapshot_overrides += 1;
            }
            AuditEvent::KeychainAlwaysAllowToggled { enabled: true } => {
                hard_count += 1;
            }
            _ => {}
        }
    }

    // Predicate fires if hard_count >= 1 OR snapshot_overrides >= 3.
    let snapshot_threshold = 3u64;
    let fired = hard_count >= 1 || snapshot_overrides >= snapshot_threshold;
    let total_evidence = hard_count + snapshot_overrides;

    FalsifierReport {
        predicate_id: "F-Cuttle-DISABLE".into(),
        status: if fired {
            FalsifierStatus::Fired
        } else {
            FalsifierStatus::Clean
        },
        evidence_count: total_evidence,
        threshold: 1,
        notes: format!(
            "hard_events={}, snapshot_overrides={} (threshold {}); per docs/falsifiers.md F-Cuttle-DISABLE",
            hard_count, snapshot_overrides, snapshot_threshold
        ),
    }
}

/// Count F-Cuttle-SNAPSHOT-DRIFT evidence: --override-snapshot-block
/// invocations during the evaluation window. Per `docs/falsifiers.md`,
/// the operator-review rubric (whether the override was wrong) runs
/// post-week and is not part of v0.1 automated evaluation.
pub fn evaluate_snapshot_drift<'a, I>(events: I, threshold_n: u64) -> FalsifierReport
where
    I: IntoIterator<Item = &'a AuditEvent>,
{
    let count = events
        .into_iter()
        .filter(|ev| matches!(ev, AuditEvent::SnapshotBlockOverridden { .. }))
        .count() as u64;
    let fired = count > threshold_n;
    FalsifierReport {
        predicate_id: "F-Cuttle-SNAPSHOT-DRIFT".into(),
        status: if fired {
            FalsifierStatus::Fired
        } else {
            FalsifierStatus::Clean
        },
        evidence_count: count,
        threshold: threshold_n,
        notes: format!(
            "override-snapshot-block invocations: {} (threshold N>{}); operator-review rubric (wrong-override count M) runs post-week; v0.1 reports the count only",
            count, threshold_n
        ),
    }
}

/// Count F-Cuttle-MEMORY-DRIFT evidence. v0.1 reports raw promotion-vs-reject
/// counts; the ratio R comparison and the normalize-against-operator-commit-rate
/// refinement (per D-25) are operator-side post-week analysis.
///
/// For v0.1, this evaluator scans audit events for ToolDispatch where the
/// tool name signals memory promotion or rejection. Production callers
/// register the canonical tool names (cuttle-cli wires; see TDD §6.2).
/// In tests we use the convention `memory-promote` / `memory-reject`.
pub fn evaluate_memory_drift<'a, I>(
    events: I,
    promote_tool_name: &str,
    reject_tool_name: &str,
    threshold_ratio: f64,
) -> FalsifierReport
where
    I: IntoIterator<Item = &'a AuditEvent>,
{
    let mut promote_count = 0u64;
    let mut reject_count = 0u64;
    for ev in events {
        if let AuditEvent::ToolDispatch { tool_name, .. } = ev {
            if tool_name == promote_tool_name {
                promote_count += 1;
            } else if tool_name == reject_tool_name {
                reject_count += 1;
            }
        }
    }
    let total = promote_count + reject_count;
    let ratio = if total == 0 {
        0.0
    } else {
        promote_count as f64 / total as f64
    };
    let status = if total == 0 {
        FalsifierStatus::InsufficientData
    } else if ratio >= threshold_ratio {
        FalsifierStatus::Fired
    } else {
        FalsifierStatus::Clean
    };
    FalsifierReport {
        predicate_id: "F-Cuttle-MEMORY-DRIFT".into(),
        status,
        evidence_count: total,
        threshold: (threshold_ratio * 100.0) as u64,
        notes: format!(
            "memory promote count: {}, reject count: {}, ratio: {:.2} (threshold R: {:.2}); per docs/falsifiers.md F-Cuttle-MEMORY-DRIFT",
            promote_count, reject_count, ratio, threshold_ratio
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gate_disable_event() -> AuditEvent {
        AuditEvent::GateDisabled {
            rule_id: "test".into(),
            operator_reason: "smoke".into(),
        }
    }

    fn snapshot_override_event() -> AuditEvent {
        AuditEvent::SnapshotBlockOverridden {
            operator_reason: "smoke".into(),
        }
    }

    fn promote_dispatch() -> AuditEvent {
        AuditEvent::ToolDispatch {
            tool_name: "memory-promote".into(),
            argument_summary: "topic1".into(),
        }
    }

    fn reject_dispatch() -> AuditEvent {
        AuditEvent::ToolDispatch {
            tool_name: "memory-reject".into(),
            argument_summary: "topic1".into(),
        }
    }

    #[test]
    fn disable_clean_with_no_events() {
        let events: Vec<AuditEvent> = vec![];
        let r = evaluate_disable(events.iter());
        assert_eq!(r.status, FalsifierStatus::Clean);
    }

    #[test]
    fn disable_fires_on_one_gate_disable() {
        let events = [gate_disable_event()];
        let r = evaluate_disable(events.iter());
        assert_eq!(r.status, FalsifierStatus::Fired);
        assert_eq!(r.evidence_count, 1);
    }

    #[test]
    fn disable_clean_on_two_snapshot_overrides() {
        let events = [snapshot_override_event(), snapshot_override_event()];
        let r = evaluate_disable(events.iter());
        // 2 < threshold 3 for snapshot overrides; no hard events.
        assert_eq!(r.status, FalsifierStatus::Clean);
    }

    #[test]
    fn disable_fires_on_three_snapshot_overrides() {
        let events = [
            snapshot_override_event(),
            snapshot_override_event(),
            snapshot_override_event(),
        ];
        let r = evaluate_disable(events.iter());
        assert_eq!(r.status, FalsifierStatus::Fired);
    }

    #[test]
    fn snapshot_drift_clean_below_threshold() {
        let events = [snapshot_override_event(), snapshot_override_event()];
        let r = evaluate_snapshot_drift(events.iter(), 3);
        assert_eq!(r.status, FalsifierStatus::Clean);
        assert_eq!(r.evidence_count, 2);
    }

    #[test]
    fn snapshot_drift_fires_above_threshold() {
        let events = [
            snapshot_override_event(),
            snapshot_override_event(),
            snapshot_override_event(),
            snapshot_override_event(),
        ];
        let r = evaluate_snapshot_drift(events.iter(), 3);
        assert_eq!(r.status, FalsifierStatus::Fired);
        assert_eq!(r.evidence_count, 4);
    }

    #[test]
    fn memory_drift_insufficient_with_no_events() {
        let events: Vec<AuditEvent> = vec![];
        let r = evaluate_memory_drift(events.iter(), "memory-promote", "memory-reject", 0.8);
        assert_eq!(r.status, FalsifierStatus::InsufficientData);
    }

    #[test]
    fn memory_drift_clean_below_ratio() {
        let events = [
            promote_dispatch(),
            reject_dispatch(),
            reject_dispatch(),
            reject_dispatch(),
        ];
        // promote=1, reject=3, ratio=0.25 < 0.8 threshold.
        let r = evaluate_memory_drift(events.iter(), "memory-promote", "memory-reject", 0.8);
        assert_eq!(r.status, FalsifierStatus::Clean);
    }

    #[test]
    fn memory_drift_fires_above_ratio() {
        let events = [
            promote_dispatch(),
            promote_dispatch(),
            promote_dispatch(),
            promote_dispatch(),
            reject_dispatch(),
        ];
        // promote=4, reject=1, ratio=0.8 >= 0.8 threshold.
        let r = evaluate_memory_drift(events.iter(), "memory-promote", "memory-reject", 0.8);
        assert_eq!(r.status, FalsifierStatus::Fired);
    }

    #[test]
    fn keychain_always_allow_counts_as_disable_evidence() {
        let events = [AuditEvent::KeychainAlwaysAllowToggled { enabled: true }];
        let r = evaluate_disable(events.iter());
        assert_eq!(r.status, FalsifierStatus::Fired);
    }

    #[test]
    fn keychain_disabling_always_allow_does_not_fire() {
        // Operator turns OFF always_allow -> NOT a F-Cuttle-DISABLE event
        // (operator restoring stricter posture).
        let events = [AuditEvent::KeychainAlwaysAllowToggled { enabled: false }];
        let r = evaluate_disable(events.iter());
        assert_eq!(r.status, FalsifierStatus::Clean);
    }
}
