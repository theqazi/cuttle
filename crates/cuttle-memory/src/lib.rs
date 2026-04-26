//! Cuttle auto-memory crate.
//!
//! Owns the text-provenance pair per `docs/TDD.md` §2.4 + §6 and
//! D-2026-04-26-17 capability-token pattern + D-2026-04-26-31 layout:
//!
//! - [`OperatorAuthoredText`]: trusted-by-default; operator typed at TTY
//!   or wrote to operator-controlled surfaces.
//! - [`ModelAuthoredText`]: untrusted-by-default; quarantined until
//!   explicit operator promotion via TTY.
//!
//! Promotion requires `&TtyInputCap` per D-17 capability-token witness.

pub mod layout;
pub mod promotion;
pub mod text;

pub use layout::{MemoryLayout, MemoryLayoutError};
pub use promotion::{prompt_promotion, PromotionDecision, MemoryError};
pub use text::{ModelAuthoredText, OperatorAuthoredText};
