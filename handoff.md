# Cuttle — Handoff

**Purpose**: top-level current-state document for Cuttle. A new session opens with
this file plus the project memory at
`/Users/m0qazi/.claude/projects/-Users-m0qazi-cuttle/memory/`.

**Version**: handoff-0.2 (after session 2 memory pruning, 2026-04-25)
**Tier**: SYSTEM (per global CLAUDE.md). Full pipeline: PRD → TDD → REVIEW-1 → REVIEW-2 → FIX-DOCS → DESIGN → API → LEGAL → PRIVACY → WRITE → COPY → REVIEW → SECURE → SBOM.

---

## What Cuttle is, in one paragraph

A from-scratch security-first terminal AI coding harness embodying the five-layer
agent-reliability framework documented in `~/claude-study/papers/paper-agent-framework.md`.
Cuttle's bedrock thesis: deterministic security AND deterministic reliability as
harness mechanics — _not_ as advisory skills the model can skip under load. Cuttle's
sharpened pitch: the framework, finally able to enforce _in front of_ execution
instead of _behind_ it, because the substrate is no longer the bottleneck. v0.1 is
single-operator, CLI-only, Anthropic-API-key-only (ToS-clean), and ships as an
implementation existence proof — not an effect claim.

## State at end of session 2 (2026-04-25)

- **Repository**: `/Users/m0qazi/cuttle`. `git init`, no commits, no source code.
  Three top-level directories: `docs/`, `sessions/`, `process/`. Session-2 added
  `sessions/2026-04-25-session-2-snapshot.md`.
- **Project memory**: 11 files (was 8), totaling 504 lines (was 513) — net slightly
  smaller, but the durability win is sharper than the count suggests. Largest file
  is now 136 lines (was 167); no file approaches the 200-line truncation ceiling.
  Three large files were pruned, one PRD-feeding sidecar was extracted, and one
  feedback memory was added (decisive execution rule). See snapshot §2 for the
  per-file breakdown.
