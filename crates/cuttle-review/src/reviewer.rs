//! ReviewClient + Finding types.
//!
//! The actual harness mechanic. `ReviewClient::review()` calls
//! Anthropic with the three-persona system prompt + the operator's
//! original spec + the model's generated code, parses the JSON
//! array of findings, and returns them.
//!
//! `block_on_critical()` is the gate the caller checks before
//! accepting the model's output.

use cuttle_anthropic::{
    AnthropicClient, ClientConfig, KnownModel, Message, Model, Request, SystemContent,
};
use cuttle_credential::primitives::ApiKey;
use serde::{Deserialize, Serialize};

use crate::error::ReviewError;
use crate::prompt::SYSTEM_PROMPT;

/// Severity classification on a finding. Matches the rubric in the
/// system prompt: CRITICAL ships an incident, HIGH ships pain
/// within a week, MEDIUM accumulates tech debt, LOW is a
/// take-it-or-leave-it suggestion.
///
/// `block_on_critical()` blocks if any CRITICAL finding is present;
/// `has_blocking()` (with a threshold) lets callers escalate the
/// blocking severity bar (e.g. block on HIGH-or-worse for stricter
/// gates).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// One adversarial review finding. The persona is recorded so the
/// operator can see *why* a particular defect class was caught and
/// can disable a persona on a per-task basis (v0.0.3+; v0.0.1 always
/// runs all three in one pass).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    /// One of "SPIDER-MAN", "IRON-MAN", "CAPTAIN-AMERICA". Kept as
    /// String here (not enum) so v0.0.2 persona expansion / custom
    /// personas don't break deserialization on stored finding logs.
    pub persona: String,
    pub location: String,
    pub message: String,
    pub fix: String,
}

/// Returns true iff the findings list contains any CRITICAL entry.
/// This is the canonical L4 block predicate cuttle's `ask`/`session`
/// command paths consult before returning a model response to the
/// operator.
pub fn block_on_critical(findings: &[Finding]) -> bool {
    findings.iter().any(|f| f.severity == Severity::Critical)
}

/// Returns true iff any finding is at or above the threshold.
/// Useful when an operator wants a stricter gate (e.g. block on
/// HIGH-or-worse for high-stakes changes). The default cuttle gate
/// is `block_on_critical()`.
pub fn has_blocking(findings: &[Finding], threshold: Severity) -> bool {
    findings.iter().any(|f| f.severity >= threshold)
}

/// Wraps `cuttle_anthropic::AnthropicClient` with the L4 review
/// system prompt and a Finding-shaped output parser. v0.0.1 hardcodes
/// the model to Haiku 4.5 (fast + cheap; review quality is good
/// enough on the eval suite); v0.0.2 will expose a `with_model()`
/// builder so operators can run a more expensive reviewer for
/// high-stakes generations.
pub struct ReviewClient {
    inner: AnthropicClient,
    model: Model,
    max_tokens: u32,
}

impl ReviewClient {
    /// Construct with default Anthropic config + Haiku 4.5 reviewer.
    pub fn new() -> Result<Self, ReviewError> {
        let inner = AnthropicClient::new(ClientConfig::default())
            .map_err(|e| ReviewError::ClientConstruction(e.to_string()))?;
        Ok(Self {
            inner,
            model: Model::Known(KnownModel::ClaudeHaiku45),
            // 4096 tokens is plenty for a 12-finding review. If the
            // reviewer hits this cap and stops mid-array, parsing
            // fails with OutputNotJsonArray and the caller can retry
            // at a higher cap. Avoiding 8K+ here keeps the per-call
            // latency budget under ~3s on Haiku.
            max_tokens: 4096,
        })
    }

    /// Run the three-persona review. Builds a single API call with
    /// the system prompt + a user message containing the operator's
    /// spec and the generated code, then parses the response into a
    /// `Vec<Finding>`.
    ///
    /// `original_prompt` may be empty (caller hasn't recorded the
    /// spec); the reviewer will still flag defects but its "is the
    /// implementation aligned with intent?" pass is weakened.
    pub async fn review(
        &self,
        api_key: &ApiKey,
        original_prompt: &str,
        generated_code: &str,
    ) -> Result<Vec<Finding>, ReviewError> {
        let user_text = format!(
            "ORIGINAL TASK SPEC:\n\n{}\n\n--- BEGIN GENERATED CODE ---\n\n{}\n\n--- END GENERATED CODE ---\n\nReturn the JSON array of findings now.",
            if original_prompt.is_empty() {
                "(spec not provided; review against general best practices)"
            } else {
                original_prompt
            },
            generated_code,
        );

        let mut req = Request::new(
            self.model.clone(),
            vec![Message::user_text(user_text)],
            self.max_tokens,
        );
        req.system = Some(SystemContent::Plain(SYSTEM_PROMPT.to_string()));
        // Force temperature low so the reviewer is consistent across
        // re-runs of the same code; reviewer flakiness erodes the
        // gate's value.
        req.temperature = Some(0.1);

        let resp = self.inner.messages(api_key, &req).await?;
        let text = resp.text();
        parse_findings(&text)
    }
}

