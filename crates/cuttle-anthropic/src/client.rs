//! `AnthropicClient`: thin HTTPS client over reqwest.
//!
//! Boundary contract:
//! - `messages()` borrows `&ApiKey` and calls `consume()` exactly once. The
//!   returned `&[u8]` is formatted into the `x-api-key` header value for
//!   the duration of the call (including retry attempts). The header value
//!   is dropped at function return; with `panic = "abort"` (workspace
//!   release profile, D-15) deterministic cleanup holds.
//! - The `cuttle-credential` crate is the sole producer of `ApiKey`. This
//!   crate is a consumer; it never constructs one.
//! - HTTPS only. The `base_url` field exists for test-server overrides
//!   (v0.0.9 wiremock integration), not for production HTTP fallback.
//!
//! Per WV-04 disclaimer in PRD §6.1.5: this client trusts the TLS chain to
//! `api.anthropic.com`; pinning is out of scope for v0.1 (operator's CA
//! store is the trust anchor). Documented as a known limitation.

use crate::error::AnthropicError;
use crate::model::{Request, Response};
use crate::retry::{RetryDecision, RetryPolicy};
use cuttle_credential::ApiKey;
use std::time::Duration;

/// Anthropic API version pinned at the wire level. Updating this is a
/// deliberate Cuttle release decision (the API behaviour can change in
/// non-backward-compatible ways across versions).
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Default base URL for the Anthropic API. Override via `ClientConfig::base_url`
/// only for test servers; production must hit api.anthropic.com over HTTPS.
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub base_url: String,
    pub request_timeout: Duration,
    pub retry: RetryPolicy,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            base_url: DEFAULT_BASE_URL.to_string(),
            // 5-minute per-request timeout. Long requests (large max_tokens
            // + reasoning models) routinely take 60-120s; 5min gives headroom
            // without leaving a hung TCP connection forever.
            request_timeout: Duration::from_secs(300),
            retry: RetryPolicy::default(),
        }
    }
}

pub struct AnthropicClient {
    http: reqwest::Client,
    config: ClientConfig,
}

impl AnthropicClient {
    /// Build a new client. Fails only if reqwest cannot construct an HTTPS
    /// transport with rustls — that is a build-environment problem, never
    /// a runtime input problem.
    pub fn new(config: ClientConfig) -> Result<Self, AnthropicError> {
        let http = reqwest::Client::builder()
            .timeout(config.request_timeout)
            // No connection pooling override; reqwest defaults are fine.
            // No proxy override; respect HTTPS_PROXY env if operator sets it.
            .build()
            .map_err(AnthropicError::Network)?;
        Ok(AnthropicClient { http, config })
    }

