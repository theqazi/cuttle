#!/usr/bin/env bash
#
# Cuttle install script. Builds the release binary and copies it to
# INSTALL_DIR (default /usr/local/bin). No sudo unless INSTALL_DIR is
# root-owned. Read this script before you run it.
#
# Usage:
#   ./install.sh                          # install to /usr/local/bin
#   INSTALL_DIR=~/.local/bin ./install.sh # install to a user-writable dir
#
# Environment overrides:
#   INSTALL_DIR  destination directory (default: /usr/local/bin)
#   CARGO        cargo binary (default: cargo)
#
# Exit codes:
#   0  success
#   1  prerequisite missing (cargo, rustc)
#   2  build failed
#   3  install (copy) failed

set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
CARGO="${CARGO:-cargo}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_BIN="$SCRIPT_DIR/target/release/cuttle"
DEST="$INSTALL_DIR/cuttle"

say() { printf 'cuttle install: %s\n' "$*"; }
die() { printf 'cuttle install: ERROR: %s\n' "$*" >&2; exit "${2:-1}"; }

# Step 1: prerequisites.
command -v "$CARGO" >/dev/null 2>&1 || die "cargo not found on PATH; install Rust 1.95+ via https://rustup.rs/" 1

say "using $($CARGO --version)"

# Step 2: pre-flight writability check. Fail fast with an actionable
# message BEFORE spending 30s on a release build only to discover the
# operator can't write to INSTALL_DIR. Two failure modes:
# - INSTALL_DIR exists but isn't writable
# - INSTALL_DIR doesn't exist and the parent isn't writable (mkdir fails)
check_installable() {
    if [ -d "$INSTALL_DIR" ]; then
        # Existing dir: writable check via -w. -w returns true under
        # sudo even for non-owner-writable dirs, so this also handles
        # `sudo ./install.sh` cleanly.
        [ -w "$INSTALL_DIR" ]
    else
        # Doesn't exist: try a probe mkdir (and clean up if it succeeds).
        if mkdir -p "$INSTALL_DIR" 2>/dev/null; then
            # Created. Leave it; the install step would have created it
            # anyway. Now confirm it's writable.
            [ -w "$INSTALL_DIR" ]
        else
            return 1
        fi
    fi
}

if ! check_installable; then
    cat >&2 <<EOF
cuttle install: ERROR: cannot write to $INSTALL_DIR.

Pick one:
  sudo $0                            # install to $INSTALL_DIR (requires sudo)
  INSTALL_DIR=\$HOME/.local/bin $0   # user-local (no sudo)
  INSTALL_DIR=\$HOME/bin $0          # user-local (no sudo)

If you pick a user-local path that is not yet on your PATH, this script
will print the exact line to add to your shell rc on success.

No build was started. Re-run with one of the options above.
EOF
    exit 3
fi

# Step 3: build.
say "building release binary (this takes ~30s on a clean build)..."
cd "$SCRIPT_DIR"
if ! "$CARGO" build --release --bin cuttle; then
    die "cargo build failed; see output above" 2
fi

if [ ! -x "$RELEASE_BIN" ]; then
    die "expected release binary not found at $RELEASE_BIN" 2
fi

# Step 4: install.
say "installing to $DEST"

if ! cp "$RELEASE_BIN" "$DEST" 2>/dev/null; then
    die "cannot copy to $DEST (permissions changed since pre-flight); re-check $INSTALL_DIR" 3
fi
chmod 0755 "$DEST" || die "chmod failed on $DEST" 3

# Step 5: report.
say "installed: $DEST"
"$DEST" --version

# Step 6: PATH check. Tell the operator the EXACT line to add to the
# EXACT rc file if INSTALL_DIR isn't on PATH yet.
on_path=false
case ":$PATH:" in
    *":$INSTALL_DIR:"*) on_path=true ;;
esac

# Detect the operator's interactive shell + its rc file. Pick the file
# the shell actually sources at interactive startup. macOS Catalina+
# defaults to zsh; bash on macOS uses ~/.bash_profile (NOT ~/.bashrc)
# for login shells, which is the daily case in Terminal.app.
shell_name="$(basename "${SHELL:-/bin/sh}")"
case "$shell_name" in
    zsh)  rc_file="$HOME/.zshrc"; rc_export="export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
    bash) rc_file="$HOME/.bash_profile"; rc_export="export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
    fish) rc_file="$HOME/.config/fish/config.fish"; rc_export="set -gx PATH $INSTALL_DIR \$PATH" ;;
    *)    rc_file="$HOME/.profile"; rc_export="export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac

cat <<EOF

Next steps:
EOF

if $on_path; then
    cat <<EOF
  1. $INSTALL_DIR is already on your PATH. cuttle is ready to run.
EOF
else
    cat <<EOF
  1. $INSTALL_DIR is NOT on your PATH yet. Add this line to $rc_file:

       $rc_export

     One-liner to do it now (idempotent, safe to re-run):

       grep -qF '$rc_export' $rc_file 2>/dev/null \\
         || echo '$rc_export' >> $rc_file

     Then reload your shell:

       source $rc_file
EOF
fi

cat <<EOF
  2. Export your Anthropic key:
       export ANTHROPIC_API_KEY="sk-ant-..."
  3. Try a one-shot:
       cuttle ask "say hi"
     Or start a session:
       cuttle session start

Session files land in ~/.cuttle/sessions/<id>/ (override with CUTTLE_HOME).
See README.md for the full surface. To remove cuttle, run ./uninstall.sh.
EOF
