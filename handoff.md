# Cuttle: Handoff

**Purpose**: top-level current-state document for Cuttle. A new session opens with
this file plus the project memory at
`/Users/m0qazi/.claude/projects/-Users-m0qazi-cuttle/memory/`.

**Version**: handoff-0.7 (after session 4 adversarial review + PRD v1.3 + delta-check, 2026-04-26)
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

- **Repository**: `/Users/m0qazi/cuttle`. 12 commits on `main`:
  - `16bc70e` seed (sessions 1-2 artifacts + archived v0 PRD)
  - `23455db` (session 4 part 1): PRD v1, DECISIONS log, Carlos integration, em-dash cleanup
  - `2860f18` (session 4 part 1): threat-model + output-quality reviews of PRD v1
  - `205f151` (session 4 part 1): handoff to v0.4
  - `b1913eb` (session 4 part 2): **PRD v1.1** + DECISIONS D-07/08/09
  - `527a399` (session 4 part 2): v1 → v1.1 delta-check artifacts
  - `7880da1` (session 4 part 2): handoff to v0.5
  - `5c0a741` (session 4 part 3): **PRD v1.2** + DECISIONS D-10/11/12 + Fowler source artifact
  - `68b40be` (session 4 part 3): v1.1 → v1.2 delta-check artifacts (Fowler pass)
  - `e863c16` (session 4 part 3): handoff to v0.6
  - `3f739ef` (session 4 part 4): adversarial review (Wolverine + Black Panther) of PRD v1.2
  - `a8fa349` (session 4 part 4): **PRD v1.3** + DECISIONS D-13 (umbrella for adversarial findings)
  - `db09b8c` (session 4 part 4): v1.2 → v1.3 delta-check artifacts
    Working tree clean modulo this handoff update.
- **PRD v1.3** at `docs/PRD.md` (committed `a8fa349`; v1.2 at `5c0a741`; v1.1 at `b1913eb`; v1 at `23455db`; v0 at `docs/archive/PRD-v0-2026-04-25.md`). v1 addressed 11 drift items + 6 Carlos decisions. v1.1 closed 4 of 4 v1 CRITICALs + 3 cross-cuts. v1.2 (Fowler pass) added external corroboration + §6.1.5 domain-primitives invariant hardening 5 v1.1 closures. v1.3 (adversarial review pass) closes 10 of 14 Wolverine + Black Panther findings; the 2 CRITICAL re-opens (WV-06 / WV-07 attestation reduces to operator-fatigue-keypress) are HONESTLY DISCLAIMED at PRD-grain with F-Cuttle-FATIGUE in §12 making them empirically falsifiable. New §8 case 9 (backup/restore as trust boundary), new OQ-11 (process isolation) + OQ-12 (PII posture). v0.1 falsifier set now 7 predicates. Em-dash discipline maintained: 0 violations across all session-3/4 deliverables.
- **Decision log** at `docs/DECISIONS.md` (committed). ADR-lite format. 13 entries:
  D-01..06 (Carlos) + D-07..09 (v1.1) + D-10..12 (v1.2 Fowler) + D-13 (v1.3
  adversarial-review umbrella). Convention: handoff carries session-1 decision
  headlines (below); DECISIONS.md carries entries from 2026-04-26 onward with full
  structure (context/options/decision/consequences).
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
- **No code, no tests, no architecture diagrams, no TDD.** Pipeline still gated
  on PRD reaching v3 (post-adversarial-review + post-pruning) per global CLAUDE.md
  SYSTEM-tier ceremony.

## Where to resume

Four paths. The first is immediately gating; paths 2 + 3 can run in parallel; path 4 (TDD) is gated on path 1 (v3 sealed).

### 1. PRD pruning to v3 (IMMEDIATE)

Promoted from path #3 in handoff-0.6 after adversarial review + v1.3 closed
the prior path #1 (commits `3f739ef` + `a8fa349` + `db09b8c`). PRD v1.3 is
~340 lines with three review cycles of accreted cross-references; the
framework's "third version is shorter and trustworthy" discipline
(`framework_methodology_document.md:34, 72`) calls for a pruning pass:

- Remove cross-references that no longer load-bear (e.g., D-### entries
  cited multiple times where one cite is enough).
- Tighten prose where v1 / v1.1 / v1.2 / v1.3 layered hedges have piled up.
- Move TDD-grain detail out of PRD bullets and into a single "TDD scope
  inheritance" section if it becomes its own surface.
- Preserve all PRD-grain commitments verbatim; pruning is rewording, not
  rescoping.

Output: PRD v3, ready for sealing. Then sealed-falsifier pre-registration in
`docs/falsifiers.md` (PRD §12, now 7 predicates) becomes immutable. Then
TDD start (path #3, formerly path #5).

If pruning surfaces structural issues that warrant another review pass,
v3 → v3.1 → seal. Otherwise v3 is the seal candidate.

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

### 4. TDD start (gated on PRD v3 sealed)

Only after PRD reaches v3 (post-pruning, sealed). Resolves OQ-1 through OQ-12
(per PRD v1.3 §10, expanded). Decisions there feed `system-design` and
`api-design` skill outputs, then implementation. TDD-grain sub-surfaces inherited
from session-4 review passes:

- v1.1 delta: TTY-provenance primitive, memory quarantine area, L5 review
  queue storage, nested-harness lockfile.
- v1.2 delta (Fowler): constructor authorization (capability scoping),
  serialization round-trip for domain primitives, FFI/native-binding boundary.
- v1.3 delta (adversarial): tool-registration tagging contract for audit-log
  digest, state-coherence file integrity (recursive), per-attestation model-
  context logging (privacy-sensitive; OQ-12 must address).

v0.1 implementation begins after seal + TDD §1 (OQ-1 language) + §2 (config /
data model) + §3 (policy gate) + §4 (sandbox) + §5 (audit log) decisions.

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

- `docs/PRD.md` (v1.3; committed `a8fa349`; v1.2 at `5c0a741`; v1.1 at `b1913eb`; v1 baseline at `23455db`). v0 archived at `docs/archive/PRD-v0-2026-04-25.md`.
- `docs/DECISIONS.md` (committed `23455db` + `b1913eb` + `5c0a741`). ADR-lite log;
  13 entries: D-01..06 (Carlos) + D-07..09 (v1.1) + D-10..12 (v1.2 Fowler) + D-13 (v1.3 adversarial umbrella).
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
