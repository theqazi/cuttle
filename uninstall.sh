#!/usr/bin/env bash
#
# Cuttle uninstall script. By default removes ONLY the binary; the
# session data under ~/.cuttle/ is preserved because it contains the
# operator's audit logs and transcripts (deleting them silently would
# be the wrong default for a security tool).
#
# Usage:
#   ./uninstall.sh                                # remove binary only
#   INSTALL_DIR=~/.local/bin ./uninstall.sh       # match where you installed
#   ./uninstall.sh --remove-data                  # also wipe ~/.cuttle/
#                                                 #   (prompts to confirm)
#   ./uninstall.sh --remove-data --yes            # skip confirmation
#                                                 #   (script-friendly)
#
# Environment overrides:
#   INSTALL_DIR    where the binary lives (default: /usr/local/bin)
#   CUTTLE_HOME    where session data lives (default: ~/.cuttle)
#
# Exit codes:
#   0  success (or already-not-installed)
#   1  bad arguments
#   3  remove failed (permissions; try sudo or pick the right INSTALL_DIR)
#   4  user declined data removal at the prompt

set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
CUTTLE_HOME_DIR="${CUTTLE_HOME:-$HOME/.cuttle}"
DEST="$INSTALL_DIR/cuttle"

REMOVE_DATA=false
ASSUME_YES=false
for arg in "$@"; do
    case "$arg" in
        --remove-data) REMOVE_DATA=true ;;
        --yes|-y)      ASSUME_YES=true ;;
        --help|-h)
            sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            printf 'cuttle uninstall: ERROR: unknown argument: %s\n' "$arg" >&2
            printf 'Run %s --help for usage.\n' "$0" >&2
            exit 1
            ;;
    esac
done

say() { printf 'cuttle uninstall: %s\n' "$*"; }
die() { printf 'cuttle uninstall: ERROR: %s\n' "$*" >&2; exit "${2:-1}"; }

# Step 1: remove the binary, if present.
if [ -e "$DEST" ]; then
    if [ ! -w "$INSTALL_DIR" ]; then
        die "cannot write to $INSTALL_DIR; re-run with sudo or set INSTALL_DIR to where you installed (current default: $INSTALL_DIR)" 3
    fi
    if rm -f "$DEST"; then
        say "removed $DEST"
    else
        die "rm $DEST failed" 3
    fi
else
    say "no binary at $DEST (already uninstalled, or wrong INSTALL_DIR)"
fi

# Step 2: optionally remove session data. Default is to PRESERVE because
# audit logs + transcripts are operator-valuable and re-installing
# cuttle later should not require re-creating them.
if $REMOVE_DATA; then
    if [ -d "$CUTTLE_HOME_DIR" ]; then
        # Count what's about to be destroyed so the operator sees the
        # blast radius before confirming.
        session_count="$(find "$CUTTLE_HOME_DIR/sessions" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')"
        say "about to remove $CUTTLE_HOME_DIR ($session_count session(s))"

        confirmed=false
        if $ASSUME_YES; then
            confirmed=true
        else
            printf 'cuttle uninstall: type "delete" to confirm: '
            read -r reply
            [ "$reply" = "delete" ] && confirmed=true
        fi

        if $confirmed; then
            rm -rf "$CUTTLE_HOME_DIR" && say "removed $CUTTLE_HOME_DIR"
        else
            say "skipped data removal (operator declined)"
            exit 4
        fi
    else
        say "no data at $CUTTLE_HOME_DIR (nothing to remove)"
    fi
else
    if [ -d "$CUTTLE_HOME_DIR" ]; then
        cat <<EOF

Session data at $CUTTLE_HOME_DIR was preserved.
To remove it too, re-run with --remove-data:
  ./uninstall.sh --remove-data
EOF
    fi
fi

# Step 3: PATH cleanup hint. We do NOT modify the operator's rc files
# automatically (too easy to break a shell config). Just remind them.
shell_name="$(basename "${SHELL:-/bin/sh}")"
case "$shell_name" in
    zsh)  rc_file="$HOME/.zshrc" ;;
    bash) rc_file="$HOME/.bash_profile" ;;
    fish) rc_file="$HOME/.config/fish/config.fish" ;;
    *)    rc_file="$HOME/.profile" ;;
esac

if grep -qF "$INSTALL_DIR" "$rc_file" 2>/dev/null; then
    cat <<EOF

If you added $INSTALL_DIR to PATH for cuttle, you may want to remove
that line from $rc_file (this script does not touch your shell config).
EOF
fi

say "done."
