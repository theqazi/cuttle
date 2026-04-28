//! Error type for cuttle-review.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReviewError {
    /// The Anthropic call itself failed (network, auth, retry exhaustion).
    /// Surfaces with the original error so callers can match on the
    /// concrete failure class.
    #[error("anthropic call failed: {0}")]
    Anthropic(#[from] cuttle_anthropic::AnthropicError),

    /// Constructing the AnthropicClient failed (bad config). v0.0.1
    /// only constructs with defaults so this is unreachable in practice
    /// but is kept on the type so the error contract stays stable when
    /// `ReviewClient::with_config()` lands in v0.0.2.
    #[error("client construction failed: {0}")]
    ClientConstruction(String),

    /// Reviewer model returned text that wasn't a JSON array of findings.
    /// Carries a snippet of the offending output (first 300 chars) for
    /// debugging without flooding the operator's terminal. Most likely
    /// cause: prompt drift; the model emitted prose around the JSON.
    #[error("reviewer output was not a JSON array: {snippet}")]
    OutputNotJsonArray { snippet: String },

    /// JSON parsed but didn't match the Finding schema (missing
    /// severity, unknown enum value, etc.). Carries the underlying
    /// serde_json error.
    #[error("reviewer output didn't match Finding schema: {0}")]
    OutputSchemaMismatch(#[from] serde_json::Error),
}
