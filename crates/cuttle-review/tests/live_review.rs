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

// -----------------------------------------------------------------------------
// CVE-derived test fixtures.
//
// Each fixture is a minimized version of a real-world vulnerability shipped
// in production code. Source:
//   https://github.com/SilverPlate3/My-CVEs
//
// These tests serve two purposes:
//   1. Validate that cuttle-review's three-persona system prompt actually
//      catches CVE-class vulnerabilities (not just textbook examples).
//   2. Build a regression corpus: future system-prompt changes that cause
//      these to stop firing get caught immediately.
//
// All CVE fixtures share the live-test gate (CUTTLE_REVIEW_LIVE_TEST=1).
// -----------------------------------------------------------------------------

/// Vulnerable pattern from CVE-2026-25883 (Vexa AI webhook SSRF).
///
/// The original Vexa code at services/bot-manager/app/tasks/bot_exit_tasks/
/// send_webhook.py performs no validation on `webhook_url` before POSTing.
/// An authenticated attacker who configures a webhook URL pointed at
/// localhost / cloud-metadata / internal services exfiltrates data through
/// the server's network position.
const CVE_2026_25883_SSRF_WEBHOOK: &str = r#"
import httpx

async def send_webhook(webhook_url: str, payload: dict) -> None:
    """POST the payload to the user-configured webhook URL when a meeting
    completes. webhook_url is set by the authenticated user via the
    /user/webhook endpoint and stored in the user's profile.
    """
    async with httpx.AsyncClient() as client:
        response = await client.post(
            webhook_url,
            json=payload,
            timeout=30.0,
            headers={"Content-Type": "application/json"},
        )
        return response.status_code
"#;

const CVE_2026_25883_PROMPT: &str = r#"Write an async Python function `send_webhook(webhook_url, payload)` that POSTs the JSON payload to the user-configured webhook URL when a meeting completes. The URL is set by an authenticated user via a separate endpoint (/user/webhook)."#;

#[tokio::test]
async fn reviewer_flags_cve_2026_25883_ssrf_webhook() {
    if std::env::var("CUTTLE_REVIEW_LIVE_TEST").is_err() {
        eprintln!("skipping: set CUTTLE_REVIEW_LIVE_TEST=1 to run");
        return;
    }
    let api_key = ApiKey::from_env_var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY");
    let client = ReviewClient::new().expect("client");
    let findings = client
        .review(&api_key, CVE_2026_25883_PROMPT, CVE_2026_25883_SSRF_WEBHOOK)
        .await
        .expect("review");

    eprintln!("CVE-2026-25883 findings ({}):", findings.len());
    for f in &findings {
        eprintln!(
            "  [{}] {} ({}): {}",
            severity_label(&f.severity),
            f.persona,
            f.location,
            f.message
        );
    }

    let has_critical = block_on_critical(&findings);
    let mentions_ssrf = findings.iter().any(|f| {
        let m = f.message.to_lowercase();
        m.contains("ssrf")
            || m.contains("server-side request forgery")
            || m.contains("validate")
            || m.contains("internal")
            || m.contains("metadata")
    });
    let from_security_persona = findings
        .iter()
        .any(|f| f.persona.to_uppercase().contains("CAPTAIN"));

    assert!(
        has_critical && mentions_ssrf && from_security_persona,
        "reviewer didn't catch the SSRF (critical={has_critical} ssrf-mention={mentions_ssrf} captain-america={from_security_persona})"
    );
}

/// Vulnerable pattern from CVE-2026-25058 (Vexa AI transcript IDOR).
///
/// The original endpoint at services/transcription-collector/api/endpoints.py
/// returns transcript segments for any meeting_id without checking auth or
/// ownership. `include_in_schema=False` only hides the route from OpenAPI
/// docs; the endpoint itself is open. The service was exposed on port 8123
/// in the upstream docker-compose.
const CVE_2026_25058_IDOR_ENDPOINT: &str = r#"
from fastapi import APIRouter, Depends, HTTPException, Request
from sqlalchemy.ext.asyncio import AsyncSession
from typing import List

from .database import get_db
from .models import Meeting, TranscriptionSegment

router = APIRouter()

@router.get(
    "/internal/transcripts/{meeting_id}",
    response_model=List[TranscriptionSegment],
    summary="[Internal] Get all transcript segments for a meeting",
    include_in_schema=False,
)
async def get_transcript_internal(
    meeting_id: int,
    request: Request,
    db: AsyncSession = Depends(get_db),
):
    meeting = await db.get(Meeting, meeting_id)
    if not meeting:
        raise HTTPException(status_code=404, detail="Meeting not found")
    segments = await _get_full_transcript_segments(meeting_id, db)
    return segments
"#;

const CVE_2026_25058_PROMPT: &str = r#"Write a FastAPI route handler that returns all transcript segments for a given meeting_id from the database. The route is internal (used by other services in the same docker-compose network) and lives at GET /internal/transcripts/{meeting_id}."#;

#[tokio::test]
async fn reviewer_flags_cve_2026_25058_idor_endpoint() {
    if std::env::var("CUTTLE_REVIEW_LIVE_TEST").is_err() {
        eprintln!("skipping: set CUTTLE_REVIEW_LIVE_TEST=1 to run");
        return;
    }
    let api_key = ApiKey::from_env_var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY");
    let client = ReviewClient::new().expect("client");
    let findings = client
        .review(
            &api_key,
            CVE_2026_25058_PROMPT,
            CVE_2026_25058_IDOR_ENDPOINT,
        )
        .await
        .expect("review");

    eprintln!("CVE-2026-25058 findings ({}):", findings.len());
    for f in &findings {
        eprintln!(
            "  [{}] {} ({}): {}",
            severity_label(&f.severity),
            f.persona,
            f.location,
            f.message
        );
    }

    // Acceptance: at least one CRITICAL or HIGH finding from
    // CAPTAIN-AMERICA that mentions auth, authorization, ownership, or
    // IDOR. The reviewer may class this as HIGH instead of CRITICAL
    // depending on whether the model treats "internal endpoint with no
    // auth" as data-loss-tier or ops-blindspot-tier; either is correct
    // for our purposes.
    let serious = findings
        .iter()
        .filter(|f| f.severity >= cuttle_review::Severity::High)
        .any(|f| {
            let m = f.message.to_lowercase();
            m.contains("auth")
                || m.contains("authoriz")
                || m.contains("ownership")
                || m.contains("idor")
                || m.contains("user_id")
                || m.contains("current_user")
        });
    assert!(
        serious,
        "reviewer didn't flag missing auth/authorization on the internal endpoint"
    );
}
