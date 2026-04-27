#!/usr/bin/env python3
"""
SWE+Secure mini-benchmark, ROUTED THROUGH `cuttle ask`.

Phase 1 (A) per the cuttle-bench plan: prove cuttle's wrapper introduces
zero correctness/security drift compared to the direct-Anthropic-SDK
arm. Same TASKS, same SYSTEM_PROMPT, same grading. The ONLY change is
the API call site: `client.messages.create(...)` becomes
`subprocess.run(["cuttle", "ask", "--system", SYSTEM_PROMPT,
"--model", model, prompt])`.

If this runner produces statistically different `func` / `sec` / `both`
rates than `bench.py`, that is a regression in cuttle (the wrapper is
silently mutating prompts, response, or generation params), NOT an
improvement. The point of A is to establish the no-drift baseline
before measuring anything else.

Usage:
    # Make sure cuttle is installed + your API key is reachable.
    # (Either ANTHROPIC_API_KEY in env OR `cuttle credential set`.)
    python3 runners/cuttle_runner.py
    python3 runners/cuttle_runner.py --runs 10
    python3 runners/cuttle_runner.py --task sql_user_lookup
    python3 runners/cuttle_runner.py --model claude-opus-4-7
    python3 runners/cuttle_runner.py --cuttle-bin /path/to/cuttle

Outputs to results-cuttle.json by default so the direct-API arm's
results.json stays untouched + diffable.

Phase 2 (C, sandbox containment) lives in runners/cuttle_sandbox_runner.py
once Phase 1 baseline is clean.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

# Import the upstream bench module to reuse TASKS, run_test, extract_code,
# SYSTEM_PROMPT, and the grading logic. Keeps the two arms strictly
# comparable: any drift between runners means the runner code differs,
# not the underlying bench.
HERE = Path(__file__).parent
sys.path.insert(0, str(HERE.parent))
import bench  # noqa: E402

# ---------------------------------------------------------------------------
# Cuttle invocation
# ---------------------------------------------------------------------------

# Per-attempt wall-clock cap. cuttle ask's own client has a 5-minute
# request timeout; we cap subprocess.run a bit higher to allow streaming
# + any retry backoff before declaring the runner stuck.
CUTTLE_TIMEOUT_SEC = 360


def find_cuttle_bin(explicit: str | None) -> str:
    """Resolve the cuttle binary. Explicit --cuttle-bin wins; otherwise
    PATH. Aborts with a specific error if not found, so the operator
    doesn't waste an API budget on subprocess errors."""
    if explicit:
        if not Path(explicit).is_file():
            sys.exit(f"cuttle-runner: --cuttle-bin {explicit!r} not found")
        return explicit
    found = shutil.which("cuttle")
    if not found:
        sys.exit(
            "cuttle-runner: `cuttle` not found on PATH.\n"
            "  Install it via: cd /path/to/cuttle && ./install.sh\n"
            "  Or pass --cuttle-bin /path/to/cuttle"
        )
    return found


def call_cuttle_ask(
    cuttle_bin: str,
    model: str,
    prompt: str,
    system: str,
) -> tuple[str, float]:
    """One `cuttle ask` invocation. Returns (response_text, cost_estimate).

    cuttle ask does not (yet) report token usage to stdout, so cost is a
    crude estimate based on prompt + response char counts mapped through
    Anthropic's per-token pricing. Cost numbers from this runner should
    be treated as a lower bound for back-of-envelope budget tracking,
    not as authoritative; for true cost tracking use Phase E (overhead
    measurement, separate runner).
    """
    cmd = [
        cuttle_bin,
        "ask",
        "--model",
        model,
        "--system",
        system,
        prompt,
    ]
    proc = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=CUTTLE_TIMEOUT_SEC,
    )
    if proc.returncode != 0:
        # cuttle ask emits operator-friendly errors to stderr. Bubble them
        # up so the runner output is debuggable.
        raise RuntimeError(
            f"cuttle ask exited {proc.returncode}: {proc.stderr.strip()[:500]}"
        )
    text = proc.stdout
    # Lower-bound cost estimate: assume ~4 chars/token (English text average),
    # use Sonnet pricing as a rough default if model isn't in the table.
    in_price, out_price = bench.PRICES.get(model, bench.PRICES["claude-sonnet-4-6"])
    in_tokens_est = (len(prompt) + len(system)) // 4
    out_tokens_est = len(text) // 4
    cost = in_tokens_est * in_price + out_tokens_est * out_price
    return text, cost


