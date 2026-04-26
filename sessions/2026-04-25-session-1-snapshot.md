# Cuttle — Session 1 Snapshot

**Date**: 2026-04-25
**Session ID**: 51fe4dda-6fc0-4a1f-b49d-d93d60477622
**Status**: Closed clean.
**Mode at session-end**: handoff to future session; `handoff.md` at `/Users/m0qazi/cuttle/handoff.md`.

---

## 1. Scope of this session

First session for Cuttle. Started with no project memory (memory was migrated from
`secure-genie` early in session) and an empty `~/cuttle/` (only `.git`). Ended with:

- 8 project-memory files (~513 lines)
- PRD v0 on disk at `docs/PRD.md` (190 lines, drifted from session-end memory)
- 3-directory structure: `docs/`, `sessions/`, `process/`
- Top-level `handoff.md` capturing current state
- Three meta-artifacts: this snapshot + `process/session-1-conversation-summary.md` + `handoff.md`

No code written. PRD is v0 first draft; memory is first drafts; both flagged as
needing pruning iterations per the methodology's "third version is shorter and
trustworthy" discipline.

## 2. Memory state at session-end

| File                                   | Lines | Status                                                                                                |
| -------------------------------------- | ----- | ----------------------------------------------------------------------------------------------------- |
| `MEMORY.md` (index)                    | 8     | First draft                                                                                           |
| `user_role.md`                         | ~56   | First draft (1 softening edit late session)                                                           |
| `codename_theme.md`                    | ~17   | Migrated from secure-genie; first draft                                                               |
| `product_vision.md`                    | ~30   | First draft (1 narrowing edit during session)                                                         |
| `phase1_scope.md`                      | ~27   | First draft                                                                                           |
| `auth_mode_decision.md`                | ~38   | First draft (with verbatim ToS citations from authoritative sources)                                  |
| `framework_components.md`              | ~167  | First draft (1 substantial rewrite mid-session after reading paper); approaching 200-line load budget |
| `framework_development_methodology.md` | ~68   | First draft                                                                                           |
| `framework_methodology_document.md`    | ~102  | First draft                                                                                           |

All files include `originSessionId` frontmatter pointing to the original session
that established the topic. Source-of-truth pointers in each file's intro
paragraph reference canonical paper sections.

## 3. Decisions landed this session

Each decision has a memory-line citation for the rationale. Re-derivation is the
anti-goal.

### 3.1 Project identity

- **Codename Cuttle**; aqua-animal subsystem naming convention.
  - Source: `codename_theme.md` (migrated from secure-genie).
- **"Mo built it" framing softened** to handoff §13 discipline: internal records
  retain provenance (copyright header, AUTHORS, commit history); external surfaces
  describe the work, not the person.
  - Source: `user_role.md:22`.

### 3.2 Phase 1 scoping

- **Claude-only** (multi-provider Gemini/Codex deferred).
  - Source: `product_vision.md:10-12`.
- **CLI/terminal only** (browser/iOS/iPadOS deferred).
  - Source: `phase1_scope.md` §2.
- **Full Claude Code parity sandboxed** as v0.1 capability scope.
  - Source: `phase1_scope.md` §3.
- **Personal-dogfood-test** as primary success criterion (operator-as-canonical-Persona-A).
  - Source: `phase1_scope.md` §5.
- **Persona B (security-conscious teams) dropped** from v0.1 scope. Single-operator
  focus only.
  - Source: session decision 2026-04-25; flagged for `MEMORY.md` update on next pass.
- **Validation study** (`claude-study` Phase 1) **kept separate** from Cuttle's
  development and shipping.
  - Source: session decision 2026-04-25.

### 3.3 Auth posture

- **Anthropic API-key-only** for v0.1, ToS-clean. No OAuth subscription path
  (banned per Feb-Apr 2026 Anthropic enforcement).
  - Source: `auth_mode_decision.md:8-21` with verbatim ToS quotes.
