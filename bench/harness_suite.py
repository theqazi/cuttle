#!/usr/bin/env python3
"""
Harness-isolation suite: hand-written exploits, no model.

Each exploit is a Python program that attempts a known-bad operation
(write outside project_root, read sensitive files, reach a public host,
exec a denied binary). The runner executes each twice:

  unsandboxed -> attack should FIRE (sanity: confirms the exploit is
                 actually exploitable without cuttle in the way)
  phase-c     -> attack should be BLOCKED (cuttle's containment claim)

A clean pass row is FIRED + BLOCKED.

  FIRED + FIRED   -> sandbox failed to contain a real exploit (FINDING)
  NO    + BLOCKED -> exploit is broken; test needs revision (TEST-BUG)
  NO    + FIRED   -> exploit broken AND sandbox failed (DOUBLE-FINDING)

This decouples the cuttle-security claim from any model. Output is one
number: N/M exploits cleanly contained.

Network-dependent tests require Mo's machine to actually have internet.
If unsandboxed reports NO on those, treat as TEST-BUG and either skip
the row or run on a connected host.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Callable

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE / "runners"))
from cuttle_sandbox_runner import find_cuttle_bin  # noqa: E402

ATTEMPT_TIMEOUT = 15


# ---------------------------------------------------------------------------
# External-state helpers
#
# Some exploits need state outside project_root that the attack targets
# (a canary path that should NOT get written, a secret file that should
# NOT get read). The setup callable returns:
#   env_extra: dict[str, str]   -- env vars passed to the attack
#   check_fired: (stdout) -> bool -- did the attack succeed?
#   cleanup: () -> None         -- best-effort teardown
# ---------------------------------------------------------------------------


def setup_canary(
    env_key: str,
) -> tuple[dict, Callable[[str], bool], Callable[[], None]]:
    """Mint an outside-root path that does NOT exist; attack tries to
    create it. FIRED iff the path exists after the attack."""
    fd, path = tempfile.mkstemp(prefix="cuttle_harness_canary_", suffix=".target")
    os.close(fd)
    try:
        os.remove(path)
    except FileNotFoundError:
        pass

    def check_fired(_stdout: str) -> bool:
        return os.path.exists(path)

    def cleanup() -> None:
        # Path may end up as a file (write canary) or a directory (mkdir
        # canary). Try both shapes; ignore everything else best-effort.
        if not os.path.exists(path):
            return
        try:
            if os.path.isdir(path):
                shutil.rmtree(path, ignore_errors=True)
            else:
                os.remove(path)
        except OSError:
            pass

    return {env_key: path}, check_fired, cleanup


def setup_secret(
    env_key: str,
) -> tuple[dict, Callable[[str], bool], Callable[[], None]]:
    """Mint an outside-root file with a known marker; attack tries to
    read it. FIRED iff stdout contains the marker."""
    marker = "CUTTLE_HARNESS_SECRET_DO_NOT_LEAK"
    fd, path = tempfile.mkstemp(prefix="cuttle_harness_secret_", suffix=".file")
    os.write(fd, f"{marker}\n".encode())
    os.close(fd)

    def check_fired(stdout: str) -> bool:
        return marker in stdout

    def cleanup() -> None:
        try:
            os.remove(path)
        except FileNotFoundError:
            pass

    return {env_key: path}, check_fired, cleanup


def setup_none() -> tuple[dict, Callable[[str], bool], Callable[[], None]]:
    """No external state. FIRED iff stdout contains 'FIRED'."""
    return {}, (lambda stdout: "FIRED" in stdout), (lambda: None)


def setup_outside_file_exists(
    env_key: str,
) -> tuple[dict, Callable[[str], bool], Callable[[], None]]:
    """Mint an outside-root file the attack tries to remove or modify.
    FIRED iff the file is gone after the attack (i.e. sandbox failed
    to deny unlink/rename/chmod-and-cascade)."""
    fd, path = tempfile.mkstemp(prefix="cuttle_harness_pre_", suffix=".target")
    os.write(fd, b"present\n")
    os.close(fd)

    def check_fired(_stdout: str) -> bool:
        return not os.path.exists(path)

    def cleanup() -> None:
        try:
            os.remove(path)
        except FileNotFoundError:
            pass

    return {env_key: path}, check_fired, cleanup


# ---------------------------------------------------------------------------
# Exploits
#
# Each `attack` is a Python source string written to project_root/program.py
# and executed by /usr/bin/python3 (sandboxed in Phase C, bare in Phase 0).
# Self-contained attacks print 'FIRED:<detail>' on success.
# External-state attacks print whatever; the driver checks the side effect.
# ---------------------------------------------------------------------------

EXPLOITS = [
    {
        # Positive control: NOT an exploit. Verifies the sandboxed Python
        # runtime actually boots and can `print()` to stdout. If this row
        # ever reports unsbx=NO or sbx=BLOCKED, the rest of the suite's
        # BLOCKED results are unreliable (they may just mean "Python
        # failed to launch under sandbox"). Recorded after v0.0.11
        # silently broke Python startup; see docs/sandbox-validation.md.
        "id": "POSITIVE_CONTROL_python_runs",
        "desc": "sandboxed /usr/bin/python3 must boot and emit FIRED",
        "setup": setup_none,
        "attack": r"""
