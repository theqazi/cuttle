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

RUSTC_VERSION="$($CARGO --version | awk '{print $2}')"
say "using $($CARGO --version)"

# Step 2: build.
say "building release binary (this takes ~30s on a clean build)..."
cd "$SCRIPT_DIR"
if ! "$CARGO" build --release --bin cuttle; then
    die "cargo build failed; see output above" 2
fi

if [ ! -x "$RELEASE_BIN" ]; then
    die "expected release binary not found at $RELEASE_BIN" 2
fi

# Step 3: install.
say "installing to $DEST"

if [ ! -d "$INSTALL_DIR" ]; then
    say "$INSTALL_DIR does not exist; creating it"
    if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
        die "cannot create $INSTALL_DIR (permissions); pick a different INSTALL_DIR or run with sudo" 3
    fi
fi

if ! cp "$RELEASE_BIN" "$DEST" 2>/dev/null; then
    die "cannot copy to $DEST (permissions); pick a writable INSTALL_DIR (e.g. ~/.local/bin) or run with sudo" 3
fi
chmod 0755 "$DEST" || die "chmod failed on $DEST" 3

# Step 4: report.
say "installed: $DEST"
"$DEST" --version

cat <<EOF

Next steps:
  1. Make sure $INSTALL_DIR is on your PATH (echo \$PATH).
  2. Export your Anthropic key:
       export ANTHROPIC_API_KEY="sk-ant-..."
  3. Try a one-shot:
       cuttle ask "say hi"
     Or start a session:
       cuttle session start

Session files land in ~/.cuttle/sessions/<id>/ (override with CUTTLE_HOME).
See README.md for the full surface.
EOF
