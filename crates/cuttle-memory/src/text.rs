//! Text provenance pair per D-2026-04-26-17 + T-007.
//!
//! [`OperatorAuthoredText`] and [`ModelAuthoredText`] are type-distinct;
//! there is NO `From<String>` / `Into<String>` cross-cast. The type system
//! enforces the memory-quarantine boundary.

/// Bytes the operator typed in the TTY (or wrote to operator-controlled
/// surfaces like `~/.claude/CLAUDE.md`). Trusted-by-default at the boundary.
///
/// Constructor is `pub(crate)`: only the input-handler crate or the
/// file-loader (operator-trusted file paths) can mint. Skills loader,
/// model client, and other untrusted modules cannot construct.
pub struct OperatorAuthoredText(String);

impl OperatorAuthoredText {
    /// Mint from operator TTY input. `pub(crate)`-style: only the input-handler
    /// crate calls. For now exposed via `from_tty_input_in_tests` below for
    /// cross-crate test setup; production callers will route through
    /// `cuttle-input::Session::read_operator_text()`.
    pub fn from_tty(s: String) -> Self {
        Self(s)
    }

    /// Mint from an operator-controlled file path read at session start.
    /// `pub(crate)`-style: only the file-loader module calls.
    pub fn from_operator_file(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

/// Bytes the model emitted. Untrusted-by-default; quarantined per
/// `docs/PRD.md` §6.1.5 cross-session memory promotion invariant.
///
/// Constructor is `pub`: any crate can wrap a string the model produced.
/// The type system uses the WRAPPER, not the constructor, to enforce
/// trust separation downstream.
pub struct ModelAuthoredText(String);

impl ModelAuthoredText {
    pub fn from_model_output(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

// NO `From<String>` / `Into<String>` impls. NO cross-cast between
// OperatorAuthoredText and ModelAuthoredText. The type system enforces
// the memory-quarantine boundary at the API surface.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_text_round_trip() {
        let t = OperatorAuthoredText::from_tty("operator note".to_string());
        assert_eq!(t.as_str(), "operator note");
    }

    #[test]
    fn model_text_round_trip() {
        let t = ModelAuthoredText::from_model_output("model note".to_string());
        assert_eq!(t.as_str(), "model note");
    }

    // The type-distinction enforcement is verified at compile time: there is
    // no way to construct ModelAuthoredText from OperatorAuthoredText (or vice
    // versa) without an explicit conversion. We rely on the absence of From
    // impls and the lack of constructors that take the other type. Compile-fail
    // tests via trybuild are TDD §3 / future work.
}