/// Parse the reviewer's response text into Findings.
///
/// The system prompt promises a top-level JSON array. We tolerate
/// surrounding whitespace AND a single enclosing markdown code fence
/// (` ```json ... ``` ` or ` ``` ... ``` `): Claude wraps structured
/// output in fences with high frequency despite explicit instructions
/// not to, and the JSON inside is otherwise correct. Stripping the
/// fence is a far cheaper recovery than re-prompting.
///
/// We do NOT tolerate prose around the array (e.g. "Here is my
/// review: [ ... ]"); that's real prompt drift the operator should
/// see. Surfaces as OutputNotJsonArray with a 300-char snippet.
fn parse_findings(text: &str) -> Result<Vec<Finding>, ReviewError> {
    let body = strip_optional_fence(text.trim());
    if !body.starts_with('[') || !body.ends_with(']') {
        let snippet = body.chars().take(300).collect::<String>();
        return Err(ReviewError::OutputNotJsonArray { snippet });
    }
    let findings: Vec<Finding> = serde_json::from_str(body)?;
    Ok(findings)
}

/// Strip an optional surrounding markdown code fence. Returns the
/// body if the input is fenced; otherwise returns the input
/// unchanged. Recognized forms:
///
///   ```json\n<body>\n```
///   ```\n<body>\n```
///
/// Conservative: requires the fence to be the FIRST and LAST tokens
/// of the trimmed input. Won't strip mid-string fences (those would
/// indicate prose-around-array drift, which we want to surface).
fn strip_optional_fence(s: &str) -> &str {
    let after_opener = if let Some(rest) = s.strip_prefix("```json") {
        rest
    } else if let Some(rest) = s.strip_prefix("```") {
        rest
    } else {
        return s;
    };
    let after_opener = after_opener.trim_start_matches('\n');
    if let Some(body) = after_opener.strip_suffix("```") {
        body.trim_end_matches('\n').trim_end()
    } else {
        // Unclosed fence: don't claim to have stripped it; let the
        // caller surface the malformed shape.
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_on_critical_fires_on_one_critical() {
        let findings = vec![
            Finding {
                severity: Severity::Low,
                persona: "SPIDER-MAN".into(),
                location: "fn foo".into(),
                message: "minor".into(),
                fix: "rename".into(),
            },
            Finding {
                severity: Severity::Critical,
                persona: "CAPTAIN-AMERICA".into(),
                location: "fn bar".into(),
                message: "SQL injection".into(),
                fix: "parameterize".into(),
            },
        ];
        assert!(block_on_critical(&findings));
    }

    #[test]
    fn block_on_critical_passes_when_no_critical() {
        let findings = vec![Finding {
            severity: Severity::High,
            persona: "IRON-MAN".into(),
            location: "fn foo".into(),
            message: "n+1".into(),
            fix: "batch".into(),
        }];
        assert!(!block_on_critical(&findings));
    }

    #[test]
    fn has_blocking_threshold_at_high() {
        let findings = vec![Finding {
            severity: Severity::High,
            persona: "IRON-MAN".into(),
            location: "fn foo".into(),
            message: "n+1".into(),
            fix: "batch".into(),
        }];
        assert!(has_blocking(&findings, Severity::High));
        assert!(!has_blocking(&findings, Severity::Critical));
    }

    #[test]
    fn parse_empty_array() {
        let parsed = parse_findings("[]").unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_array_with_whitespace() {
        let parsed = parse_findings("  []\n").unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_one_critical_finding() {
        let raw = r#"[
            {
              "severity": "CRITICAL",
              "persona": "CAPTAIN-AMERICA",
              "location": "render_comment",
              "message": "user input is concatenated into HTML without escape; XSS",
              "fix": "use html.escape() on the input before substitution"
            }
        ]"#;
        let parsed = parse_findings(raw).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].severity, Severity::Critical);
        assert_eq!(parsed[0].persona, "CAPTAIN-AMERICA");
        assert!(parsed[0].message.contains("XSS"));
        assert!(block_on_critical(&parsed));
    }

    #[test]
    fn parse_rejects_prose_around_array() {
        // Common prompt-drift failure: model emits explanatory text
        // before the JSON. Without this guard the next steps would
        // crash on serde_json with a confusing error.
        let raw = "Here is my review:\n\n[]";
        let err = parse_findings(raw).unwrap_err();
        assert!(matches!(err, ReviewError::OutputNotJsonArray { .. }));
    }

    #[test]
    fn parse_strips_json_fence_and_accepts() {
        // Claude commonly wraps structured output in ```json fences
        // despite the system prompt's instruction not to. Stripping
        // is cheaper than re-prompting; the JSON inside is correct.
        let raw = "```json\n[]\n```";
        let parsed = parse_findings(raw).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_strips_bare_fence_and_accepts() {
        let raw = "```\n[]\n```";
        let parsed = parse_findings(raw).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_strips_fence_around_real_finding() {
        let raw = r#"```json
[{"severity":"CRITICAL","persona":"CAPTAIN-AMERICA","location":"x","message":"y","fix":"z"}]
```"#;
        let parsed = parse_findings(raw).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].severity, Severity::Critical);
    }

    #[test]
    fn parse_still_rejects_unclosed_fence() {
        // If the closing fence is missing, treat as malformed: the
        // body may have been truncated mid-array and parsing it
        // anyway would silently lose findings.
        let raw = "```json\n[]";
        let err = parse_findings(raw).unwrap_err();
        assert!(matches!(err, ReviewError::OutputNotJsonArray { .. }));
    }

    #[test]
    fn parse_rejects_unknown_severity() {
        let raw = r#"[{"severity":"NUCLEAR","persona":"SPIDER-MAN","location":"x","message":"y","fix":"z"}]"#;
        let err = parse_findings(raw).unwrap_err();
        assert!(matches!(err, ReviewError::OutputSchemaMismatch(_)));
    }

    #[test]
    fn severity_ordering_is_low_to_critical() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }
}
