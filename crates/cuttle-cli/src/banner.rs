//! ASCII banner for `cuttle session start`. Shown once at REPL launch
//! so the operator knows which tool is on the other side of stdin.
//!
//! Cuttle's codename theme is aquatic animals; the banner is a stylized
//! cuttlefish silhouette with the version + the load-bearing security
//! posture sentence under it. Kept short so it doesn't dominate the
//! terminal at session start.

/// Render the banner with the live binary version. Returns a `String`
/// so the caller decides where it goes (stdout for sessions, captured
/// in tests).
pub fn render() -> String {
    let version = env!("CARGO_PKG_VERSION");
    // 7 lines of art + 2 of metadata. The cuttlefish is intentionally
    // small enough to fit in an 80-column terminal without wrapping.
    format!(
        r#"
        .--.       .--.
       (    `-._.-'    )      cuttle {version}
        \    (o)(o)   /       security-first BYOK Claude harness
         \  ( <  > )  /       deterministic gate, HMAC-chained audit,
          \  '----'  /        zeroize-on-drop credentials
           '-.____.-'
            )))    (((        type /quit or Ctrl+D to exit.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_includes_live_version() {
        let s = render();
        assert!(s.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn render_mentions_quit_command() {
        // If we ever change the exit instruction, this test catches the
        // banner getting out of sync with session_cmd.rs.
        let s = render();
        assert!(s.contains("/quit"));
    }

    #[test]
    fn render_fits_within_80_columns() {
        for (i, line) in render().lines().enumerate() {
            assert!(
                line.chars().count() <= 80,
                "line {i} is {} chars, exceeds 80: {line:?}",
                line.chars().count()
            );
        }
    }
}
