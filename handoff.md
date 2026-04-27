# Cuttle: Handoff

**Purpose**: top-level current-state document for Cuttle. A new session opens with
this file plus the project memory at
`/Users/m0qazi/.claude/projects/-Users-m0qazi-cuttle/memory/`.

**Version**: handoff-0.15 (session 6: cuttle session start REPL works end-to-end; 248/248 tests; per-session HMAC-chained audit + 0600 transcript layout; provenance-without-content separation working as designed; 2026-04-26)
**Tier**: SYSTEM (per global CLAUDE.md). Full pipeline: PRD → TDD → REVIEW-1 → REVIEW-2 → FIX-DOCS → DESIGN → API → LEGAL → PRIVACY → WRITE → COPY → REVIEW → SECURE → SBOM.

---

## What Cuttle is, in one paragraph

A from-scratch security-first terminal AI coding harness embodying the five-layer
agent-reliability framework documented in `~/claude-study/papers/paper-agent-framework.md`.
Cuttle's bedrock thesis: deterministic security AND deterministic reliability as
harness mechanics, _not_ as advisory skills the model can skip under load. Cuttle's
sharpened pitch: the framework, finally able to enforce _in front of_ execution
instead of _behind_ it, because the substrate is no longer the bottleneck. v0.1 is
single-operator, CLI-only, Anthropic-API-key-only (ToS-clean), and ships as an
implementation existence proof, not an effect claim.

## Session 6 update (handoff-0.15)

**Headline**: `cuttle session start` lands. Multi-turn streaming REPL with
per-session audit chain and transcript. The full daily-driver loop works
end-to-end with `ANTHROPIC_API_KEY` exported. 248/248 tests; clippy clean.

Mo's session-6 design decisions (per the open-questions block in
handoff-0.14) baked into code:

- **Chain key**: fresh per session, written to
  `~/.cuttle/sessions/<id>/chain.key` mode 0600 + create_new (refuses to
  clobber). Operator points `cuttle audit verify --chain-key-file <PATH>`
  at this file post-session.
- **Multi-turn audit-event shape**: new `UserPrompt { content_sha256,
length }` and `AssistantResponse { content_sha256, length, input_tokens,
output_tokens }` variants on `AuditEvent`. Content lives in transcript;
  audit log carries digest + length only. **Provenance without content** —
  chain proves "these N turns of these lengths happened in this order
  with these token costs" without holding PII.
- **Tool dispatch**: deferred. v0.0.14 ships `--tools=none` shape (no
  tool dispatch at all). Bash + read + write + dispatch infrastructure
  is v0.0.15+.
- **Transcript persistence**: `~/.cuttle/sessions/<id>/transcript.jsonl`
  mode 0600. JSONL `{role, content, timestamp_utc}` per turn. No
  auto-resume; the audit log + transcript are the durable record.

Commits added in session 6:

- `3aa311b` cuttle-audit v0.0.4 + cuttle-telemetry v0.0.11: new event
  variants + SessionSummary aggregate + 5-tuple summarize() return.
- `cd0abe8` cuttle-cli v0.0.14: cuttle session start REPL implementation
  (~465 LOC in session_cmd.rs, including 14 unit tests).

After a session, the full audit cycle works:
`cuttle session start` → multi-turn conversation → exit
`cuttle audit verify --audit-log ~/.cuttle/sessions/<id>/audit.jsonl
                        --chain-key-file ~/.cuttle/sessions/<id>/chain.key`
→ "audit log verified. chain head: <hex>"
`cuttle telemetry --audit-log ~/.cuttle/sessions/<id>/audit.jsonl
                    --falsifier-eval` → aggregate + falsifier eval

## Mid-session 5 update (handoff-0.12)

**Headline**: streaming + first daily-driver-shape command works.
`cuttle ask "anything"` with `ANTHROPIC_API_KEY` exported now produces a
streaming Claude response. 204/204 tests pass; clippy clean.

Commits added since handoff-0.11:

- `b5e51c5` cuttle-anthropic v0.0.9: SSE streaming via eventsource-stream;
  StreamEvent enum (8 variants + Unknown forward-compat); messages_stream()
  with first-byte-aware retry safety; PartialStream error variant.
- `ad223e4` cuttle-credential v0.0.2: ApiKey::from_env_var with footgun
  rejection (NotSet / Empty / NonUtf8 / SurroundingWhitespace).
- `5e6c440` cuttle-cli v0.0.12: cuttle ask command. Hand-rolled argv parser
  extended for the new subcommand; --model / --max-tokens / --api-key-env /
  --stdin flags; positional prompt with multi-token concatenation; tokio
  runtime built on demand; streaming output flushed after each delta so
  the operator sees real-time text.
- `82c6941` cuttle-anthropic v0.0.10: prompt cache support. Request.system
  is now Option<SystemContent>, an untagged serde union of Plain(String)
  and Blocks(Vec<SystemBlock>). SystemBlock carries an optional
  CacheControl::ephemeral() that maps to Anthropic's 5-min ephemeral cache.
  Daily-driver impact: ~10x cost reduction on repeated long-prefix calls.
  Backward-compat preserved via From<String> + From<&str> impls.
