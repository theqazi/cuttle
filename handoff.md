# Cuttle: Handoff

**Purpose**: top-level current-state document for Cuttle. A new session opens with
this file plus the project memory at
`/Users/m0qazi/.claude/projects/-Users-m0qazi-cuttle/memory/`.

**Version**: handoff-0.3 (after session 3 PRD v1 + DECISIONS log + Carlos integration, 2026-04-26)
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

## State at end of session 3 (2026-04-26)

- **Repository**: `/Users/m0qazi/cuttle`. 1 commit on `main` (`16bc70e` seed,
  containing handoff + sessions + process + archived v0 PRD). Session 3 PRD v1,
  DECISIONS log, Carlos source artifact, and two new memory files were authored
  but had not yet committed at the snapshot point (PreToolUse skill-gate hook
  blocked the second commit pending threat-model + output-quality skill markers;
  see "Where to resume" path #1 below).
- **PRD v1** at `docs/PRD.md` (302 lines). Replaces v0 (now at `docs/archive/PRD-v0-2026-04-25.md`).
  Addresses all 11 drift items from `sessions/2026-04-25-session-1-snapshot.md` §4
  plus 6 Carlos-derived decisions (D-2026-04-26-01 through D-2026-04-26-06). Adds
  three new sections (§12 sealed falsifier pre-registration; §13 content-vs-guidance;
  §14 dual use of framework). Carries one self-flagged cleanup task: 18 em-dash
  violations against CLAUDE.md rule 7d.1, identified post-write, fix queued.
- **Decision log** at `docs/DECISIONS.md` (189 lines). New file. ADR-lite format.
  Seeds with 6 Carlos-driven entries. Convention: handoff carries session-1
  decision headlines (below); DECISIONS.md carries entries from 2026-04-26 onward
  with full structure (context/options/decision/consequences). Carries 7 em-dash
  violations queued for the same cleanup pass.
- **Process artifact** at `process/carlos-arguelles-input.md` (161 lines). New
  file. Preserves the Carlos Arguelles source material (3 articles, verbatim
  quotes where load-bearing) so PRD does not carry citation weight inline.
  Three-layer separation: process/ source → DECISIONS.md decisions → PRD.md
  implementation.
- **Project memory**: 14 files (was 12), 560 lines (was 523). Two new feedback
  memories from session 3: `feedback_save_memory_at_intervals.md` (19 lines)
  and `feedback_handoff_between_context_switches.md` (18 lines). Largest file
  unchanged at 136 lines. No file approaches 200-line truncation ceiling.
- **No code, no tests, no architecture diagrams, no TDD.** Pipeline still gated
  on PRD reaching v3 (post-adversarial-review + post-pruning) per global CLAUDE.md
  SYSTEM-tier ceremony.

## Where to resume

Six paths. The first is immediately gating; the rest can be sequenced.

### 1. Em-dash cleanup pass + commit session-3 artifacts (IMMEDIATE)

Session 3 left 4 uncommitted artifacts (`docs/PRD.md` v1, `docs/DECISIONS.md`,
`process/carlos-arguelles-input.md`, two new memory files). Commit was blocked by
PreToolUse skill-gate hook (threat-model + output-quality markers missing) and by
self-flagged em-dash violations (18 in PRD, 7 in DECISIONS) against CLAUDE.md rule
7d.1.

Sequence:

1. Em-dash cleanup pass on `docs/PRD.md` and `docs/DECISIONS.md`. Replace with
   commas, parens, colons, semicolons, or sentence breaks per rule 7d.1. Carlos
   source artifact has fewer; check it too.
2. Run `threat-model` skill against `docs/PRD.md` §6 (genuine new security
   architecture being declared in v0.1 scope). Refreshes `/tmp/.claude-threat-model-gate`.
3. Run `output-quality` skill against `docs/PRD.md`. Refreshes
   `/tmp/.claude-output-quality-gate`.
4. Stage by specific path (no `-A`): PRD, DECISIONS, archive (already in seed),
   process artifact, two memory files. Note memory files live OUTSIDE the cuttle
   repo at `~/.claude/projects/-Users-m0qazi-cuttle/memory/` and are not staged
   here; they live in the global memory store, not git.
5. Commit. Suggested subject: `docs: PRD v1 + DECISIONS log + Carlos integration`.

### 2. Karpathy review (Mo queued 2026-04-26)

Apply same lens as Carlos against Cuttle's bedrock thesis. Read karpathy.ai +
public AI/eng writing (talks, blog posts, GitHub READMEs, twitter/X long-form).
Three-layer artifact discipline: `process/karpathy-input.md` source preservation
→ `docs/DECISIONS.md` entries D-2026-04-26-07 onward → fold into PRD as v1.1
or queue for v2 incorporation.

Anti-goal: do NOT compare Cuttle to nanoGPT or Karpathy's tooling. Single-target
lens per `feedback_review_as_lens_not_comparator.md`.

### 3. Adversarial review on the post-Karpathy PRD

Sequence Karpathy AFTER em-dash cleanup but BEFORE adversarial review, so the
adversary attacks the most-complete version. Then run Claude+Codex (or
Claude+Gemini) adversarial defense modeled on the framework's `DUAL_AGENT_REVIEW.md`
discipline. Output: PRD v2.

### 4. PRD pruning to v3

Per `framework_methodology_document.md:34, 72`: "third version is shorter and
trustworthy." Output: PRD v3, ready for sealing. Then sealed-falsifier
pre-registration in `docs/falsifiers.md` (PRD §12) becomes immutable.

### 5. Framework-doc updates from Carlos (parallel stream)

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

Can run in parallel with paths 2-4. Tracked as task #8.

### 6. TDD start (gated on PRD v3)

Only after PRD reaches v3 (post-adversarial, post-pruning, sealed). Resolves
OQ-1 through OQ-6 + OQ-9 (per PRD v1 §10). Decisions there feed `system-design`
and `api-design` skill outputs, then implementation.

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

**New artifacts on disk (session 3, uncommitted at snapshot):**

- `docs/PRD.md` (v1, 302 lines). v0 archived at `docs/archive/PRD-v0-2026-04-25.md`.
- `docs/DECISIONS.md` (189 lines). ADR-lite parallel decision log; seeds with
  D-2026-04-26-01 through D-2026-04-26-06 (Carlos-driven).
- `process/carlos-arguelles-input.md` (161 lines). Source artifact preserving
  Carlos Arguelles Medium articles (URLs, verbatim quotes, what was used vs set
  aside).

## What this handoff intentionally omits

- Conversation transcripts (the durable record is the snapshot + memory; transcripts
  evaporate per claude-study handoff §13 discipline).
- Personal context that shaped the project (handoff §13 discipline; provenance lives
  in internal records).
- Per-file pruning rationale (lives in `sessions/2026-04-25-session-2-snapshot.md §2`).

## Next session resume prompt

```
I'm resuming Cuttle. Read handoff.md first, then the project memory at
/Users/m0qazi/.claude/projects/-Users-m0qazi-cuttle/memory/. Memory grew in
session 3 (2026-04-26) to 14 files; new feedback rules govern long-session
discipline (save memory at intervals; update handoff between context switches).
Cite memory line numbers, don't re-derive.

Recommended first move: handoff.md "Where to resume" path #1 (em-dash cleanup
pass + commit session-3 artifacts). Four files were authored but uncommitted
at snapshot: docs/PRD.md (v1, 302 lines), docs/DECISIONS.md (189 lines),
process/carlos-arguelles-input.md (161 lines), and two memory files outside
the cuttle repo. Self-flagged 18 em-dash violations in PRD and 7 in DECISIONS
against CLAUDE.md rule 7d.1; clean those before committing. PreToolUse hook
will require threat-model and output-quality skill markers; run them on
PRD §6 (security architecture) and PRD whole-doc (structured deliverable)
before the commit attempt.

After commit lands, path #2 (Karpathy review) is queued by Mo. Three-layer
artifact discipline: process/karpathy-input.md preserves source, DECISIONS.md
gains entries D-2026-04-26-07 onward, fold into PRD as v1.1 or queue for v2.
Single-target lens per feedback_review_as_lens_not_comparator.md; do NOT
compare Cuttle to nanoGPT or Karpathy's tooling.

Anti-goals: don't re-derive the 12 session-1 decisions or the 6 session-3
decisions (cite line numbers in handoff.md and DECISIONS.md respectively);
don't re-prune memory (done in session 2); don't re-derive post-rectification
framing (cite framework_development_methodology.md lines 22-27); don't inline
cuttle_v01_option_c_enumeration.md back into framework_components.md
(deliberately separated); don't re-research Carlos Arguelles articles
(captured in process/carlos-arguelles-input.md).
```
