//! Live integration test: actually call Anthropic and have it review
//! a known-vulnerable function. Gated behind the
//! `CUTTLE_REVIEW_LIVE_TEST` env var so `cargo test` in CI / local
//! workflows doesn't burn API budget by default.
//!
//! Run with:
//!
//!     CUTTLE_REVIEW_LIVE_TEST=1 \
//!     ANTHROPIC_API_KEY=$(security find-generic-password \
//!         -s dev.cuttle.api-keys -a ANTHROPIC_API_KEY -w) \
//!     cargo test -p cuttle-review --test live_review -- --nocapture
//!
//! Expected outcome: the reviewer flags at least one CRITICAL finding
//! tagged CAPTAIN-AMERICA on the CSV-formula-injection bug. Cost on
//! Haiku 4.5: ~$0.005 per invocation.

use cuttle_credential::primitives::ApiKey;
use cuttle_review::{block_on_critical, ReviewClient, Severity};

const VULNERABLE_CSV_EXPORT: &str = r#"
import csv, io

def export_users_csv(users):
    """Return a CSV string with columns name,email for each user dict."""
    buf = io.StringIO()
    w = csv.writer(buf)
    w.writerow(['name', 'email'])
    for u in users:
        # Vulnerable: user['name'] and user['email'] are written
        # without neutralizing the spreadsheet formula triggers
        # (=, +, -, @, \t, \r). When the resulting CSV is opened in
        # Excel / LibreOffice, an attacker-controlled cell starting
        # with '=' is interpreted as a formula.
        w.writerow([u['name'], u['email']])
    return buf.getvalue()
"#;

const ORIGINAL_PROMPT: &str = r#"Write a Python function `export_users_csv(users)` that returns a CSV string. Each user is a dict with `name` and `email` keys with str values. The user content is from untrusted input (e.g. signup form). The output will be opened by spreadsheet applications."#;

#[tokio::test]
async fn reviewer_flags_csv_formula_injection_as_critical() {
    if std::env::var("CUTTLE_REVIEW_LIVE_TEST").is_err() {
        eprintln!(
            "skipping: set CUTTLE_REVIEW_LIVE_TEST=1 to run the live API test (~$0.005 on Haiku)"
        );
        return;
    }

    let api_key = ApiKey::from_env_var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY required for live review test");

    let client = ReviewClient::new().expect("ReviewClient default config");
    let findings = client
        .review(&api_key, ORIGINAL_PROMPT, VULNERABLE_CSV_EXPORT)
        .await
        .expect("review call");

    eprintln!("findings ({}):", findings.len());
    for f in &findings {
        eprintln!(
            "  [{}] {} ({}): {} -> {}",
            severity_label(&f.severity),
            f.persona,
            f.location,
            f.message,
            f.fix
        );
    }

    assert!(
        !findings.is_empty(),
        "reviewer returned zero findings on a known-vulnerable CSV export"
    );

    let has_critical = block_on_critical(&findings);
    let mentions_formula = findings.iter().any(|f| {
        let lower = f.message.to_lowercase();
        lower.contains("formula") || lower.contains("=") || lower.contains("excel")
    });

    assert!(
        has_critical || mentions_formula,
        "reviewer didn't flag the CSV formula injection (no CRITICAL and no formula/excel mention)"
    );
}

fn severity_label(s: &Severity) -> &'static str {
    match s {
        Severity::Critical => "CRITICAL",
        Severity::High => "HIGH",
        Severity::Medium => "MEDIUM",
        Severity::Low => "LOW",
    }
}
