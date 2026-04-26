//! `SandboxedCommand`: builder + blocking spawn for the macOS sandbox.
//!
//! Workflow:
//! 1. Caller builds a `SandboxedCommand` with program + args + (optional)
//!    cwd + (optional) env-overrides.
//! 2. `spawn_blocking()` renders the SBPL profile, writes it to a private
//!    temp file (mode 0600 in the operator's tmp dir), invokes the standard
//!    macOS sandbox binary as `<binary> -f <profile-file> <program> <args>`,
//!    waits for exit, returns `SandboxedOutput`.
//! 3. The temp file is deleted when the `NamedTempFile` handle drops (end
//!    of `spawn_blocking`).
//!
//! Environment scoping: by default the spawned process inherits the caller's
//! env (matching `std::process::Command` defaults). Callers that want a
//! pruned env should use `with_clear_env()` followed by `env(k, v)` calls,
//! same shape as the std API.
//!
//! Failure semantics: every reachable failure surfaces as `SandboxError`.
//! Non-zero exit is `SandboxError::NonZeroExit` carrying both stdout and
//! stderr bytes for the gate to inspect — even a non-zero exit is
//! information the agent needs (compile errors, test failures, etc.).

use crate::error::SandboxError;
use crate::profile::SandboxProfile;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Path of the macOS sandbox binary on a stock install. v0.0.9 hard-codes
/// the install path; v0.2 will probe via `which` to support nonstandard
/// paths (rare in practice on macOS).
const SANDBOX_BIN: &str = "/usr/bin/sandbox-exec";

#[derive(Debug)]
pub struct SandboxedCommand {
    profile: SandboxProfile,
    program: OsString,
    args: Vec<OsString>,
    cwd: Option<PathBuf>,
    env_clear: bool,
    env_overrides: Vec<(OsString, OsString)>,
}

