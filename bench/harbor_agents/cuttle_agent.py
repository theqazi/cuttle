"""CuttleAgent: Claude Code agent routed through cuttle's BYOK
credential vault + per-trial audit log.

This is Harbor Option 1 (Phase A semantics): the agent runs
inside Harbor's container envs (Linux), so cuttle-sandbox's
macOS-Seatbelt SBPL is *not* in this loop. What it does add over
Harbor's stock `claude-code`:

  - Credential from cuttle's macOS Keychain entry
    (`security find-generic-password -s dev.cuttle.api-keys
    -a ANTHROPIC_API_KEY`) when ANTHROPIC_API_KEY isn't already
    in env. Operator never has to `export ANTHROPIC_API_KEY=...`
    before invoking `harbor run`.
  - One JSONL audit line per trial start + trial end at
    `<logs_dir>/cuttle-audit.jsonl`. Lightweight; not tied into
    cuttle's HMAC-chained audit primitive yet (that would
    require the cuttle binary on PATH and a `cuttle audit
    append` subcommand we haven't shipped).

Run this agent via:

    harbor run \\
      -d "<dataset>" \\
      --agent-import-path bench.harbor_agents.cuttle_agent:CuttleAgent \\
      -m "anthropic/claude-haiku-4-5-20251001"

The `name()` returned is "cuttle" so leaderboards / job logs
attribute the trial to cuttle, not claude-code, even though the
underlying execution path is identical.

For Phase C semantics (cuttle-sandbox containment in the eval
loop), see Option 2 in docs/sandbox-validation.md: a custom
`CuttleEnvironment(BaseEnvironment)` that runs commands on the
macOS host through `cuttle sandbox run`. Not yet implemented.
"""

import json
import os
import subprocess
import time
from pathlib import Path

from harbor.agents.installed.claude_code import ClaudeCode
from harbor.environments.base import BaseEnvironment
from harbor.models.agent.context import AgentContext

KEYCHAIN_SERVICE = "dev.cuttle.api-keys"
KEYCHAIN_ACCOUNT = "ANTHROPIC_API_KEY"


def _resolve_keychain_credential() -> str | None:
    """Read ANTHROPIC_API_KEY from cuttle's macOS Keychain entry.

    Returns None on any failure: missing `security` binary (non-mac
    host), missing entry, locked Keychain. Caller falls back to the
    existing env-var path.
    """
    try:
        key = subprocess.check_output(
            [
                "security",
                "find-generic-password",
                "-s",
                KEYCHAIN_SERVICE,
                "-a",
                KEYCHAIN_ACCOUNT,
                "-w",
            ],
            stderr=subprocess.DEVNULL,
            text=True,
            timeout=5,
        ).strip()
        return key or None
    except (
        subprocess.CalledProcessError,
        subprocess.TimeoutExpired,
        FileNotFoundError,
    ):
        return None


class CuttleAgent(ClaudeCode):
    """ClaudeCode subclass with cuttle's BYOK + audit layer."""

    @staticmethod
    def name() -> str:
        return "cuttle"

    def version(self) -> str | None:
        # Track the cuttle-cli version this agent was developed against.
        # Bumped manually when the agent's behavior changes; the underlying
        # ClaudeCode version is reflected in trial config under
        # `agent.version` separately.
        return "0.0.1"

    async def setup(self, environment: BaseEnvironment) -> None:
        # Resolve ANTHROPIC_API_KEY from cuttle's Keychain entry if it
        # isn't already in env. This lets `harbor run` work without the
        # operator pre-exporting the key.
        if not (
            os.environ.get("ANTHROPIC_API_KEY")
            or os.environ.get("ANTHROPIC_AUTH_TOKEN")
            or os.environ.get("CLAUDE_CODE_OAUTH_TOKEN")
            or os.environ.get("AWS_BEARER_TOKEN_BEDROCK")
        ):
            key = _resolve_keychain_credential()
            if key:
                os.environ["ANTHROPIC_API_KEY"] = key
                self.logger.info("cuttle: resolved ANTHROPIC_API_KEY from Keychain")
            else:
                self.logger.warning(
                    "cuttle: no ANTHROPIC_API_KEY in env and Keychain "
                    "lookup failed; downstream auth will fail"
                )

        self._cuttle_audit("trial_setup", {})
        await super().setup(environment)

    async def run(
        self,
        instruction: str,
        environment: BaseEnvironment,
        context: AgentContext,
    ) -> None:
        self._cuttle_audit(
            "trial_start",
            {"instruction_len": len(instruction), "model": self.model_name},
        )
        try:
            await super().run(instruction, environment, context)
            self._cuttle_audit("trial_end", {"status": "ok"})
        except Exception as exc:
            self._cuttle_audit(
                "trial_end",
                {"status": "error", "exc_type": type(exc).__name__},
            )
            raise

    def _cuttle_audit(self, event: str, payload: dict) -> None:
        """Append a JSONL audit line to the trial's logs dir.

        Best-effort: any IO failure is silenced. The audit chain is
        not HMAC-signed in this MVP; promotion to cuttle's
        HMAC-chained audit primitive is tracked as a follow-up
        (would need `cuttle audit append <path> <event>` subcommand,
        not yet shipped).
        """
        try:
            line = {
                "ts": time.time(),
                "event": f"harbor.{event}",
                "agent": self.name(),
                "agent_version": self.version(),
                **payload,
            }
            audit_log = self.logs_dir / "cuttle-audit.jsonl"
            audit_log.parent.mkdir(parents=True, exist_ok=True)
            with open(audit_log, "a") as f:
                f.write(json.dumps(line) + "\n")
        except OSError:
            pass
