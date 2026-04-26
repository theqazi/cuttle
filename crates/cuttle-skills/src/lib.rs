//! Cuttle skills loader crate.
//!
//! Implements the WV-05 / D-2026-04-26-25 Unicode allowlist (skills
//! containing Unicode characters outside known-safe categories fail to load
//! rather than load-with-stripping).
//!
//! Per `docs/PRD.md` §6.1.1 skills loader bullet + `docs/TDD.md` §3 + §3.6.
//!
//! The allowlist is conservative: Latin alphanumeric + common ASCII
//! punctuation + ASCII whitespace + a small set of named-safe extras
//! (CJK ranges, accented Latin, dashes, quotes). Categories not on the
//! list are treated as novel and trigger fail-closed.

pub mod allowlist;
pub mod content;

pub use allowlist::{is_codepoint_safe, scan_for_disallowed, AllowlistViolation};
pub use content::{SkillContent, SkillContentError};
