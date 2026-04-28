#!/usr/bin/env bash
# Cuttle bench: Phase A in a clean Linux container.
#
# Wrapper around `docker run cuttle-bench ...` that handles:
#   - Building the image if it isn't built (or --rebuild was passed).
#   - Sourcing ANTHROPIC_API_KEY from the macOS Keychain entry that
#     cuttle itself uses (service `dev.cuttle.api-keys`, account
#     `ANTHROPIC_API_KEY`), so the operator doesn't need to export
#     the key into the calling shell.
#   - Mounting the live `bench/` tree so iterating on tasks doesn't
#     require a rebuild.
#   - Routing all positional args through to bench.py.
#
# Usage:
#     ./bench/run-in-docker.sh --runs 5 --model claude-haiku-4-5-20251001
#     ./bench/run-in-docker.sh --task csv_export --runs 10
#     ./bench/run-in-docker.sh --rebuild --runs 3   # force image rebuild
#
# Limitations:
#   - This is Phase A only. cuttle-sandbox is macOS-Seatbelt-specific
#     so Docker on Mac (Linux VM) cannot exercise cuttle's containment.
#     Phase C runs must stay native: `python3 runners/cuttle_sandbox_runner.py`.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="cuttle-bench:latest"
RESULTS_DIR="$REPO_ROOT/bench/results"
REBUILD=0

# Strip our own flags out before forwarding to bench.py.
ARGS=()
for a in "$@"; do
    if [[ "$a" == "--rebuild" ]]; then
        REBUILD=1
    else
        ARGS+=("$a")
    fi
done

# Resolve the API key from the cuttle Keychain entry. This avoids
# requiring `export ANTHROPIC_API_KEY=...` in the calling shell. The
# secret is read into a single `docker run -e` invocation; it doesn't
# land in shell history or env beyond that subprocess.
if [[ -z "${ANTHROPIC_API_KEY:-}" ]]; then
    if KEY="$(security find-generic-password \
                -s dev.cuttle.api-keys \
                -a ANTHROPIC_API_KEY \
                -w 2>/dev/null)"; then
        export ANTHROPIC_API_KEY="$KEY"
        unset KEY
    else
        echo "run-in-docker: ANTHROPIC_API_KEY not in env and not in" >&2
        echo "  Keychain (service=dev.cuttle.api-keys, account=" >&2
        echo "  ANTHROPIC_API_KEY). Set it via:" >&2
        echo "    cuttle credential set" >&2
        echo "  or:" >&2
        echo "    export ANTHROPIC_API_KEY=sk-..." >&2
        exit 2
    fi
fi

# Build the image if missing or --rebuild was passed.
if [[ "$REBUILD" -eq 1 ]] || ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    echo "run-in-docker: building $IMAGE..."
    docker build -t "$IMAGE" "$REPO_ROOT/bench"
fi

mkdir -p "$RESULTS_DIR"

# Default --out lands in the mounted results dir so it survives the
# container removal. If the user passed their own --out, we don't
# override it.
HAS_OUT=0
for a in "${ARGS[@]}"; do
    if [[ "$a" == "--out" ]] || [[ "$a" == "--out="* ]]; then
        HAS_OUT=1
    fi
done
if [[ "$HAS_OUT" -eq 0 ]]; then
    ARGS+=("--out" "/bench/results/results-docker.json")
fi

echo "run-in-docker: launching bench.py" "${ARGS[@]}"
exec docker run --rm \
    -e ANTHROPIC_API_KEY \
    -v "$REPO_ROOT/bench:/bench" \
    "$IMAGE" \
    python3 /bench/bench.py "${ARGS[@]}"
