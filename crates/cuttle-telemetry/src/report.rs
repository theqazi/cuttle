//! `TelemetryReport`: assembled aggregate + optional falsifier evaluations.
//!
//! Two ways to build a report:
//! - `TelemetryReport::from_events(events)` → aggregates only.
//! - `TelemetryReport::with_falsifiers(events, ratio)` → aggregates plus
//!   the three v0.1 falsifier evaluators (DISABLE, SNAPSHOT-DRIFT,
//!   MEMORY-DRIFT) per `cuttle-falsifiers`.
//!
//! The report is serde-serializable (operator gets JSON via `cuttle
//! telemetry --json`) and `Display`-formatted for the default human view.
//!
//! Per PRD §6.1.6 + D-04: nothing in this struct ever leaves the local
//! machine. The CLI surface in `cuttle-cli` writes to stdout / a file the
//! operator chose; there is no upload path.

use crate::aggregate::{
    summarize, AbandonSummary, OverrideSummary, PolicyDecisionSummary, ToolDispatchSummary,
};
use cuttle_audit::AuditEvent;
use cuttle_falsifiers::{
    evaluate_disable, evaluate_memory_drift, evaluate_snapshot_drift, FalsifierReport,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Default tool names used when promoting / rejecting auto-memory entries.
/// `cuttle-cli` overrides via config when registering canonical names; tests
/// and the default-config path use these strings.
pub const DEFAULT_MEMORY_PROMOTE_TOOL: &str = "memory-promote";
pub const DEFAULT_MEMORY_REJECT_TOOL: &str = "memory-reject";

/// Default snapshot-drift threshold N. Per docs/falsifiers.md this is a
/// sealed pre-registration value; bumping it requires a DECISIONS entry.
pub const DEFAULT_SNAPSHOT_DRIFT_THRESHOLD: u64 = 3;

/// Default memory-drift ratio threshold R. Per docs/falsifiers.md.
pub const DEFAULT_MEMORY_DRIFT_RATIO: f64 = 0.8;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TelemetryReport {
    pub dispatch: ToolDispatchSummary,
    pub decisions: PolicyDecisionSummary,
    pub overrides: OverrideSummary,
    pub abandons: AbandonSummary,
    /// Present only when `with_falsifiers()` was used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub falsifiers: Vec<FalsifierReport>,
}

impl TelemetryReport {
    pub fn from_events<'a, I>(events: I) -> Self
    where
        I: IntoIterator<Item = &'a AuditEvent>,
    {
        let (dispatch, decisions, overrides, abandons) = summarize(events);
        TelemetryReport {
            dispatch,
            decisions,
            overrides,
            abandons,
            falsifiers: Vec::new(),
        }
    }

    /// Build a report and additionally run the three v0.1 falsifier
    /// evaluators with default thresholds. Iterates the event slice
    /// FOUR times (once for `summarize`, once per evaluator); v0.0.10
    /// optimizes only after the live-CLI cost is measured.
    pub fn with_falsifiers(events: &[AuditEvent]) -> Self {
        let mut report = Self::from_events(events.iter());
        report.falsifiers.push(evaluate_disable(events.iter()));
        report.falsifiers.push(evaluate_snapshot_drift(
            events.iter(),
            DEFAULT_SNAPSHOT_DRIFT_THRESHOLD,
        ));
        report.falsifiers.push(evaluate_memory_drift(
            events.iter(),
            DEFAULT_MEMORY_PROMOTE_TOOL,
            DEFAULT_MEMORY_REJECT_TOOL,
            DEFAULT_MEMORY_DRIFT_RATIO,
        ));
        report
    }

    /// Render as JSON (sorted keys via serde_json's pretty-print). Used
    /// by `cuttle telemetry --json`.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl fmt::Display for TelemetryReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Cuttle telemetry report")?;
        writeln!(f, "=======================")?;
        writeln!(f)?;

        writeln!(f, "Tool dispatches (total {}):", self.dispatch.total)?;
        if self.dispatch.by_tool.is_empty() {
            writeln!(f, "  (none)")?;
        } else {
            for (tool, count) in &self.dispatch.by_tool {
                writeln!(f, "  {tool:32} {count}")?;
            }
        }
        writeln!(f)?;

        writeln!(f, "Policy decisions:")?;
        writeln!(f, "  Allow:  {}", self.decisions.allow)?;
        writeln!(f, "  Warn:   {}", self.decisions.warn)?;
        writeln!(f, "  Deny:   {}", self.decisions.deny)?;
        writeln!(f, "  Prompt: {}", self.decisions.prompt)?;
        if self.decisions.other > 0 {
            writeln!(
                f,
                "  Other:  {} (unknown decision class)",
                self.decisions.other
            )?;
        }
        writeln!(f)?;

        writeln!(f, "Operator overrides (F-Cuttle-DISABLE evidence):")?;
        writeln!(
            f,
            "  GateDisabled:                {}",
            self.overrides.gate_disabled
        )?;
        writeln!(
            f,
            "  ChainRotated:                {}",
            self.overrides.chain_rotated
        )?;
        writeln!(
            f,
            "  RestoredFromBackup:          {}",
            self.overrides.restored_from_backup
        )?;
        writeln!(
            f,
            "  SnapshotBlockOverridden:     {}",
            self.overrides.snapshot_block_overridden
        )?;
        writeln!(
            f,
            "  KeychainAlwaysAllow ON:      {}",
            self.overrides.keychain_always_allow_enabled
        )?;
        writeln!(
            f,
            "  KeychainAlwaysAllow OFF:     {}",
            self.overrides.keychain_always_allow_disabled
        )?;
        writeln!(f)?;

        writeln!(f, "Abandon points (total {}):", self.abandons.total)?;
        if self.abandons.by_tool.is_empty() {
            writeln!(f, "  (none — every dispatch returned)")?;
        } else {
            for (tool, count) in &self.abandons.by_tool {
                writeln!(f, "  {tool:32} {count}")?;
            }
        }

        if !self.falsifiers.is_empty() {
            writeln!(f)?;
            writeln!(f, "Falsifier evaluations:")?;
            for fr in &self.falsifiers {
                writeln!(
                    f,
                    "  {:30} status={:?} evidence={} threshold={}",
                    fr.predicate_id, fr.status, fr.evidence_count, fr.threshold
                )?;
                writeln!(f, "    notes: {}", fr.notes)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttle_falsifiers::FalsifierStatus;

    fn dispatch(tool: &str) -> AuditEvent {
        AuditEvent::ToolDispatch {
            tool_name: tool.into(),
            argument_summary: "x".into(),
        }
    }

    fn gate_disabled() -> AuditEvent {
        AuditEvent::GateDisabled {
            rule_id: "r1".into(),
            operator_reason: "smoke".into(),
        }
    }

    #[test]
    fn from_events_aggregates_without_falsifiers() {
        let events = [dispatch("bash"), gate_disabled()];
        let r = TelemetryReport::from_events(events.iter());
        assert_eq!(r.dispatch.total, 1);
        assert_eq!(r.overrides.gate_disabled, 1);
        assert!(r.falsifiers.is_empty());
    }

    #[test]
    fn with_falsifiers_runs_all_three_evaluators() {
        let events = [dispatch("memory-promote"), gate_disabled()];
        let r = TelemetryReport::with_falsifiers(&events);
        assert_eq!(r.falsifiers.len(), 3);
        let ids: Vec<&str> = r
            .falsifiers
            .iter()
            .map(|f| f.predicate_id.as_str())
            .collect();
        assert!(ids.contains(&"F-Cuttle-DISABLE"));
        assert!(ids.contains(&"F-Cuttle-SNAPSHOT-DRIFT"));
        assert!(ids.contains(&"F-Cuttle-MEMORY-DRIFT"));
    }

    #[test]
    fn with_falsifiers_disable_fires_when_gate_disabled_event_present() {
        let events = [gate_disabled()];
        let r = TelemetryReport::with_falsifiers(&events);
        let disable = r
            .falsifiers
            .iter()
            .find(|f| f.predicate_id == "F-Cuttle-DISABLE")
            .unwrap();
        assert_eq!(disable.status, FalsifierStatus::Fired);
    }

    #[test]
    fn to_json_round_trips() {
        let events = [dispatch("bash")];
        let r = TelemetryReport::from_events(events.iter());
        let s = r.to_json().expect("json render");
        // Sanity-check shape: dispatched bash count present.
        assert!(s.contains("\"bash\""), "json missing bash: {s}");
        assert!(s.contains("\"total\": 1"), "json missing total: {s}");
    }

    #[test]
    fn display_renders_sections_in_stable_order() {
        let events = [dispatch("bash"), gate_disabled()];
        let r = TelemetryReport::with_falsifiers(&events);
        let s = format!("{r}");
        let dispatch_idx = s.find("Tool dispatches").unwrap();
        let decisions_idx = s.find("Policy decisions").unwrap();
        let overrides_idx = s.find("Operator overrides").unwrap();
        let abandons_idx = s.find("Abandon points").unwrap();
        let falsifiers_idx = s.find("Falsifier evaluations").unwrap();
        assert!(dispatch_idx < decisions_idx);
        assert!(decisions_idx < overrides_idx);
        assert!(overrides_idx < abandons_idx);
        assert!(abandons_idx < falsifiers_idx);
    }

    #[test]
    fn display_handles_empty_event_stream_without_panic() {
        let events: Vec<AuditEvent> = vec![];
        let r = TelemetryReport::from_events(events.iter());
        let s = format!("{r}");
        assert!(s.contains("(none)"));
    }

    #[test]
    fn json_excludes_falsifiers_field_when_empty() {
        let events: Vec<AuditEvent> = vec![];
        let r = TelemetryReport::from_events(events.iter());
        let s = r.to_json().unwrap();
        // skip_serializing_if drops the empty Vec, so the field name
        // should not appear at all.
        assert!(
            !s.contains("falsifiers"),
            "should skip empty falsifiers: {s}"
        );
    }
}
