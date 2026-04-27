//! `SandboxError`: every failure mode is fail-closed by design.
//!
//! There is no `Permissive` variant. There is no "sandbox unavailable" fallback
//! that runs the subprocess unconfined. If the sandbox cannot be applied, the
//! call fails. This is the v0.1 deterministic-security bedrock per PRD §6.1.1.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    /// `project_root` (or any explicitly-allowed subpath) contains a character
    /// that would require quoting in SBPL. Cuttle rejects rather than escapes
    /// because escape rules differ subtly across macOS releases and a wrong
    /// escape silently widens the profile. Fail closed: caller must
    /// rename / move the project before sandboxing.
    #[error(
        "path contains characters unsafe for SBPL serialization: {path:?}; offending char: {ch:?}"
    )]
    UnsafePath { path: PathBuf, ch: char },

    /// `project_root` is not absolute. SBPL requires absolute paths in
    /// `(subpath ...)` clauses; relative paths would be ambiguous and the
    /// sandbox would deny them. Fail closed at construction time so the
    /// error surfaces before any subprocess spawn.
    #[error("path must be absolute, got: {0:?}")]
    NonAbsolutePath(PathBuf),

    /// The standard macOS sandbox binary was not found on PATH. macOS ships
    /// it at `/usr/bin/sandbox-exec`; absence indicates a corrupted install
    /// or a non-macOS host. Fail closed: caller must verify the install.
    #[error("macOS sandbox binary missing from PATH or {expected:?}")]
    BinaryMissing { expected: PathBuf },

    /// SBPL profile compilation failed (the sandbox binary's parser rejected
    /// our generated scheme). Carries the binary's stderr verbatim so the
    /// operator can diagnose. Profile contents are written to disk only on
    /// the operator's request via `cuttle telemetry --dump-last-profile` so
    /// they don't leak project-path bytes into stable logs.
    #[error("sandbox profile rejected by macOS parser: {stderr}")]
    ProfileRejected { stderr: String },

    /// Subprocess spawn or wait failed at the OS layer (out of FDs, fork
    /// failure, signal during wait). Carries the underlying io::Error.
    #[error("sandboxed spawn failed: {0}")]
    SpawnFailed(#[from] std::io::Error),

    /// Sandboxed subprocess returned a non-zero exit status. This is NOT
    /// always a Cuttle-side failure — operator's tool may legitimately exit
    /// non-zero. The variant exists so the gate can decide policy
    /// (e.g., bash-tool exit=1 is a normal outcome the agent should see).
    #[error("sandboxed subprocess exited with status {status}")]
    NonZeroExit {
        status: i32,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },

    /// `project_root` could not be canonicalized via `realpath(3)`.
    /// Either the path does not exist, a parent has no execute
    /// permission, or there is an I/O error stat'ing the chain. Fail
    /// closed: you cannot sandbox-protect a directory we cannot resolve.
    #[error("could not canonicalize project_root {path:?}: {source}")]
    ProjectRootCanonicalize {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_path_error_message_includes_offending_char() {
        let e = SandboxError::UnsafePath {
            path: PathBuf::from("/tmp/foo\"bar"),
            ch: '"',
        };
        let msg = format!("{e}");
        // Path is rendered via `{:?}` (Debug) which escapes the embedded
        // quote; check for the unique non-quote substring instead.
        assert!(
            msg.contains("foo"),
            "expected path fragment in message, got {msg}"
        );
        assert!(
            msg.contains("offending char: '\"'"),
            "expected offending char marker, got {msg}"
        );
    }

    #[test]
    fn non_absolute_path_error_message_includes_path() {
        let e = SandboxError::NonAbsolutePath(PathBuf::from("relative/path"));
        let msg = format!("{e}");
        assert!(msg.contains("relative/path"));
    }
}