- `63dce86` cuttle-cli v0.0.13: cuttle audit verify subcommand. Hand-rolled
  hex parser + raw-bytes-or-hex auto-detect on the keyfile (~30 LOC, no
  hex-crate dep). 2-level argv dispatch. Smoke verified: bad-key path
  produces "is 9 bytes; expected exactly 32 raw bytes or 64 hex characters".
  HMAC mismatch surfaces "verification failed at entry seq=N".

## Open design questions for `cuttle session start` (next big move)

The interactive REPL is the highest-value remaining build, but it crosses
product-shape lines that need explicit operator decisions:

1. **Chain key persistence**: where does the per-session chain key live?
   Options: (a) generated fresh per session, written to
   `~/.cuttle/sessions/<id>/chain.key` mode 0600; (b) stored in macOS
   Keychain alongside the API-key entry; (c) operator generates once and
   reuses across sessions. Each has different audit-verify ergonomics.
2. **Multi-turn audit-event shape**: AuditEvent currently has no variant
   for "user message" or "assistant message". Options: (a) add new
   variants `UserPrompt { content_sha256, length }` + `AssistantResponse
{ content_sha256, length, usage }`; (b) treat conversation as a
   ToolDispatch with synthetic tool name; (c) keep audit log strictly for
   gate decisions and store conversation transcript separately. PRD §6.1.6
   suggests (a) but the TDD §5 audit-log shape was designed with (c) in
   mind.
3. **Tool dispatch**: with what tool definitions? Bash + file-read + file-
   write covers the daily-driver minimum. Each tool needs (a) JSON-schema
   for the API; (b) cuttle-side handler that goes through cuttle-sandbox;
   (c) ToolRegistry entry tagging secret-bearing-ness.
4. **Conversation history shape**: in-memory only? persisted to
   `~/.cuttle/sessions/<id>/transcript.jsonl`? Resume across cuttle
   restarts? Per CLAUDE.md privacy mandate, transcripts can hold PII.

