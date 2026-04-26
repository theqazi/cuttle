# Cuttle: Handoff

**Purpose**: top-level current-state document for Cuttle. A new session opens with
this file plus the project memory at
`/Users/m0qazi/.claude/projects/-Users-m0qazi-cuttle/memory/`.

**Version**: handoff-0.5 (after session 4 PRD v1.1 application + delta-check, 2026-04-26)
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

## State at end of session 4 (2026-04-26)

- **Repository**: `/Users/m0qazi/cuttle`. 6 commits on `main`:
  - `16bc70e` seed (sessions 1-2 artifacts + archived v0 PRD)
  - `23455db` (session 4 part 1): PRD v1, DECISIONS log, Carlos integration, em-dash cleanup
  - `2860f18` (session 4 part 1): threat-model + output-quality reviews of PRD v1
  - `205f151` (session 4 part 1): handoff bumped to v0.4
  - `b1913eb` (session 4 part 2): **PRD v1.1** + DECISIONS D-07/08/09
  - `527a399` (session 4 part 2): v1 → v1.1 delta-check artifacts
    Working tree clean modulo this handoff update.
- **PRD v1.1** at `docs/PRD.md` (committed `b1913eb`; v1 in git history at `23455db`; v0 at `docs/archive/PRD-v0-2026-04-25.md`).
  v1 addressed all 11 drift items from `sessions/2026-04-25-session-1-snapshot.md` §4
  plus 6 Carlos-derived decisions (D-2026-04-26-01 through D-2026-04-26-06). v1.1
  applies 15 deduplicated edits closing 4 of 4 v1 CRITICAL threat-model findings
  (T-001 attestation provenance separation; T-002 apiKeyHelper hash-pin; T-007
  cross-session memory quarantine; T-009 exception-table operator-load) plus the 3
  cross-cuts (CC-1 fail-closed gate, CC-2 in-memory key zeroization, CC-3 privacy
  skill in §11). Adds three sections from v1: §12 sealed falsifier pre-registration
  (now 6 predicates including F-Cuttle-SNAPSHOT-DRIFT and F-Cuttle-MEMORY-DRIFT);
  §13 content-vs-guidance; §14 dual use of framework. Em-dash discipline maintained;
  0 violations against CLAUDE.md rule 7d.1 across all session-3/4 deliverables.
- **Decision log** at `docs/DECISIONS.md` (committed). ADR-lite format. 9 entries:
  D-2026-04-26-01..06 (Carlos-driven) + D-2026-04-26-07..09 (v1.1 application:
  implementation softening, trust-boundary tightening umbrella, pipeline expansion).
  Convention: handoff carries session-1 decision headlines (below); DECISIONS.md
  carries entries from 2026-04-26 onward with full structure
  (context/options/decision/consequences).
- **Process artifact** at `process/carlos-arguelles-input.md` (161 lines, committed).
  Preserves the Carlos Arguelles source material (3 articles, verbatim quotes
  where load-bearing) so PRD does not carry citation weight inline. Three-layer
  separation: process/ source → DECISIONS.md decisions → PRD.md implementation.
- **Review artifacts (v1)** at `docs/threat-model-prd-v1.md` and
  `docs/output-quality-prd-v1.md` (committed `2860f18`). v1 STRIDE+ register and v1
  PRD-checklist + filler/hedging audit. Authoritative per-finding source for D-07/08/09.
- **Delta-check artifacts (v1 → v1.1)** at `docs/threat-model-prd-v1.1-delta.md` and
  `docs/output-quality-prd-v1.1-delta.md` (committed `527a399`). Verifies all 4 v1
  CRITICALs closed at PRD-grain plus 3 cross-cuts; flags 4 new sub-surfaces for
  TDD-grain treatment (TTY-provenance primitive, memory quarantine area, L5 review
  queue storage, nested-harness lockfile). All 5 v1 output-quality FIX-BEFORE-V2
  items closed. Wolverine + Black Panther adversarial review deferred to path #3
  against PRD v2.
- **Project memory**: 14 files, 560 lines (unchanged from session 3). No new
  feedback rules learned in session 4 (existing rules executed cleanly: decisive
  execution, save-memory-at-intervals, handoff-between-context-switches all
  applied).
- **No code, no tests, no architecture diagrams, no TDD.** Pipeline still gated
  on PRD reaching v3 (post-adversarial-review + post-pruning) per global CLAUDE.md
  SYSTEM-tier ceremony.

## Where to resume

Five paths. The first is immediately gating; the rest can be sequenced.

### 1. Karpathy review (IMMEDIATE; Mo queued 2026-04-26)

Promoted from path #2 in handoff-0.4 after PRD v1.1 application closed the prior
path #1 in session 4 (commits `b1913eb` + `527a399`). Apply the same lens
discipline used for Carlos: source-preservation in `process/`, decisions logged
to DECISIONS.md, fold into PRD as v1.2.

Apply Karpathy's lens against Cuttle's bedrock thesis. Read karpathy.ai + public
AI/eng writing (talks, blog posts, GitHub READMEs, twitter/X long-form).
Three-layer artifact discipline: `process/karpathy-input.md` source preservation
→ `docs/DECISIONS.md` entries D-2026-04-26-10 onward → fold into PRD as v1.2 (or
queue for v2 incorporation if findings are deep enough to justify holding for
adversarial review).