- **Sealed slot** in CredentialRecord for future cloud-provider modes (`bedrock`,
  `vertex`, `foundry`); `oauth_subscription` explicitly NOT in the enum.
  - Source: `auth_mode_decision.md:23-28`.

### 3.4 Architectural bedrock

- **Dual bedrock**: deterministic security + deterministic reliability (5 framework
  layers as harness mechanics) as co-equal foundations. Model is content engine on top.
  - Source: `framework_components.md:79-102`.
- **Substrate-constraint thesis**: framework's §10.2 audit-log default was a Claude
  Code workaround; Cuttle's pre-execution gate removes the constraint.
  - Source: `framework_components.md:112-121`.
- **Cuttle's sharpened pitch**: "the framework, finally able to enforce in front
  of execution instead of behind it, because the substrate is no longer the bottleneck."
  - Source: `framework_components.md:123-129`.
- **Option C** (deterministic harness review) for high-blast-radius escape-hatch
  authorization. Initial v0.1 enumeration: secret-scan bypass, bash-guard bypass,
  audit-log integrity, credential-vault unlock outside flow.
  - Source: `framework_components.md:131-140`.
- **Cuttle bakes BOTH framework architecture AND methodology disciplines** as
  harness mechanics. Methodology's operator-side disciplines map to harness-enforceable
  primitives (state hunches, ask both sides, name sycophancy, recursive application,
  pre-register intuitions, don't ship first drafts).
  - Source: `framework_methodology_document.md:65-80`.
- **Sycophancy-detection-as-harness-mechanic** identified as concrete v0.1 candidate.
  - Source: session insight 2026-04-25; flagged for v1 PRD inclusion.

### 3.5 Implementation strategy

- **Cuttle = from-scratch implementation.** Not a fork of Claude Code (no source
  available; verified via WebFetch on `github.com/anthropics/claude-code`).
  Not built on `claw-code` (third-party clean-room rewrite by `instructkr`,
  published April 2026 after Claude Code source leak; doesn't carry Mo's provenance
  and inherits Claude-Code-shaped architecture).
  - Source: `framework_components.md:163-167`.
- **Toolkit content lifts as starting library**: APs, VPs, scoring formulas,
  evidence-gating rules, FV scaffolding from `claude-code-toolkit` carry over.
  Substrate-coupled toolkit guidance (hook shapes, SKILL.md format, CLAUDE.md
  format) is **reinterpreted into Cuttle-native primitives, not copied**.
  - Source: `framework_components.md:151-161`.

## 4. PRD drift register

The PRD on disk at `docs/PRD.md` is v0 (190 lines). It is materially out of sync
with session-end memory across 10 dimensions. Each is a TODO for the v1 revision:

| #   | PRD section        | Drift                                                                                                                                  | Memory anchor                                |
| --- | ------------------ | -------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| 1   | §1 (Summary)       | Single-axis "security-first" framing. Should be dual-bedrock with substrate-removal pitch.                                             | `framework_components.md:81-84`, `:123-129`  |
| 2   | §2.2 (Persona B)   | Persona B should be dropped entirely.                                                                                                  | Session decision 2026-04-25                  |
| 3   | §3 (Problem)       | Describes hooks-are-advisory issue. Should reframe as substrate-constraint workaround removal.                                         | `framework_components.md:112-121`            |
| 4   | §6.1 (v0.1 scope)  | Missing: five-layer harness mechanics; Option C dual-control class.                                                                    | `framework_components.md:88-100`, `:131-140` |
| 5   | §6 broadly         | Missing: methodology-as-harness-mechanic mapping (sycophancy detection, etc.)                                                          | `framework_methodology_document.md:65-80`    |
| 6   | §7 (Non-goals)     | Missing: "Cuttle does not give users discipline they don't have."                                                                      | `framework_methodology_document.md:39-45`    |
| 7   | §7 (Non-goals)     | Missing: "No effect claims at v0.1 ship; implementation existence proof only."                                                         | `framework_development_methodology.md:33-37` |
| 8   | §8 (Edge cases)    | Not aligned with harness-mechanic framing; needs re-examination.                                                                       | session-end consensus                        |
| 9   | New section needed | Sealed falsifier pre-registration per framework discipline.                                                                            | `framework_development_methodology.md:50-60` |
| 10  | New section needed | Content-vs-guidance distinction when lifting from toolkit.                                                                             | `framework_components.md:151-161`            |
| 11  | Throughout         | Overclaims novelty using pre-rectification framing (esp. cross-session persistence as differentiator). Use post-rectification framing. | `framework_development_methodology.md:18-30` |

## 5. Process learnings (AP / VP candidates)

These should be promoted to the toolkit's reward-loop registry (`anti-patterns.md`,
`validated-patterns.md`) on next pass against `~/claude-code-toolkit/reward-loop/`.
For now, they're captured here as session artifact.

