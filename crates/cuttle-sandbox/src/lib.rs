//! Cuttle macOS sandbox primitive.
//!
//! Per `docs/TDD.md` §4 + D-2026-04-26-26: v0.1 sandboxes every
//! Cuttle-dispatched subprocess with the standard macOS sandbox binary,
//! generating an SBPL (Sandbox Profile Language) profile per project root.
//! Every Bash and file-op tool call goes through here; the gate is the
//! choke point that calls `cuttle-sandbox::spawn()`.
//!
//! v0.0.9 scope:
//! - `SandboxProfile` struct + builder.
//! - `render_sbpl()` pure function with path validation (rejects unsafe
//!   characters rather than attempting risky shell-escape gymnastics).
//! - `SandboxedCommand::spawn_blocking()` over `std::process::Command`.
//! - `SandboxError` taxonomy with fail-closed semantics on every
//!   diagnosable failure (sandbox binary missing, profile compile fail,
//!   spawn fail).
//!
//! v0.0.10 scope (deferred):
//! - rlimit enforcement (RLIMIT_CPU / DATA / NOFILE / NPROC) via
//!   `pre_exec` hook. This crosses an unsafe libc boundary and gets its
//!   own threat-model pass before landing.
//! - Async `spawn()` over `tokio::process::Command` for the streaming
//!   subprocess case (only relevant once `cuttle-runtime` ties in).
//!
//! Failure-mode discipline (CLAUDE.md §0b): every error path documents
//! whether it fails open or closed. The whole crate fails closed by
//! design — there is no permissive fallback for "sandbox unavailable."

pub mod error;
pub mod profile;
pub mod spawn;

pub use error::SandboxError;
pub use profile::{SandboxProfile, default_allowed_binaries};
pub use spawn::{SandboxedCommand, SandboxedOutput};
