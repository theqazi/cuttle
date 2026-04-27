//! Cuttle audit log crate.
//!
//! HMAC-chained append-only JSONL audit log per `docs/TDD.md` §5.1 and
//! D-2026-04-26-27. Tool-registration tagging contract per D-2026-04-26-28
//! (WV-03 closure). PII redaction via `Redactor` trait per D-2026-04-26-30
//! (OQ-12 resolution).
//!
//! See `docs/DECISIONS.md` D-27, D-28, D-29, D-30 for full provenance.

pub mod chain;
pub mod event;
pub mod redact;
pub mod tagging;

pub use chain::{
    read_entries_unverified, verify_chain, AuditChain, AuditChainKey, AuditEntry, AuditError,
};
pub use event::AuditEvent;
pub use redact::{DefaultRedactor, Redactor};
pub use tagging::{PiiPosture, ToolRegistry, ToolTag};