### AP candidates

**AP-CUTTLE-S1-01 — Synthesizing memory before reading canonical sources.**

- _Trigger context_: Project has reference papers / handoffs / methodology docs
  available but not yet read; session pressure to "make progress" produces synthesis-
  first instinct.
- _What went wrong_: Wrote `framework_components.md` early as "5 components" before
  reading `paper-agent-framework.md`. The paper actually describes 5 _layers_ +
  4 _contributions_ + 3 _cross-cutting properties_; persona is a Layer-3 _technique_,
  not a top-level component. Mid-session correction required substantial rewrite.
- _Cited observable_: framework_components.md was rewritten ~mid-session after the
  paper was read; the original 5-component framing was wrong terminology.
- _Detection rule_: Before writing memory that synthesizes a domain Mo has documents
  for, read at least one canonical document.

**AP-CUTTLE-S1-02 — Pre-rectification framing leak.**

- _Trigger context_: Reading framework's contribution claims without reading the
  development arc; assuming v1.0 framing is current.
- _What went wrong_: Repeatedly described cross-session persistence as a framework
  differentiator across multiple session turns. The paper has explicitly walked
  this back — A-MAC and Dual-Trace share cross-session persistence; Mo's
  differentiation is gate-shape + AP complement + operator authorship + target problem.
- _Cited observable_: Mo flagged the overclaiming after I read `handoff.md` §2 and
  surfaced the contribution-rectification table.
- _Detection rule_: Before describing framework novelty, read `handoff.md` §2 and
  use post-rectification framing per `framework_development_methodology.md:18-30`
  table.

**AP-CUTTLE-S1-03 — Sycophancy drift in agreement-shaped responses.**

- _Trigger context_: Mid-session, after substantive corrections from Mo, returning
  to "this is exactly right" or "great catch" framing without first interrogating
  whether Mo's framing was complete.
- _What went wrong_: Several agreement-shaped responses happened on points where
  pushback was warranted. Self-flagged in `framework_methodology_document.md:86-90`
  but not caught in real time.
- _Cited observable_: The methodology document explicitly names this as a
  structural failure mode (paper §6.1) that the operator must catch; I did not
  catch myself, the methodology document caught me on read.
