//! Memory promotion workflow per D-2026-04-26-31.
//!
//! Per `docs/TDD.md` §6.2. Promotion of a quarantine entry to canonical
//! requires a [`cuttle_gate::TtyInputCap`] capability-token witness; only
//! `cuttle-input::Session` holds the cap (per D-17 + cuttle-input crate).
//! The cap-required signature ensures the type system blocks promotion from
//! untrusted modules (skills loader, model client).

use crate::layout::MemoryLayout;
use crate::text::OperatorAuthoredText;
use cuttle_gate::TtyInputCap;
use std::path::PathBuf;

#[derive(Debug)]
pub enum PromotionDecision {
    Promote {
        canonical_path: PathBuf,
    },
    Reject {
        reason: String,
    },
    /// Operator chose to leave in quarantine for now.
    Defer,
}

#[derive(thiserror::Error, Debug)]
pub enum MemoryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("quarantine entry not found at {path:?}")]
    QuarantineMissing { path: PathBuf },
    #[error("canonical destination already exists at {path:?}")]
    CanonicalExists { path: PathBuf },
}

/// Promote a quarantine entry to canonical.
///
/// `_cap: &TtyInputCap` is the load-bearing parameter: the type system
/// requires the caller to hold the TTY input capability; the model client
/// or skills loader cannot synthesize one. The cap reference is unused at
/// runtime (its presence at compile time is the proof).
///
/// `operator_decision_text` is `OperatorAuthoredText` (per D-17): the
/// operator's promotion decision is itself operator-authored, not model-emitted.
pub fn prompt_promotion(
    layout: &MemoryLayout,
    quarantine_entry: &PathBuf,
    canonical_topic: &str,
    operator_decision_text: OperatorAuthoredText,
    _cap: &TtyInputCap,
) -> Result<PromotionDecision, MemoryError> {
    if !quarantine_entry.exists() {
        return Err(MemoryError::QuarantineMissing {
            path: quarantine_entry.clone(),
        });
    }
    let canonical_path = layout
        .canonical_dir()
        .join(format!("{}.md", canonical_topic));
    if canonical_path.exists() {
        return Err(MemoryError::CanonicalExists {
            path: canonical_path,
        });
    }
    // Read quarantine entry, prepend operator's decision text as a header,
    // write to canonical, remove from quarantine.
    let quarantine_content = std::fs::read_to_string(quarantine_entry)?;
    let promoted_content = format!(
        "<!-- Promoted by operator: {} -->\n{}",
        operator_decision_text.as_str(),
        quarantine_content
    );
    std::fs::write(&canonical_path, promoted_content)?;
    std::fs::remove_file(quarantine_entry)?;
    Ok(PromotionDecision::Promote { canonical_path })
}

/// Reject a quarantine entry: move to `rejected/`. Kept N=30 days then
/// auto-purged (cleanup logic lives in cuttle-cli session-startup).
pub fn reject_quarantine_entry(
    layout: &MemoryLayout,
    quarantine_entry: &PathBuf,
    operator_reason: OperatorAuthoredText,
    _cap: &TtyInputCap,
) -> Result<PromotionDecision, MemoryError> {
    if !quarantine_entry.exists() {
        return Err(MemoryError::QuarantineMissing {
            path: quarantine_entry.clone(),
        });
    }
    let file_name = quarantine_entry
        .file_name()
        .ok_or_else(|| MemoryError::QuarantineMissing {
            path: quarantine_entry.clone(),
        })?;
    let rejected_path = layout.rejected_dir().join(file_name);
    let content = std::fs::read_to_string(quarantine_entry)?;
    let with_reason = format!(
        "<!-- Rejected by operator: {} -->\n{}",
        operator_reason.as_str(),
        content
    );
    std::fs::write(&rejected_path, with_reason)?;
    std::fs::remove_file(quarantine_entry)?;
    Ok(PromotionDecision::Reject {
        reason: "operator-rejected".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttle_gate::capabilities::__internal_input_cap_factory;

    #[test]
    fn promote_quarantine_to_canonical() {
        let tmp = tempfile::tempdir().unwrap();
        let layout = MemoryLayout::ensure(tmp.path().join("memory")).unwrap();
        let q_path = layout.quarantine_path_for("session1", 1);
        std::fs::write(&q_path, "model-authored note").unwrap();
        let cap = __internal_input_cap_factory::issue();
        let op_text = OperatorAuthoredText::from_tty("looks good".to_string());
        let decision = prompt_promotion(&layout, &q_path, "topic1", op_text, &cap).unwrap();
        match decision {
            PromotionDecision::Promote { canonical_path } => {
                assert!(canonical_path.exists());
                let content = std::fs::read_to_string(&canonical_path).unwrap();
                assert!(content.contains("Promoted by operator: looks good"));
                assert!(content.contains("model-authored note"));
                assert!(!q_path.exists(), "quarantine entry should be removed");
            }
            other => panic!("expected Promote, got {:?}", other),
        }
    }

    #[test]
    fn reject_moves_to_rejected_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let layout = MemoryLayout::ensure(tmp.path().join("memory")).unwrap();
        let q_path = layout.quarantine_path_for("session1", 2);
        std::fs::write(&q_path, "low quality").unwrap();
        let cap = __internal_input_cap_factory::issue();
        let reason = OperatorAuthoredText::from_tty("not relevant".to_string());
        let decision = reject_quarantine_entry(&layout, &q_path, reason, &cap).unwrap();
        match decision {
            PromotionDecision::Reject { .. } => {
                let rejected_files: Vec<_> = std::fs::read_dir(layout.rejected_dir())
                    .unwrap()
                    .filter_map(|e| e.ok())
                    .collect();
                assert_eq!(rejected_files.len(), 1);
                assert!(!q_path.exists());
            }
            other => panic!("expected Reject, got {:?}", other),
        }
    }

    #[test]
    fn promote_missing_quarantine_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let layout = MemoryLayout::ensure(tmp.path().join("memory")).unwrap();
        let q_path = layout.quarantine_path_for("session1", 1);
        let cap = __internal_input_cap_factory::issue();
        let op_text = OperatorAuthoredText::from_tty("ok".to_string());
        let r = prompt_promotion(&layout, &q_path, "topic1", op_text, &cap);
        assert!(matches!(r, Err(MemoryError::QuarantineMissing { .. })));
    }

    #[test]
    fn promote_existing_canonical_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let layout = MemoryLayout::ensure(tmp.path().join("memory")).unwrap();
        let q_path = layout.quarantine_path_for("session1", 1);
        std::fs::write(&q_path, "note").unwrap();
        std::fs::write(layout.canonical_dir().join("topic1.md"), "existing").unwrap();
        let cap = __internal_input_cap_factory::issue();
        let op_text = OperatorAuthoredText::from_tty("ok".to_string());
        let r = prompt_promotion(&layout, &q_path, "topic1", op_text, &cap);
        assert!(matches!(r, Err(MemoryError::CanonicalExists { .. })));
    }
}
