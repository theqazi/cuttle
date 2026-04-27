//! Banner for `cuttle session start`. Shown once at REPL launch.
//!
//! Design history (kept short for future-Mo who wonders why this is text):
//! - v0.0.15: literal cuttlefish silhouette. Looked like a deformed face.
//! - v0.0.16-a: Unicode quadrant wordmark. Linter shifted top-row spacing
//!   and the letters fell out of column alignment.
//! - v0.0.16-b: full-block wordmark. Aligned, but the 1-px-thick strokes
//!   read as spindly + the 1-space letter gaps made the letters merge.
//! - v0.0.16-c (this version): killed the wordmark. Rounded box draw
//!   with title in the top border, the way `gh`, `kubectl`, etc. do
//!   their startup splash. Clean, reliable, no glyph-alignment risk.
//!
//! No emojis (per CLAUDE.md). No em-dashes (per CLAUDE.md §7d.1).

/// Render the banner with the live binary version.
pub fn render() -> String {
    let version = env!("CARGO_PKG_VERSION");
    // Box width chosen so the longest content line ("deterministic ...
    // zeroize on drop") fits with 2 spaces of inner padding either side.
    // 60 chars wide is comfortable in any terminal >= 80 cols.
    let title = format!(" cuttle {version} ");
    // Top border: "╭─ cuttle 0.0.16 ──────...─╮"
    let inner_width: usize = 58;
    let dashes_after_title = inner_width.saturating_sub(title.chars().count());
    let top = format!("╭─{title}{}─╮", "─".repeat(dashes_after_title));
    let bottom = format!("╰{}╯", "─".repeat(inner_width + 2));

    let lines = [
        "  security-first BYOK Claude harness",
        "  deterministic gate, HMAC-chained audit, zeroize-on-drop",
    ];
    let body: String = lines
        .iter()
        .map(|l| format!("│ {l:<inner_width$} │\n", inner_width = inner_width))
        .collect();

    format!("\n{top}\n{body}{bottom}\n   type /quit or Ctrl+D to exit.\n")
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

    #[test]
    fn box_top_and_bottom_are_same_width() {
        // If the title-padding math drifts, the rounded corners no
        // longer line up vertically. Asserting equal widths catches
        // that the moment it happens.
        let s = render();
        let lines: Vec<&str> = s.lines().collect();
        // First non-empty line is the top border; find the matching
        // bottom border (the line starting with ╰).
        let top = lines
            .iter()
            .find(|l| l.starts_with("╭"))
            .expect("missing top border");
        let bottom = lines
            .iter()
            .find(|l| l.starts_with("╰"))
            .expect("missing bottom border");
        assert_eq!(
            top.chars().count(),
            bottom.chars().count(),
            "top {top:?} != bottom {bottom:?}"
        );
    }
}
