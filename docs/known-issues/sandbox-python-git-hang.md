# KI-1: Python-subprocess hang on disallowed binaries under sandbox

**Status:** **FIXED in `cuttle-sandbox` v0.0.13** (2026-04-27).
**Affected:** `cuttle-sandbox` v0.0.10 - v0.0.12.
**Discovered:** 2026-04-27 (git case via `command_injection`).
**Generalized:** 2026-04-27 (sips case via `subprocess_shell_inject`).
**Root cause:** Python 3.9's `_posixsubprocess` fork-exec helper
opens `/dev/fd/` to enumerate inherited file descriptors and close
them. Cuttle's SBPL didn't allow `/dev/fd` reads (only specific
`/dev/null`, `/dev/random`, etc.), so `opendir("/dev/fd")` failed
with EPERM and Python fell back to `_close_fds_by_brute_force`,
which loops `close(fd)` from 3 to `RLIMIT_NOFILE`. Under the
sandbox, `RLIMIT_NOFILE` was effectively unbounded, making the loop
spin indefinitely at 99% CPU before the spawn could even fail.
**Fix:** added `(subpath "/dev/fd")` to the `(allow file-read* ...)`
block in `crates/cuttle-sandbox/src/profile.rs` so Python takes the
fast path. Sandboxed `subprocess.run` of any non-allowlisted binary
now returns `PermissionError: Operation not permitted` immediately
instead of hanging.

The rest of this doc is preserved as a record of the diagnosis path
(stack-sample bisection via `sample(1)` against the spinning Python
child) since the same technique applies to future SBPL regressions.

**Filename note:** this file is named `sandbox-python-git-hang.md` for
historical reasons; the issue is _not_ git-specific. Any binary that
is not in cuttle's default `(allow process-exec ...)` literal set
triggers the same hang when spawned via `subprocess.run` from
sandboxed `/usr/bin/python3`. Confirmed on git (commit `df5ef19` era)
and sips (Phase C bench run 2026-04-27 dated below).

## Symptom

`cuttle sandbox run /usr/bin/python3 program.py` hangs indefinitely
when `program.py` does:

```python
import subprocess
subprocess.run(['git', 'init', '-q', '/some/path'])
```

The sandboxed Python child process spins at 99% CPU and never returns.
The subprocess.run timeout (15s default in the bench harness) does not
fire. Killing the parent process leaves the child as an orphan that
must be `pkill`'d manually.

## Negative cases

The hang is specific to **Python-as-parent + sandboxed Python +
spawning some grandchild**. The exact trigger isn't fully nailed
down. Confirmed reproductions:

- git: `subprocess.run(['git', 'init', '-q', '<path>'])` from
  sandboxed `/usr/bin/python3`. Hang.
- sips: `subprocess.run(['/usr/bin/sips', '-s', 'format', ...])`
  from sandboxed `/usr/bin/python3`. Hang.

Confirmed non-reproductions:

- `cuttle sandbox run /usr/bin/git init -q /some/path` (bare, no
  Python wrapper). Returns in <100ms with exit 128 and
  `fatal: unable to access '~/.gitconfig': Operation not permitted`.
- `subprocess.run(['git', ...])` from `/usr/bin/python3` _outside_
  the sandbox. Returns normally.
- `subprocess.run(['/bin/echo', 'hello'])` _inside_ the sandbox.
  Returns normally.
- `subprocess.run(['/usr/bin/uptime'])` _inside_ the sandbox. Fast
  deny: harness suite's `exec_disallowed_binary` row reports
  BLOCKED in <1s. uptime is not in cuttle's default allowed-exec
  set, same as sips, yet uptime fast-denies and sips hangs.

The uptime-vs-sips asymmetry is interesting. Both binaries are
absent from the `(allow process-exec ...)` literal set, so both
should fail the exec at the same SBPL check. The fact that only
sips hangs suggests the trigger is something the _grandchild
exec setup_ does differently between the two. Plausible
distinguishing factors (none verified):