#[derive(Debug, Clone)]
pub struct SandboxedOutput {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl SandboxedCommand {
    pub fn new<S: Into<OsString>>(profile: SandboxProfile, program: S) -> Self {
        SandboxedCommand {
            profile,
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env_clear: false,
            env_overrides: Vec::new(),
        }
    }

    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.args.push(a.as_ref().to_os_string());
        }
        self
    }

    pub fn current_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.cwd = Some(dir.as_ref().to_path_buf());
        self
    }

    pub fn with_clear_env(mut self) -> Self {
        self.env_clear = true;
        self
    }

    pub fn env<K: AsRef<OsStr>, V: AsRef<OsStr>>(mut self, key: K, val: V) -> Self {
        self.env_overrides
            .push((key.as_ref().to_os_string(), val.as_ref().to_os_string()));
        self
    }

    /// Render the profile, write to a temp file, invoke the standard sandbox
    /// binary, wait for exit, return captured output. Blocks the calling
    /// thread; callers in async contexts should wrap with
    /// `tokio::task::spawn_blocking`.
    pub fn spawn_blocking(self) -> Result<SandboxedOutput, SandboxError> {
        // Pre-flight: the sandbox binary must exist. Fail closed at the
        // earliest possible point so the operator never sees a confusing
        // "command not found" buried in a wider error.
        let sandbox_path = Path::new(SANDBOX_BIN);
        if !sandbox_path.exists() {
            return Err(SandboxError::BinaryMissing {
                expected: sandbox_path.to_path_buf(),
            });
        }

        let sbpl = self.profile.render_sbpl();
        let mut profile_file = tempfile::Builder::new()
            .prefix("cuttle-sbpl-")
            .suffix(".sb")
            .tempfile()?;
        profile_file.write_all(sbpl.as_bytes())?;
        profile_file.as_file_mut().sync_all()?;

        let mut cmd = Command::new(SANDBOX_BIN);
        cmd.arg("-f")
            .arg(profile_file.path())
            .arg(&self.program)
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        if self.env_clear {
            cmd.env_clear();
        }
        for (k, v) in &self.env_overrides {
            cmd.env(k, v);
        }

        let output = cmd.output()?;

        // Map exit status. On Unix, .code() returns None when killed by
        // signal; we synthesize -1 in that case (the caller still sees
        // stdout/stderr, and the negative code is a clear signal-killed
        // marker).
        let status = output.status.code().unwrap_or(-1);

        // Sniff for SBPL parse rejection: the standard sandbox binary
        // prints "sandbox-exec: ... compile failed" on stderr and exits
        // non-zero before it ever exec's the target. Surface that as a
        // distinct error variant so the operator gets a profile-specific
        // diagnostic instead of a generic NonZero.
        if status != 0 {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            if stderr_str.contains("sandbox-exec") && stderr_str.contains("failed") {
                return Err(SandboxError::ProfileRejected {
                    stderr: stderr_str.into_owned(),
                });
            }
            return Err(SandboxError::NonZeroExit {
                status,
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }

        Ok(SandboxedOutput {
            status,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_profile() -> SandboxProfile {
        SandboxProfile::for_project(PathBuf::from("/tmp")).unwrap()
    }

    #[test]
    fn builder_records_program_and_args() {
        let c = SandboxedCommand::new(test_profile(), "/bin/echo")
            .arg("hello")
            .arg("world");
        assert_eq!(c.program, OsString::from("/bin/echo"));
        assert_eq!(c.args.len(), 2);
        assert_eq!(c.args[0], OsString::from("hello"));
    }

    #[test]
    fn builder_args_iter_preserves_order() {
        let c = SandboxedCommand::new(test_profile(), "/bin/echo").args(["a", "b", "c"]);
        assert_eq!(c.args.len(), 3);
        assert_eq!(c.args[2], OsString::from("c"));
    }

    #[test]
    fn builder_clears_env_on_request() {
        let c = SandboxedCommand::new(test_profile(), "/bin/echo")
            .with_clear_env()
            .env("FOO", "bar");
        assert!(c.env_clear);
        assert_eq!(c.env_overrides.len(), 1);
    }

    #[test]
    fn builder_records_cwd() {
        let c = SandboxedCommand::new(test_profile(), "/bin/echo").current_dir("/tmp/somewhere");
        assert_eq!(c.cwd, Some(PathBuf::from("/tmp/somewhere")));
    }

    /// macOS-only smoke: actually invoke a sandboxed `/bin/echo` and
    /// verify stdout. This is the single integration test that proves the
    /// whole pipeline (profile render → tempfile → spawn → capture)
    /// works end-to-end on a real host. cfg-gated so non-macOS CI passes.
    #[cfg(target_os = "macos")]
    #[test]
    fn sandboxed_echo_returns_expected_stdout() {
        let c = SandboxedCommand::new(test_profile(), "/bin/echo").arg("cuttle-sandbox-smoke");
        let out = match c.spawn_blocking() {
            Ok(o) => o,
            Err(SandboxError::BinaryMissing { .. }) => {
                // CI without the sandbox binary: skip silently rather than
                // fail; the profile render tests still cover the codegen.
                return;
            }
            Err(e) => panic!("unexpected error: {e}"),
        };
        assert_eq!(out.status, 0);
        let s = String::from_utf8_lossy(&out.stdout);
        assert!(
            s.contains("cuttle-sandbox-smoke"),
            "stdout did not contain expected echo: {s}"
        );
    }

    /// macOS-only adversarial test: prove the sandbox actually DENIES
    /// writes outside `project_root`. A renderer that produces the right
    /// string but fails to enforce is worse than no sandbox at all
    /// (false sense of security). Tries to write to /etc which the
    /// profile does not list under file-write*; the touch must fail.
    #[cfg(target_os = "macos")]
    #[test]
    fn sandbox_denies_write_outside_project_root() {
        let profile = SandboxProfile::for_project(PathBuf::from("/tmp"))
            .unwrap()
            .with_allowed_binaries(vec![PathBuf::from("/usr/bin/touch")])
            .unwrap();
        // /etc is read-allowed but NOT write-allowed; touch should fail.
        let c = SandboxedCommand::new(profile, "/usr/bin/touch")
            .arg("/etc/cuttle-sandbox-should-be-denied");
        match c.spawn_blocking() {
            Err(SandboxError::BinaryMissing { .. }) => {}
            Err(SandboxError::NonZeroExit { stderr, .. }) => {
                let s = String::from_utf8_lossy(&stderr);
                assert!(
                    s.contains("Permission denied")
                        || s.contains("Operation not permitted")
                        || s.contains("Read-only"),
                    "expected denial in stderr, got: {s}"
                );
            }
            Ok(out) => panic!(
                "sandbox FAILED to block write to /etc; status={} stdout={:?} stderr={:?}",
                out.status,
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ),
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
}
