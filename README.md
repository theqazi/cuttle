# Cuttle

A security-first BYOK harness for Anthropic Claude. Single-operator,
CLI-only, macOS-first. v0.1 dogfood.

Cuttle's bedrock: deterministic security AND deterministic reliability
as harness mechanics, not as advisory skills the model can skip under
load. Every tool dispatch passes through a fail-closed gate. Every
conversation turn lands in an HMAC-chained audit log whose integrity
the operator can verify offline.

## What works today (v0.1)

| Command                                                          | What it does                                                               |
| ---------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `cuttle session start`                                           | Multi-turn streaming Claude REPL with per-session audit chain + transcript |
| `cuttle ask "<prompt>"`                                          | Single-turn streaming Claude call (no audit chain; for one-shot use)       |
| `cuttle audit verify --audit-log <P> --chain-key-file <K>`       | Verify HMAC chain integrity of a session's audit log                       |
| `cuttle telemetry [--json] [--falsifier-eval] [--audit-log <P>]` | Aggregate signal + (optional) sealed-falsifier evaluation                  |

Tool dispatch (Bash, file ops) is **deferred** in v0.0.14 and lands in
v0.0.15+. The model can describe what it would do but cannot execute
yet. The sandbox primitive (macOS Seatbelt + SBPL) is in place;
wiring it through to the model loop is the remaining piece.

## Status

Pre-release. Working dogfood for the audit + conversation surfaces.
Not packaged, not signed, not on Homebrew. The sole supported install
path right now is `cargo build --release` from this repository.

## Install

Requires Rust 1.95+ (`brew install rust` or [rustup.rs](https://rustup.rs/))
and macOS 14+. Linux mostly works but the sandbox primitive is
macOS-only.

```bash
git clone <this-repo> cuttle && cd cuttle
./install.sh                       # builds + installs to /usr/local/bin/cuttle
# or pick a different location:
INSTALL_DIR=~/.local/bin ./install.sh
```

The install script is a thin wrapper around `cargo build --release`
plus `cp`. Read it before running it. No sudo unless `INSTALL_DIR` is
root-owned.

## Quickstart

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
cuttle ask "say hi in three words"
# claude> Hello, hi there!
```

For an interactive session with full audit + transcript:

```bash
cuttle session start
# cuttle session: 2026-04-26T15-30-45Z-a1b2c3d4
#   audit log:  /Users/you/.cuttle/sessions/<id>/audit.jsonl
#   chain key:  /Users/you/.cuttle/sessions/<id>/chain.key
#   transcript: /Users/you/.cuttle/sessions/<id>/transcript.jsonl
# type /quit or Ctrl+D to exit. enter your prompt below.
#
# you> what's 2+2?
# claude> 2 + 2 = 4.
# you> /quit
```

After the session, verify the chain integrity and inspect aggregate signal:

```bash
SESSION=~/.cuttle/sessions/2026-04-26T15-30-45Z-a1b2c3d4

cuttle audit verify --audit-log $SESSION/audit.jsonl \
                    --chain-key-file $SESSION/chain.key
# audit log verified.
# chain head: a3f0...

cuttle telemetry --audit-log $SESSION/audit.jsonl --falsifier-eval
# (full report with token spend, falsifier predicate evaluations, etc.)
```

## File layout

| Path                                       | Purpose                                                           |
| ------------------------------------------ | ----------------------------------------------------------------- |
| `~/.cuttle/sessions/<id>/audit.jsonl`      | HMAC-chained audit log (provenance, no content)                   |
| `~/.cuttle/sessions/<id>/chain.key`        | 32-byte session chain key, mode 0600                              |
| `~/.cuttle/sessions/<id>/transcript.jsonl` | Full conversation text, mode 0600                                 |
| `~/.cuttle/audit.jsonl`                    | Default audit log path for `cuttle telemetry` without --audit-log |

Override the parent directory via `CUTTLE_HOME=/some/path`.

## The audit-log design

The audit log is a chain of digests, not a chain of content. Each turn
appends a `UserPrompt` or `AssistantResponse` event carrying the SHA-256
of the content plus its length plus (for assistant turns) token usage.
The actual conversation text lives in `transcript.jsonl` in the same
session directory.

This separation lets you share a session's audit log + chain key for an
external integrity check without exposing what you actually talked
about. The chain proves "these N turns of these lengths happened in
this order with these token costs." The transcript proves what the
content was, and stays behind 0600 perms.

## Configuration knobs

Environment variables Cuttle reads:

- `ANTHROPIC_API_KEY` (required for `ask` + `session start`). Override
  the variable name with `--api-key-env <OTHER_VAR>`.
- `CUTTLE_HOME` (optional). Defaults to `~/.cuttle`. Used by every
  command that needs a session directory or default audit log path.

Per-command flags: `cuttle <subcommand> --help` (or `cuttle --help` for
the full surface).

## Security posture

- Anthropic API key is read once per call and zeroized on drop. Never
  written to disk by Cuttle, never logged, never echoed.
- All HTTPS traffic is rustls (no OpenSSL).
- Tool dispatch (when it lands) routes through macOS Seatbelt with a
  per-project SBPL profile that fails closed if the sandbox can't be
  applied.
- Audit log uses HMAC-SHA-256 chain. Tampering with any entry breaks
  the chain at that point and `cuttle audit verify` will surface
  exactly which entry sequence failed.
- Em-dash discipline + Unicode allowlist on auto-memory + capability-
  token witness on memory promotion (per the framework's L1-L5 layers).

For the full threat model, see `docs/PRD.md` §6 + `docs/TDD.md` §5.

## Limitations and known gaps (v0.1)

- macOS-only for the sandbox primitive. Linux works for `ask` /
  `session start` / `telemetry` / `audit verify` but Bash dispatch
  needs a Landlock equivalent (v0.2 scope).
- No tool dispatch in v0.0.14. The REPL is conversational only.
- No conversation resume across `cuttle` invocations. Each
  `cuttle session start` is a fresh session; the audit log + transcript
  are the durable record.
- No Keychain integration yet. `ANTHROPIC_API_KEY` must be in your
  environment. Keychain backend is v0.2 scope.
- TLS pinning is not implemented. Cuttle trusts the operator's CA store
  for `api.anthropic.com`. Documented limitation; v0.2 scope.

## Build + test

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --all-targets -- -D warnings
```

The release binary is at `target/release/cuttle` after
`cargo build --release`.

## License

Apache 2.0. See workspace `Cargo.toml`.

## Contributing

Pre-release; not yet open for external contributions. The repository
documents its own design via `docs/PRD.md`, `docs/TDD.md`, and
`docs/DECISIONS.md`. Read those before proposing changes.
