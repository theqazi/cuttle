//! Cuttle policy gate crate.
//!
//! Owns the load-bearing trust-boundary primitives per `docs/TDD.md` §2.4 + §2.5
//! + §3 and D-2026-04-26-17 / D-2026-04-26-21:
//!
//! - [`AttestationBody`]: TTY vs Model provenance, type-level enforcement of T-001.
//! - [`TtyInputCap`]: capability token; only `cuttle-input` crate can mint via
//!   the `__internal_input_cap_factory` re-export. Other crates cannot construct.
//! - [`Decision`]: Allow / Warn / Deny / Prompt graduation per D-2026-04-26-21
//!   (resolves OQ-9).
//!
//! See `docs/DECISIONS.md` D-17 for full provenance.

pub mod capabilities;
pub mod decision;
pub mod primitives;

pub use capabilities::TtyInputCap;
pub use decision::{Decision, ExceptionSuggestion, GateError};
pub use primitives::attestation::{AttestationBody, Provenance};
