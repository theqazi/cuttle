//! [`TierClassification`]: ceremony tier per CLAUDE.md SCOPE TIERS table.
//!
//! Per `docs/PRD.md` §6.1.2 row L2 + `docs/TDD.md` §2.4. Model proposes a tier
//! as a string; deserialization rejects anything outside the four variants.

use serde::{Deserialize, Serialize};

/// Tier classification proposed by the model and verified by the harness.
///
/// `serde` does NOT use `default` or `other` per D-2026-04-26-18: missing or
/// unknown variant fails deserialization (mismatch becomes a parse error,
/// not a runtime string compare).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TierClassification {
    Patch,
    Feature,
    Refactor,
    System,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_each_variant() {
        for tier in [
            TierClassification::Patch,
            TierClassification::Feature,
            TierClassification::Refactor,
            TierClassification::System,
        ] {
            let json = serde_json::to_string(&tier).unwrap();
            let restored: TierClassification = serde_json::from_str(&json).unwrap();
            assert_eq!(tier, restored);
        }
    }

    #[test]
    fn rejects_unknown_variant() {
        let bad = r#""Hotfix""#;
        let r: Result<TierClassification, _> = serde_json::from_str(bad);
        assert!(r.is_err());
    }
}