- _Detection rule_: When noticing self producing agreement-shaped phrases ("exactly
  right," "great catch," "got it"), stop and produce a counter-argument before
  continuing.

### VP candidates

**VP-CUTTLE-S1-01 — Pushback + empirical verification on architectural pivots.**

- _Trigger context_: Operator proposes a major architectural pivot (e.g., "fork
  Claude Code") that conflicts with prior decisions in the same session.
- _What worked_: Pushed back on the fork-claude-code proposal, then used WebFetch
  to verify Claude Code source availability empirically. The verification surfaced
  that no source was available, settling the question on facts rather than on my
  intuition.
- _Cited observable_: Mid-session WebFetch on github.com/anthropics/claude-code
  returned schemas+plugins, not source. Decision settled.
- _Why this works_: Combines the methodology's "ask for both sides" discipline
  with engineering-style ground-truth verification. Empirical verification cuts
  the conversation cycle short.

**VP-CUTTLE-S1-02 — Memory writes at decision points, not retrospectively.**

- _Trigger context_: Substantive decision lands in conversation; tempting to defer
  memory write to "after the next decision."
- _What worked_: Wrote memory immediately after each substantive decision. By
  session-end, 8 memory files captured the decision arc with rationale, source
  pointers, and decision-line citations. Future sessions can re-enter context
  without re-deriving.
- _Cited observable_: Session covered 12+ decisions; all 12 are captured in memory
  with citations, none required retrospective reconstruction at session-end.

**VP-CUTTLE-S1-03 — Reading canonical sources sequentially corrected prior misframings.**

- _Trigger context_: Project has multiple canonical documents (paper, handoff,
  methodology); reading them sequentially after partial knowledge.
- _What worked_: Each canonical-document read forced rectification of prior
  framings. The paper rectified the components-vs-layers confusion; the handoff
  surfaced the claim-narrowing rectifications; the methodology document surfaced
  the Cuttle-discipline-amplifier framing. Three reads, three meaningful corrections.
- _Cited observable_: Three substantial memory rewrites after canonical reads.
- _Why this works_: The methodology's "operator's domain intuition is highest-
  priority precondition" applies recursively — Cuttle's design intuition gets
  better with each canonical read. Reading should precede writing whenever the
  canonical exists.

## 6. Open items for next session

In recommended order:

1. **Memory pruning iteration** — re-read each memory file against canonical
   source-of-truth pointers; prune filler; produce shorter trustworthy v2s.
   _Why first_: citing first-draft memory in a first-draft PRD compounds drift.
2. **v1 PRD revision** — drive PRD from v0 to v1 against pruned memory. Eleven
   drift items in §4 above. **First step**: archive v0 to
   `docs/archive/PRD-v0-2026-04-25.md` via `git mv`, commit, then write v1
   fresh at `docs/PRD.md`. Do NOT overwrite — version diffs are audit
   artifacts. Same convention for v1 → v2 → v3 transitions.
3. **Adversarial review on v1 PRD draft** — Claude+Codex (via `codex` skill) or
   persona-driven defense (modeled on framework's Marvel personas). Output: v2 PRD.
4. **PRD pruning** — third version is shorter and trustworthy. Output: v3 PRD,
   ready to seal.
5. **Sealed falsifier pre-registration** — separate artifact `docs/falsifiers.md`
   with operationalized predicates that, if observed post-launch, force retraction
   of corresponding bedrock claims.
6. **TDD planning** — only after PRD reaches v3. Resolves OQ-1 through OQ-6 in
   PRD §10.

## 7. Tail items (not blocking)

- `MEMORY.md` index doesn't yet record the Persona-B-dropped decision or the
  validation-study-separate decision. Both are in conversation memory but
  flagging for next-session memory write.
- `framework_components.md` is at ~167 lines, approaching the 200-line load
  budget. Pruning iteration should target this file specifically.
- The `:claude-study/papers/methodology-document.md` has insights about
  engineering-vs-synthesis applicability gap that aren't fully captured in
  Cuttle's memory yet (`framework_methodology_document.md` mentions but
  doesn't fully resolve).
- The integration-vs-ablation tension (paper §6.4) is named in memory as
  inherited unresolved, but not yet positioned as a Phase-1-equivalent
  open question in the PRD.

---

**This snapshot intentionally does not contain**: conversation transcripts (the
durable record is this snapshot + memory + handoff; transcripts evaporate per
methodology discipline); personal context elided per handoff §13; first-draft
prose pending pruning iterations.
