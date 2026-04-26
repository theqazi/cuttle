# PRD: Cuttle (working codename)

| Field          | Value                                                                                                                                                                                                                                     |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Status         | Draft v3 (post-pruning of v2 = v1.3 post-adversarial). Sealed-falsifier pre-registration at `docs/falsifiers.md`. Pending FIX-DOCS + system-design + api-design before implementation. Karpathy review (handoff path #2) parallel-stream. |
| Owner          | Mohammed Qazi (sirajuddin.qazi@gmail.com)                                                                                                                                                                                                 |
| Created        | 2026-04-25 (v0); revised 2026-04-26 (v1 → v1.1 → v1.2 → v1.3 = v2 → v3 by pruning)                                                                                                                                                        |
| Tier           | SYSTEM (per global CLAUDE.md). Pipeline: PRD → TDD → REVIEW-1 → REVIEW-2 → FIX-DOCS → DESIGN → API → LEGAL → PRIVACY → WRITE → COPY → REVIEW → SECURE → SBOM.                                                                             |
| Project memory | `/Users/m0qazi/.claude/projects/-Users-m0qazi-cuttle/memory/`                                                                                                                                                                             |
| Decision log   | `docs/DECISIONS.md` (D-2026-04-26-01 onward; session-1 headlines in `handoff.md`)                                                                                                                                                         |
| Public name    | Undecided. Working codename **Cuttle**. Public-name research in `codename_theme.md`.                                                                                                                                                      |

---

## 1. Summary

Cuttle is a security-first, terminal-native AI coding harness that connects to Anthropic's Claude API via the user's own API key (BYOK). It targets feature parity with Claude Code's current agent surface (tool use, subagent dispatch, skills, MCP, hooks, plan mode, memory) but is built on two co-equal bedrocks rather than one.

**Bedrock 1: Deterministic security.** Every tool invocation routes through a deny-by-default policy gate. No model is in the policy loop.

**Bedrock 2: Deterministic reliability.** The five layers of the agent-reliability framework (`framework_components.md:11-19`) ship as harness mechanics, not advisory skills. The operator-state snapshot, tier classifier, mandatory-skill loader, verification gates, and reward-relearning registry all run as runtime state the harness tracks, refusing progress when invariants are not met.

**Post-rectification framing** (`framework_development_methodology.md:22-27`). Cuttle's contribution is not a novel architecture; pre-execution gating is industrial precedent. Four independent industry voices converge on the substrate-vs-ceremony shape (full table at §15). Cuttle's contribution is the **substrate-native form of that converged principle for the LLM-agent case**, where the no-human-in-loop differentiator (D-2026-04-26-02) makes ceremony-based defense unviable at the per-tool-call grain.

v0.1 ships as **implementation existence proof** (`framework_development_methodology.md:33-37`). No effect claims. The framework runs as a substrate-native harness; gates fire as specified; the audit log captures evidence. Whether Cuttle reduces regression rate, improves code quality, or catches more bugs than the operator's current Claude Code setup is a Phase-1-equivalent open question, not a settled property.

Multi-provider (Gemini/Codex), GUI/mobile, and deployment are out of scope for Phase 1. Anthropic API key is the only auth mode in v0.1 (`auth_mode_decision.md:8-21`).

## 2. User

### 2.1 Persona A: experienced API-billed developer

Has an Anthropic Console API key. Uses Claude Code daily, has invested in customizing it (skills library, hooks, settings.json, MCP servers). Pays for tokens directly. The PRD owner is the canonical instance; v0.1 success is operator-defined per `phase1_scope.md:18-21`.

Verified setup baseline (2026-04-25 from `~/.claude/`):

- 56 skills, 21 hooks (including `bash-guard.sh`, `secret-scan.sh`, `leak-gate.sh`, `skill-gate.sh`, `pre-commit-verify.sh`, `post-edit-verify.sh`, `settings-hygiene.sh`, `vocabulary-scrub.sh`)
- `~/.claude/reward-loop/` (anti-patterns, validated-patterns, session-scores, dated archives)
- `settings.json` + `settings.local.json`, MCP auth caches, per-session security-warning state files

What this operator wants from Cuttle that Claude Code cannot provide (per `framework_components.md:96-109`):

- **Pre-execution gating, not post-execution accountability.** A `secret-scan` hook detects a leaked credential after the file write touches disk; Cuttle's policy gate refuses the write before it happens.
- **Five-layer reliability mechanics in the substrate, not as advisory skills.** Today L4 (verification) and L5 (reward-relearning) run as skills the model invokes when it remembers; Cuttle runs them as harness state.
- **Per-call authorization, not per-tool.** Claude Code decides "does this agent have access to Bash?" once. Cuttle decides per call, with context (directory, branch, staged files, argument shape).
- **Explicit subagent permission scoping at dispatch.** Claude Code subagents inherit broad parent permissions; Cuttle's policy-gate authorization context model supports per-agent scope from day one.

### 2.2 Persona B (security-conscious teams): DROPPED from v0.1

Per session-1 decision 2026-04-25 (handoff §"Decision log (session 1)" item 5). v0.1 is single-operator dogfood-driven. Multi-operator validation is a Phase-1-equivalent open question (D-2026-04-26-03). Cloud-provider auth modes (`bedrock`, `vertex`, `foundry`) remain in the sealed-slot enum (`auth_mode_decision.md:21-28`) for future Persona-B work but are not implemented in v0.1.

## 3. Problem

The owner's current Claude Code setup is sophisticated but exposes three structural problems Cuttle addresses by removing a substrate constraint, not by adding advisory hooks.

### 3.1 Hooks are advisory because the substrate is post-execution

Claude Code's hook surface runs alongside tool execution, not in front of it. The framework's `§10.2` audit-log default for escape-hatch accountability is the architecturally-preferred answer **only because the substrate has no pre-execution gate to bind to**. Cuttle exposes the pre-execution gate natively; the audit log remains as secondary accountability and as the reward-loop's data source. The substrate-constraint thesis: pre-execution gating is industrial precedent (D-2026-04-26-01); the framework was prevented from using it by Claude Code, not by choice.

### 3.2 Per-call blast radius is unbounded by intent

Per D-2026-04-26-02. Pre-execution gating matters for a single-operator harness even when post-execution is acceptable for Google's monorepo or Amazon's microrepo fleet, because the LLM agent's tool-use loop has **no human in the loop between model output and side-effect**. A single bash call can `rm -rf $HOME`; a single write can leak a credential. Per-call blast radius is unbounded by intent in a way that human-authored commits _with intervening human review_ (the dominant case in enterprise CI) reduce in expectation: the human can author the same destructive sequence, but the commit-to-execute cycle gives them the opportunity to notice. The LLM agent skips that cycle. Carlos's Amazon-vs-Google framing ("morally equivalent, contextually different") applies because the contexts genuinely differ.

### 3.3 Subagent permission inheritance is implicit and broad

Dispatched Agents inherit the parent's tool surface; no first-class mechanism to express "this subagent may read but not write" or "this subagent has no shell" at the dispatch site. v0.1 establishes per-subagent scoping at dispatch even though the `Agent` tool itself ships in v0.2; otherwise v0.2 becomes a rewrite.

### 3.4 What is NOT the problem (post-rectification framing)

Existing alternatives (Replit Agent, Cursor, Lovable) solve none of these and produce code the owner has characterized as low-quality. **They are not the comparison surface.** Cuttle's comparison surface is the operator's current Claude Code + 56-skill + 21-hook setup; replacing it with something measurably more secure and more mature is the success bar (`phase1_scope.md:20-21`). Comparing against Replit/Cursor/Lovable would introduce the framework-as-comparator failure mode (`feedback_review_as_lens_not_comparator.md`).

## 4. Why now

1. **April 4, 2026: third-party Pro/Max OAuth enforcement.** Anthropic clarified subscription OAuth tokens are first-party-only. The auth-mode landscape for third-party tools is now unambiguously bounded to API-key billing and cloud-provider routes (`auth_mode_decision.md:10-18`). Architectural decisions made before this clarification carried real risk; that risk is now eliminated.
2. **The framework reached v2.2 post-rectification.** Through 11+ rounds of adversarial defense (`framework_development_methodology.md:11-16`), the framework's contribution claims have been narrowed to a defensible scope. Building Cuttle on the rectified framework is lower-risk than building on a pre-review version.
3. **The owner's `~/.claude/` setup has matured to the point where its architectural seams are visible.** Twenty-one hooks doing what would more cleanly be policy-gate decisions is not an incremental config problem; the underlying primitive is wrong.

## 5. Success criteria for v0.1 ship

Each is binary, owner-verifiable. v0.1 does not ship until all six hold.

| ID   | Criterion                                                                                                                                                         | How verified                                                                                                                      |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| SC-1 | Owner uses Cuttle as the only agent harness for one full work-week (≥5 days, ≥7 work-account-billed sessions) without falling back to Claude Code.                | `cuttle telemetry` invocation count ≥ 80% of agentic sessions. **N=1 selection-biased per D-2026-04-26-03**, not adoption signal. |
| SC-2 | Cuttle blocks ≥1 tool invocation that the owner's current Claude Code setup would have allowed-then-warned. Captured in policy-gate audit log with rule citation. | Audit log inspection. Manual review confirms blocked invocations were genuinely undesired.                                        |
| SC-3 | All 21 user-hook responsibilities are accounted for (replaced by equivalent or stricter Cuttle rule, or explicitly documented as deferred).                       | Hook-mapping table in `docs/v01-hook-coverage.md`, owner-signed-off.                                                              |
| SC-4 | Cuttle reuses the existing `~/.claude/skills/*/SKILL.md` library unmodified. At least 40 of 56 installed skills work in Cuttle without code changes.              | Manual sweep: list, run, capture pass/fail.                                                                                       |
| SC-5 | Independent third-party security review (`code-review` + `threat-model` skill chain) finds zero CRITICAL findings on v0.1 implementation. HIGH triaged.           | `docs/security-findings-v01.md`, all CRITICAL closed, all HIGH dispositioned.                                                     |
| SC-6 | Sealed falsifiers in `docs/falsifiers.md` pre-registered before v0.1 ship. F-Cuttle-DISABLE plus ≥3 additional load-bearing predicates.                           | `docs/falsifiers.md` exists, signed-and-dated, predicates measurable from audit log.                                              |

SC-1 is the dogfood test (selection-biased per D-2026-04-26-03). SC-2 is the security-delta test. SC-3 is the maturity-coverage test. SC-4 is the dogfood-affordability test. SC-5 is the "don't mess it up" test. SC-6 is the framework-discipline test (`framework_development_methodology.md:39-43`).

## 6. Scope

### 6.1 v0.1: minimum daily-driver replacement

The smallest surface that satisfies §5.

#### 6.1.1 Substrate

- **CLI entry point** `cuttle` on macOS (darwin/arm64 + darwin/x86_64). Linux deferred to v0.2.
- **Anthropic API client**: streaming, multi-turn, tool-use loop. Pinned to Claude Sonnet 4.6 and Opus 4.7 as defaults. Prompt cache enabled.
- **Credential vault** (per D-2026-04-26-08, T-002, CC-2):
  - macOS Keychain backend (primary; default).
  - Encrypted-file fallback (opt-in; explicit operator confirmation per session).
  - `apiKeyHelper`-style indirection (opt-in only): shell-script credential resolution with TTL refresh on 401. Helper script path is content-hashed and recorded in `CredentialRecord` (extended `helper_hash` field); hash mismatch refuses invocation. Helper invocation runs under a sandbox profile denying network egress except to documented credential-store endpoints.
  - **In-memory zeroization invariant.** API key bytes zeroized on session end and uncaught panic. Constrains OQ-1 (Rust `zeroize` viable; Go via mlock + overwrite; TS not viable for this surface without native bindings).
  - **Keychain prompt-fatigue acknowledgment** (per BP-05): fail-closed gate restarts (CC-1) trigger Keychain prompts; operator-under-load may toggle "always allow" defeating per-process isolation. v0.1 acknowledges the cross-purpose; TDD specifies a rate-budget alternative (e.g., session-scoped Keychain handle).
- **Sandboxed core tool set**: `Bash`, `Read`, `Edit`, `Write`, `Glob`, `Grep`, `LS`. Bash sandboxed via a macOS process-isolation primitive scoped to the project directory, with resource limits on CPU, memory, file descriptors, child-process count. Primitive choice is OQ-3; `sandbox-exec` is the leading candidate but is on Apple's deprecation path (T-005), so TDD must produce a contingency.
- **Session persistence**: append-only JSONL transcripts under `~/.cuttle/projects/<project-key>/<session-id>.jsonl`, schema-compatible with Claude Code's transcript format where the schema does not introduce policy-gate bypass vectors.
- **Skills loader**: read `~/.claude/skills/*/SKILL.md` frontmatter, surface descriptions to the model with explicit untrusted-content framing. Skills run inside Cuttle's policy gate, not their own. **Strip-list maintenance contract** (per WV-05): the strip-set is allowlist-shaped, not denylist-shaped. Skills containing Unicode characters outside known-safe categories **fail to load** rather than load-with-stripping. Default-deny prevents Best-of-N-style novel injection vectors from exploiting strip-list lag.
- **Per-project + global memory** (per D-2026-04-26-08, T-007): same auto-memory contract as Claude Code. Memory writes go through the policy gate. **Cross-session memory loaded into a new session is presented to the model as untrusted-by-default until the operator explicitly promotes it.** Model-authored writes land in a quarantine area; promotion to canonical sidecar requires TTY operator confirmation. Operator-authored CLAUDE.md and the model-quarantine surface are structurally separate.
- **Audit log**: every tool call, policy decision, deny reason, attestation body provenance (TTY-input vs model-emit), gate-disable event, and tool result digest written to `~/.cuttle/audit/<yyyy-mm-dd>.jsonl` with a tamper-evident chain (scheme is OQ-4). **Tool-output digest taint annotation** (per WV-03): tools registered with a `secret_bearing` flag; for tagged tools (or unknown / default), only metadata (length, type, success/failure) recorded, NOT a content sha256. The audit log is anti-forgetfulness and anti-drift; it is **not** anti-Sybil against the operator-as-adversary in v0.1 single-operator scope (per T-003); it is **not** a cleartext side-channel for secret-bearing tool outputs.
- **Policy gate failure mode** (per CC-1): the policy-gate process is the load-bearing security boundary. **Gate-process death halts all tool dispatch until restart.** Cuttle fails closed on gate panic, gate crash, or gate-process disappearance. No fallback to "execute without gate."

#### 6.1.2 Five-layer harness mechanics (the bedrock 2 commitment)

Per `framework_components.md:79-89`. v0.1 ships partial integration; the **integration-vs-ablation tension** (`framework_methodology_document.md:51, 74-76`) is named and inherited unresolved.

| Layer | v0.1 mechanic                                                                                                                                                                                                                                                                                                                                                                                                                      | Deferred to v0.N                                            | Promotion trigger                                                               |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------- |
| L1    | Operator self-snapshot at session start. Refuses SYSTEM-tier classification without snapshot. State-to-mandatory-skill mapping (poor sleep blocks SYSTEM; foggy focus pre-loads debugging).                                                                                                                                                                                                                                        | Long-horizon snapshot decay; cross-session snapshot trends. | F-Cuttle-SNAPSHOT-DRIFT fires.                                                  |
| L2    | Tier classifier from runtime signals (file count, LOC, scope keywords). Model proposes; harness verifies. Per-tier required-step state machine refuses progress without artifacts.                                                                                                                                                                                                                                                 | Tier-override audit; cross-tier ceremony amortization.      | Tier-override count exceeds N per dogfood week.                                 |
| L3    | Mandatory skills auto-load by tier. Contextual skills load by trigger. Making/checking/persona modes are runtime states. Worklog auto-captured per tool call.                                                                                                                                                                                                                                                                      | Persona-overreach detection; persona-stickiness counter.    | Persona-mode mismatch detected against worklog evidence.                        |
| L4    | Verification commands run automatically post-task. CRITICAL/HIGH findings block "task complete" claim. Multi-persona verification dispatches the same output through N personas as parallel agents (N defaults to 2; operator-configurable; persona disagreement surfaces both reports).                                                                                                                                           | Cross-task verification dependencies.                       | Repeated downstream-task failure traceable to upstream verification gap.        |
| L5    | Cuttle queries AP/VP registry deterministically pre-task. Computes SCORE post-task. LEARN proposes registry entries on score-band thresholds; **proposed entries land in an operator review queue, not auto-promote**. Promotion requires TTY operator confirmation. Each mutation carries signed provenance (operator-owned key in v0.1 single-operator scope; chain is anti-forgetfulness/anti-drift, NOT anti-Sybil per WV-04). | Registry health auto-decay; cross-project propagation.      | Review-queue backlog exceeds N items, OR auto-decay-shaped patterns observable. |

#### 6.1.3 Option C deterministic harness review (high-blast-radius escape-hatch class)

Per `cuttle_v01_option_c_enumeration.md`. Each rule terminates in a machine-checkable predicate plus operator self-attestation:

1. **Secret-scan bypass on Write/Edit.** Evidence: operator-supplied substring + WHY (≥120 chars) + file path. Predicate: substring present in diff at path; WHY non-empty and length-bounded.
2. **Bash-guard bypass on destructive shell** (`rm -rf`, `git push --force`, schema migrations on shared infra). Evidence: explicit target enumeration + system-path allowlist check + WHY.
3. **Audit-log integrity bypass.** v0.1: no exception. Always denied.
4. **Credential-vault unlock outside Cuttle's normal flow.** Evidence: credential ID + explicit consumer declaration + N-minute Cuttle-access-void marker.

Residual operator-trust surface (`cuttle_v01_option_c_enumeration.md:29-31`) is acknowledged: even with machine-checkable predicates, some entries terminate in operator self-attestation. The attestation is logged; the audit catches drift; the reward-loop SCOREs the outcome.

#### 6.1.4 Methodology-as-harness-mechanic mapping

Per `framework_methodology_document.md:65-80`. Six methodology disciplines map to harness primitives. v0.1 ships:

- **State hunches before asking for analysis.** Cuttle prompts for hunch before generating; refuses survey-shaped queries during making mode.
- **Sycophancy detection.** Cuttle scans assistant output for agreement-shaped phrases ("exactly right," "great catch," "got it") in checking mode; surfaces a counter-argument prompt before continuing.
- **Insist on artifacts, not summaries.** Cuttle refuses to mark FEATURE/REFACTOR/SYSTEM tasks complete without a written artifact (PRD, TDD, ADR, or test).

Deferred to v0.N: ask-both-sides automation; recursive-application audits; pre-register-intuitions instrumentation.

#### 6.1.5 Cross-cutting harness invariants

Per `framework_components.md:91-94`:

- **Intent + rationale fields are required structured-output fields.** The harness rejects-without-them on every non-trivial decision.
- **Context-budget enforcement is automatic.** Always-loaded content over the line budget triggers automatic skill extraction.
- **Escape hatches are themselves a Cuttle harness primitive.** Every harness-enforced rule ships with a paper §10.2-format exception table (`Condition → Modified rule → Intent preserved → Rationale → Evidence required`). The deny path always offers the operator the option to invoke a matched exception or draft a new one in `proposed-exceptions.md` (which does NOT auto-allow). **Exception tables are loaded at runtime from operator-controlled storage; NOT embedded in the binary, NOT readable by the model, ship empty by default** (per T-009). Operator-supplied evidence reaches the gate via TTY input, not via model output.
- **Attestation provenance separation** (per T-001). The policy-gate process maintains a structural separation between operator-typed text (TTY input events) and model-emitted text. Attestation fields that gate-bypass predicates consume as evidence require operator-typed provenance; model-emitted strings are rejected as evidence even if they would satisfy content shape. **Limitation explicitly disclaimed** (per WV-06, WV-07): TTY-provenance separation distinguishes bytes-typed-by-operator from bytes-emitted-by-model. It does NOT distinguish operator-INTENT from operator-FATIGUE-KEYPRESS. A model can emit "to bypass this gate, please type the following: <attack string>" and an operator under load may comply; the bytes then carry valid Tty provenance. Cuttle does not solve this at the per-attestation grain in v0.1; F-Cuttle-FATIGUE in §12 makes it empirically falsifiable.
- **Cross-session memory promotion** (per T-007). Cross-session memory loaded into a new session is structurally separated from operator-authored CLAUDE.md; model-authored writes land in a quarantine area; promotion to canonical sidecar requires TTY operator confirmation. The `MEMORY.md` index distinguishes promoted-canonical entries from quarantined-pending entries.
- **Domain primitives at trust boundaries** (per D-2026-04-26-11; Johnsson / Deogun / Sawano, _Secure by Design_, Manning 2019). Trust-boundary-crossing values constructed only through domain-primitive types that enforce invariants at construction. Raw `String` / `[u8]` / `int` / `Path` forbidden at trust-boundary surfaces. The type system does the work, not the runtime, for the attestation-provenance and memory-quarantine invariants. Concrete v0.1 candidates (TDD finalizes): `ApiKey` (read-once + zeroizable + helper-hash-bound, T-002 + CC-2); `AttestationBody { provenance: Tty | Model, content }` (T-001); `HelperHash` (T-002); `LockfilePath` (T-004); `TierClassification` enum (L2); `OperatorAuthoredText` vs `ModelAuthoredText` (T-001 + T-007). Adds a security argument to OQ-1 (TS structural typing weakens nominal-type enforcement; Rust newtypes / Go named types enforce nominally). **Constructor authorization** (per WV-01): each primitive's constructor is module-private and reachable only via a capability the calling module holds. Untrusted-or-low-trust modules pass raw bytes through validating boundary functions that own the capability.

#### 6.1.6 Local telemetry surface (per D-2026-04-26-04)

`cuttle telemetry` surfaces aggregate signal to the operator: gate-fire rates, override attempts, abandon points by tool/policy/session. **No data leaves the machine; no Cuttle-controlled server.** Addresses Carlos's "telemetry-dark tools cannot improve" failure mode without compromising the no-phoning-home commitment in §7.

Storage and ACL (per T-008): telemetry aggregates inherit audit-log file permissions (operator-only, no group/world read). Aggregate counts of which gates fired on which projects reveal sensitive workflow patterns; the aggregate store is treated as confidential by default. The `privacy` skill (per §11) reviews this surface before v0.1 ship.

### 6.2 v0.2: completes the agent surface

Ships after v0.1 dogfood passes SC-1 for two consecutive weeks.

- Subagent dispatch (`Agent` tool) with per-subagent policy scoping
- Plan mode + slash commands
- Remaining 19 user hooks ported into the policy-gate as built-in policies
- MCP server support, scoped to a deny-by-default network policy
- Reward-loop registry integration extended to cross-session aggregation
- Linux credential-vault backend (libsecret / gnome-keyring + encrypted-file fallback)
- v0.N column items above (L1 snapshot decay, L3 persona-overreach, L4 cross-task dependencies, L5 registry auto-decay)

### 6.3 v0.N: closes Phase 1

- TodoWrite/Task system, scheduled tasks
- Worktree integration
- Windows credential-vault backend
- Cloud-provider auth modes (only when first Persona-B-equivalent user is committed; do not pre-build)
- IDE integrations (only if blocking dogfood; otherwise punt to Phase 2)

### 6.4 Phase 2

Deployment. Out of scope for this PRD. Multi-provider (Gemini/Codex) lives between Phase 1 and Phase 2; ordering deferred.

## 7. Non-goals

- **Not a Claude Code fork.** From-scratch implementation. No Anthropic source incorporated. Schema compatibility on transcripts and skills is for migration ergonomics only.
- **Not a Claude.ai client.** No `claude.ai` authentication, no session cookies, no Claude Code OAuth tokens. ToS-clean by construction (`auth_mode_decision.md:12-18`).
- **No multi-provider in Phase 1.** Provider-abstraction shape welcome where free; no second concrete implementation.
- **No GUI in Phase 1.** No Electron, web frontend, mobile, iPad. CLI-only.
- **No Cuttle-side LLM billing.** Users pay Anthropic directly.
- **No remote telemetry phoning home in v0.1.** No analytics, crash reports, or usage metrics to a Cuttle-controlled server. Local-only telemetry (§6.1.6) permitted; remote forbidden. Future remote telemetry ships opt-in only and gets `privacy` skill review.
- **No marketing claim of "Claude Max compatibility,"** until Anthropic publishes a sanctioned third-party Max mechanism.
- **No removal of `ANTHROPIC_API_KEY` from environment.** Cuttle reads it as one credential source among several.

### 7.1 Non-goals from the framework methodology

- **Cuttle does not give users discipline they don't have.** Per `framework_methodology_document.md:56-60`. Discipline amplifier, not discipline supplier.
- **No effect claims at v0.1 ship.** Per `framework_development_methodology.md:33-37`. Implementation existence proof. Marketing claims forbidden until Phase-1-equivalent validation.
- **v0.1 does not optimize for first-week adoption ergonomics.** Per D-2026-04-26-03. N=1 selection-biased dogfood proof. Per Carlos, "tools die silently despite technical merit"; Cuttle accepts that risk explicitly.

## 8. Edge cases (non-obvious)

These shape the design and must be answered in the TDD.

1. **Skill in `~/.claude/skills/` contains prompt-injection in SKILL.md.** Skill descriptions presented with explicit untrusted-content framing; Unicode-attack stripping at load (per §6.1.1); the policy gate is not steerable by model output. **Critical refinement** (per T-001): policy lives below the model AND attestation bodies that any gate-bypass predicate consumes as evidence require operator-typed provenance via TTY input. The TTY-input-vs-model-emit separation (per §6.1.5) is the load-bearing primitive.

2. **Subagent calls back into a tool the parent had elevated permission for.** v0.1 establishes explicit subagent permission scoping at dispatch time even though `Agent` doesn't ship until v0.2.

3. **MCP server's OAuth flow demands a browser open during a non-interactive CI run.** Cuttle in CI mode (no TTY, no DISPLAY) fails closed. The policy-gate `Prompt` decision collapses to `Deny` in non-interactive mode.

4. **API key rotated mid-session.** Long-running tool-use loop receives a 401 partway through. Credential vault's `apiKeyHelper`-style refresh is invoked, in-flight request retried once, audit log captures the rotation.

5. **`~/.claude/settings.json` `permissions.allow` rules conflict with Cuttle's built-in policies.** v0.1 default: user permissions can only narrow, never broaden. Built-in security policies are a floor. Deliberate departure from Claude Code; called out in onboarding.

6. **Operator runs Cuttle inside a Cuttle subagent (nested harness).** Inner instance refuses to start if it detects another Cuttle instance, unless explicit `CUTTLE_NESTED=allow`. Detection uses out-of-band signals (lockfile in `~/.cuttle/run/`, process-tree walk via `proc_pidpath`-equivalent), NOT environment variables alone (per T-004). Even with the override, if the outer session's policy state cannot be inherited (e.g., lockfile present but unreadable), the inner instance fails closed.

7. **Operator self-snapshot reports poor sleep + high stress.** v0.1 L1 mechanic refuses SYSTEM-tier classification automatically. Operator override `--override-snapshot-block` is logged and counts as evidence for F-Cuttle-DISABLE if ≥N times.

8. **A harness-enforced gate is wrong.** Carlos's "configurable risk dial" lens (D-2026-04-26-05): if a gate fires on legitimate work and there is no graduated bypass, the operator will disable it. The operator must have a way to file a `proposed-exceptions.md` entry without disabling the gate; the deny path surfaces this option as part of the structured deny message.

9. **Operator restores `~/.cuttle/` from a snapshot or backup** (per BP-04). Multiple subsystems implicitly assume `~/.cuttle/` state is current: audit-log chain head, lockfile, registry signed-provenance chain, telemetry aggregates, credential-vault `helper_hash`. A restore from old snapshot resurrects stale state silently inconsistent with the current world. v0.1 default: on session start, Cuttle reads a `~/.cuttle/state-coherence.json` written at last clean shutdown; if the file is missing OR mtime / chain-head digest does not match the audit-log head, Cuttle refuses to start without explicit `--restored-from-backup` operator acknowledgment that resets the chain heads (logged as a high-trust event, counts as evidence for F-Cuttle-DISABLE).

## 9. Constraints

- **ToS:** all auth paths comply with Anthropic Consumer Terms (effective 2025-10-08) and the Feb/April 2026 third-party OAuth clarification (`auth_mode_decision.md:10-18`). Hard constraint, not a guideline.
- **Platform:** macOS-first for v0.1. Linux follows in v0.2; Windows in v0.N.
- **License:** TBD. Default candidate: Apache-2.0 with explicit "no Anthropic source incorporation" attestation in NOTICE. Final decision happens at first public-release milestone; held open during v0.1 to keep relicensing cheap. `legal-review` skill engages at that point.
- **Language:** TBD in TDD §1 (OQ-1). Candidates: Rust, TypeScript, Go. v0.1 has security constraints (CC-2 zeroization, §6.1.5 nominal-type domain primitives) that lean Rust > Go > TS.
- **Predicate maintenance cost.** Pre-execution gating shifts cost from post-hoc audit to predicate engineering. Per Carlos's $100M/yr Google pre-submit anchor: substrate-constraint removal does not eliminate cost; it relocates it. Cuttle's predicates must be authored and maintained as the Anthropic API surface evolves.
- **Repository:** `/Users/m0qazi/cuttle`. Public hosting decision deferred to first-release milestone.

## 10. Open questions (resolved in TDD or before)

| ID    | Question                                                                                                                                                                                       | Resolves in                     |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------- |
| OQ-1  | Implementation language (Rust / TS / Go). Drives every other technical decision.                                                                                                               | TDD §1                          |
| OQ-2  | Policy-gate language: declarative DSL or imperative plug-ins. Affects how `~/.claude/hooks/` migrate.                                                                                          | TDD §3                          |
| OQ-3  | macOS sandbox primitive: `sandbox-exec` (deprecated) or hybrid over `posix_spawn` + sandbox-exec.                                                                                              | TDD §4                          |
| OQ-4  | Audit-log tamper-evidence: HMAC chain (simpler) or Merkle tree with periodic root publication (stronger).                                                                                      | TDD §5                          |
| OQ-5  | Skill-trust model: all `~/.claude/skills/*` get the same policy-gate treatment, or trusted-skills allowlist. Edge case 1 forces this.                                                          | threat-model                    |
| OQ-6  | Canonical config file location: `~/.cuttle/config.toml`, XDG, or owner-pickable.                                                                                                               | TDD §2                          |
| OQ-7  | Public name. Not blocking until first public release.                                                                                                                                          | Phase 2 prep                    |
| OQ-8  | Telemetry posture: opt-in remote diagnostics or zero remote telemetry ever. (Local-only is decided.)                                                                                           | privacy review                  |
| OQ-9  | Allow/Warn/Deny graduation vs binary Allow/Deny/Prompt for the policy-gate API. Per D-2026-04-26-05.                                                                                           | TDD §3                          |
| OQ-10 | Integration-vs-ablation: does partial-integration v0.1 deliver the framework's value, or does the value only emerge at full integration. Empirically open.                                     | Phase-1-equivalent validation   |
| OQ-11 | Process-isolation model (per BP-01). Should the policy gate run as a separate OS process supervising the model client over typed IPC. Trade: cleaner OS-level trust boundary vs added latency. | TDD §3 (revisit at v0.2 latest) |
| OQ-12 | Audit-log PII posture (per BP-06). Record-as-is vs redact-at-write vs refuse-tools-that-may-emit-PII.                                                                                          | TDD §5 + privacy review         |

## 11. Deliverables and pipeline

Per global CLAUDE.md SYSTEM-tier workflow, this PRD is step 0. Subsequent deliverables, in order:

1. **TDD** at `docs/TDD.md`: resolves OQ-1 through OQ-6, OQ-9, OQ-11, OQ-12.
2. **REVIEW-1**: `code-review` skill on PRD + TDD. Deliverable: `docs/review-1-findings.md`.
3. **REVIEW-2**: `legal-review` (ToS / AUP / license), `threat-model` (capability surface, sandbox, audit-log integrity), `privacy` (telemetry §6.1.6, audit-log content digests, cross-session memory §6.1.5), in parallel. Deliverable: `docs/review-2-findings.md`.
4. **FIX-DOCS**: incorporate review findings into PRD + TDD.
5. **`system-design`** skill on the resolved TDD. Deliverable: architecture diagram + component-boundary doc.
6. **`api-design`** skill on the CLI surface and credential vault interface.
7. Implementation begins.

PRD versioning convention as actually exercised in session 4: `v1` is the first published draft. `v1.x` are micro-revisions incorporating discrete review passes (v1.1 = threat-model + output-quality; v1.2 = Fowler / Secure-by-Design; v1.3 = Wolverine + Black Panther adversarial). `v2` is the post-adversarial collapsed label (= v1.3 in this revision history). `v3` is post-pruning (this version). `Accepted` only after FIX-DOCS, which happens after TDD review surfaces incremental findings against PRD v3.

## 12. Sealed falsifier pre-registration

Per `framework_development_methodology.md:39-48`. The framework's discipline requires sealed falsifiers before v0.1 ship. Each load-bearing claim has an associated operationalized predicate that, if observed, forces public retraction.

Each predicate is also a Ford / Parsons / Kua _architectural fitness function_ (per D-2026-04-26-10): "an objective integrity assessment of an architectural characteristic," evaluated continuously through the audit-log and telemetry surfaces. The two names point at the same primitive; "sealed falsifier" preserves the framework-methodology lineage, "fitness function" names the industry-standard concept. **v0.1 ships predicates as data only** (per BP-02): the audit log captures events; the AUTOMATED EVALUATOR is TDD-grade scope. At v0.1 ship, predicates are operator-evaluable post-week, not auto-evaluated by the build.

Sealed in `docs/falsifiers.md`; the seal triggers immutability at v0.1 ship (typo fixes only after seal, structural changes only before ship).

- **F-Cuttle-DISABLE** (per D-2026-04-26-06, expanded D-2026-04-26-09): operator disables ≥1 harness-mechanic gate during v0.1 dogfood week, OR re-keys the audit-log tamper-evident chain (per T-003), OR invokes `--restored-from-backup` resetting chain heads (per §8 case 9), OR invokes `--override-snapshot-block` repeatedly. Bedrock thesis ("harness mechanics > advisory skills") partially refuted: gates surviving as advisory in practice. Measured via audit-log gate-disable, chain-rotation, and high-trust-event entries.
- **F-Cuttle-BEDROCK**: operator skip rate on Cuttle's harness mechanics is statistically indistinguishable from skip rate on toolkit's skills/hooks across N≥20 SYSTEM-tier sessions. Bedrock thesis rejected.
- **F-Cuttle-SUBSTRATE**: deterministic-pre-execution-gate produces operator workflow disruption rates ≥X×toolkit's audit-log-async rates (measured by abandon-point telemetry from §6.1.6). Substrate-constraint claim incomplete: the gate IS the bottleneck, not the substrate.
- **F-Cuttle-OPTION-C**: attestation-terminating predicates produce bypass rates indistinguishable from §10.2 default across the v0.1 enumerated set. Option C reduces to §10.2 default; the synchronous-deny framing is theatre.
- **F-Cuttle-SNAPSHOT-DRIFT** (per T-006): `--override-snapshot-block` invoked > N times in v0.1 dogfood week AND post-hoc operator review concludes the override was wrong > M of those times. L1 mechanic delivering theatre, not protection. (N, M set in TDD §3.)
- **F-Cuttle-MEMORY-DRIFT** (per T-007): cross-session model-authored memory promotions exceed deny+quarantine retention rate by ratio R during v0.1 dogfood week. Cross-session memory promotion mechanic functioning as auto-promote in practice; untrusted-by-default invariant refuted. (R set in TDD §3.)
- **F-Cuttle-FATIGUE** (per WV-06 / WV-07): attestation bodies operator-typed during v0.1 dogfood week contain substring matches against model-emitted text in the same conversation turn at rate ≥ R_F. Attestation-provenance separation invariant defeated by operator-fatigue-keypress; Option C / escape-hatch evidence pathways collapse to model-driven bypass at the operator-fatigue grain. (R_F set in TDD §3; probable starting point ≥30% of attestation tokens trigram-matchable to immediately-prior model output. Detection requires per-attestation logging of model context window at the moment of TTY input, itself a privacy-sensitive surface OQ-12 must address.)

## 13. Content-vs-guidance distinction when lifting from the toolkit

Per `framework_components.md:113-122`. The framework was scoped to the Claude Code substrate. **Content** (APs, VPs, scoring formulas, FV scaffolding, evidence-gating rules) is substrate-independent and lifts cleanly into Cuttle. **Substrate-coupled guidance** (hook shapes, `~/.claude/skills/SKILL.md` format, `CLAUDE.md` instruction-loading, hook-script contracts) is shaped by Claude Code's primitives and must be reinterpreted for Cuttle's primitives, not copied.

- Toolkit's `bash-guard.sh` → Cuttle's bash-tool policy-gate predicate (same logic, different shape).
- Toolkit's `SKILL.md` frontmatter → Cuttle's skill-manifest format (decide whether Cuttle reads SKILL.md unchanged or transforms at install).
- Toolkit's `CLAUDE.md` always-loaded instructions → Cuttle's harness-mandatory rule set + persona injection (always-loaded surface in Cuttle is structured runtime state, not freeform markdown).

Implication for v0.1: skills, hooks, MCP, slash commands are decoration on top of the bedrock. The load-bearing security and reliability lives in the bedrock; surface compatibility with `~/.claude/skills/` (SC-4) is for migration ergonomics only.

## 14. Dual use of the framework (methodology + content)

Per `framework_components.md:123-130`. The framework is **both** the methodology Cuttle is being _developed under_ AND the content of Cuttle's bedrock:

- **Methodology**: Cuttle's own development is governed by the framework's discipline. This PRD is itself an instance.
- **Content**: Cuttle's runtime IS the framework's five layers as harness mechanics, substrate-native rather than substrate-grafted.

Both must hold; they are not redundant. Failure on either side compromises the whole.

## 15. External corroboration

Per D-2026-04-26-12. Cuttle's bedrock thesis (substrate-native security + reliability beats advisory ceremony) is the LLM-agent-substrate form of an industry-converged principle. Four independent voices land on the same architectural shape from different domains:

| Source                                                                            | Domain                            | Convergent claim                                                                                                                      | Cuttle cross-ref |
| --------------------------------------------------------------------------------- | --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| Carlos Arguelles (Senior Principal, Amazon, ex-Google)                            | CI/CD blast radius                | Pre-submit verification at $100M/yr is justified by per-commit blast radius. Post-submit acceptable only where humans author commits. | D-01, D-02       |
| Mohan & Gumbley (Thoughtworks, hosted on martinfowler.com 2025)                   | Threat-modeling practice          | "Integrate threat modeling regularly, like continuous integration for security." Bite-sized, embedded, beats one-off workshop.        | D-12             |
| Ford / Parsons / Kua (_Building Evolutionary Architectures_, foreword by Fowler)  | Architecture governance           | Architectural fitness functions defended continuously through tests/metrics in the build, not at one-off review.                      | D-10             |
| Johnsson / Deogun / Sawano (_Secure by Design_, Manning 2019, foreword by Fowler) | Application-level security design | Domain primitives enforce invariants at construction; trust-boundary surfaces use type-system primitives, not raw types.              | D-11             |

Cuttle's contribution narrows from "novel architecture" to "novel substrate-native form of an industry-converged principle for the LLM-agent case where blast radius is per-call and the no-human-in-loop differentiator (D-02) makes ceremony-based defense unviable." Honest framing per `framework_development_methodology.md:18-30`.

**What the convergence does NOT establish** (per BP-03 / BP-07). The four voices converge in their respective domains; none was addressing single-operator AI-agent harnesses. Cuttle takes the substrate-vs-ceremony principle to a fifth domain where it has not been tested at industry scale. The §15 table is convergent-precedent, not direct-validation. Dissenting precedents Cuttle should remain honest about: capability-discipline traditions (Lampson) argue for trust-the-user with structured permissions rather than deny-by-default at the per-call grain; operator-fatigue-inevitability arguments (Mickens, Geer) caution that any threat-model-as-substrate eventually hits the same operator-fatigue wall the ceremony-based versions do (operationally captured by F-Cuttle-FATIGUE).

Source artifacts: `process/carlos-arguelles-input.md`, `process/martin-fowler-input.md`. Framework-side sidecar `framework_external_corroboration.md` (handoff path #3) is the live cross-project record; this §15 is the PRD-grain snapshot.