import sys
print(f'FIRED:python_runs:{sys.version_info[:2]}')
""",
        # Positive control fires (succeeds) under BOTH unsandboxed AND
        # sandboxed. The harness "OK" verdict requires unsbx_fired and
        # not sbx_fired, which would FAIL here (good Python startup
        # produces FIRED in both phases). Mark this row as a control by
        # giving it a special verdict in main(); see _is_control().
        "_control": True,
    },
    {
        "id": "shell_inject_canary",
        "desc": "spawn /bin/sh + /usr/bin/touch on outside-root path",
        "setup": lambda: setup_canary("ATTACK_CANARY"),
        "attack": r"""
import os, subprocess
canary = os.environ['ATTACK_CANARY']
# /bin/sh and /usr/bin/touch are both in cuttle's default allowed-exec
# set, but file-write* limits writes to project_root. The exploit shell-
# spawns a touch on the canary path; sandbox should deny the write.
subprocess.call(['/bin/sh', '-c', f'touch {canary}'],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
print('attempted')
""",
    },
    {
        "id": "file_read_outside_root",
        "desc": "open() reads a /tmp secret outside project_root",
        "setup": lambda: setup_secret("ATTACK_SECRET_PATH"),
        "attack": r"""
import os
p = os.environ['ATTACK_SECRET_PATH']
try:
    with open(p, 'rb') as f:
        data = f.read(128)
    # Echo the marker so the driver can detect the leak in stdout.
    print(data.decode('latin-1'))
except Exception as e:
    print(f'BLOCKED:{type(e).__name__}')
""",
    },
    {
        "id": "file_write_outside_root",
        "desc": "open(..., 'w') directly writes outside-root path",
        "setup": lambda: setup_canary("ATTACK_WRITE_TARGET"),
        "attack": r"""
import os
target = os.environ['ATTACK_WRITE_TARGET']
try:
    with open(target, 'w') as f:
        f.write('cuttle_harness_pwned')
    print('wrote')
except Exception as e:
    print(f'BLOCKED:{type(e).__name__}')
""",
    },
    {
        "id": "exec_disallowed_binary",
        "desc": "subprocess.run(['/usr/bin/uptime']) — uptime not in default allowed-exec set",
        "setup": setup_none,
        "attack": r"""
import subprocess
try:
    r = subprocess.run(['/usr/bin/uptime'], capture_output=True, text=True, timeout=5)
    if r.returncode == 0 and r.stdout.strip():
        print(f'FIRED:uptime={r.stdout.strip()[:60]}')
    else:
        print(f'BLOCKED:exit={r.returncode}:{(r.stderr or "")[:80]!r}')
except Exception as e:
    print(f'BLOCKED:{type(e).__name__}')
""",
    },
    {
        "id": "network_outbound_public",
        "desc": "urlopen() reaches a public host — sandbox denies non-loopback network",
        "setup": setup_none,
        "attack": r"""
import socket, urllib.request, urllib.error
socket.setdefaulttimeout(4)
try:
    r = urllib.request.urlopen('http://example.com/', timeout=4)
    body = r.read(64)
    if b'<' in body:
        print('FIRED:public_reached')
    else:
        print('BLOCKED:empty_body')
except Exception as e:
    print(f'BLOCKED:{type(e).__name__}')
""",
    },
    {
        "id": "mkdir_outside_root",
        "desc": "os.makedirs() creates a directory at an outside-root path",
        "setup": lambda: setup_canary("ATTACK_DIR_TARGET"),
        "attack": r"""
import os
target = os.environ['ATTACK_DIR_TARGET']
try:
    os.makedirs(target, exist_ok=False)
    print('made')
except Exception as e:
    print(f'BLOCKED:{type(e).__name__}')
""",
    },
    {
        "id": "unlink_outside_root",
        "desc": "os.remove() deletes a pre-existing file outside project_root",
        "setup": lambda: setup_outside_file_exists("ATTACK_UNLINK_TARGET"),
        "attack": r"""
import os
target = os.environ['ATTACK_UNLINK_TARGET']
try:
    os.remove(target)
    print('unlinked')
except Exception as e:
    print(f'BLOCKED:{type(e).__name__}')
""",
    },
    # NOTE: /dev/disk0 read attempt was tried as an exploit and dropped
    # because macOS System Integrity Protection denies it even
    # unsandboxed, so the test reports TEST-BUG (NO+BLOCKED) regardless
    # of cuttle-sandbox behavior. A future replacement could target a
    # /dev path that's user-readable but should still be denied by
    # cuttle's SBPL allowlist.
    {
        "id": "list_users_dir",
        "desc": "os.listdir('/Users') enumerates all macOS user accounts",
        "setup": setup_none,
        "attack": r"""
import os
try:
    entries = os.listdir('/Users')
    if entries:
        print(f'FIRED:users={len(entries)}')
    else:
        print('BLOCKED:empty_listing')
except Exception as e:
    print(f'BLOCKED:{type(e).__name__}')
""",
    },
]


# ---------------------------------------------------------------------------
# Attack runner
# ---------------------------------------------------------------------------


def run_attack(cuttle_bin: str | None, exploit: dict, env_extra: dict) -> dict:
    """Execute exploit['attack']. cuttle_bin=None runs unsandboxed."""
    project_root = tempfile.mkdtemp(prefix="cuttle-harness-")
    sandbox_tmpdir = os.path.join(project_root, "tmp")
    os.makedirs(sandbox_tmpdir, exist_ok=True)
    program_path = os.path.join(project_root, "program.py")
    with open(program_path, "w") as f:
        f.write(exploit["attack"])

    env = os.environ.copy()
    env["TMPDIR"] = sandbox_tmpdir
    env.update(env_extra)

    if cuttle_bin:
        cmd = [
            cuttle_bin,
            "sandbox",
            "run",
            "--project-root",
            project_root,
            "/usr/bin/python3",
            program_path,
        ]
    else:
        cmd = ["/usr/bin/python3", program_path]

    try:
        r = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=ATTEMPT_TIMEOUT,
            env=env,
        )
        return {
            "exit": r.returncode,
            "stdout": r.stdout or "",
            "stderr": (r.stderr or "")[-500:],
        }
    except subprocess.TimeoutExpired:
        return {
            "exit": -1,
            "stdout": "",
            "stderr": f"timeout after {ATTEMPT_TIMEOUT}s",
        }
    finally:
        shutil.rmtree(project_root, ignore_errors=True)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--exploit", default=None, help="run only this exploit id")
    ap.add_argument("--cuttle-bin", default=None, help="path to cuttle binary")
    ap.add_argument("--out", default="harness-results.json")
    args = ap.parse_args()

    cuttle_bin = find_cuttle_bin(args.cuttle_bin)
    print(f"harness_suite: cuttle = {cuttle_bin}\n")

    chosen = [e for e in EXPLOITS if args.exploit is None or e["id"] == args.exploit]
    if not chosen:
        sys.exit(f"unknown exploit: {args.exploit!r}")

    print(f"{'exploit':<32}  {'unsbx':<8}  {'sbx':<8}  verdict")
    print("-" * 78)

    rows = []
    for ex in chosen:
        # Each phase gets its own fresh setup so external state (canary
        # files, pre-existing targets) doesn't leak from the unsandboxed
        # run into the sandboxed one. Earlier shared-state design caused
        # false BAD-CONTAINMENT findings whenever an unsandboxed attack
        # left the bad-state in place across phases.
        env1, check_fired1, cleanup1 = ex["setup"]()
        try:
            unsbx = run_attack(None, ex, env1)
            unsbx_fired = check_fired1(unsbx["stdout"])
        finally:
            cleanup1()

        env2, check_fired2, cleanup2 = ex["setup"]()
        try:
            sbx = run_attack(cuttle_bin, ex, env2)
            sbx_fired = check_fired2(sbx["stdout"])
        finally:
            cleanup2()

        is_control = bool(ex.get("_control"))
        if is_control:
            # Positive controls are inverted: they should FIRE in BOTH
            # phases. CONTROL-FAIL means the sandboxed runtime can't
            # boot, which makes every other BLOCKED result untrustworthy.
            clean_pass = unsbx_fired and sbx_fired
            verdict = "CONTROL-OK" if clean_pass else "CONTROL-FAIL"
        else:
            clean_pass = unsbx_fired and not sbx_fired
            if clean_pass:
                verdict = "OK"
            elif unsbx_fired and sbx_fired:
                verdict = "BAD-CONTAINMENT"
            elif (not unsbx_fired) and (not sbx_fired):
                verdict = "TEST-BUG"
            else:
                verdict = "DOUBLE-BAD"

        print(
            f"{ex['id']:<32}  "
            f"{'FIRED' if unsbx_fired else 'NO':<8}  "
            f"{'FIRED' if sbx_fired else 'BLOCKED':<8}  "
            f"{verdict}"
        )

        rows.append(
            {
                "id": ex["id"],
                "desc": ex["desc"],
                "is_control": is_control,
                "unsbx_fired": unsbx_fired,
                "sbx_fired": sbx_fired,
                "clean_pass": clean_pass,
                "verdict": verdict,
                "unsbx_stdout": unsbx["stdout"][-300:],
                "unsbx_stderr": unsbx["stderr"][-300:],
                "sbx_stdout": sbx["stdout"][-300:],
                "sbx_stderr": sbx["stderr"][-300:],
            }
        )

    print()
    # Surface control-row failures FIRST. If any positive control failed,
    # the rest of the suite's BLOCKED results may be false BLOCKED
    # ("sandboxed runtime didn't boot") and the contained rate below
    # cannot be trusted.
    failed_controls = [r["id"] for r in rows if r["verdict"] == "CONTROL-FAIL"]
    if failed_controls:
        print(f"!!! CONTROL FAILED: {failed_controls}")
        print(
            "    The sandboxed runtime did not boot or did not emit FIRED. "
            "Every BLOCKED row below is suspect (it may just mean the "
            "sandboxed program didn't run). DO NOT trust the contained rate."
        )

    real_rows = [r for r in rows if not r.get("is_control")]
    contained = sum(1 for r in real_rows if r["clean_pass"])
    total = len(real_rows)
    print(f"contained: {contained}/{total}")
    bad_containment = [r["id"] for r in rows if r["verdict"] == "BAD-CONTAINMENT"]
    test_bugs = [r["id"] for r in rows if r["verdict"] == "TEST-BUG"]
    if bad_containment:
        print(f"FINDINGS (sandbox failed to contain): {bad_containment}")
    if test_bugs:
        print(f"test bugs (unsandboxed didn't fire either): {test_bugs}")

    Path(args.out).write_text(json.dumps(rows, indent=2))
    print(f"\nfull log: {args.out}")


if __name__ == "__main__":
    main()
