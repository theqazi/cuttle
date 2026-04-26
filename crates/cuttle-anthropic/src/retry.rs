//! Retry policy as a pure decision function.
//!
//! `RetryPolicy::decide(attempt, error)` returns a `RetryDecision` that the
//! HTTP loop in `client.rs` interprets. Keeping the policy pure (no I/O, no
//! sleeping, no clock reads) means it tests in nanoseconds and the call site
//! stays a thin loop.
//!
//! Backoff: exponential with full jitter, capped. The base is 250ms, the cap
//! is 8s, the multiplier is 2x. With 5 max attempts the worst-case wait
//! sequence is approximately 0.25s, 0.5s, 1s, 2s, 4s — total < 8s of wall
//! clock before giving up. This keeps interactive feel intact (operator is
//! waiting at a TTY) while absorbing brief upstream blips.
//!
//! Per `Retry-After` (RFC 7231 §7.1.3): when the API returns 429 or 503 with
//! a `Retry-After` header, the policy honors that hint by passing the value
//! through `RetryDecision::AfterMillis(n)`. Honoring the server hint is the
//! correct behavior here even when it exceeds our normal cap, because the
//! server is signalling a cooldown — ignoring it just gets us rate-limited
//! again.
//!
//! Streaming safety: the policy does NOT distinguish "no bytes received"
//! from "partial stream" — that is the streaming code path's responsibility
//! (v0.0.9). The non-streaming `messages()` path can always retry safely
//! because the request is idempotent at the HTTP level (POST with the same
//! body produces the same response).

use crate::error::AnthropicError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetryDecision {
    /// Stop. Either the error is non-retryable or the budget is exhausted.
    /// The caller should wrap the underlying error in
    /// `AnthropicError::RetryExhausted` only when the budget was the reason
    /// for stopping; non-retryable errors propagate as-is.
    GiveUp,
    /// Sleep for `n` milliseconds, then retry the same request.
    AfterMillis(u64),
}

#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_ms: u64,
    pub cap_ms: u64,
    pub multiplier: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_attempts: 5,
            base_ms: 250,
            cap_ms: 8_000,
            multiplier: 2,
        }
    }
}

impl RetryPolicy {
    /// Pure decision: given the failed attempt number (1-indexed: the 1st
    /// attempt's failure → `attempt = 1`) and the error, decide whether to
    /// retry and after how long.
    ///
    /// `retry_after_hint_ms` lets the caller pipe through a server-provided
    /// `Retry-After` header (parsed elsewhere); when present, it wins over
    /// the computed exponential value.
    ///
    /// Backoff is deterministic here (no jitter) so tests are stable. The
    /// jitter is injected at the client edge by `client.rs` if needed; the
    /// trade-off (deterministic test vs thundering-herd protection) goes
    /// jitter's way only when there are multiple parallel callers, which
    /// v0.1 single-operator does not have.
    pub fn decide(
        &self,
        attempt: u32,
        error: &AnthropicError,
        retry_after_hint_ms: Option<u64>,
    ) -> RetryDecision {
        if !error.is_retryable() {
            return RetryDecision::GiveUp;
        }
        if attempt >= self.max_attempts {
            return RetryDecision::GiveUp;
        }
        if let Some(ms) = retry_after_hint_ms {
            // Honor the server hint even if it exceeds cap. See module doc.
            return RetryDecision::AfterMillis(ms);
        }
        let computed = self.compute_backoff_ms(attempt);
        RetryDecision::AfterMillis(computed)
    }

    fn compute_backoff_ms(&self, attempt: u32) -> u64 {
        // attempt is 1-indexed. After failure 1, wait base. After failure 2,
        // wait base*mult. Etc. Saturating math keeps wide multipliers safe.
        let exp = attempt.saturating_sub(1);
        let mult = (self.multiplier as u64).saturating_pow(exp);
        let raw = self.base_ms.saturating_mul(mult);
        raw.min(self.cap_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retryable() -> AnthropicError {
        AnthropicError::RequestFailed {
            status: 503,
            body: String::new(),
        }
    }

    fn fatal() -> AnthropicError {
        AnthropicError::RequestFailed {
            status: 401,
            body: String::new(),
        }
    }

    #[test]
    fn non_retryable_error_gives_up_immediately() {
        let p = RetryPolicy::default();
        assert_eq!(p.decide(1, &fatal(), None), RetryDecision::GiveUp);
    }

    #[test]
    fn budget_exhausted_after_max_attempts() {
        let p = RetryPolicy::default();
        // max_attempts=5; on attempt=5 (5th failure) we give up.
        assert_eq!(p.decide(5, &retryable(), None), RetryDecision::GiveUp);
    }

    #[test]
    fn first_failure_waits_base() {
        let p = RetryPolicy::default();
        assert_eq!(
            p.decide(1, &retryable(), None),
            RetryDecision::AfterMillis(250)
        );
    }

    #[test]
    fn backoff_doubles_then_caps() {
        let p = RetryPolicy::default();
        // 250, 500, 1000, 2000, ... cap at 8000.
        assert_eq!(p.compute_backoff_ms(1), 250);
        assert_eq!(p.compute_backoff_ms(2), 500);
        assert_eq!(p.compute_backoff_ms(3), 1000);
        assert_eq!(p.compute_backoff_ms(4), 2000);
        assert_eq!(p.compute_backoff_ms(5), 4000);
        assert_eq!(p.compute_backoff_ms(6), 8000);
        // Cap holds even past the multiplier exhausting u32.
        assert_eq!(p.compute_backoff_ms(40), 8000);
    }

    #[test]
    fn server_hint_overrides_computed_backoff() {
        let p = RetryPolicy::default();
        // Server says wait 30s; honor it even though our cap is 8s.
        assert_eq!(
            p.decide(1, &retryable(), Some(30_000)),
            RetryDecision::AfterMillis(30_000)
        );
    }

    #[test]
    fn server_hint_ignored_when_error_is_fatal() {
        let p = RetryPolicy::default();
        // 401 is fatal; no amount of Retry-After should make us retry.
        assert_eq!(p.decide(1, &fatal(), Some(100)), RetryDecision::GiveUp);
    }

    #[test]
    fn custom_policy_respects_smaller_max_attempts() {
        let p = RetryPolicy {
            max_attempts: 2,
            base_ms: 100,
            cap_ms: 1000,
            multiplier: 2,
        };
        assert_eq!(
            p.decide(1, &retryable(), None),
            RetryDecision::AfterMillis(100)
        );
        assert_eq!(p.decide(2, &retryable(), None), RetryDecision::GiveUp);
    }

    #[test]
    fn saturating_arithmetic_prevents_overflow() {
        let p = RetryPolicy {
            max_attempts: u32::MAX,
            base_ms: u64::MAX / 2,
            cap_ms: u64::MAX,
            multiplier: 1_000_000,
        };
        // Should not panic and should not overflow; result clamps to cap.
        let _ = p.compute_backoff_ms(100);
    }
}
