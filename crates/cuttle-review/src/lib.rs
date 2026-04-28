//! Cuttle L4 verification gate.
//!
//! Runs model-generated code through an adversarial three-persona
//! review and returns severity-classified findings. The harness
//! mechanic is `block_on_critical()`: cuttle's caller refuses to
//! return / commit / accept the model's output if any CRITICAL
//! finding is present. This is the framework's L4 (Verification
//! Loop) implemented as deterministic-bedrock instead of as a
//! skill-shape advisory the model can skip.
//!
//! v0.0.1 ships:
//! - Synchronous `review_code()` API.
//! - Anthropic-backed reviewer via `cuttle-anthropic`.
//! - Inline three-persona system prompt (compressed from
//!   `~/.claude/skills/code-review/SKILL.md`); full skill content
//!   lifting is deferred to v0.0.2 once `cuttle-skills` exposes a
//!   loader for SKILL.md files.
//!
//! v0.0.2+ will add: multi-persona-parallel dispatch (each persona
//! gets its own API call, findings merged); streaming output for
//! long reviews; integration with `cuttle-skills` so the system
//! prompt is the canonical skill content rather than a hand-coded
//! compression.

pub mod error;
pub mod prompt;
pub mod reviewer;

pub use error::ReviewError;
pub use prompt::SYSTEM_PROMPT;
pub use reviewer::{Finding, ReviewClient, Severity, block_on_critical, has_blocking};

/// Convenience wrapper: full review pipeline. Builds a
/// `ReviewClient`, calls the reviewer, returns findings.
///
/// `original_prompt` is the operator's original task spec (so the
/// reviewer knows what the code is meant to do). `generated_code`
/// is the model's output to be reviewed.
pub async fn review_code(
    api_key: &cuttle_credential::primitives::ApiKey,
    original_prompt: &str,
    generated_code: &str,
) -> Result<Vec<Finding>, ReviewError> {
    let client = ReviewClient::new()?;
    client
        .review(api_key, original_prompt, generated_code)
        .await
}
