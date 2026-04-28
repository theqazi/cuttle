//! `SandboxProfile`: builder + SBPL renderer.
//!
//! Path safety: rather than implement SBPL string escape (rules differ across
//! macOS releases; a wrong escape silently widens the profile), we reject any
//! path containing a character that would require quoting. The set of
//! disallowed characters: `"` (string delimiter), `\` (escape), `\n` / `\r`
//! (line breaks), `\0` (null), and any C0 control character. Operators with
//! exotic project paths get a clear error and rename rather than a silent
//! sandbox bypass.
//!
//! Per TDD §4.1 / D-26 the profile MUST be regenerated per project root.
//! Caching by project-root hash is a v0.2 optimization; v0.0.9 always
//! re-renders.

use crate::error::SandboxError;
use std::path::{Path, PathBuf};

/// Default set of binaries the sandbox allows the agent to exec. The
/// guiding principle is: deny by default, allow exactly the binaries a
/// daily-driver software-engineering session needs. Anything else
/// requires the operator to extend the allowlist via `cuttle config`.
pub fn default_allowed_binaries() -> Vec<PathBuf> {
    [
        "/bin/sh",
        "/bin/bash",
        "/bin/zsh",
        "/bin/cat",
        "/bin/cp",
        "/bin/echo",
        "/bin/ls",
        "/bin/mkdir",
        "/bin/mv",
        "/bin/rm",
        "/bin/test",
        "/usr/bin/awk",
        "/usr/bin/diff",
        "/usr/bin/env",
        "/usr/bin/find",
        "/usr/bin/git",
        "/usr/bin/grep",
        "/usr/bin/head",
        "/usr/bin/python3",
        // The single /usr/bin/python3 stub above is the entry point.
        // The render_sbpl() side ALSO emits subpath rules for the
        // Python framework dirs so the 4-binary re-exec chain
        // (stub -> CLT symlink -> versioned -> .app/Contents/MacOS/Python)
        // works without enumerating each binary.
        "/usr/bin/sed",
        "/usr/bin/sort",
        "/usr/bin/tail",
        "/usr/bin/uniq",
        "/usr/bin/wc",
        "/usr/bin/which",
        "/usr/bin/xargs",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

#[derive(Clone, Debug)]
pub struct SandboxProfile {
    project_root: PathBuf,
    allowed_subprocess_paths: Vec<PathBuf>,
    /// rlimit fields: stored but NOT yet enforced in v0.0.9. v0.0.10 wires
    /// them via `pre_exec`. They are recorded here so callers can already
    /// configure them and the storage shape stays stable across the
    /// rlimit-enforcement landing.
    pub cpu_limit_secs: u32,
    pub mem_limit_mb: u32,
    pub max_open_fds: u32,
    pub max_subprocesses: u32,
}

impl SandboxProfile {
    /// Construct with default rlimits + default allowed-binaries set.
    /// Validates the project root is absolute + SBPL-safe; returns
    /// `SandboxError` on either failure.
    ///
    /// Canonicalizes the project_root before storing: macOS `/tmp` is a
    /// symlink to `/private/tmp` and SBPL `(subpath ...)` does not
    /// follow symlinks, so a sandboxed program's `getcwd()` would fail
    /// the file-read check if we kept the symlink form. Canonicalizing
    /// also requires the path to EXIST, which is the right contract:
    /// you can't sandbox-protect a directory that isn't there.
    pub fn for_project(project_root: PathBuf) -> Result<Self, SandboxError> {
        validate_path_for_sbpl(&project_root)?;
        let canonical =
            project_root
                .canonicalize()
                .map_err(|e| SandboxError::ProjectRootCanonicalize {
                    path: project_root.clone(),
                    source: e,
                })?;
        // The canonical form must also be SBPL-safe. realpath(3) can in
        // theory produce paths with characters that need escaping; check
        // again so we never silently widen the SBPL via canonicalization.
        validate_path_for_sbpl(&canonical)?;
        Ok(SandboxProfile {
            project_root: canonical,
            allowed_subprocess_paths: default_allowed_binaries(),
            cpu_limit_secs: 60,
            mem_limit_mb: 1024,
            max_open_fds: 256,
            max_subprocesses: 16,
        })
    }

    /// Replace the default allowed-binaries set. Each path is validated
    /// the same way `project_root` is; mixing safe + unsafe paths fails
    /// closed.
    pub fn with_allowed_binaries(mut self, binaries: Vec<PathBuf>) -> Result<Self, SandboxError> {
        for p in &binaries {
            validate_path_for_sbpl(p)?;
        }
        self.allowed_subprocess_paths = binaries;
        Ok(self)
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn allowed_subprocess_paths(&self) -> &[PathBuf] {
        &self.allowed_subprocess_paths
    }

    /// Render the SBPL profile string suitable for passing to the macOS
    /// sandbox binary via `-f <file>`. Pure function: same input → same
    /// output. No I/O; no clock; no env reads.
    pub fn render_sbpl(&self) -> String {
        let project = self.project_root.display();
        let allowed_execs: String = self
            .allowed_subprocess_paths
            .iter()
            .map(|p| format!("    (literal \"{}\")\n", p.display()))
            .collect();

        // The file-read subpath set is what dyld + libsystem actually
        // need on a stock macOS install: lib + share + system frameworks
        // + /private/var (dyld cache lives there) + the binary directories
        // we allow process-exec on. The (literal "/") line lets dyld stat
        // the root directory itself; without it, the loader aborts the
        // child before main() runs. The IP "localhost" form is the only
        // accepted host literal in modern SBPL — IP literals like
        // "127.0.0.1" are rejected at parse time on macOS 14+.
        format!(
            "(version 1)\n\
             (deny default)\n\
             (allow process-fork)\n\
             (allow signal (target self))\n\
             (allow file-read*\n\
             \x20   (subpath \"{project}\")\n\
             \x20   (subpath \"/bin\")\n\
             \x20   (subpath \"/sbin\")\n\
             \x20   (subpath \"/usr/bin\")\n\
             \x20   (subpath \"/usr/sbin\")\n\
             \x20   (subpath \"/usr/lib\")\n\
             \x20   (subpath \"/usr/share\")\n\
             \x20   (subpath \"/System/Library\")\n\
             \x20   (subpath \"/Library\")\n\
             \x20   (subpath \"/private/etc\")\n\
             \x20   (subpath \"/etc\")\n\
             \x20   (subpath \"/var\")\n\
             \x20   (subpath \"/private/var\")\n\
             \x20   (literal \"/\")\n\
             \x20   (literal \"/dev/null\")\n\
             \x20   (literal \"/dev/random\")\n\
             \x20   (literal \"/dev/urandom\")\n\
             \x20   (literal \"/dev/dtracehelper\")\n\
             \x20   (subpath \"/dev/fd\"))\n\
             (deny file-read*\n\
             \x20   (subpath \"/var/folders\")\n\
             \x20   (subpath \"/private/var/folders\"))\n\
             (allow file-read*\n\
             \x20   (subpath \"{project}\"))\n\
             (allow file-write*\n\
             \x20   (subpath \"{project}\")\n\
             \x20   (literal \"/dev/null\")\n\
             \x20   (literal \"/dev/dtracehelper\"))\n\
             (allow file-write-data\n\
             \x20   (literal \"/dev/stdout\")\n\
             \x20   (literal \"/dev/stderr\"))\n\
             (allow process-exec\n{allowed_execs}\
             \x20   (subpath \"/Library/Developer/CommandLineTools/Library/Frameworks/Python3.framework\")\n\
             \x20   (subpath \"/Library/Developer/CommandLineTools/usr/bin\")\n\
             \x20   (subpath \"/opt/homebrew/Cellar\")\n\
             \x20   (subpath \"/opt/homebrew/opt\")\n\
             \x20   (subpath \"/usr/local/Cellar\")\n\
             \x20   (subpath \"/usr/local/opt\"))\n\
             (deny network*)\n\
             (allow network* (remote ip \"localhost:*\"))\n\
             (allow sysctl-read)\n\
             (allow mach-lookup)\n\
             (allow ipc-posix-shm)\n"
        )
    }
}

/// Validate that `p` is absolute and contains only SBPL-safe characters.
/// Returns the first offending character on failure so the operator can
/// fix the specific byte.
fn validate_path_for_sbpl(p: &Path) -> Result<(), SandboxError> {
    if !p.is_absolute() {
        return Err(SandboxError::NonAbsolutePath(p.to_path_buf()));
    }
    let s = match p.to_str() {
        Some(s) => s,
        None => {
            // Non-UTF8 path. We treat this as unsafe — SBPL profile is
            // textual; serializing arbitrary bytes is a footgun.
            return Err(SandboxError::UnsafePath {
                path: p.to_path_buf(),
                ch: '\u{FFFD}',
            });
        }
    };
    for ch in s.chars() {
        if is_sbpl_unsafe(ch) {
            return Err(SandboxError::UnsafePath {
                path: p.to_path_buf(),
                ch,
            });
        }
    }
    Ok(())
}

fn is_sbpl_unsafe(ch: char) -> bool {
    // Reject SBPL string delimiters + escape + control chars + null.
    // is_control() covers C0 (U+0000..U+001F) and DEL (U+007F).
    ch == '"' || ch == '\\' || ch.is_control()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tempdir-based fixture. Returns a (TempDir, canonical_path) pair so
    /// the dir lives until the test scope ends. Most tests need a real
    /// existing path now that for_project canonicalizes (canonicalize
    /// requires the path to exist). We pre-canonicalize the expected path
    /// so equality checks against project_root() match the canonical form
    /// SandboxProfile stores.
    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let td = tempfile::tempdir().expect("tempdir");
        let canonical = td.path().canonicalize().expect("canonicalize tempdir");
        (td, canonical)
    }

    #[test]
    fn for_project_rejects_relative_path() {
        let r = SandboxProfile::for_project(PathBuf::from("relative/foo"));
        assert!(matches!(r, Err(SandboxError::NonAbsolutePath(_))));
    }

    #[test]
    fn for_project_rejects_path_with_double_quote() {
        let r = SandboxProfile::for_project(PathBuf::from("/tmp/foo\"bar"));
        match r {
            Err(SandboxError::UnsafePath { ch, .. }) => assert_eq!(ch, '"'),
            other => panic!("expected UnsafePath, got {other:?}"),
        }
    }

    #[test]
    fn for_project_rejects_path_with_backslash() {
        let r = SandboxProfile::for_project(PathBuf::from("/tmp/foo\\bar"));
        match r {
            Err(SandboxError::UnsafePath { ch, .. }) => assert_eq!(ch, '\\'),
            other => panic!("expected UnsafePath, got {other:?}"),
        }
    }

    #[test]
    fn for_project_rejects_path_with_newline() {
        let r = SandboxProfile::for_project(PathBuf::from("/tmp/foo\nbar"));
        assert!(matches!(r, Err(SandboxError::UnsafePath { .. })));
    }

    #[test]
    fn for_project_rejects_path_with_control_char() {
        let r = SandboxProfile::for_project(PathBuf::from("/tmp/foo\x07bar"));
        assert!(matches!(r, Err(SandboxError::UnsafePath { .. })));
    }

    #[test]
    fn for_project_accepts_typical_unix_path() {
        let (_td, root) = fixture();
        let p = SandboxProfile::for_project(root.clone()).expect("typical path should validate");
        assert_eq!(p.project_root(), root);
        assert!(!p.allowed_subprocess_paths().is_empty());
    }

    #[test]
    fn for_project_errors_when_path_does_not_exist() {
        // Regression for the canonicalize() semantics: a non-existent
        // path can no longer be sandbox-protected. Operator gets a
        // specific error rather than a runtime sandbox-exec failure.
        let r = SandboxProfile::for_project(PathBuf::from("/nonexistent/cuttle/scratch"));
        assert!(
            matches!(r, Err(SandboxError::ProjectRootCanonicalize { .. })),
            "expected ProjectRootCanonicalize, got {r:?}"
        );
    }

    #[test]
    fn rendered_sbpl_starts_with_version_and_deny_default() {
        let (_td, root) = fixture();
        let p = SandboxProfile::for_project(root).unwrap();
        let s = p.render_sbpl();
        assert!(s.starts_with("(version 1)"), "{s}");
        assert!(s.contains("(deny default)"));
        assert!(s.contains("(deny network*)"));
    }

    #[test]
    fn rendered_sbpl_includes_project_root_in_read_and_write_subpaths() {
        let (_td, root) = fixture();
        let p = SandboxProfile::for_project(root.clone()).unwrap();
        let s = p.render_sbpl();
        let expected = format!("(subpath \"{}\")", root.display());
        assert!(s.contains(&expected), "missing {expected:?} in {s}");
        // Project root must appear under both file-read* and file-write*.
        let read_section_idx = s.find("(allow file-read*").unwrap();
        let write_section_idx = s.find("(allow file-write*").unwrap();
        assert!(read_section_idx < write_section_idx);
    }

    #[test]
    fn rendered_sbpl_lists_allowed_binaries_as_literals() {
        let (_td, root) = fixture();
        let p = SandboxProfile::for_project(root).unwrap();
        let s = p.render_sbpl();
        assert!(s.contains("(literal \"/bin/bash\")"));
        assert!(s.contains("(literal \"/usr/bin/git\")"));
    }

    #[test]
    fn rendered_sbpl_allows_localhost_loopback_only() {
        let (_td, root) = fixture();
        let p = SandboxProfile::for_project(root).unwrap();
        let s = p.render_sbpl();
        // Modern SBPL accepts only "localhost" or "*" as the host literal;
        // IP literals like "127.0.0.1" are rejected at parse time.
        assert!(s.contains("(allow network* (remote ip \"localhost:*\"))"));
        // No allow for non-loopback addresses.
        assert!(!s.contains("(allow network* (remote ip \"0.0.0.0"));
        assert!(!s.contains("(allow network* (remote ip \"127.0.0.1"));
    }

    #[test]
    fn with_allowed_binaries_validates_each_path() {
        let (_td, root) = fixture();
        let p = SandboxProfile::for_project(root).unwrap();
        let r = p.with_allowed_binaries(vec![
            PathBuf::from("/bin/safe"),
            PathBuf::from("/bin/un\"safe"),
        ]);
        assert!(matches!(r, Err(SandboxError::UnsafePath { .. })));
    }

    #[test]
    fn with_allowed_binaries_replaces_set() {
        let (_td, root) = fixture();
        let p = SandboxProfile::for_project(root)
            .unwrap()
            .with_allowed_binaries(vec![PathBuf::from("/bin/only")])
            .unwrap();
        assert_eq!(p.allowed_subprocess_paths().len(), 1);
        let s = p.render_sbpl();
        assert!(s.contains("(literal \"/bin/only\")"));
        assert!(!s.contains("(literal \"/bin/bash\")"));
    }

    #[test]
    fn default_rlimits_are_recorded() {
        let (_td, root) = fixture();
        let p = SandboxProfile::for_project(root).unwrap();
        assert_eq!(p.cpu_limit_secs, 60);
        assert_eq!(p.mem_limit_mb, 1024);
        assert_eq!(p.max_open_fds, 256);
        assert_eq!(p.max_subprocesses, 16);
    }

    #[test]
    fn default_allowed_binaries_includes_core_set() {
        let bins = default_allowed_binaries();
        assert!(bins.iter().any(|p| p == Path::new("/bin/bash")));
        assert!(bins.iter().any(|p| p == Path::new("/usr/bin/git")));
        assert!(bins.iter().any(|p| p == Path::new("/usr/bin/python3")));
    }
}
