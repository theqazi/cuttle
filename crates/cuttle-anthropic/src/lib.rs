//! Cuttle Anthropic API client.
//!
//! Thin wrapper over `reqwest` + `serde` per `docs/TDD.md` §1. v0.1 deliberately
//! does NOT depend on a third-party Anthropic SDK to keep the supply-chain
//! attack surface narrow (PRD §9 + sbom-license posture; CamoLeak / postmark-mcp
//! incident class). Move to a community SDK in v0.2 only after `legal-review`
//! + `sbom-license` pass.
//!
//! v0.0.8 scope:
//! - Domain types (`Model`, `Role`, `Message`, `Content`, `Request`, `Response`,
//!   `Usage`).
//! - `AnthropicClient` with non-streaming `messages()` over HTTPS + rustls.
//! - `AnthropicError` taxonomy distinguishing retryable from terminal errors.
//! - `RetryPolicy` as a pure testable function (exponential backoff on
//!   429 + 5xx; NEVER retry once the first byte of a streamed response has
//!   landed; that is the SSE-replay double-billing trap called out in TDD §1).
//!
//! Deferred to subsequent commits:
//! - `messages_stream()` (SSE) → v0.0.9.
//! - `cache_control` on system blocks (prompt cache) → v0.0.10.
//!
//! Credential boundary: `messages()` borrows `&ApiKey` and calls
//! `ApiKey::consume()` exactly once per call. Retries within a single
//! `messages()` invocation reuse the consumed bytes (lifetime-bound to the
//! `&ApiKey` reference). Per-request fresh-from-Keychain fetch is the
//! credential-vault crate's contract, not this crate's.

pub mod client;
pub mod error;
pub mod model;
pub mod retry;
pub mod stream;

pub use client::{AnthropicClient, ClientConfig};
pub use error::AnthropicError;
pub use model::{Content, Message, Model, Request, Response, Role, StopReason, Usage};
pub use retry::{RetryDecision, RetryPolicy};
pub use stream::{
    ContentBlockDelta, ContentBlockStartPayload, ErrorPayload, MessageDeltaPayload,
    MessageStartPayload, StreamEvent, parse_response_stream,
};
