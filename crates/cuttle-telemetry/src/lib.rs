//! Cuttle local-only telemetry surface.
//!
//! Per PRD §6.1.6 + D-2026-04-26-04 + D-2026-04-26-09: aggregates audit-log
//! events into operator-readable signal (gate-fire rates, override counts,
//! abandon points). NOTHING in this crate writes to a network socket. The
//! whole point is to close the "telemetry-dark tools cannot improve" gap
//! (Carlos's input) without compromising the no-phoning-home commitment.
//!
//! This crate is a LIBRARY. The `cuttle telemetry` CLI surface lives in
//! `cuttle-cli`; that crate composes:
//!
//!   `cuttle-audit::AuditChain::open(path)` →
//!   `aggregate::summarize(events)` →
//!   `report::TelemetryReport` →
//!   formatted text or JSON.
//!
//! For `--falsifier-eval` (per D-09), `cuttle-cli` additionally composes
//! `cuttle-falsifiers::evaluate_*` over the same event stream and merges
//! the predicate reports into `TelemetryReport`.
//!
//! v0.0.10 scope:
//! - Per-tool dispatch counts.
//! - Per-decision-class counts (Allow / Warn / Deny / Prompt).
//! - Per-override-event-type counts (the hard-evidence set for
//!   F-Cuttle-DISABLE per D-25).
//! - Abandon-point heuristic: dispatches without matching results,
//!   bucketed by tool name. The strict-correlation version (per-session,
//!   per-call-id) is a v0.2 refinement once the audit log carries call
//!   IDs.
//! - `TelemetryReport::with_falsifiers()` composes the three v0.1
//!   falsifier evaluators.

pub mod aggregate;
pub mod report;

pub use aggregate::{
    AbandonSummary, OverrideSummary, PolicyDecisionSummary, ToolDispatchSummary,
    summarize,
};
pub use report::TelemetryReport;
