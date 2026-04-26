//! Cuttle falsifier-evaluator crate.
//!
//! Implements the data-collection side of the 7 sealed-falsifier predicates
//! per `docs/falsifiers.md` + `docs/TDD.md` §3.9 + §5.6 + BP-02 partial close
//! (D-2026-04-26-25 + D-2026-04-26-13).
//!
//! Per BP-02 / D-13: v0.1 ships predicates as data; the AUTOMATED EVALUATOR
//! is TDD-grade scope. This crate provides the per-predicate evaluator
//! callable on-demand via `cuttle telemetry --falsifier-eval` (cuttle-cli
//! wires the CLI surface).
//!
//! v0.1 evaluators implemented:
//! - F-Cuttle-DISABLE: counts (gate-disable + chain-rotate +
//!   restored-from-backup + override-snapshot + keychain-always-allow) events.
//! - F-Cuttle-SNAPSHOT-DRIFT: counts override-snapshot events and exposes
//!   the count for operator-review-rubric-based assessment.
//! - F-Cuttle-MEMORY-DRIFT: counts memory promotion vs reject events.
//!
//! v0.1 evaluators DEFERRED (need additional event emitters or external data):
//! - F-Cuttle-BEDROCK: needs toolkit baseline data + statistical test (R / Python).
//! - F-Cuttle-SUBSTRATE: needs abandon-point telemetry from cuttle-telemetry.
//! - F-Cuttle-OPTION-C: needs per-rule normalized bypass-rate computation.
//! - F-Cuttle-FATIGUE: needs per-attestation logging (separate fatigue-log).

pub mod evaluator;
pub mod report;

pub use evaluator::{evaluate_disable, evaluate_memory_drift, evaluate_snapshot_drift};
pub use report::{FalsifierReport, FalsifierStatus};
