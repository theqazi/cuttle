//! Cuttle credential vault crate.
//!
//! Owns two trust-boundary domain primitives per `docs/TDD.md` §2.4 and
//! D-2026-04-26-17 capability-token pattern:
//!
//! - [`primitives::ApiKey`]: read-once + zeroize-on-drop.
//! - [`primitives::HelperHash`]: constant-time-comparable SHA-256 of helper script.
//!
//! Constructors are `pub(crate)`; only this crate's submodules can mint.
//! See `docs/DECISIONS.md` D-15, D-17, D-18 for full provenance.

pub mod primitives;
pub mod record;

pub use primitives::api_key::{ApiKey, ApiKeyEnvError};
pub use primitives::helper_hash::HelperHash;
pub use record::{CredentialBackend, CredentialRecord};
