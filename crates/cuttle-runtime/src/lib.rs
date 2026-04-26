//! Cuttle runtime crate.
//!
//! Owns three trust-boundary domain primitives per `docs/TDD.md` §2.4 and
//! D-2026-04-26-17 capability-token pattern:
//!
//! - [`primitives::LockfilePath`]: validated canonical path under `~/.cuttle/run/`.
//! - [`primitives::TierClassification`]: enum for SYSTEM-tier ceremony classification.
//! - [`primitives::SessionId`]: opaque session identifier wrapper.
//!
//! Implements the lockfile HMAC + parent-PID + signing-key-in-memory mechanism
//! per D-2026-04-26-23 (WV-02 closure).
//!
//! See `docs/DECISIONS.md` D-17, D-23, D-29 for full provenance.

pub mod lockfile;
pub mod primitives;
pub mod signing_key;

pub use lockfile::{LockfileContents, LockfileError, read_lockfile, write_lockfile};
pub use primitives::{LockfilePath, LockfilePathError, SessionId, TierClassification};
pub use signing_key::SigningKey;