Anti-goal: do NOT compare Cuttle to nanoGPT or Karpathy's tooling. Single-target
lens per `feedback_review_as_lens_not_comparator.md`.

After Karpathy v1.2 lands, re-run threat-model + output-quality delta-check
against v1.2 (same shape as v1 → v1.1 delta artifacts at
`docs/threat-model-prd-v1.1-delta.md`).

### 2. Adversarial review on the post-Karpathy PRD

Run Claude+Codex (or Claude+Gemini) adversarial defense modeled on the framework's
`DUAL_AGENT_REVIEW.md` discipline. Targets PRD v1.2 (post-Karpathy). Output: PRD v2.
This is also the natural home for the deferred Wolverine + Black Panther review-board
pass that the threat-model skill itself queued (see Step 8 in
`docs/threat-model-prd-v1.md`).

### 3. PRD pruning to v3

Per `framework_methodology_document.md:34, 72`: "third version is shorter and
trustworthy." Output: PRD v3, ready for sealing. Then sealed-falsifier
pre-registration in `docs/falsifiers.md` (PRD §12) becomes immutable.

### 4. Framework-doc updates from Carlos (parallel stream)

Mo's directive 2026-04-26: "start updating our framework docs based on what we
learn." Two homes:

- **Cuttle-side memory**: note Carlos corroboration in
  `framework_components.md` re: substrate-constraint thesis. Possibly new sidecar
  `framework_external_corroboration.md` tracking all independent industry/academic
  convergence.
- **claude-study side**: propose new sidecar
  `~/claude-study/papers/external-convergence.md` tracking corroboration +
  identified gaps. Do NOT touch canonical `paper-agent-framework.md` directly
  (sealed artifact per methodology).

Can run in parallel with paths 1-3.

### 5. TDD start (gated on PRD v3)

Only after PRD reaches v3 (post-adversarial, post-pruning, sealed). Resolves
OQ-1 through OQ-6 + OQ-9 (per PRD v1.1 §10). Decisions there feed `system-design`
and `api-design` skill outputs, then implementation. v1.1 added new TDD-grain
sub-surfaces flagged in `docs/threat-model-prd-v1.1-delta.md` ("New surface
introduced by v1.1 edits"): TTY-provenance primitive, memory quarantine area,
L5 review queue storage, nested-harness lockfile.

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

- `docs/PRD.md` (v1.1; committed `b1913eb`; v1 baseline at `23455db`). v0 archived at `docs/archive/PRD-v0-2026-04-25.md`.
- `docs/DECISIONS.md` (committed `23455db` + `b1913eb`). ADR-lite parallel decision log;
  9 entries: D-2026-04-26-01..06 (Carlos-driven) + D-07/08/09 (v1.1 application).
- `process/carlos-arguelles-input.md` (161 lines, committed `23455db`). Source
  artifact preserving Carlos Arguelles Medium articles (URLs, verbatim quotes,
  what was used vs set aside).
- `docs/threat-model-prd-v1.md` (committed `2860f18`). PRD-grain STRIDE+ register on PRD v1;
  10 threats (4 CRITICAL); 12 required PRD v1.1 edits enumerated. Authoritative per-finding source for D-08.
- `docs/output-quality-prd-v1.md` (committed `2860f18`). 0 BLOCK / 5 FIX-BEFORE-V2
  / 5 NICE-TO-HAVE; PRD v1 passes structural checklist.
- `docs/threat-model-prd-v1.1-delta.md` (committed `527a399`). Verifies v1.1 closes
  4 of 4 v1 CRITICALs at PRD-grain + 3 cross-cuts; flags 4 new sub-surfaces for TDD.
- `docs/output-quality-prd-v1.1-delta.md` (committed `527a399`). Verifies all 5 v1
  FIX-BEFORE-V2 items closed; NICE-TO-HAVE items carry forward.

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

Repo state: 6 commits on main. Session 4 closed with PRD v1 (23455db) +
v1 reviews (2860f18) + handoff v0.4 (205f151) + PRD v1.1 (b1913eb) + v1.1
delta-check (527a399). Working tree clean. PRD v1.1 closes all 4 v1 CRITICAL
threat-model findings + 3 cross-cuts at PRD-grain.

Recommended first move: handoff.md "Where to resume" path #1 (Karpathy review).
Promoted from path #2 in handoff-0.4 after v1.1 application closed the prior
path #1. Apply same lens discipline as Carlos: source-preservation in
process/karpathy-input.md, decisions logged D-2026-04-26-10 onward, fold into
PRD as v1.2. Single-target lens per feedback_review_as_lens_not_comparator.md;
do NOT compare Cuttle to nanoGPT or Karpathy's tooling. After v1.2 lands,
re-run threat-model + output-quality delta-check against v1.2.

After Karpathy v1.2 commits, path #2 is the deferred adversarial review against
v1.2 -> PRD v2. The threat-model skill already queued Wolverine + Black Panther
(Step 8 in docs/threat-model-prd-v1.md); that pass is the natural fit for path
#2.

Anti-goals: don't re-derive the 12 session-1 decisions or the 9 session-3/4
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
