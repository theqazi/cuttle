//! Memory filesystem layout per D-2026-04-26-31.
//!
//! Per `docs/TDD.md` §6.1 + §6.3:
//! ```text
//! ~/.cuttle/memory/
//! ├── canonical/                 # operator-promoted; loaded as trusted
//! │   ├── MEMORY.md
//! │   └── <topic>.md
//! └── quarantine/
//!     ├── pending/                # awaiting operator review
//!     └── rejected/               # kept N=30 days for re-review
//! ```
//! Per-project at `~/.cuttle/memory/projects/<project-key>/` mirrors
//! the same structure.

use std::path::{Path, PathBuf};

pub struct MemoryLayout {
    root: PathBuf,
}

impl MemoryLayout {
    /// Initialize a memory layout rooted at `root` (typically `~/.cuttle/memory/`
    /// or `~/.cuttle/memory/projects/<project-key>/`). Creates the canonical/
    /// and quarantine/{pending,rejected}/ directories if they do not exist.
    pub fn ensure(root: PathBuf) -> Result<Self, MemoryLayoutError> {
        std::fs::create_dir_all(root.join("canonical"))?;
        std::fs::create_dir_all(root.join("quarantine").join("pending"))?;
        std::fs::create_dir_all(root.join("quarantine").join("rejected"))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn canonical_dir(&self) -> PathBuf {
        self.root.join("canonical")
    }

    pub fn pending_dir(&self) -> PathBuf {
        self.root.join("quarantine").join("pending")
    }

    pub fn rejected_dir(&self) -> PathBuf {
        self.root.join("quarantine").join("rejected")
    }

    /// Return the canonical MEMORY.md index path (operator-authored index).
    pub fn memory_md(&self) -> PathBuf {
        self.canonical_dir().join("MEMORY.md")
    }

    /// Compute the quarantine path for a model-authored memory write.
    /// Naming: `<session-id>-<seq>.md` per D-31.
    pub fn quarantine_path_for(&self, session_id: &str, seq: u64) -> PathBuf {
        self.pending_dir()
            .join(format!("{}-{}.md", session_id, seq))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MemoryLayoutError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_creates_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let layout = MemoryLayout::ensure(tmp.path().join("memory")).unwrap();
        assert!(layout.canonical_dir().is_dir());
        assert!(layout.pending_dir().is_dir());
        assert!(layout.rejected_dir().is_dir());
    }

    #[test]
    fn quarantine_path_includes_session_and_seq() {
        let tmp = tempfile::tempdir().unwrap();
        let layout = MemoryLayout::ensure(tmp.path().join("memory")).unwrap();
        let p = layout.quarantine_path_for("abc123", 42);
        let s = p.to_string_lossy().into_owned();
        assert!(s.contains("abc123-42.md"));
        assert!(s.contains("/quarantine/pending/"));
    }
}
