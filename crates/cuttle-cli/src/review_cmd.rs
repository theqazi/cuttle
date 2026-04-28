//! `cuttle review` subcommand handler. Drives `cuttle-review`'s
//! L4 verification gate from the terminal.
//!
//! Contract:
//!
//!   stdin:  the code to review (model output, file contents,
//!           anything; reviewer treats it as opaque source).
//!   --prompt <s>:  the original task spec (optional but improves
//!                  finding quality because the reviewer can match
//!                  intent vs. implementation).
//!   --threshold critical|high|medium|low:  block severity bar.
//!                  Default is "critical".
//!   --json:        emit findings as a JSON array (suitable for
//!                  piping to jq); default is human-readable lines.
//!
//! Exit codes:
//!   0:  reviewer ran AND no finding at-or-above threshold.
//!   1:  reviewer ran AND found at-or-above threshold (block).
//!   2:  reviewer or auth error (couldn't reach the bar one way or
//!       the other).
//!
//! Pattern matches `cuttle ask`: builds a tokio runtime on demand,
//! resolves the API key via `cuttle_credential::ApiKey::resolve`, and
//! writes results to the caller-supplied writer.

use crate::args::{ReviewArgs, ReviewThreshold};
use cuttle_credential::{ApiKey, ResolveError};
use cuttle_review::{has_blocking, Finding, ReviewClient, ReviewError, Severity};
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReviewCmdError {
    #[error("could not load API key: {0}")]
    ApiKey(#[from] ResolveError),

    #[error("could not read code from stdin: {0}")]
    StdinRead(std::io::Error),

    #[error("stdin was empty; pipe code in or use --code-file")]
    EmptyStdinCode,

    #[error("reviewer failed: {0}")]
    Review(#[from] ReviewError),

    #[error("could not build tokio runtime: {0}")]
    Runtime(std::io::Error),

    #[error("writing review output failed: {0}")]
    WriteFailed(std::io::Error),
}

/// Synchronous entry point. Returns Ok with the (findings, blocked)
/// pair so main.rs can map blocked → exit(1). The `out` writer
/// receives the formatted findings (human or JSON).
pub fn run<W: Write>(
    args: &ReviewArgs,
    out: &mut W,
) -> Result<(Vec<Finding>, bool), ReviewCmdError> {
    let code = read_stdin_code()?;
    let api_key = ApiKey::resolve(&args.api_key_env)?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(ReviewCmdError::Runtime)?;

    let client = ReviewClient::new()?;
    let prompt = args.prompt.as_deref().unwrap_or("");
    let findings = rt.block_on(client.review(&api_key, prompt, &code))?;

    let threshold = match args.threshold {
        ReviewThreshold::Critical => Severity::Critical,
        ReviewThreshold::High => Severity::High,
        ReviewThreshold::Medium => Severity::Medium,
        ReviewThreshold::Low => Severity::Low,
    };
    let blocked = has_blocking(&findings, threshold);

    if args.json {
        write_json(&findings, out)?;
    } else {
        write_human(&findings, blocked, out)?;
    }
    Ok((findings, blocked))
}

fn read_stdin_code() -> Result<String, ReviewCmdError> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(ReviewCmdError::StdinRead)?;
    if buf.trim().is_empty() {
        return Err(ReviewCmdError::EmptyStdinCode);
    }
    Ok(buf)
}

fn write_json<W: Write>(findings: &[Finding], out: &mut W) -> Result<(), ReviewCmdError> {
    // serde_json::to_vec can't fail for owned Findings (no IO,
    // no NaN), so unwrap is safe here. The IO write IS fallible.
    let bytes = serde_json::to_vec_pretty(findings).expect("Finding serialization is infallible");
    out.write_all(&bytes).map_err(ReviewCmdError::WriteFailed)?;
    out.write_all(b"\n").map_err(ReviewCmdError::WriteFailed)?;
    Ok(())
}

fn write_human<W: Write>(
    findings: &[Finding],
    blocked: bool,
    out: &mut W,
) -> Result<(), ReviewCmdError> {
    if findings.is_empty() {
        writeln!(out, "no findings.").map_err(ReviewCmdError::WriteFailed)?;
        return Ok(());
    }

    writeln!(out, "{} finding(s):", findings.len()).map_err(ReviewCmdError::WriteFailed)?;
    for f in findings {
        let sev = severity_label(&f.severity);
        writeln!(
            out,
            "  [{}] {} ({})\n    {}\n    fix: {}",
            sev, f.persona, f.location, f.message, f.fix
        )
        .map_err(ReviewCmdError::WriteFailed)?;
    }
    if blocked {
        writeln!(
            out,
            "\nBLOCKED: at least one finding meets or exceeds the configured threshold."
        )
        .map_err(ReviewCmdError::WriteFailed)?;
    }
    Ok(())
}

fn severity_label(s: &Severity) -> &'static str {
    match s {
        Severity::Critical => "CRITICAL",
        Severity::High => "HIGH",
        Severity::Medium => "MEDIUM",
        Severity::Low => "LOW",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_finding(sev: Severity, persona: &str, msg: &str) -> Finding {
        Finding {
            severity: sev,
            persona: persona.to_string(),
            location: "fn x".to_string(),
            message: msg.to_string(),
            fix: "do the thing".to_string(),
        }
    }

    #[test]
    fn write_human_empty_findings() {
        let mut buf: Vec<u8> = Vec::new();
        write_human(&[], false, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("no findings"));
    }

    #[test]
    fn write_human_one_critical_blocked() {
        let findings = vec![fixture_finding(
            Severity::Critical,
            "CAPTAIN-AMERICA",
            "SQL injection",
        )];
        let mut buf: Vec<u8> = Vec::new();
        write_human(&findings, true, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("[CRITICAL]"));
        assert!(s.contains("CAPTAIN-AMERICA"));
        assert!(s.contains("SQL injection"));
        assert!(s.contains("BLOCKED"));
    }

    #[test]
    fn write_human_one_low_not_blocked() {
        let findings = vec![fixture_finding(
            Severity::Low,
            "SPIDER-MAN",
            "rename this var",
        )];
        let mut buf: Vec<u8> = Vec::new();
        write_human(&findings, false, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("[LOW]"));
        assert!(!s.contains("BLOCKED"));
    }

    #[test]
    fn write_json_round_trip() {
        let findings = vec![fixture_finding(
            Severity::High,
            "IRON-MAN",
            "n+1 query in hot path",
        )];
        let mut buf: Vec<u8> = Vec::new();
        write_json(&findings, &mut buf).unwrap();
        let parsed: Vec<Finding> = serde_json::from_slice(&buf).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].severity, Severity::High);
        assert_eq!(parsed[0].persona, "IRON-MAN");
    }
}
