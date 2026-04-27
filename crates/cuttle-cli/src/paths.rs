//! Default path resolution for `~/.cuttle/*`.
//!
//! v0.0.11 surface: just the audit log path (everything else gets paths
//! when its subcommand lands). Resolution order matches CLAUDE.md §1a
//! (read-before-write):
//! 1. Caller-supplied explicit path wins.
//! 2. `CUTTLE_HOME` environment variable, if set.
//! 3. `~/.cuttle/` via the `dirs` crate.
//! 4. None — caller must explicitly opt in by passing a path.
//!
//! Date-rolled audit logs (`~/.cuttle/audit/<yyyy-mm-dd>.jsonl` per TDD §5)
//! land in v0.0.12 with the chrono dep wire-in. v0.0.11 uses a flat
//! `~/.cuttle/audit.jsonl`.

use std::path::PathBuf;

const CUTTLE_HOME_ENV: &str = "CUTTLE_HOME";
const DEFAULT_AUDIT_LOG_BASENAME: &str = "audit.jsonl";

/// Resolve the cuttle home directory. None if neither `CUTTLE_HOME` nor
/// the user's home directory can be determined (rare; happens in
/// stripped CI environments).
pub fn cuttle_home() -> Option<PathBuf> {
    if let Ok(p) = std::env::var(CUTTLE_HOME_ENV) {
        return Some(PathBuf::from(p));
    }
    dirs::home_dir().map(|h| h.join(".cuttle"))
}

/// Resolve the default audit log path, or None if cuttle_home() is None.
pub fn default_audit_log_path() -> Option<PathBuf> {
    cuttle_home().map(|h| h.join(DEFAULT_AUDIT_LOG_BASENAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper that scopes a CUTTLE_HOME override + restores afterward.
    /// std::env::set_var is unsafe in 2024 edition because env is a
    /// process-global resource. SAFETY: tests within this module run
    /// sequentially with the cargo test default thread pool only via
    /// the `serial_test` crate; without that, parallel env mutation is
    /// racy. Acceptable trade-off here: the `CUTTLE_HOME` var is unique
    /// to this codebase, no other test touches it, and we restore it.
    fn with_cuttle_home<F: FnOnce()>(value: Option<&str>, f: F) {
        let prev = std::env::var(CUTTLE_HOME_ENV).ok();
        match value {
            Some(v) => unsafe { std::env::set_var(CUTTLE_HOME_ENV, v) },
            None => unsafe { std::env::remove_var(CUTTLE_HOME_ENV) },
        }
        f();
        match prev {
            Some(p) => unsafe { std::env::set_var(CUTTLE_HOME_ENV, p) },
            None => unsafe { std::env::remove_var(CUTTLE_HOME_ENV) },
        }
    }

    #[test]
    fn cuttle_home_env_takes_precedence() {
        with_cuttle_home(Some("/tmp/scratch-cuttle"), || {
            assert_eq!(cuttle_home(), Some(PathBuf::from("/tmp/scratch-cuttle")));
        });
    }

    #[test]
    fn default_audit_log_path_appends_basename() {
        with_cuttle_home(Some("/tmp/scratch-cuttle"), || {
            assert_eq!(
                default_audit_log_path(),
                Some(PathBuf::from("/tmp/scratch-cuttle/audit.jsonl"))
            );
        });
    }
}