- sips is a re-execing shim (similar to `/usr/bin/python3`'s
  shim-to-Frameworks chain) and the re-exec attempt loops; uptime
  is a single-binary that fails exec immediately.
- sips opens a Mach service that the SBPL doesn't grant; uptime
  doesn't.
- sips's argv has flag arguments; uptime's doesn't (probably not
  the cause but worth ruling out).

## Root-cause hypothesis (unverified)

Python 3.9 (`/usr/bin/python3`, the only Python in cuttle's default
`(allow process-exec ...)` literal set) uses `posix_spawn` to start
subprocesses on macOS. When the sandbox profile is in effect, the
child-side setup phase of `posix_spawn` appears to enter a busy loop
on a syscall that the SBPL profile denies in a way the spawn code
doesn't gracefully handle.

Plausible candidates:

- A Mach port lookup the spawn code retries. Cuttle's profile has
  `(allow mach-lookup)` but maybe the inheritance to the grandchild
  context drops it.
- A signal-handler installation that loops on EPERM. We have
  `(allow signal (target self))` but that may not cover the
  spawn-internal mechanisms.
- An attempt to read a path under `/var/folders/<user>/<hash>/T/` for
  Python's site-init or temp-dir resolution. Cuttle v0.0.12 explicitly
  denies that subtree (per the per-user-TMPDIR-deny finding). The
  child may be retrying instead of falling through.

None of these are confirmed. Investigation requires `dtruss` or a
similar syscall tracer, and the macOS sandbox is hostile to dtrace
without root + SIP-disabled.

## Why it isn't a containment hole

The hang is a _liveness_ failure (the sandboxed process never
finishes), not a _safety_ failure (the sandbox still denies whatever
the child tried to do). Containment is intact: the only observable
side effect of the hang is the spinning child, not any leak outside
`project_root`. The harness suite's positive control row
(`POSITIVE_CONTROL_python_runs`) confirms basic Python startup works
under v0.0.12, so it is specifically the `subprocess.run(['git',
...])` shape that triggers the spin, not Python startup itself.

## Why it matters anyway

Operators using cuttle to harness an agent that legitimately runs
git from sandboxed Python (test suites, CI scripts, agentic-coding
workflows) will see their sandboxed program hang. The denial is
correct; the symptom is wrong. v0.1 ships with the workaround
documented here and the bench's `phase_c_skip` flag.

## Workarounds for operators

1. **Prebuild git state outside the sandbox.** The bench's
   `command_injection` task uses this pattern: a `setup` field runs
   `git init`/`git commit` _unsandboxed_ in `BENCH_PROJECT_ROOT`,
   then the sandboxed test only invokes the model's `git_log()`
   on the prebuilt repo. See `bench/bench.py` lines 132-180 for the
   shape. (Note: the `git_log` call inside the sandbox may still
   trigger the hang depending on what git does internally; this
   workaround helps when only init/commit need to run.)

2. **Use a different binary.** If git isn't strictly required, a
   sandboxed Python program that uses pure-Python file ops (no
   subprocess to git) won't hit this issue.

3. **Skip the task in Phase C.** Add `phase_c_skip: "<reason>"` to
   the task dict and the bench's Phase C runner will record a SKIP
   row rather than block waiting for the timeout.

## Future investigation

Steps to pursue when this gets prioritized:

1. Reproduce on a clean macOS install with SIP disabled to allow
   `dtruss -fW <pid>` against the spinning Python child. Identify
   the looping syscall.
2. Add the syscall's permitted operation to the SBPL profile (or
   confirm the rule is already there but not effective for
   posix_spawn-internal context).
3. Add a regression test in `cuttle-sandbox` exercising
   `subprocess.run(['git', ...])` from a sandboxed Python child.
   Until that test exists, any future SBPL change risks
   regressing this same shape.

## See also

- `docs/sandbox-validation.md` § "Known issues" → KI-1 (inline summary).
- `bench/bench.py:141` (the `phase_c_skip` flag and its reason string).
- `bench/runners/cuttle_sandbox_runner.py` (the runner that honors the flag).
