//! Error taxonomy for the Anthropic client.
//!
//! Errors carry the information the gate + audit log + retry policy need to
//! make routing decisions. Specifically: each variant exposes a stable
//! classification that the retry policy reads via `is_retryable()` so
//! retry logic stays out of the call site.
//!
//! Per CLAUDE.md §0c (operational empathy): error messages must be specific,
//! contextual, actionable. The `RequestFailed` variant carries the HTTP
//! status + the body text returned by the API; the `Network` variant carries
//! the underlying reqwest error. We deliberately do NOT redact response
//! bodies here — redaction happens at the audit-log boundary
//! (`cuttle-audit::DefaultRedactor`).

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnthropicError {
    /// Network-layer failure: DNS, TCP, TLS handshake, connection reset.
    /// Always retryable per the retry policy.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// HTTP request returned a non-2xx status. `status` + `body` together
    /// give the caller everything needed to diagnose without a packet capture.
    /// Retry behavior depends on `status`: see `is_retryable()`.
    #[error("HTTP {status}: {body}")]
    RequestFailed { status: u16, body: String },

    /// Response body deserialized cleanly as JSON but the structure did not
    /// match the expected `Response` schema. Indicates an upstream API change
    /// or a Cuttle model-type bug; never retried.
    #[error("response shape mismatch: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// Request-construction failure: bad URL, bad header value. Programmer
    /// bug; never retried.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Retry budget exhausted. Carries the last error encountered so the
    /// caller can surface it to the operator + the audit log.
    #[error("retry budget exhausted after {attempts} attempts; last error: {last}")]
    RetryExhausted {
        attempts: u32,
        last: Box<AnthropicError>,
    },
}

impl AnthropicError {
    /// Classification used by `RetryPolicy::decide()`.
    ///
    /// `RequestFailed` is retryable on 408 (request timeout), 409 (conflict),
    /// 425 (too early), 429 (rate limit), 500, 502, 503, 504, 529 (overloaded).
    /// Authentication failures (401, 403) and client errors (400, 404, 422)
    /// are NEVER retried — they would just burn the retry budget for the
    /// same outcome. The 408/409/425 set is included because Anthropic and
    /// upstream proxies do return these on transient conditions.
    pub fn is_retryable(&self) -> bool {
        match self {
            AnthropicError::Network(_) => true,
            AnthropicError::RequestFailed { status, .. } => {
                matches!(status, 408 | 409 | 425 | 429 | 500 | 502 | 503 | 504 | 529)
            }
            AnthropicError::Deserialize(_)
            | AnthropicError::InvalidRequest(_)
            | AnthropicError::RetryExhausted { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_status_is_retryable() {
        let e = AnthropicError::RequestFailed {
            status: 429,
            body: "rate limited".into(),
        };
        assert!(e.is_retryable());
    }

    #[test]
    fn server_error_5xx_is_retryable() {
        for s in [500, 502, 503, 504, 529] {
            let e = AnthropicError::RequestFailed {
                status: s,
                body: String::new(),
            };
            assert!(e.is_retryable(), "status {} should be retryable", s);
        }
    }

    #[test]
    fn auth_failure_is_not_retryable() {
        for s in [401, 403] {
            let e = AnthropicError::RequestFailed {
                status: s,
                body: String::new(),
            };
            assert!(!e.is_retryable(), "status {} must not be retryable", s);
        }
    }

    #[test]
    fn client_error_4xx_is_not_retryable() {
        for s in [400, 404, 422] {
            let e = AnthropicError::RequestFailed {
                status: s,
                body: String::new(),
            };
            assert!(!e.is_retryable(), "status {} must not be retryable", s);
        }
    }

    #[test]
    fn deserialize_error_is_not_retryable() {
        let json_err = serde_json::from_str::<u32>("not json").unwrap_err();
        let e: AnthropicError = json_err.into();
        assert!(!e.is_retryable());
    }
}
