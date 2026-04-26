//! Cuttle reward-loop registry crate.
//!
//! Implements the AP/VP (anti-pattern / validated-pattern) registry with
//! operator-review queue + signed-provenance per `docs/PRD.md` §6.1.2 row L5
//! and `docs/TDD.md` §3 + T-010 / D-22 / WV-04.
//!
//! Key invariants:
//! - LEARN proposes registry entries; entries land in pending/, NOT canonical.
//! - Promotion to canonical requires a `&TtyInputCap` capability witness.
//! - Each registry mutation carries signed provenance: session ID, model
//!   output that triggered the proposal, score, operator-confirmation timestamp.
//! - Provenance signing key is operator-owned in v0.1 single-operator scope;
//!   per WV-04 / D-2026-04-26-13, the registry chain is anti-forgetfulness and
//!   anti-drift, NOT anti-Sybil against the operator-as-adversary (symmetric
//!   to the audit-log T-003 disclaimer).

pub mod entry;
pub mod registry;
pub mod signing;

pub use entry::{EntryKind, EntryProvenance, RegistryEntry, RegistryEntryError};
pub use registry::{Registry, RegistryError};
pub use signing::{ProvenanceSigningKey, SignedEntry};
