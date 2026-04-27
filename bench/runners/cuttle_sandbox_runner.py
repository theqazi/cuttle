#!/usr/bin/env python3
"""
SWE+Secure mini-benchmark, with TEST EXECUTION wrapped in `cuttle sandbox run`.

Phase 2 (C) per the cuttle-bench plan: the model still gets called via
the Anthropic SDK directly (so this runner's call_model = bench.call_model
unchanged). The change is at the EXEC site: instead of bench.run_test
running model code in an unconfined subprocess, this runner writes the
program to a per-attempt project_root directory and runs it inside
`cuttle sandbox run --project-root <root> /usr/bin/python3 program.py`.

What this measures:

The bench's "interesting failure mode" is correct-but-INSECURE code.
Phase A measured how often the model produces such code. Phase C asks
the next question: WHEN the model produces such code, does cuttle's
sandbox layer prevent the lateral effect?

Concretely:
- `command_injection`: model's vulnerable git_log() shells out to
  `touch /tmp/__pwned_canary__`. Under sandbox, touch is not in the
  default allowed-exec set + /tmp/__pwned_canary__ is outside
  project_root → both layers deny → SEC_PASS even though the model is
  vulnerable.
- `path_traversal`: model's vulnerable read_upload() reads
  ../secret.txt. Under sandbox, the secret file path is outside
  project_root → file-read denied → SEC_PASS even when model is
  vulnerable.
- `ssrf_fetch`: bench harness already monkeypatches the network so the
  sandbox network-deny doesn't add value here in v1. (To exercise the
  sandbox network rule, strip the monkeypatch — separate experiment.)

The DELTA between Phase A SEC% and Phase C SEC% per task IS the value
cuttle's sandbox layer adds. Phase A baselines the model alone; Phase C
adds the harness containment.

For the "no model bug" cases (jwt_verify, session_token, html_render,
csv_export, sql_user_lookup), Phase A and Phase C should be
indistinguishable (sandbox is a no-op when the model is correct).

Usage:
    pip install anthropic
    export ANTHROPIC_API_KEY=...    # for the model call (NOT for cuttle)
    python3 runners/cuttle_sandbox_runner.py
    python3 runners/cuttle_sandbox_runner.py --runs 10
    python3 runners/cuttle_sandbox_runner.py --task command_injection

Outputs to results-cuttle-sandbox.json by default so the Phase A
results-cuttle.json and the direct-API results.json stay diffable.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE.parent))
import bench  # noqa: E402

# ---------------------------------------------------------------------------
# Sandbox-wrapped execution. Mirrors bench.run_test but routes through
# `cuttle sandbox run` with a per-attempt project_root + TMPDIR override.
# ---------------------------------------------------------------------------

SANDBOX_TIMEOUT_SEC = 30  # double the bench's 15s; sandbox-exec adds ~50ms.


def find_cuttle_bin(explicit: str | None) -> str:
    """Resolve the cuttle binary. Same logic as cuttle_runner.py."""
    if explicit:
        if not Path(explicit).is_file():
            sys.exit(f"cuttle-sandbox-runner: --cuttle-bin {explicit!r} not found")
        return explicit
    found = shutil.which("cuttle")
    if not found:
        sys.exit(
            "cuttle-sandbox-runner: `cuttle` not found on PATH.\n"
            "  Install it via: cd /path/to/cuttle && ./install.sh\n"
            "  Or pass --cuttle-bin /path/to/cuttle"
        )
    return found


def run_test_sandboxed(
    cuttle_bin: str,
    model_code: str,
    test_code: str,
    setup_code: str = "",
) -> dict:
    """Execute model+test inside cuttle sandbox. Returns same shape as
    bench.run_test: {func_pass, sec_pass, stdout, stderr}.

    Per-attempt fresh project_root under tempfile so concurrent attempts
    don't trip over each other and so the sandbox view of the filesystem
    is minimal (read+write ONLY here, not the operator's whole project).

    If setup_code is provided, run it UNSANDBOXED first with
    BENCH_PROJECT_ROOT=project_root. This is how command_injection
    prebuilds its git repo without git fighting SBPL — the setup writes
    into project_root, then the sandboxed test reads/operates on it.
    The model's vulnerable code still runs sandboxed, so the sec-delta
    measurement remains intact.

    TMPDIR is overridden to point inside project_root so any
    `tempfile.mkdtemp()` calls inside the sandboxed test land in
    sandbox-readable space.
    """
    # tempfile.mkdtemp returns an absolute path; required by SBPL.
    project_root = tempfile.mkdtemp(prefix="cuttle-bench-attempt-")
    sandbox_tmpdir = os.path.join(project_root, "tmp")
    os.makedirs(sandbox_tmpdir, exist_ok=True)

    env = os.environ.copy()
    env["TMPDIR"] = sandbox_tmpdir
    env["BENCH_PROJECT_ROOT"] = project_root

    if setup_code:
        try:
            r = subprocess.run(
                [sys.executable, "-c", setup_code],
                capture_output=True,
                text=True,
                timeout=SANDBOX_TIMEOUT_SEC,
                env=env,
            )
            if r.returncode != 0:
                return {
                    "func_pass": False,
                    "sec_pass": False,
                    "stdout": "",
                    "stderr": f"setup failed: {r.stderr[-400:]}",
                    "exit_code": r.returncode,
                }
        except subprocess.TimeoutExpired:
            return {
                "func_pass": False,
                "sec_pass": False,
                "stdout": "",
                "stderr": f"setup timeout after {SANDBOX_TIMEOUT_SEC}s",
                "exit_code": -1,
            }

    program = (
        f"__MODEL_SOURCE__ = {model_code!r}\n\n"
        f"{model_code}\n\n"
        "# --- test harness ---\n"
        f"{test_code}"
    )
    program_path = os.path.join(project_root, "program.py")
    with open(program_path, "w") as f:
        f.write(program)

    cmd = [
        cuttle_bin,
        "sandbox",
        "run",
        "--project-root",
        project_root,
        "/usr/bin/python3",
        program_path,
    ]
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=SANDBOX_TIMEOUT_SEC,
            env=env,
        )
        out = result.stdout
        return {
            "func_pass": "FUNC_PASS" in out,
            "sec_pass": "SEC_PASS" in out,
            "stdout": out[-500:],
            "stderr": result.stderr[-500:],
            "exit_code": result.returncode,
        }
    except subprocess.TimeoutExpired:
        return {
            "func_pass": False,
            "sec_pass": False,
            "stdout": "",
            "stderr": f"sandbox timeout after {SANDBOX_TIMEOUT_SEC}s",
            "exit_code": -1,
        }
    finally:
        # Best-effort cleanup. If the sandbox left behind a process or a
        # file we can't remove, don't fail the whole bench run.
        try:
            shutil.rmtree(project_root, ignore_errors=True)
        except Exception:
            pass


# ---------------------------------------------------------------------------
# Main loop. Same shape as bench.main but with sandboxed run_test.
# Model call site is bench.call_model unchanged (Phase C measures
# containment, not call-path drift; that was Phase A's job).
# ---------------------------------------------------------------------------


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default=bench.DEFAULT_MODEL)
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--task", default=None, help="run only this task id")
    ap.add_argument("--budget", type=float, default=bench.BUDGET_USD)
    ap.add_argument("--out", default="results-cuttle-sandbox.json")
    ap.add_argument(
        "--cuttle-bin",
        default=None,
        help="explicit path to the cuttle binary (default: PATH lookup)",
    )
    args = ap.parse_args()

    cuttle_bin = find_cuttle_bin(args.cuttle_bin)
    print(f"cuttle-sandbox-runner: using {cuttle_bin}", flush=True)

    # Phase C uses the direct Anthropic SDK for the model call, NOT
    # cuttle ask. Mixing in the cuttle-ask wrapper would make the
    # delta-vs-Phase-A interpretation muddier (is the change from
    # sandbox or from the wrapper?). One change at a time.
    from anthropic import Anthropic

    client = Anthropic()

    tasks = [t for t in bench.TASKS if args.task is None or t["id"] == args.task]

    # Tasks that opt out of Phase C entirely. These still run under Phase A
    # (direct API + unsandboxed test) so the SEC% baseline is intact; we
    # just don't measure containment-delta on them. Reason is recorded so
    # comparisons across runs surface the gap explicitly rather than as
    # silent zeros.
    skipped = []
    runnable_tasks = []
    for t in tasks:
        reason = t.get("phase_c_skip")
        if reason:
            skipped.append((t["id"], reason))
            print(f"[{t['id']}] SKIP (phase C): {reason}", flush=True)
        else:
            runnable_tasks.append(t)

    results = []
    total_cost = 0.0
    t0 = time.time()

    for task in runnable_tasks:
        for run_idx in range(args.runs):
            if total_cost >= args.budget:
                print(f"[budget] hit ${args.budget:.2f} cap, stopping")
                break

            print(f"\n[{task['id']}] run {run_idx + 1}/{args.runs}", flush=True)
            try:
                text, cost = bench.call_model(client, args.model, task["prompt"])
            except Exception as e:
                print(f"  api error: {e}")
                continue

            if cost > bench.PER_ATTEMPT_CAP_USD:
                print(
                    f"  warn: attempt cost ${cost:.3f} exceeded "
                    f"per-attempt cap ${bench.PER_ATTEMPT_CAP_USD}"
                )

            total_cost += cost
            code = bench.extract_code(text)
            grade = run_test_sandboxed(
                cuttle_bin, code, task["test"], task.get("setup", "")
            )

            print(
                f"  func={'PASS' if grade['func_pass'] else 'FAIL'}  "
                f"sec={'PASS' if grade['sec_pass'] else 'FAIL'}  "
                f"sandbox_exit={grade['exit_code']}  "
                f"cost=${cost:.4f}  total=${total_cost:.3f}"
            )

            results.append(
                {
                    "task": task["id"],
                    "run": run_idx,
                    "model": args.model,
                    "via": "cuttle-sandbox",
                    "cost_usd": cost,
                    "func_pass": grade["func_pass"],
                    "sec_pass": grade["sec_pass"],
                    "both_pass": grade["func_pass"] and grade["sec_pass"],
                    "sandbox_exit_code": grade["exit_code"],
                    "response": text,
                    "stderr": grade["stderr"],
                }
            )

    # ----- summary -----
    print("\n" + "=" * 60)
    print(f"runner: cuttle-sandbox ({cuttle_bin})")
    print(f"model:  {args.model}")
    print(f"total cost: ${total_cost:.3f}   wall: {time.time() - t0:.1f}s")
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
    for tid, reason in skipped:
        print(f"{tid:<22} {'SKIP':>6} {'SKIP':>6} {'SKIP':>6}  ({reason})")

    # Persist skip records as flat-list members so compare_runs.py keeps
    # working unchanged. Consumers filter on `skipped == True`.
    payload = list(results)
    for tid, reason in skipped:
        payload.append(
            {
                "task": tid,
                "model": args.model,
                "via": "cuttle-sandbox",
                "skipped": True,
                "skip_reason": reason,
            }
        )
    Path(args.out).write_text(json.dumps(payload, indent=2))
    print(f"\nfull log: {args.out}")
    print(
        "\nThe sandbox-containment value shows up as a SEC delta on tasks"
        "\nwhere the model can produce vulnerable code that touches files"
        "\noutside its working dir. Diff against Phase A:"
        "\n  python3 bench.py --runs {0}".format(args.runs)
        + "\n  diff <(jq -r '.[] | \"\\(.task) sec=\\(.sec_pass)\"' results.json | sort -u) \\"
        + "\n       <(jq -r '.[] | \"\\(.task) sec=\\(.sec_pass)\"' "
        + f"{args.out} | sort -u)"
    )


if __name__ == "__main__":
    main()
