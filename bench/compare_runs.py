#!/usr/bin/env python3
"""
Side-by-side comparison of bench result files.

Reads up to three JSON files produced by:
  - bench.py                              -> results.json           (baseline)
  - runners/cuttle_runner.py              -> results-cuttle.json    (Phase A)
  - runners/cuttle_sandbox_runner.py      -> results-cuttle-sandbox.json (Phase C)

Any subset of the three is fine; missing files are skipped, not errors,
so partial runs are still inspectable.

Output: per-task table with func/sec/both rates per arm + the deltas
(Phase A - baseline, Phase C - baseline). Deltas exceeding the
noise threshold for the run count are flagged with `*`.

Noise threshold (rough, not a statistical test): with N runs, the
binomial standard error at p=0.5 is ~0.5/sqrt(N). For 3 runs that's
~29 percentage points; deltas under that are within noise. For 10 runs
it's ~16pp. compare_runs derates the threshold by 0.5 to flag suggestive
trends (anything that looks like a real signal at the run count).

Usage:
    python3 compare_runs.py
    python3 compare_runs.py --runs 10        # informs the noise threshold
    python3 compare_runs.py --json           # machine-readable output
    python3 compare_runs.py \
        --baseline results.json \
        --phase-a results-cuttle.json \
        --phase-c results-cuttle-sandbox.json
"""

import argparse
import json
import math
import sys
from pathlib import Path


def load(path: str | None) -> list[dict] | None:
    """Load a results file. None if path is None or file missing."""
    if not path:
        return None
    p = Path(path)
    if not p.exists():
        return None
    return json.loads(p.read_text())


def per_task_rates(results: list[dict]) -> dict[str, dict[str, float]]:
    """Aggregate {task_id: {func, sec, both, n}}.

    Entries with `skipped: True` are recorded as a sentinel rate dict so
    the renderer can show "SKIP" instead of a fabricated zero. We never
    average a skip into a real run.
    """
    by_task: dict[str, list[dict]] = {}
    for r in results:
        by_task.setdefault(r["task"], []).append(r)
    out = {}
    for tid, rs in by_task.items():
        # If the task was opted out (e.g. phase_c_skip), short-circuit.
        skip_rs = [r for r in rs if r.get("skipped")]
        if skip_rs and not [r for r in rs if not r.get("skipped")]:
            out[tid] = {
                "skipped": True,
                "skip_reason": skip_rs[0].get("skip_reason", ""),
                "n": 0,
            }
            continue
        real_rs = [r for r in rs if not r.get("skipped")]
        n = len(real_rs)
        out[tid] = {
            "func": sum(r["func_pass"] for r in real_rs) / n,
            "sec": sum(r["sec_pass"] for r in real_rs) / n,
            "both": sum(r["both_pass"] for r in real_rs) / n,
            "n": n,
        }
    return out


def fmt_pct(x: float) -> str:
    """Render a 0..1 rate as a 4-char percentage string."""
    return f"{x * 100:>3.0f}%"


def fmt_delta(x: float | None, threshold_pp: float) -> str:
    """Render a -1..1 delta in pp, with `*` flag if |delta| > threshold."""
    if x is None:
        return " --- "
    pp = x * 100
    flag = "*" if abs(pp) > threshold_pp else " "
    sign = "+" if pp > 0 else ("-" if pp < 0 else " ")
    return f"{sign}{abs(pp):>2.0f}pp{flag}"


