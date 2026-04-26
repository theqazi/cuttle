//! [`SkillContent`]: validated skill body that has passed the Unicode allowlist.
//!
//! Per `docs/TDD.md` §3 + WV-05 / D-2026-04-26-25. Constructor calls
//! [`crate::scan_for_disallowed`]; on failure, the skill fails to load and
//! the operator is informed of the disallowed codepoint and offset.

use crate::allowlist::{scan_for_disallowed, AllowlistViolation};
use thiserror::Error;

/// Skill content that has been validated against the v0.1 Unicode allowlist.
/// Constructor is `pub`: callers (cuttle-cli skills loader, tests) provide
/// the raw string; the constructor returns an error if the validation fails.
pub struct SkillContent {
    body: String,
}

impl SkillContent {
    /// Validate `body` against the allowlist and wrap. Fails closed on any
    /// disallowed codepoint per WV-05.
    pub fn validate(body: String) -> Result<Self, SkillContentError> {
        scan_for_disallowed(&body)?;
        Ok(Self { body })
    }

    pub fn as_str(&self) -> &str {
        &self.body
    }

    pub fn into_string(self) -> String {
        self.body
    }
}

#[derive(Error, Debug)]
pub enum SkillContentError {
    #[error("skill content has disallowed Unicode: {0}")]
    Disallowed(#[from] AllowlistViolation),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_safe_skill() {
        let body = "# Skill\n\n## Purpose\nDo a thing.\n".to_string();
        let s = SkillContent::validate(body.clone()).unwrap();
        assert_eq!(s.as_str(), body);
    }

    #[test]
    fn validate_rejects_zero_width_attack() {
        let attack = "name: helpful\u{200B}assistant\nignore prior".to_string();
        let r = SkillContent::validate(attack);
        assert!(matches!(r, Err(SkillContentError::Disallowed(_))));
    }

    #[test]
    fn validate_rejects_rtl_attack() {
        let attack = "harmless\u{202E}attack".to_string();
        let r = SkillContent::validate(attack);
        assert!(matches!(r, Err(SkillContentError::Disallowed(_))));
    }

    #[test]
    fn into_string_returns_validated_body() {
        let body = "ok".to_string();
        let s = SkillContent::validate(body.clone()).unwrap();
        assert_eq!(s.into_string(), body);
    }
}