Recommend Mo decide #1 + #2 before the v0.0.14 commit (each is a load-
bearing PRD-grain decision; making them implicitly via code wires them
into permanent shape). #3 + #4 can be deferred behind a `cuttle session
start --tools=none` switch that ships a no-tools REPL first.

Smoke verified by invoking the actual binary (no API key needed for error
paths): missing-prompt error names exact next action; invalid --max-tokens
names the offending option + value; missing API key names the env var.

## State at end of session 5 (2026-04-26)

**Headline**: path #1 of handoff-0.10 closed. All 12 v0.1 crates committed; the
`cuttle` binary builds, runs, and executes the first end-to-end CLI surface
(`cuttle telemetry [--json] [--falsifier-eval]`) over a real audit log.
Workspace: 173/173 tests pass; `cargo clippy --all-targets -- -D warnings` clean.

**Crates added in session 5** (commits on `main` after `1bfd7f0`):

- `cuttle-skills` v0.0.5, `cuttle-rewardloop` v0.0.6 (landed before session-5 capture
  but in the same milestone): unicode-allowlist scan + AP/VP signed-provenance registry.
- `cuttle-falsifiers` v0.0.7 (`6cd12d6`): three data-collection evaluators
  (DISABLE / SNAPSHOT-DRIFT / MEMORY-DRIFT) per BP-02 / D-25 + sealed-falsifiers doc.
- `cuttle-anthropic` v0.0.8 (`1a42d58`): non-streaming `messages()` over reqwest+rustls;
  AnthropicError taxonomy with explicit retryable classification; RetryPolicy as a
  pure decision function. Streaming + prompt-cache are scoped for v0.0.9 / v0.0.10.
- `cuttle-sandbox` v0.0.9 (`2343451`): SBPL profile generator + sync spawn over the
  standard macOS sandbox binary. Live integration tests prove (a) end-to-end
  sandboxed echo works and (b) the sandbox actually denies writes outside
  `project_root` (adversarial test caught two TDD-grade bugs in initial run).
  rlimit enforcement deferred to v0.0.10 (warrants its own threat-model pass).
- `cuttle-telemetry` v0.0.10 (`8cf159d`): library backing the `cuttle telemetry`
  CLI surface. Aggregate primitives (dispatch / decisions / overrides / abandons)
  plus `TelemetryReport::with_falsifiers` composition over cuttle-falsifiers.
- `cuttle-cli` v0.0.11 (`598dd39`): top-level binary. Hand-rolled argv parser
  (~140 LOC); subcommand handlers in their own modules; centralized path
  resolution; exit codes 0/1/2 for success/usage/subcommand. Smoke-tested:
  `cuttle --version` → `cuttle 0.0.11`, missing-audit-log error is specific +
  actionable + correct exit code.
- `cuttle-audit` gained a small read-only addition: `read_entries_unverified()`
  for read-only consumers (telemetry, future session-resume) that don't own the
  chain key. The verifying path (`cuttle audit verify`) lands in v0.0.12+.

**Process learnings from session 5**:

1. **Adversarial / live-integration tests on policy code are mandatory, not optional.**
   The cuttle-sandbox initial profile compiled fine and produced a syntactically
   valid SBPL — but every sandboxed program died with SIGABRT because dyld
   couldn't stat `/`. A pure renderer test would have shipped a useless sandbox.
   The "test what you claim to enforce" rule (CLAUDE.md §8g) is non-negotiable
   for security primitives. Two TDD-grade bugs caught: SBPL rejects IP literals
   in `(remote ip ...)` (only `localhost` / `*` are accepted hosts on macOS 14+);
   `(literal "/")` is required in `file-read*` for the loader to not abort.

2. **Scope discipline holds even when "continue until interrupted" is the standing
   directive.** Each crate landed at a clean unit-of-thinking boundary
   (cuttle-anthropic v0.0.8 = non-streaming + retry only; streaming was
   deliberately deferred to v0.0.9). The alternative (one giant cuttle-anthropic
   commit covering streaming + prompt-cache + retry) would have made review
   harder and forced larger rollback if any subsystem turned out wrong.

3. **Pre-existing legacy state from end of session 4 (the 6-crates-landed era)
   is preserved verbatim below for traceability**; the session-5 block above is
   the authoritative current view. Do not re-derive state from the older block.

## Where to resume (session 6 priorities)

Path #1 of handoff-0.10 is now CLOSED (all v0.1 crates land). The next-up
ranking, in order of how much each unblocks the daily-driver UX:

1. **`cuttle-anthropic` v0.0.9 (streaming SSE)** — the largest unblocker for
   the interactive-REPL surface. Adds `messages_stream()` over
   `eventsource-stream`. Critical retry-safety contract: NEVER retry once
   the first byte of a streamed response has landed (double-billing trap
   documented in retry.rs module doc). Also adds HTTP-date Retry-After
   parsing (delta-seconds form already lands in v0.0.8).

2. **`cuttle-cli` v0.0.12 (interactive `cuttle session start`)** — depends on
   #1. The first daily-driver-shape command. Composes credential vault →
   AnthropicClient::messages_stream → sandbox-dispatch loop → audit chain.
   This is the dogfood-readiness gate per PRD SC-1.

3. **`cuttle-sandbox` v0.0.10 (rlimit enforcement)** — adds `pre_exec` hook
   for RLIMIT_CPU / DATA / NOFILE / NPROC. Crosses an unsafe libc boundary;
   trigger the `threat-model` skill on the unsafe surface BEFORE landing
   (per CLAUDE.md mandatory-skill table).

4. **`cuttle-cli` v0.0.13 (`cuttle audit verify` + `cuttle config init/show`)**
   — composes `cuttle_audit::verify_chain`. Date-rolled audit log paths
   (`~/.cuttle/audit/<yyyy-mm-dd>.jsonl`) per TDD §5; needs chrono wire-in.

5. **REVIEW-1 / REVIEW-2 / FIX-DOCS pass** (parallel-stream) — once #1 + #2
   are in place, run `code-review` skill on all 12 crates as a single batch,
   then `threat-model` re-pass against the running binary, then a
   `sbom-license` scan on the full reqwest+rustls dep tree before any v0.1
   release tag. Per CLAUDE.md §5 (REVIEW step is mandatory at SYSTEM tier
   before COMPLETE).

6. **TDD §2-§6 doc completion** (lower priority; pending task #38) — the TDD
   v0 covers §1 (language), §2 (config + primitives), §3 (gate), §4
   (sandbox), §5 (audit), §6 (memory). Filling out §7 (skills), §8
   (rewardloop), §9 (CLI surface) reflects the implementation that now
   exists. Doc-after-code is acceptable here because the code commits
   carry inline rationale; the TDD additions are for future-Mo / future
   reviewer.

## Legacy: state at end of session 4 (2026-04-26)

- **Repository**: `/Users/m0qazi/cuttle`. 26 commits on `main`:
  - `16bc70e` seed (sessions 1-2)
  - Part 1 (PRD v1 baseline + reviews + handoff v0.4): `23455db`, `2860f18`, `205f151`
  - Part 2 (PRD v1.1 + delta + handoff v0.5): `b1913eb`, `527a399`, `7880da1`
  - Part 3 (PRD v1.2 Fowler + delta + handoff v0.6): `5c0a741`, `68b40be`, `e863c16`
  - Part 4 (adversarial review + PRD v1.3 + delta + handoff v0.7): `3f739ef`, `a8fa349`, `db09b8c`, `baa0dc9`
  - Part 5 (PRD v3 pruning + sealed falsifiers + delta + handoff v0.8): `4f0ffbb`, `2ce8b1b`, `a9252c6`, `b38aaa4`
  - Part 6 (TDD v0 complete: §1..§6 + DECISIONS D-15..31): `389edb7`, `9c88cf2`, `f1c6d02`, `045b4ce`
  - Part 7 (v0.0.1 scaffolding + handoff v0.9): `67ba1a3`, `8a7deae`
  - Part 8 (cuttle-runtime v0.0.2 + cuttle-audit v0.0.3 + cuttle-memory v0.0.4): `f6e503a`, `8b0f81a`, `1bfd7f0`
    Working tree clean modulo this handoff update.
- **PRD v3** at `docs/PRD.md` (committed `4f0ffbb`; v1.3 at `a8fa349`; v1.2 at `5c0a741`; v1.1 at `b1913eb`; v1 at `23455db`; v0 at `docs/archive/PRD-v0-2026-04-25.md`). v3 = post-pruning of v1.3 (= v2 per §11 versioning convention). 319 lines (was 336 in v1.3). All PRD-grain commitments preserved verbatim per `docs/threat-model-prd-v3-delta.md` audit; pruning targeted redundant cross-references and the §11 convention text. v3 is the seal candidate; FIX-DOCS at end of REVIEW-1 + REVIEW-2 produces the Accepted version. **Sealed-falsifier pre-registration** at `docs/falsifiers.md` (committed `2ce8b1b`) in pre-seal-draft state with all 7 predicates (F-Cuttle-DISABLE / BEDROCK / SUBSTRATE / OPTION-C / SNAPSHOT-DRIFT / MEMORY-DRIFT / FATIGUE) structurally locked; threshold refinements permitted up to v0.1 ship.
- **Decision log** at `docs/DECISIONS.md` (committed). ADR-lite format. 31 entries:
  D-01..06 (Carlos) + D-07..09 (v1.1) + D-10..12 (v1.2 Fowler) + D-13 (v1.3
  adversarial umbrella) + D-14 (v3 pruning umbrella) + D-15..31 (TDD §1-§6
  decisions). Convention: handoff carries session-1 decision headlines (below);
  DECISIONS.md carries entries from 2026-04-26 onward with full structure
  (context/options/decision/consequences).
- **TDD v0** at `docs/TDD.md` (committed across `389edb7`, `9c88cf2`, `f1c6d02`,
  `045b4ce`; ~1080 lines). Resolves 8 of 12 OQs at TDD-grain. §1 Rust language
  choice (D-15) with full reasoning; §2 config + 6 domain primitives + capability
  tokens; §3 policy gate (workspace structure, supervisor + restart, IPC schema,
  Allow/Warn/Deny graduation, imperative plug-ins, predicate maintenance, lockfile
  HMAC, Keychain rate-budget, falsifier threshold refinement); §4 sandbox; §5
  audit log + tool-tagging + state-coherence + PII posture; §6 memory + quarantine.
- **6 of 12 v0.1 crates landed** at `crates/` (Cargo workspace pinned to Rust 1.95
  stable + 11 workspace deps + release panic=abort). 50/50 tests pass; cargo clippy
  --all-targets -- -D warnings clean across the whole workspace.
  - `cuttle-credential` v0.0.1 (`67ba1a3`): ApiKey + HelperHash + CredentialRecord. 9 tests.
  - `cuttle-gate` v0.0.1 (`67ba1a3`): AttestationBody + TtyInputCap + Decision. 7 tests.
  - `cuttle-input` v0.0.1 (`67ba1a3`): Session that mints TtyInputCap. 1 test.
  - `cuttle-runtime` v0.0.2 (`f6e503a`): LockfilePath HMAC (WV-02 closure) +
    TierClassification + SigningKey. 16 tests.
  - `cuttle-audit` v0.0.3 (`8b0f81a`): HMAC chain (D-27) + ToolRegistry safe-by-default
    (WV-03 closure) + DefaultRedactor (D-30 OQ-12). 9 tests.
  - `cuttle-memory` v0.0.4 (`1bfd7f0`): OperatorAuthoredText / ModelAuthoredText pair
    (T-007) + canonical/quarantine layout + cap-witnessed promotion (D-31). 8 tests.
- **Process artifacts** (three-layer discipline: source preservation):
  `process/carlos-arguelles-input.md` (161 lines, committed `23455db`) preserves
  Carlos Arguelles articles. `process/martin-fowler-input.md` (committed `5c0a741`)
  preserves Mohan/Gumbley + Ford + Johnsson source material with verbatim quotes
  and a "Convergent thesis" table. Both feed DECISIONS → PRD without inline citation
  weight; the Fowler artifact also becomes the source for the framework-side
  `framework_external_corroboration.md` sidecar (path #4).
- **Review artifacts (v1)** at `docs/threat-model-prd-v1.md` and
  `docs/output-quality-prd-v1.md` (committed `2860f18`). v1 STRIDE+ register and v1
  PRD-checklist + filler/hedging audit. Authoritative per-finding source for D-07/08/09.
- **Delta-check artifacts (v1 → v1.1)** at `docs/threat-model-prd-v1.1-delta.md` and
  `docs/output-quality-prd-v1.1-delta.md` (committed `527a399`). Verifies all 4 v1
  CRITICALs closed at PRD-grain plus 3 cross-cuts; flags 4 new sub-surfaces for
  TDD-grain treatment (TTY-provenance primitive, memory quarantine area, L5 review
  queue storage, nested-harness lockfile). All 5 v1 output-quality FIX-BEFORE-V2
  items closed.
- **Delta-check artifacts (v1.1 → v1.2)** at `docs/threat-model-prd-v1.2-delta.md`
  and `docs/output-quality-prd-v1.2-delta.md` (committed `68b40be`). Verifies v1.2
  introduces zero new PRD-grain attack surface and that the §6.1.5 domain-primitives
  invariant hardens 5 of v1.1's CRITICAL closures with type-system defense-in-depth.
  Three new sub-surfaces flagged for TDD: constructor authorization, serialization
  round-trip, FFI/native-binding boundary. Wolverine + Black Panther adversarial
  review still queued for path #1 against PRD v1.2.
- **Project memory**: 14 files, 560 lines (unchanged from session 3). No new
  feedback rules learned in session 4 (existing rules executed cleanly: decisive
  execution, save-memory-at-intervals, handoff-between-context-switches all
  applied).
- **First Rust code lands** in `crates/cuttle-credential`, `crates/cuttle-gate`,
  `crates/cuttle-input` per v0.0.1 scaffolding above. 17/17 tests pass; cargo
  clippy clean. The type-system enforcement of T-001 attestation-provenance
  separation, T-002 helper-hash binding, CC-2 zeroization, D-17 capability-token
  witness pattern is now demonstrable in compiling code.
- **9 of 12 v0.1 crates pending**: `cuttle-runtime`, `cuttle-anthropic`,
  `cuttle-sandbox`, `cuttle-audit`, `cuttle-telemetry`, `cuttle-memory`,
  `cuttle-skills`, `cuttle-rewardloop`, `cuttle-falsifiers`, `cuttle-cli`.
- **No architecture diagrams** (DESIGN step pending REVIEW-1 + REVIEW-2).

## Where to resume

Three paths. Path 1 (TDD) is on the critical path to implementation; paths 2 + 3 are parallel-stream and can run before, during, or after path 1.

### 1. v0.0.x crate scaffolding continuation (IMMEDIATE)

Promoted from path #4 (TDD start) after TDD v0 + v0.0.1 scaffolding closed the
prior path #1 (commits `389edb7`..`045b4ce` + `67ba1a3`). 3 of 12 v0.1 crates
shipped; 9 remaining. Suggested next-up order (highest TDD-coverage value first):

- **`cuttle-runtime` (v0.0.2)**: `LockfilePath` domain primitive (T-004 / D-23
  HMAC + parent-PID + signing-key-in-memory); `TierClassification` enum (L2
  mechanic); session orchestration skeleton; state-coherence.json read-at-start
  invariant (BP-04 / D-29 two-fence).
- **`cuttle-audit` (v0.0.3)**: HMAC chain implementation (D-27); `AuditEntry` +
  `AuditChain` + `sync_data()` per entry; `ToolRegistry` with `secret_bearing`
  safe-by-default (D-28); state-coherence.json self-HMAC writer; `Redactor` trait
  for PII (D-30).
- **`cuttle-memory` (v0.0.4)**: `OperatorAuthoredText` vs `ModelAuthoredText`
  newtypes (D-17 + T-007); promotion workflow with `&TtyInputCap` requirement
  (D-31); canonical/quarantine filesystem layout.
- **`cuttle-sandbox` (v0.0.5)**: SBPL profile generation (D-26); resource limits
  via tokio `pre_exec`; T-005 contingency stubs.
- **`cuttle-anthropic` (v0.0.6)**: thin client over reqwest + serde +
  eventsource-stream (D-15 supply-chain decision); streaming, retry, prompt-cache.
- **`cuttle-skills` (v0.0.7)**: skills loader with Unicode allowlist + fail-closed
  on novel category (WV-05).
- **`cuttle-rewardloop` (v0.0.8)**: AP/VP registry with operator-review-queue
  promotion + signed provenance (D-22 + T-010 + WV-04).
- **`cuttle-falsifiers` (v0.0.9)**: data-collection side of the 7 sealed
  predicates; `cuttle telemetry --falsifier-eval` runner.
- **`cuttle-telemetry` (v0.0.10)**: `cuttle telemetry` CLI; on-demand audit-log
  scan; aggregation contract per D-04 + D-09.
- **`cuttle-cli` (v0.0.11)**: top-level `cuttle` bin; argument parsing; wires
  the runtime; brings everything together for first end-to-end smoke test.

Each crate commit lands separately for incremental review. Each commit verifies
cargo test + cargo clippy --all-targets -- -D warnings before commit.

### REVIEW-1 / REVIEW-2 / FIX-DOCS (parallel-stream)

Once enough crates land for end-to-end smoke testing (~v0.0.6 milestone, after
`cuttle-anthropic`), invoke `code-review` skill on PRD v3 + TDD v0 + landed code
as REVIEW-1. Then `legal-review` + `threat-model` + `privacy` as REVIEW-2.
FIX-DOCS produces the Accepted PRD + TDD; the Accepted milestone seals
`docs/falsifiers.md` per D-14.

### TDD-resolved sub-surfaces (work-items inherited by path #1 crates)

Sub-surfaces flagged across session-4 review passes; each new crate inherits
the relevant items as work-items:

- v1.1 delta: TTY-provenance primitive (cuttle-gate ✓ done in v0.0.1), memory
  quarantine area (cuttle-memory pending), L5 review queue storage (cuttle-rewardloop
  pending), nested-harness lockfile (cuttle-runtime pending).
- v1.2 delta (Fowler): constructor authorization capability scoping (cuttle-gate ✓
  done in v0.0.1; cross-crate verification at REVIEW-1), serialization round-trip
  for domain primitives (cuttle-credential ✓ done; cuttle-gate ✓ done; cuttle-runtime
  pending), FFI/native-binding boundary (none yet, may surface at v0.2 if TS surface
  is added).
- v1.3 delta (adversarial): tool-registration tagging contract for audit-log digest
  (cuttle-audit pending), state-coherence file integrity recursive (cuttle-runtime +
  cuttle-audit pending), per-attestation model-context logging privacy-sensitive
  (cuttle-falsifiers pending).

Implementation begins on path #1; REVIEW-1/REVIEW-2/FIX-DOCS as parallel-stream.

### 2. Karpathy review (parallel stream; deferred from v0.5 path #1)

Apply Karpathy's lens against Cuttle's bedrock thesis. Read karpathy.ai +
public AI/eng writing (talks, blog posts, GitHub READMEs, twitter/X long-form).
Three-layer artifact discipline: `process/karpathy-input.md` source preservation
→ `docs/DECISIONS.md` entries from D-2026-04-26-14 onward → fold into PRD as a
later revision (post-pruning would target v3.1) or §15 row update.

Anti-goal: do NOT compare Cuttle to nanoGPT or Karpathy's tooling. Single-target
lens per `feedback_review_as_lens_not_comparator.md`.

Lower priority than path #1 (pruning): adversarial review already attacked v1.2
defensibility from Wolverine + Black Panther angles; Karpathy's lens is
complementary not redundant but not on the critical path to v3 seal. Can run
in parallel with path #1 (pruning) if a parallel agent is available, or
queued for post-seal.

### 3. Framework-doc updates from Carlos + Fowler (parallel stream)

Mo's directive 2026-04-26: "start updating our framework docs based on what we
learn." Two homes:

- **Cuttle-side memory**: note Carlos + Mohan/Gumbley + Ford + Johnsson
  corroboration in `framework_components.md` re: substrate-constraint thesis.
  New sidecar `framework_external_corroboration.md` tracking all independent
  industry/academic convergence (sourced from `process/carlos-arguelles-input.md`
  - `process/martin-fowler-input.md` "Convergent thesis" table).
- **claude-study side**: propose new sidecar
  `~/claude-study/papers/external-convergence.md` tracking corroboration +
  identified gaps. Do NOT touch canonical `paper-agent-framework.md` directly
  (sealed artifact per methodology).

Can run in parallel with paths #1 + #2.

### TDD scope inheritance (cross-cutting reference for path #1)

TDD-grain sub-surfaces flagged across session-4 review passes; path #1 (TDD
start) inherits all of these as work-items:

- v1.1 delta: TTY-provenance primitive, memory quarantine area, L5 review
  queue storage, nested-harness lockfile.
- v1.2 delta (Fowler): constructor authorization (capability scoping),
  serialization round-trip for domain primitives, FFI/native-binding boundary.
- v1.3 delta (adversarial): tool-registration tagging contract for audit-log
  digest, state-coherence file integrity (recursive), per-attestation
  model-context logging (privacy-sensitive; OQ-12 must address).
- v3 delta (pruning): F-Cuttle-FATIGUE detection requires per-attestation
  logging contract that intersects OQ-12; falsifier threshold refinement
  (N, M, R, R_F, X) per `docs/falsifiers.md`.

v0.1 implementation begins after path #1 TDD + REVIEW-1 + REVIEW-2 + FIX-DOCS

- DESIGN + API.

## Anti-goal for resume

**Do NOT**:

- Re-derive the framework's contribution claims from scratch (they're in
  `framework_components.md`, post-rectification).
- Re-investigate claw-code or claude-code source availability (settled in
  `framework_components.md:107`).
- Re-do the auth-mode landscape research (settled in `auth_mode_decision.md` with
  verbatim ToS citations).
- Rewrite `framework_methodology_document.md` from the canonical source; it's a
  pointer + synthesis; if conflicts arise, re-read `~/claude-study/papers/methodology-document.md`.
- Treat first-draft memory files as ground truth without verifying against
  source-of-truth pointers.

## Decision log (session 1)

For full detail see `sessions/2026-04-25-session-1-snapshot.md` §3. Headlines:

1. Codename **Cuttle**; aqua-animal subsystem naming convention.
2. Phase 1 narrowed to **Claude-only** (multi-provider deferred).
3. v0.1 platform **CLI/terminal only** (browser/iOS/iPadOS deferred).
4. **API-key-only auth, ToS-clean.** No OAuth subscription path. Sealed slot for
   future cloud-provider modes (`bedrock`, `vertex`, `foundry`).
5. **Persona B dropped** from v0.1 scope (single-operator focus).
6. Validation study (`claude-study` Phase 1) **separate** from Cuttle.
7. **Bedrock thesis** locked: deterministic security + deterministic reliability
   (5 framework layers as harness mechanics) as co-equal foundations.
8. **Substrate-constraint thesis** locked: framework's `§10.2` audit-log default
   was a Claude Code workaround; Cuttle's pre-execution gate removes the constraint.
9. **Option C** (deterministic harness review) for high-blast-radius escape-hatch
   class. Initial enumeration: secret-scan bypass, bash-guard bypass, audit-log
   integrity, credential-vault unlock outside flow.
10. **Cuttle = from-scratch implementation**. Not a fork of Claude Code (no source
    available). Not built on `claw-code` (third-party clean-room rewrite by `instructkr`).
11. **"Mo built it" framing softened** to handoff §13 discipline: internal records
    retain provenance, external surfaces describe the work.
12. **Cuttle bakes BOTH framework architecture AND methodology disciplines** as
    harness mechanics. Sycophancy-detection-as-harness-mechanic identified as
    concrete v0.1 candidate.

## Decision log (session 3 onward): see `docs/DECISIONS.md`

Session-3 decisions (2026-04-26) are captured with full structure
(context/options/decision/consequences) in `docs/DECISIONS.md`. Headlines:

- **D-2026-04-26-01**: substrate-constraint thesis anchored in industry
  blast-radius argument (Carlos Arguelles pre-submit philosophy as independent
  corroboration). PRD §1 reframed.
- **D-2026-04-26-02**: per-call blast-radius differentiator made explicit in
  PRD §3 (no human in loop between LLM model output and side-effect).
- **D-2026-04-26-03**: adoption-ergonomics non-goal added to PRD §7
  (v0.1 ships as N=1 selection-biased dogfood; multi-operator adoption signal
  is a Phase-1-equivalent open question).
- **D-2026-04-26-04**: local telemetry surface added to v0.1 scope
  (`cuttle telemetry` command, no remote phoning-home).
- **D-2026-04-26-05**: Allow/Warn/Deny graduation as new open question OQ-9
  (deferred to TDD §3).
- **D-2026-04-26-06**: falsifier F-Cuttle-DISABLE seeded
  (gate-disable counts during dogfood week as bedrock-thesis falsifier signal).

Convention going forward: session-1 headlines stay above; session-N decisions
land in `docs/DECISIONS.md` with full structure. Source artifacts (e.g.,
`process/carlos-arguelles-input.md`) preserve external inputs that drove the
decisions, separate from the decisions themselves.

## Memory index (14 files at end of session 3)

| File                                           | Lines | Purpose                                                                                                                       |
| ---------------------------------------------- | ----- | ----------------------------------------------------------------------------------------------------------------------------- |
| `MEMORY.md`                                    | 10    | Index of project memory                                                                                                       |
| `user_role.md`                                 | 55    | Operator: Principal/Staff at Apple AI/ML, ex-Twitter/Meta. Calibration + complementary perspective                            |
| `codename_theme.md`                            | 16    | Cuttle codename + aqua-animal subsystem naming                                                                                |
| `product_vision.md`                            | 28    | Vision; Phase 1 narrowed to Claude-only                                                                                       |
| `phase1_scope.md`                              | 27    | Phase 1 PRD inputs: CLI-only, full-Claude-Code parity sandboxed, dogfood test                                                 |
| `auth_mode_decision.md`                        | 35    | API-key-only with verbatim Anthropic ToS citations + sealed slot for cloud-provider modes                                     |
| `framework_components.md`                      | 136   | The 5-layer framework + 4 contributions + 3 cross-cutting; substrate-constraint thesis                                        |
| `framework_development_methodology.md`         | 62    | How Mo developed the framework: red-team → 2-agent → 11-round duel; claim-narrowing                                           |
| `framework_methodology_document.md`            | 88    | Pattern-matching-foil technique; preconditions, failure modes, recursive applicability                                        |
| `cuttle_v01_option_c_enumeration.md`           | 31    | NEW (session 2). Initial 4-candidate dual-control class for deterministic harness review                                      |
| `feedback_decisive_execution.md`               | 16    | NEW (session 2). When recommendation is evidence-grounded, execute; don't pause for confirmation                              |
| `feedback_review_as_lens_not_comparator.md`    | 18    | Review/eval instructions default to single-target lens, not comparator (session 2)                                            |
| `feedback_save_memory_at_intervals.md`         | 19    | NEW (session 3). Long-session discipline: save state after each major artifact / new feedback rule; clear context proactively |
| `feedback_handoff_between_context_switches.md` | 18    | NEW (session 3). Update handoff.md before strand-shifts/context-clear/session-end; pairs with save-memory-at-intervals        |

Total: 560 lines across 14 files (was 523 across 12). 200-line load-budget
ceiling per CLAUDE.md hook means future sessions truncate any single file
above 200 lines. Largest unchanged at 136 lines, comfortably under the ceiling.

**Cross-repo artifact created session 2:** `~/claude-study/review-claude-code-setup.md`
(203 lines). Reviewer-prompt for a fresh Claude Code session to audit any Claude Code
setup against the 5-layer framework. Invoke with: `Read /Users/m0qazi/claude-study/review-claude-code-setup.md and review the Claude Code setup at <target>`.

**Artifacts on disk (sessions 3 + 4, all committed):**

- `docs/PRD.md` (v3 sealed candidate; committed `4f0ffbb`; v1.3 at `a8fa349`; v1.2 at `5c0a741`; v1.1 at `b1913eb`; v1 baseline at `23455db`). v0 archived at `docs/archive/PRD-v0-2026-04-25.md`.
- `docs/DECISIONS.md` (committed across `23455db`, `b1913eb`, `5c0a741`, `a8fa349`, `4f0ffbb`). ADR-lite log;
  14 entries: D-01..06 (Carlos) + D-07..09 (v1.1) + D-10..12 (v1.2 Fowler) + D-13 (v1.3 adversarial umbrella) + D-14 (v3 pruning umbrella).
- `docs/falsifiers.md` (committed `2ce8b1b`). Sealed-falsifier pre-registration in pre-seal-draft state. 7 predicates, structural shape locked, threshold values refined in TDD.
- `process/carlos-arguelles-input.md` (161 lines, committed `23455db`). Carlos source.
- `process/martin-fowler-input.md` (committed `5c0a741`). Mohan/Gumbley + Ford + Johnsson source.
- `docs/threat-model-prd-v1.md` (committed `2860f18`). v1 PRD-grain STRIDE+ register;
  10 threats (4 CRITICAL); 12 v1.1 edits enumerated. Authoritative per-finding source for D-08.
- `docs/output-quality-prd-v1.md` (committed `2860f18`). v1 0 BLOCK / 5 FIX-BEFORE-V2
  / 5 NICE-TO-HAVE; PRD v1 passes structural checklist.
- `docs/threat-model-prd-v1.1-delta.md` + `docs/output-quality-prd-v1.1-delta.md`
  (committed `527a399`). Verifies v1 → v1.1: all 4 v1 CRITICALs + 3 cross-cuts + all
  5 FIX-BEFORE-V2 closed; flags 4 TDD sub-surfaces.
- `docs/threat-model-prd-v1.2-delta.md` + `docs/output-quality-prd-v1.2-delta.md`
  (committed `68b40be`). Verifies v1.1 → v1.2: zero new attack surface; 5 prior
  closures hardened by §6.1.5 domain-primitives invariant; 3 new TDD sub-surfaces
  (constructor authorization, serialization round-trip, FFI/native-binding boundary).
- `docs/adversarial-review-prd-v1.2.md` (committed `3f739ef`). Wolverine + Black
  Panther review of PRD v1.2; 7 + 7 findings (3 Wolverine CRITICAL re-opens
  T-001/T-007/T-008; 7 Black Panther structural). Authoritative per-finding source for D-13.
- `docs/threat-model-prd-v1.3-delta.md` + `docs/output-quality-prd-v1.3-delta.md`
  (committed `db09b8c`). Verifies v1.2 → v1.3: 10 PRD-grain closures + 2 TDD-deferred;
  WV-06/WV-07 honestly disclaimed (operator-fatigue-keypress not solved at per-attestation
  grain). 3 new TDD sub-surfaces (tool-registration tagging, state-coherence file
  recursive integrity, per-attestation model-context logging privacy-sensitive).
- `docs/threat-model-prd-v3-delta.md` + `docs/output-quality-prd-v3-delta.md`
  (committed `a9252c6`). Verifies v1.3 → v3 pruning preserved every PRD-grain
  commitment verbatim. Section-by-section audit. Zero new attack surface from
  pruning; §11 versioning convention rewritten honestly.

## What this handoff intentionally omits

- Conversation transcripts (the durable record is the snapshot + memory; transcripts
  evaporate per claude-study handoff §13 discipline).
- Personal context that shaped the project (handoff §13 discipline; provenance lives
  in internal records).
- Per-file pruning rationale (lives in `sessions/2026-04-25-session-2-snapshot.md §2`).

## Next session resume prompt

```
I'm resuming Cuttle. Read handoff.md first, then the project memory at
/Users/m0qazi/.claude/projects/-Users-m0qazi-cuttle/memory/. Memory has 14
files; feedback rules govern long-session discipline (save memory at intervals;
update handoff between context switches; decisive execution when evidence-grounded).
Cite memory line numbers, don't re-derive.

