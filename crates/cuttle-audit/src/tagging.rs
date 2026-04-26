//! Tool-registration tagging contract per D-2026-04-26-28 (WV-03 closure)
//! and D-2026-04-26-30 (OQ-12 PII posture).
//!
//! Per `docs/TDD.md` §5.2 + §5.4. Each tool registers with `(secret_bearing,
//! pii_bearing)` flags. Unknown tools default to safe-by-default:
//! `secret_bearing = true, pii_bearing = RedactAtWrite`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PiiPosture {
    /// Tool refused at registration; cannot be invoked. v0.2+.
    Refused,
    /// PII redacted at audit-log write time via `Redactor`. v0.1 default.
    RedactAtWrite,
    /// Tool output recorded as-is in the audit log. Requires explicit
    /// per-tool operator opt-in.
    RecordAsIs,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolTag {
    pub tool_name: String,
    pub secret_bearing: bool,
    pub pii_bearing: PiiPosture,
    pub registered_at: DateTime<Utc>,
}

/// Tool registry; populated at session start. Unknown tools default to
/// safe-by-default per WV-03: secret_bearing = true, pii_bearing = RedactAtWrite.
pub struct ToolRegistry {
    tools: HashMap<String, ToolTag>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool with explicit tags. Used at session start by the
    /// runtime to register built-in tools.
    pub fn register(&mut self, tag: ToolTag) {
        self.tools.insert(tag.tool_name.clone(), tag);
    }

    /// Look up the tag for `tool_name`. Returns the safe-by-default tag if
    /// the tool is not registered.
    pub fn lookup_or_safe_default(&self, tool_name: &str) -> ToolTag {
        self.tools.get(tool_name).cloned().unwrap_or(ToolTag {
            tool_name: tool_name.to_string(),
            secret_bearing: true,
            pii_bearing: PiiPosture::RedactAtWrite,
            registered_at: Utc::now(),
        })
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_tool_defaults_safe() {
        let r = ToolRegistry::new();
        let tag = r.lookup_or_safe_default("unknown-tool");
        assert!(tag.secret_bearing);
        assert!(matches!(tag.pii_bearing, PiiPosture::RedactAtWrite));
    }

    #[test]
    fn registered_tool_returns_tag() {
        let mut r = ToolRegistry::new();
        r.register(ToolTag {
            tool_name: "Bash".into(),
            secret_bearing: false,
            pii_bearing: PiiPosture::RedactAtWrite,
            registered_at: Utc::now(),
        });
        let tag = r.lookup_or_safe_default("Bash");
        assert!(!tag.secret_bearing);
    }
}
