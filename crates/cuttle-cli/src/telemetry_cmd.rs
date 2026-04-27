//! `cuttle telemetry` subcommand handler.
//!
//! Wires:
//!   `cuttle_audit::read_entries_unverified(path)` →
//!   extract `entry.event` for each →
//!   `cuttle_telemetry::TelemetryReport::with_falsifiers(...)` (or
//!   `from_events(...)` when `--falsifier-eval` is absent) →
//!   render to stdout (text or JSON).
//!
//! HMAC verification is intentionally NOT done here: telemetry is a
//! read-only inspection surface and v0.0.11 doesn't yet require the
//! operator to surface a chain key. `cuttle audit verify` (v0.0.12+)
//! will own the verification path.

use crate::args::TelemetryArgs;
use crate::paths;
use cuttle_audit::AuditEvent;
use cuttle_telemetry::TelemetryReport;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TelemetryCmdError {
    #[error("could not resolve audit log path; pass --audit-log <PATH> or set CUTTLE_HOME")]
    NoAuditLogPath,

    #[error("audit log not found at {0}; run a Cuttle session first or pass --audit-log <PATH>")]
    AuditLogMissing(PathBuf),

    #[error("audit log read failed: {0}")]
    AuditRead(#[from] cuttle_audit::AuditError),

    #[error("rendering JSON failed: {0}")]
    JsonRender(#[from] serde_json::Error),

    #[error("writing report to stdout failed: {0}")]
    WriteFailed(#[from] std::io::Error),
}

pub fn run<W: Write>(args: &TelemetryArgs, out: &mut W) -> Result<(), TelemetryCmdError> {
    let path = match &args.audit_log {
        Some(p) => p.clone(),
        None => paths::default_audit_log_path().ok_or(TelemetryCmdError::NoAuditLogPath)?,
    };
    if !path.exists() {
        return Err(TelemetryCmdError::AuditLogMissing(path));
    }

    let entries = cuttle_audit::read_entries_unverified(&path)?;
    let events: Vec<AuditEvent> = entries.into_iter().map(|e| e.event).collect();

    let report = if args.falsifier_eval {
        TelemetryReport::with_falsifiers(&events)
    } else {
        TelemetryReport::from_events(events.iter())
    };

    if args.json {
        writeln!(out, "{}", report.to_json()?)?;
    } else {
        write!(out, "{report}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttle_audit::{AuditChain, AuditChainKey};
    use std::path::Path;
    use tempfile::TempDir;

    fn write_test_log(path: &Path, events: Vec<AuditEvent>) {
        let key = AuditChainKey::from_bytes([7u8; 32]);
        let mut chain = AuditChain::open(path.to_path_buf(), key).unwrap();
        for ev in events {
            chain.append(ev).unwrap();
        }
    }

    #[test]
    fn run_errors_when_audit_log_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nope.jsonl");
        let args = TelemetryArgs {
            json: false,
            falsifier_eval: false,
            audit_log: Some(path.clone()),
        };
        let mut out = Vec::new();
        let err = run(&args, &mut out).unwrap_err();
        assert!(matches!(err, TelemetryCmdError::AuditLogMissing(p) if p == path));
    }

    #[test]
    fn run_emits_text_report_for_present_log() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("audit.jsonl");
        write_test_log(
            &path,
            vec![AuditEvent::ToolDispatch {
                tool_name: "bash".into(),
                argument_summary: "ls".into(),
            }],
        );
        let args = TelemetryArgs {
            json: false,
            falsifier_eval: false,
            audit_log: Some(path),
        };
        let mut out = Vec::new();
        run(&args, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("Tool dispatches"));
        assert!(s.contains("bash"));
    }

    #[test]
    fn run_emits_json_when_flag_set() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("audit.jsonl");
        write_test_log(
            &path,
            vec![AuditEvent::ToolDispatch {
                tool_name: "bash".into(),
                argument_summary: "ls".into(),
            }],
        );
        let args = TelemetryArgs {
            json: true,
            falsifier_eval: false,
            audit_log: Some(path),
        };
        let mut out = Vec::new();
        run(&args, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        // Round-trip through serde_json to confirm valid JSON.
        let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed["dispatch"]["total"], 1);
    }

    #[test]
    fn run_includes_falsifier_section_when_flag_set() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("audit.jsonl");
        write_test_log(
            &path,
            vec![AuditEvent::GateDisabled {
                rule_id: "r1".into(),
                operator_reason: "x".into(),
            }],
        );
        let args = TelemetryArgs {
            json: false,
            falsifier_eval: true,
            audit_log: Some(path),
        };
        let mut out = Vec::new();
        run(&args, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("Falsifier evaluations"));
        assert!(s.contains("F-Cuttle-DISABLE"));
    }

    #[test]
    fn run_emits_empty_report_for_empty_log() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("audit.jsonl");
        // Touch the file so it exists but has no entries.
        std::fs::File::create(&path).unwrap();
        let args = TelemetryArgs {
            json: false,
            falsifier_eval: false,
            audit_log: Some(path),
        };
        let mut out = Vec::new();
        run(&args, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("(none)"));
    }
}
