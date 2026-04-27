//! `cuttle sandbox profile` + `cuttle sandbox run` subcommand handlers.
//!
//! Direct operator access to the cuttle-sandbox primitive. Useful for:
//! - Inspecting the SBPL profile cuttle would apply to a project
//!   (`cuttle sandbox profile --project-root ~/code/foo`).
//! - Demonstrating that the sandbox actually blocks malicious operations
//!   (`cuttle sandbox run --project-root /tmp /usr/bin/touch /etc/foo`).
//!
//! These commands are NOT used by `cuttle session start` yet; tool
//! dispatch from the model loop is v0.0.16+. They give the operator a
//! way to validate the sandbox layer in isolation before trusting it
//! with model-driven dispatch.

use crate::args::{SandboxProfileArgs, SandboxRunArgs};
use cuttle_sandbox::{SandboxError, SandboxProfile, SandboxedCommand};
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxCmdError {
    #[error("could not resolve current directory: {0}")]
    NoCwd(std::io::Error),

    #[error("project-root must be absolute; got {0:?}")]
    NotAbsolute(PathBuf),

    #[error("sandbox profile build failed: {0}")]
    Profile(#[from] cuttle_sandbox::SandboxError),

    #[error("writing to stdout failed: {0}")]
    WriteFailed(#[from] std::io::Error),
}

/// Resolve project_root: explicit arg wins; otherwise canonicalized cwd.
fn resolve_project_root(arg: Option<&PathBuf>) -> Result<PathBuf, SandboxCmdError> {
    let p = match arg {
        Some(p) => p.clone(),
        None => std::env::current_dir().map_err(SandboxCmdError::NoCwd)?,
    };
    if !p.is_absolute() {
        return Err(SandboxCmdError::NotAbsolute(p));
    }
    Ok(p)
}

pub fn run_profile<W: Write>(
    args: &SandboxProfileArgs,
    out: &mut W,
) -> Result<(), SandboxCmdError> {
    let project_root = resolve_project_root(args.project_root.as_ref())?;
    let profile = SandboxProfile::for_project(project_root)?;
    write!(out, "{}", profile.render_sbpl())?;
    Ok(())
}

pub fn run_run<W: Write>(args: &SandboxRunArgs, out: &mut W) -> Result<i32, SandboxCmdError> {
    let project_root = resolve_project_root(args.project_root.as_ref())?;
    let profile = SandboxProfile::for_project(project_root)?;
    let mut cmd = SandboxedCommand::new(profile, &args.program);
    for a in &args.args {
        cmd = cmd.arg(a);
    }

    match cmd.spawn_blocking() {
        Ok(output) => {
            out.write_all(&output.stdout)?;
            // The command's own stderr lands on cuttle's stderr so the
            // operator sees it in real time; we propagate exit status.
            std::io::stderr().write_all(&output.stderr).ok();
            Ok(output.status)
        }
        Err(SandboxError::NonZeroExit {
            status,
            stdout,
            stderr,
        }) => {
            // Non-zero exit is information, not a Cuttle-side bug. Mirror
            // the child's stdout/stderr to ours and propagate the status.
            out.write_all(&stdout)?;
            std::io::stderr().write_all(&stderr).ok();
            Ok(status)
        }
        Err(other) => Err(SandboxCmdError::Profile(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_project_root_uses_arg_when_absolute() {
        let p = PathBuf::from("/tmp");
        let resolved = resolve_project_root(Some(&p)).unwrap();
        assert_eq!(resolved, PathBuf::from("/tmp"));
    }

    #[test]
    fn resolve_project_root_rejects_relative_arg() {
        let p = PathBuf::from("relative/path");
        let r = resolve_project_root(Some(&p));
        assert!(matches!(r, Err(SandboxCmdError::NotAbsolute(_))));
    }

    #[test]
    fn resolve_project_root_falls_back_to_cwd_when_none() {
        let r = resolve_project_root(None).unwrap();
        // cwd is always absolute on Unix.
        assert!(r.is_absolute(), "cwd should be absolute, got {r:?}");
    }

    #[test]
    fn run_profile_renders_sbpl_for_explicit_project_root() {
        let args = SandboxProfileArgs {
            project_root: Some(PathBuf::from("/tmp/example-project")),
        };
        let mut out = Vec::new();
        run_profile(&args, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.starts_with("(version 1)"), "{s}");
        assert!(s.contains("(deny default)"));
        assert!(s.contains("(subpath \"/tmp/example-project\")"));
    }

    #[test]
    fn run_profile_errors_on_relative_project_root() {
        let args = SandboxProfileArgs {
            project_root: Some(PathBuf::from("relative")),
        };
        let mut out = Vec::new();
        let r = run_profile(&args, &mut out);
        assert!(matches!(r, Err(SandboxCmdError::NotAbsolute(_))));
    }

    /// macOS-only: actually run a sandboxed echo through the CLI handler.
    /// Proves the wiring (parser → SandboxProfile → SandboxedCommand →
    /// stdout) is intact end-to-end.
    #[cfg(target_os = "macos")]
    #[test]
    fn run_run_executes_sandboxed_echo() {
        let args = SandboxRunArgs {
            project_root: Some(PathBuf::from("/tmp")),
            program: PathBuf::from("/bin/echo"),
            args: vec!["sandbox-cli-smoke".to_string()],
        };
        let mut out = Vec::new();
        let status = match run_run(&args, &mut out) {
            Ok(s) => s,
            // CI without the sandbox binary: skip silently. The non-macos
            // unit tests above already cover the parser + render path.
            Err(SandboxCmdError::Profile(SandboxError::BinaryMissing { .. })) => return,
            Err(e) => panic!("unexpected error: {e}"),
        };
        assert_eq!(status, 0);
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("sandbox-cli-smoke"), "stdout: {s}");
    }

    /// macOS-only adversarial test: the CLI handler MUST surface a
    /// sandbox denial when the operator tries something the profile
    /// blocks. `/usr/bin/touch` is NOT in the default-allowed exec
    /// set, so the sandbox kills the spawn at the exec step (before
    /// touch even runs). Either outcome is a correct denial; this
    /// test accepts both rather than asserting a specific failure
    /// mode of the sandbox-exec stderr-sniff heuristic.
    #[cfg(target_os = "macos")]
    #[test]
    fn run_run_surfaces_sandbox_denial() {
        let args = SandboxRunArgs {
            project_root: Some(PathBuf::from("/tmp")),
            program: PathBuf::from("/usr/bin/touch"),
            args: vec!["/etc/cuttle-cli-should-be-denied".to_string()],
        };
        let mut out = Vec::new();
        match run_run(&args, &mut out) {
            // CI without the sandbox binary: skip silently.
            Err(SandboxCmdError::Profile(SandboxError::BinaryMissing { .. })) => {}
            // Sandbox correctly killed at the SBPL parse / exec step.
            // Both outcomes (Err::Profile and Err::Profile via ProfileRejected
            // sniff) are valid denials.
            Err(SandboxCmdError::Profile(_)) => {}
            // OR: exec succeeded but the inner write was blocked, surfacing
            // as a non-zero exit with touch's own stderr.
            Ok(status) => {
                assert_ne!(
                    status, 0,
                    "touch /etc/<x> must NOT succeed under the default sandbox"
                );
            }
            Err(e) => panic!("unexpected error class: {e}"),
        }
    }
}