Repo state: 9 commits on main. Session 4 closed with PRD v1 + reviews +
PRD v1.1 + v1.1 delta + PRD v1.2 (Fowler integration) + v1.2 delta + handoff
v0.6. Working tree clean. PRD v1.2 closes all 4 v1 CRITICAL threat-model
findings + 3 cross-cuts at PRD-grain, hardened by §6.1.5 domain-primitives
invariant adding type-system defense-in-depth.

Recommended first move: handoff.md "Where to resume" path #1 (adversarial
review of PRD v1.2). Per session-4 expansion ("don't stop until interrupted ...
PRD then implementation if it makes sense"), adversarial review dominates
Karpathy review at this grain: v1.2 is dense and externally-anchored
(§15 four corroborating voices); adversarial attack tests defensibility, while
Karpathy adds another corroborating voice. Adversarial dominates.

Run Wolverine + Black Panther personas per the threat-model skill's Step 8
methodology against PRD v1.2 directly. Output `docs/adversarial-review-prd-v1.2.md`.
If new CRITICAL findings: PRD v1.3 → delta-check → commit. Then PRD pruning to
v3 (path #3), seal falsifiers, then TDD start (path #5).

Karpathy review (path #2) is parallel-stream lower-priority; defer to after
v1.3 or run concurrently if a parallel agent is available.

Anti-goals: don't re-derive the 12 session-1 decisions or the 12 session-3/4
decisions (cite line numbers in handoff.md and DECISIONS.md respectively);
don't re-prune memory (done in session 2); don't re-derive post-rectification
framing (cite framework_development_methodology.md lines 22-27); don't inline
cuttle_v01_option_c_enumeration.md back into framework_components.md
(deliberately separated); don't re-research Carlos Arguelles articles
(captured in process/carlos-arguelles-input.md); don't re-run the v1 reviews
(artifacts at docs/threat-model-prd-v1.md / docs/output-quality-prd-v1.md
committed 2860f18; v1.1 delta artifacts at docs/threat-model-prd-v1.1-delta.md /
docs/output-quality-prd-v1.1-delta.md committed 527a399); don't re-apply the
v1.1 punchlist (already in PRD v1.1 commit b1913eb, traced via D-07/08/09).
```