    /// POST `/v1/messages` (non-streaming). Calls `api_key.consume()` once;
    /// reuses the consumed bytes across retry attempts.
    ///
    /// On success: a deserialized `Response` with text + usage.
    /// On failure: `AnthropicError`. Retryable failures wrap into
    /// `RetryExhausted` only after the budget is spent; non-retryable
    /// failures (4xx auth/validation, deserialize errors) propagate
    /// immediately.
    pub async fn messages(
        &self,
        api_key: &ApiKey,
        request: &Request,
    ) -> Result<Response, AnthropicError> {
        // Read the API key bytes ONCE. The borrow lives until the end of
        // this function, so all retry attempts share it. ApiKey panics on
        // a second consume() call by design — we must not call it inside
        // the retry loop.
        let key_bytes = api_key.consume();
        let key_header_value = std::str::from_utf8(key_bytes).map_err(|_| {
            AnthropicError::InvalidRequest(
                "API key is not valid UTF-8; refusing to format header".into(),
            )
        })?;

        let url = format!("{}/v1/messages", self.config.base_url);
        let body_bytes = serde_json::to_vec(request)?;

        let mut attempt: u32 = 0;
        loop {
            attempt += 1;
            let result = self
                .send_once(&url, key_header_value, body_bytes.clone())
                .await;

            match result {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    let hint = retry_after_hint_from(&err);
                    match self.config.retry.decide(attempt, &err, hint) {
                        RetryDecision::GiveUp => {
                            // Distinguish budget-exhausted from non-retryable.
                            if err.is_retryable() {
                                return Err(AnthropicError::RetryExhausted {
                                    attempts: attempt,
                                    last: Box::new(err),
                                });
                            }
                            return Err(err);
                        }
                        RetryDecision::AfterMillis(ms) => {
                            tokio::time::sleep(Duration::from_millis(ms)).await;
                        }
                    }
                }
            }
        }
    }

    /// One HTTPS POST attempt. Maps reqwest errors + non-2xx statuses into
    /// `AnthropicError`. The `Retry-After` header (if present on the
    /// response) is preserved on the error so `messages()` can pass it
    /// through to the retry policy.
    async fn send_once(
        &self,
        url: &str,
        api_key_header: &str,
        body: Vec<u8>,
    ) -> Result<Response, AnthropicError> {
        let resp = self
            .http
            .post(url)
            .header("x-api-key", api_key_header)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let bytes = resp.bytes().await?;
            let parsed: Response = serde_json::from_slice(&bytes)?;
            return Ok(parsed);
        }

        // Non-2xx: capture Retry-After before we consume the body.
        let retry_after_ms = parse_retry_after_header(&resp);
        let body_text = resp.text().await.unwrap_or_default();

        // Embed the hint inside the body string with a stable prefix so
        // retry_after_hint_from can recover it. This avoids enlarging the
        // public error type just to thread one optional field.
        let body_with_hint = match retry_after_ms {
            Some(ms) => format!("__retry_after_ms__={ms}\n{body_text}"),
            None => body_text,
        };

        Err(AnthropicError::RequestFailed {
            status: status.as_u16(),
            body: body_with_hint,
        })
    }
}

/// Extract the embedded Retry-After hint left by `send_once` if present.
fn retry_after_hint_from(err: &AnthropicError) -> Option<u64> {
    if let AnthropicError::RequestFailed { body, .. } = err
        && let Some(stripped) = body.strip_prefix("__retry_after_ms__=")
    {
        let end = stripped.find('\n').unwrap_or(stripped.len());
        return stripped[..end].parse::<u64>().ok();
    }
    None
}

/// Parse Retry-After per RFC 7231 §7.1.3. Two valid formats:
/// - delta-seconds (integer): "120"
/// - HTTP-date: "Wed, 21 Oct 2026 07:28:00 GMT"
///
/// v0.0.8 honors the integer-seconds form (the form Anthropic + most CDNs
/// emit). HTTP-date is parsed loosely (returns None on failure) — operator
/// can still rely on the computed exponential backoff.
fn parse_retry_after_header(resp: &reqwest::Response) -> Option<u64> {
    let raw = resp.headers().get("retry-after")?.to_str().ok()?;
    if let Ok(secs) = raw.trim().parse::<u64>() {
        return Some(secs.saturating_mul(1_000));
    }
    // HTTP-date form: deferred to v0.0.9 (chrono parsing). Returning None
    // is safe; the policy falls back to its computed backoff.
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_targets_production_base_url() {
        let c = ClientConfig::default();
        assert_eq!(c.base_url, "https://api.anthropic.com");
        assert!(c.request_timeout.as_secs() >= 60);
    }

    #[test]
    fn client_builds_with_default_config() {
        let _client = AnthropicClient::new(ClientConfig::default()).expect("build");
    }

    #[test]
    fn retry_after_hint_round_trips_through_error_body() {
        let err = AnthropicError::RequestFailed {
            status: 429,
            body: "__retry_after_ms__=4000\nrate limited".into(),
        };
        assert_eq!(retry_after_hint_from(&err), Some(4000));
    }

    #[test]
    fn retry_after_hint_returns_none_when_absent() {
        let err = AnthropicError::RequestFailed {
            status: 500,
            body: "internal error".into(),
        };
        assert_eq!(retry_after_hint_from(&err), None);
    }

    #[test]
    fn retry_after_hint_returns_none_for_non_request_failed() {
        // Construct via serde_json failure to avoid manually creating reqwest::Error.
        let json_err = serde_json::from_str::<u32>("not json").unwrap_err();
        let e: AnthropicError = json_err.into();
        assert_eq!(retry_after_hint_from(&e), None);
    }
}