- **PRD v0** at `docs/PRD.md` (190 lines, unchanged from session 1). Still out of
  sync with memory; v1 revision is now the recommended next move (path #2 below).
- **No code, no tests, no architecture diagrams, no TDD.** None of those start until
  PRD reaches v2 (post-adversarial-review) per global CLAUDE.md SYSTEM-tier ceremony.

## Where to resume

The next session has three viable starting points, ordered by recommendation:

### 1. ~~Memory pruning iteration~~ — DONE in session 2

Session 2 (2026-04-25) executed this path. Three large files pruned, PRD-feeding
sidecar extracted, decisive-execution feedback memory added. Details in
`sessions/2026-04-25-session-2-snapshot.md §2`. Future sessions should NOT re-run
this sweep; if a specific memory file is wrong, fix that file directly.

### 2. v1 PRD revision (now recommended first)

**Step 0 — archive v0 first** (decision 2026-04-25, end of session 1):
`mkdir -p docs/archive && git mv docs/PRD.md docs/archive/PRD-v0-2026-04-25.md`,
commit, then start v1 fresh at `docs/PRD.md`. Do NOT overwrite v0 in place —
the diff between v0 and v1 is itself an audit artifact for what changed and
why, and the methodology's "third version is shorter and trustworthy" discipline
implies versions are preserved, not overwritten. Same convention applies to
all subsequent revisions (v1 → archive, v2 fresh; v2 → archive, v3 fresh).

Drive the PRD from v0 to v1 against current memory. Specific corrections required
(captured in `sessions/2026-04-25-session-1-snapshot.md` §4):

- §1 framing: single-axis "security-first" → dual-bedrock with substrate-removal pitch
- §2.2 (Persona B): drop entirely (decision 2026-04-25)
- §3 (Problem): reframe as substrate-constraint workaround removal
- §6 v0.1 scope: add five-layer harness mechanics; add Option C dual-control class
- §7 non-goals: add explicit "Cuttle does not give users discipline they don't have"
- New §12: sealed falsifier pre-registration (per framework's own discipline)
- New §13: content-vs-guidance distinction when lifting from toolkit
- New §14: dual use of framework (methodology + content)
- Throughout: post-rectification framing for novelty claims (no overclaiming
  cross-session persistence as a differentiator)

Estimated effort: 1 session for v1 draft + 1 session for adversarial review +
1 session for v2 (post-review) + pruning to v3.

### 3. Adversarial review of bedrock thesis

Before v1 PRD is sealed, the bedrock thesis should pass through Claude+Codex (or
Claude+Gemini) adversarial defense, modeled on the framework's `DUAL_AGENT_REVIEW.md`
discipline. This was identified mid-session as a gap — the bedrock thesis had only
me as a counter-voice, which is not adversarial review by the framework's own
standard.

Recommended sequencing: run adversarial review **on the v1 PRD draft** rather than on
the bare bedrock thesis (gives the adversary something concrete to attack).

## Anti-goal for resume

**Do NOT**:

- Re-derive the framework's contribution claims from scratch (they're in
  `framework_components.md`, post-rectification).
- Re-investigate claw-code or claude-code source availability (settled in
  `framework_components.md:107`).
- Re-do the auth-mode landscape research (settled in `auth_mode_decision.md` with
  verbatim ToS citations).
- Rewrite `framework_methodology_document.md` from the canonical source — it's a
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

## Memory index (12 files at end of session 2)

| File                                        | Lines | Purpose                                                                                            |
| ------------------------------------------- | ----- | -------------------------------------------------------------------------------------------------- |
| `MEMORY.md`                                 | 10    | Index of project memory                                                                            |
| `user_role.md`                              | 55    | Operator: Principal/Staff at Apple AI/ML, ex-Twitter/Meta. Calibration + complementary perspective |
| `codename_theme.md`                         | 16    | Cuttle codename + aqua-animal subsystem naming                                                     |
| `product_vision.md`                         | 28    | Vision; Phase 1 narrowed to Claude-only                                                            |
| `phase1_scope.md`                           | 27    | Phase 1 PRD inputs: CLI-only, full-Claude-Code parity sandboxed, dogfood test                      |
| `auth_mode_decision.md`                     | 35    | API-key-only with verbatim Anthropic ToS citations + sealed slot for cloud-provider modes          |
| `framework_components.md`                   | 136   | The 5-layer framework + 4 contributions + 3 cross-cutting; substrate-constraint thesis             |
| `framework_development_methodology.md`      | 62    | How Mo developed the framework: red-team → 2-agent → 11-round duel; claim-narrowing                |
| `framework_methodology_document.md`         | 88    | Pattern-matching-foil technique; preconditions, failure modes, recursive applicability             |
| `cuttle_v01_option_c_enumeration.md`        | 31    | NEW (session 2). Initial 4-candidate dual-control class for deterministic harness review           |
| `feedback_decisive_execution.md`            | 16    | NEW (session 2). When recommendation is evidence-grounded, execute — don't pause for confirmation  |
| `feedback_review_as_lens_not_comparator.md` | 18    | NEW (session 2). Review/eval instructions default to single-target lens, not comparator            |

Total: 523 lines across 12 files (was 513 across 8). 200-line load-budget ceiling
per CLAUDE.md hook means future sessions truncate any single file >200 lines.
Largest is now 136 (was 167) — comfortably under the ceiling.

**Cross-repo artifact created session 2:** `~/claude-study/review-claude-code-setup.md`
(203 lines). Reviewer-prompt for a fresh Claude Code session to audit any Claude Code
setup against the 5-layer framework. Invoke with: `Read /Users/m0qazi/claude-study/review-claude-code-setup.md and review the Claude Code setup at <target>`.

## What this handoff intentionally omits

- Conversation transcripts (the durable record is the snapshot + memory; transcripts
  evaporate per claude-study handoff §13 discipline).
- Personal context that shaped the project (handoff §13 discipline; provenance lives
  in internal records).
- Per-file pruning rationale (lives in `sessions/2026-04-25-session-2-snapshot.md §2`).

## Next session resume prompt

```
I'm resuming Cuttle. Read handoff.md first, then the project memory at
/Users/m0qazi/.claude/projects/-Users-m0qazi-cuttle/memory/. Memory was
pruned in session 2 (2026-04-25); cite memory line numbers, don't re-derive.

Recommended first move: PRD v0 → v1 revision (handoff.md "Where to resume"
path #2). Step 0 = archive v0 to docs/archive/PRD-v0-2026-04-25.md, then
draft v1 §1 (framing) and §6 (v0.1 scope) against pruned memory before the
rest of the document — those carry the most session-1 staleness.

Anti-goals: don't re-derive any of the 12 decisions in handoff.md "Decision
log" (each in memory with rationale, cite the line); don't re-prune memory
(done); don't re-derive post-rectification framing (cite framework_development_methodology.md
lines 22-27); don't inline cuttle_v01_option_c_enumeration.md back into
framework_components.md (deliberately separated).
```