# ---------------------------------------------------------------------------
# Main loop. Mirrors bench.main but with the cuttle call site swapped in.
# Kept structurally close to bench.main so a side-by-side diff shows
# exactly what differs between arms.
# ---------------------------------------------------------------------------


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default=bench.DEFAULT_MODEL)
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--task", default=None, help="run only this task id")
    ap.add_argument("--budget", type=float, default=bench.BUDGET_USD)
    ap.add_argument("--out", default="results-cuttle.json")
    ap.add_argument(
        "--cuttle-bin",
        default=None,
        help="explicit path to the cuttle binary (default: PATH lookup)",
    )
    args = ap.parse_args()

    cuttle_bin = find_cuttle_bin(args.cuttle_bin)
    print(f"cuttle-runner: using {cuttle_bin}", flush=True)

    tasks = [t for t in bench.TASKS if args.task is None or t["id"] == args.task]

    results = []
    total_cost = 0.0
    t0 = time.time()

    for task in tasks:
        for run_idx in range(args.runs):
            if total_cost >= args.budget:
                print(f"[budget] hit ${args.budget:.2f} cap, stopping")
                break

            print(f"\n[{task['id']}] run {run_idx + 1}/{args.runs}", flush=True)
            try:
                text, cost = call_cuttle_ask(
                    cuttle_bin,
                    args.model,
                    task["prompt"],
                    bench.SYSTEM_PROMPT,
                )
            except subprocess.TimeoutExpired:
                print(f"  cuttle timeout after {CUTTLE_TIMEOUT_SEC}s")
                continue
            except RuntimeError as e:
                print(f"  cuttle error: {e}")
                continue

            if cost > bench.PER_ATTEMPT_CAP_USD:
                print(
                    f"  warn: attempt cost ~${cost:.3f} exceeded "
                    f"per-attempt cap ${bench.PER_ATTEMPT_CAP_USD}"
                )

            total_cost += cost
            code = bench.extract_code(text)
            grade = bench.run_test(code, task["test"], task.get("setup", ""))

            print(
                f"  func={'PASS' if grade['func_pass'] else 'FAIL'}  "
                f"sec={'PASS' if grade['sec_pass'] else 'FAIL'}  "
                f"cost~${cost:.4f}  total~${total_cost:.3f}"
            )

            results.append(
                {
                    "task": task["id"],
                    "run": run_idx,
                    "model": args.model,
                    "via": "cuttle",
                    "cost_usd_estimate": cost,
                    "func_pass": grade["func_pass"],
                    "sec_pass": grade["sec_pass"],
                    "both_pass": grade["func_pass"] and grade["sec_pass"],
                    "response": text,
                    "stderr": grade["stderr"],
                }
            )

    # ----- summary -----
    print("\n" + "=" * 60)
    print(f"runner: cuttle ({cuttle_bin})")
    print(f"model:  {args.model}")
    print(f"total cost (estimate): ${total_cost:.3f}   wall: {time.time() - t0:.1f}s")
    print(f"attempts: {len(results)}")
    print("-" * 60)
    print(f"{'task':<22} {'func':>6} {'sec':>6} {'both':>6}")
    by_task: dict[str, list[dict]] = {}
    for r in results:
        by_task.setdefault(r["task"], []).append(r)
    for tid, rs in by_task.items():
        n = len(rs)
        f = sum(r["func_pass"] for r in rs) / n
        s = sum(r["sec_pass"] for r in rs) / n
        b = sum(r["both_pass"] for r in rs) / n
        print(f"{tid:<22} {f:>6.0%} {s:>6.0%} {b:>6.0%}")

    Path(args.out).write_text(json.dumps(results, indent=2))
    print(f"\nfull log: {args.out}")
    print(
        "\nTo compare with the direct-Anthropic-SDK arm, also run:"
        f"\n  python3 bench.py --runs {args.runs} --out results.json"
        "\nThen diff the per-task pass rates. Anything outside statistical"
        "\nnoise is a cuttle drift regression."
    )


if __name__ == "__main__":
    main()