def noise_threshold_pp(n: int) -> float:
    """Rough flag threshold in percentage points. Derated half of the
    binomial SE at p=0.5 for the given run count, so we flag anything
    that LOOKS like signal at this n. Not a statistical test."""
    if n <= 0:
        return 0.0
    se_pp = 100.0 * 0.5 / math.sqrt(n)
    return se_pp * 0.5


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--baseline",
        default="results.json",
        help="bench.py output (default: results.json)",
    )
    ap.add_argument(
        "--phase-a", default="results-cuttle.json", help="cuttle_runner.py output"
    )
    ap.add_argument(
        "--phase-c",
        default="results-cuttle-sandbox.json",
        help="cuttle_sandbox_runner.py output",
    )
    ap.add_argument(
        "--runs",
        type=int,
        default=None,
        help="override the run count used for noise-flag math "
        "(default: max n across loaded arms)",
    )
    ap.add_argument(
        "--json",
        action="store_true",
        help="emit machine-readable JSON instead of a table",
    )
    args = ap.parse_args()

    arms = {
        "baseline": load(args.baseline),
        "phase-a": load(args.phase_a),
        "phase-c": load(args.phase_c),
    }
    loaded = {k: v for k, v in arms.items() if v is not None}
    if not loaded:
        sys.exit(
            "compare_runs: no results files found. Run at least one of:\n"
            "  python3 bench.py\n"
            "  python3 runners/cuttle_runner.py\n"
            "  python3 runners/cuttle_sandbox_runner.py"
        )

    rates = {arm: per_task_rates(rs) for arm, rs in loaded.items()}

    # Union of all task ids across all loaded arms (so a partial run
    # against a single task still shows up cleanly).
    all_tasks = sorted({t for r in rates.values() for t in r})

    # Pick noise threshold from observed N (max across arms) unless
    # operator overrode it.
    observed_n = max(
        (rates[a][t]["n"] for a in rates for t in rates[a]),
        default=0,
    )
    n_for_threshold = args.runs if args.runs else observed_n
    threshold_pp = noise_threshold_pp(n_for_threshold)

    if args.json:
        out = {
            "noise_threshold_pp": round(threshold_pp, 2),
            "arms_loaded": list(loaded.keys()),
            "n_runs_for_threshold": n_for_threshold,
            "per_task": {
                t: {arm: rates[arm].get(t) for arm in rates} for t in all_tasks
            },
        }
        print(json.dumps(out, indent=2))
        return

    # ----- table -----
    arm_labels = {
        "baseline": "base",
        "phase-a": "ph-A ",
        "phase-c": "ph-C ",
    }
    arms_present = list(rates.keys())

    print(
        f"compare_runs: arms={','.join(arms_present)}  "
        f"n={n_for_threshold} runs/task  "
        f"noise>{threshold_pp:.0f}pp flagged with *"
    )
    print()

    # Header: task | for each arm: func sec both | for A and C: Δsec
    cols = ["task".ljust(22)]
    for arm in arms_present:
        lbl = arm_labels[arm]
        cols.append(f"{lbl} func sec both")
    if "baseline" in rates and "phase-a" in rates:
        cols.append("ΔsecA")
    if "baseline" in rates and "phase-c" in rates:
        cols.append("ΔsecC")
    print("  ".join(cols))
    print("-" * (len("  ".join(cols)) + 4))

    for tid in all_tasks:
        row = [tid.ljust(22)]
        for arm in arms_present:
            r = rates[arm].get(tid)
            if r is None:
                row.append(f"{arm_labels[arm]} ---  ---  --- ")
            elif r.get("skipped"):
                row.append(f"{arm_labels[arm]} SKIP SKIP SKIP")
            else:
                row.append(
                    f"{arm_labels[arm]} "
                    f"{fmt_pct(r['func'])} {fmt_pct(r['sec'])} {fmt_pct(r['both'])}"
                )
        # Deltas (sec axis only; that's where containment shows up).
        # Skipped arms can't produce a delta — render `---` instead of
        # silently zeroing.
        base = rates.get("baseline", {}).get(tid)
        a = rates.get("phase-a", {}).get(tid)
        c = rates.get("phase-c", {}).get(tid)

        def _real(rt: dict | None) -> dict | None:
            return None if rt is None or rt.get("skipped") else rt

        if "baseline" in rates and "phase-a" in rates:
            ba, aa = _real(base), _real(a)
            if ba and aa:
                row.append(fmt_delta(aa["sec"] - ba["sec"], threshold_pp))
            else:
                row.append(fmt_delta(None, threshold_pp))
        if "baseline" in rates and "phase-c" in rates:
            bc, cc = _real(base), _real(c)
            if bc and cc:
                row.append(fmt_delta(cc["sec"] - bc["sec"], threshold_pp))
            else:
                row.append(fmt_delta(None, threshold_pp))
        print("  ".join(row))

    print()
    print("Reading the deltas:")
    print("  ΔsecA: cuttle ask drift vs direct API. Should be ~0pp.")
    print("         Anything flagged is a cuttle-wrapper regression to investigate.")
    print("  ΔsecC: cuttle sandbox containment value vs direct API. Should be")
    print("         POSITIVE on path_traversal + command_injection (sandbox blocks")
    print("         the lateral effect even when model is vulnerable). Should be")
    print("         ~0pp on pure-compute tasks (sandbox is a no-op there).")


if __name__ == "__main__":
    main()
