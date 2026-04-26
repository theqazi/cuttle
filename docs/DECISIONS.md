# Cuttle Decision Log

Running ADR-lite log of design decisions made during Cuttle's development.

**Scope split**:

- `handoff.md` carries the headline list of session-1 decisions (1-12) with one-line rationale.
- This file picks up from 2026-04-26 with full structure: context, options considered, decision, consequences. Each entry traces back to a source artifact (in `process/`) and forward to the deliverables it changed (PRD, TDD, memory, code).
- The session-1 decisions are not backfilled into this format. If a session-1 decision is later revisited, the revisit goes here as a new entry that supersedes the original.

**Entry format**:

```
## D-YYYY-MM-DD-NN: Title

| Field        | Value                                                           |
| ------------ | --------------------------------------------------------------- |
| Date         | YYYY-MM-DD                                                      |
| Status       | Proposed / Accepted / Superseded by D-...                       |
| Source       | process/<artifact>.md or session-N reference                    |
| Affects      | docs/PRD.md §N, docs/TDD.md §N, memory/<file>.md, ...           |

**Context**: 1-3 sentences. What changed in the world that put this decision on the table?

**Options considered**:
A. ...
B. ...
C. ...

**Decision**: A / B / C, with one-line why.

**Consequences**: What changes in the deliverables. What is now true that was not true before.
```

---

## D-2026-04-26-01: Substrate-constraint thesis anchored in industry blast-radius argument

| Field   | Value                                                                        |
| ------- | ---------------------------------------------------------------------------- |
| Date    | 2026-04-26                                                                   |
| Status  | Accepted                                                                     |
| Source  | `process/carlos-arguelles-input.md` §"How Amazon and Google view CI/CD..."   |
| Affects | `docs/PRD.md` §1 (Summary), §3 (Problem); future framework sidecar (task #8) |

**Context**: Cuttle's bedrock pitch (`framework_components.md:96-109`) framed pre-execution gating as architectural novelty. Carlos Arguelles, Senior Principal at Amazon (ex-Google, ex-MS), independently argues for pre-submit verification on monorepo blast-radius grounds, with Google's $100M/yr investment as evidence of industrial commitment. The pre-submit philosophy is not novel; it is industrial precedent at the largest scale.

**Options considered**:
A. Keep Cuttle's pitch as architectural novelty. Defensible only by ignoring industrial CI/CD literature.
B. Anchor in Carlos's blast-radius argument; reframe Cuttle's contribution as the LLM-agent application of an established CI/CD principle.
C. Drop the substrate-constraint thesis as not-novel-enough.

**Decision**: B. Honest framing per `framework_development_methodology.md:18-30` (post-rectification discipline). Cuttle's contribution narrows from "novel architecture" to "novel application of pre-submit philosophy to the LLM-agent case, where the substrate-constraint argument differs in shape (no human in loop) but the pre-vs-post tradeoff is the same."

**Consequences**:

- PRD §1 framing rewritten to reference Carlos as independent industry anchor.
- PRD §3 problem statement gains an explicit pre-submit/post-submit positioning.
- Cuttle's claim register loses "novel architecture" and gains "novel LLM-agent application of established architecture." Net defensibility increases (claim is narrower and better-anchored).
- Memory update queued (task #8): note corroboration in `framework_components.md` synthesis or new sidecar `framework_external_corroboration.md`.

---

## D-2026-04-26-02: Per-call blast-radius differentiator made explicit in PRD §3

| Field   | Value                                                           |
| ------- | --------------------------------------------------------------- |
| Date    | 2026-04-26                                                      |
| Status  | Accepted                                                        |
| Source  | `process/carlos-arguelles-input.md` §"How Amazon and Google..." |
| Affects | `docs/PRD.md` §3 (Problem)                                      |

**Context**: Carlos says Amazon (post-submit) and Google (pre-submit) are "morally equivalent, contextually different" (driven by blast radius). Cuttle's bedrock thesis claims pre-execution > post-execution architecturally. Without an explicit blast-radius argument, Cuttle's claim is not defensible against Carlos's lens.

**Options considered**:
A. Leave the differentiator implicit (current v0 PRD shape).
B. State explicitly in §3: an LLM agent's tool-use loop has no human in the loop between model output and side-effect, so per-call blast radius is unbounded by intent (one bash call can `rm -rf $HOME`). That is the load-bearing differentiator vs. enterprise CI, where post-submit is acceptable because human-authored commits land first.

**Decision**: B. The differentiator is the entire reason Cuttle's pre-execution gating is justified for a single-operator harness when post-submit is acceptable for a 120K-engineer monorepo.

**Consequences**:

- PRD §3 gains explicit no-human-in-loop framing.
- Falsifier candidate (D-2026-04-26-06) becomes operationally meaningful: if the operator routinely disables gates, the no-human-in-loop differentiator collapses (gates become advisory, the human is back in the loop only when they choose to be, which is exactly the failure mode the framework names).

---

## D-2026-04-26-03: Adoption-ergonomics non-goal added to PRD §7

| Field   | Value                                                         |
| ------- | ------------------------------------------------------------- |
| Date    | 2026-04-26                                                    |
| Status  | Accepted                                                      |
| Source  | `process/carlos-arguelles-input.md` §"Adventures in 30 Years" |
| Affects | `docs/PRD.md` §5 (Success criteria), §7 (Non-goals)           |

**Context**: Carlos's load-bearing claim: "technical excellence is necessary but nowhere near sufficient... [it] separates a tool that gets used from one that quietly dies." Cuttle's framework methodology says "Cuttle does not give users discipline they don't have" (`framework_methodology_document.md:60`), which is honest but addresses discipline, not adoption. The framework is silent on the engineering work of adoption design.

**Options considered**:
A. Stay silent on adoption (current v0 state). Risks shipping a technically pure tool that no second operator adopts, which would refute the bedrock thesis without ever testing it on more than N=1.
B. Add explicit non-goal: v0.1 does not optimize for first-week adoption ergonomics; ships as N=1 dogfood proof; multi-operator adoption signal is the Phase-1-equivalent open question, not a settled property.
C. Make adoption a v0.1 goal. Would expand scope significantly and conflict with single-operator dogfood success criterion.

**Decision**: B. Honest scoping per `framework_development_methodology.md:33-37` (no effect claims; implementation existence proof only). Surfaces the methodology's own multi-operator-validation open question into the PRD where it can drive Phase-1-equivalent scope.

**Consequences**:

- PRD §7 gains explicit non-goal on first-week adoption ergonomics.
- PRD §5 success criteria SC-1 reframed: dogfood is N=1 selection-biased proof, not adoption signal. Multi-operator validation moved to a future "Phase-1-equivalent validation" milestone (parallel to claude-study Phase 1).
- Cuttle's pitch gains a load-bearing negative claim: this is not a tool for users who do not bring discipline. Already foreshadowed in `framework_methodology_document.md:60`; now PRD-explicit.

---

## D-2026-04-26-04: Local telemetry surface added to v0.1 scope

| Field   | Value                                                         |
| ------- | ------------------------------------------------------------- |
| Date    | 2026-04-26                                                    |
| Status  | Accepted                                                      |
| Source  | `process/carlos-arguelles-input.md` §"Adventures in 30 Years" |
| Affects | `docs/PRD.md` §6.1 (v0.1 scope), §7 (Non-goals)               |

**Context**: Carlos: "imperfect data gathered pragmatically proved more value than waiting for perfect measurement"; telemetry-dark tools cannot improve. v0 PRD §7 said "no telemetry phoning home in v0.1" (correct for ToS/privacy posture per `auth_mode_decision.md:33-34`). The non-goal was over-broad: it precluded local visibility, not just remote phoning-home.

**Options considered**:
A. Keep zero telemetry posture (audit log is sufficient). The audit log is event-grain and append-only; it does not surface aggregate signal (gate-fire rates, override attempts, abandon points).
B. Add `cuttle telemetry` local-only command surfacing aggregate signal to the operator. No data leaves the machine; no Cuttle-controlled server.
C. Add opt-in remote telemetry. Out of scope per v0.1 ToS/privacy posture.

**Decision**: B. Local-only telemetry is L4-as-feedback-loop, not just L4-as-gate. Privacy-clean (operator-visible only, no remote transmission). Closes the "telemetry-dark tool cannot improve" failure mode without compromising the no-phoning-home commitment.

**Consequences**:

- PRD §6.1 v0.1 scope gains `cuttle telemetry` surface (gate-fire rates, override attempts, abandon points by tool/policy/session).
- PRD §7 non-goal narrowed: remote phoning-home remains forbidden in v0.1; local surfacing is permitted.
- TDD §5 (audit log design) gains an aggregation requirement: the audit log must support efficient queries for the telemetry surface without re-scanning the full event stream.

---

## D-2026-04-26-05: Allow/Warn/Deny graduation as new open question OQ-9

| Field   | Value                                                                                  |
| ------- | -------------------------------------------------------------------------------------- |
| Date    | 2026-04-26                                                                             |
| Status  | Accepted (as open question; resolution deferred to TDD §3)                             |
| Source  | `process/carlos-arguelles-input.md` §"Adventures in 30 Years" (configurable risk dial) |
| Affects | `docs/PRD.md` §10 (Open questions); future `docs/TDD.md` §3 (policy-gate API)          |

**Context**: Carlos's "configurable risk dial": humans own which uncovered lines carry business risk; one-size-fits-all enforcement misses the operator-specific risk surface. Cuttle's policy gate is currently binary Allow/Deny/Prompt (per v0 PRD §6.1 line 92 in `docs/archive/PRD-v0-2026-04-25.md`).

**Options considered**:
A. Stay binary Allow/Deny/Prompt. Simpler API; matches v0 PRD; reduces TDD surface.
B. Adopt Allow/Warn/Deny with operator-configurable per-policy threshold. Preserves deny-by-default for high-blast-radius rules; gives the operator a "log + continue" middle ground for low-blast-radius rules where strictness costs too much.
C. Defer to TDD §3 as OQ-9. Resolves alongside the broader policy-gate API question (declarative DSL vs imperative plug-ins, which is OQ-2 in v0 PRD).

**Decision**: C. The choice has architectural implications for the policy-gate API; resolving it in PRD without the broader API design would prematurely lock the shape. Add as OQ-9 in PRD §10.

**Consequences**:

- PRD §10 gains OQ-9.
- TDD §3 will resolve OQ-9 alongside OQ-2 (declarative vs imperative). Whichever DSL/API shape wins, the Allow/Warn/Deny question is part of the same surface.
- Open question explicitly notes the tradeoff: binary keeps the API smaller and the audit story simpler; graduated matches Carlos's risk-dial framing and is more defensible against "this gate is too strict" pushback.

---

## D-2026-04-26-06: Falsifier F-Cuttle-DISABLE seeded in docs/falsifiers.md

| Field   | Value                                                                                                   |
| ------- | ------------------------------------------------------------------------------------------------------- |
| Date    | 2026-04-26                                                                                              |
| Status  | Proposed (first-draft predicate; refine in TDD per `framework_development_methodology.md:48`)           |
| Source  | `process/carlos-arguelles-input.md` (silent-disable-under-load failure mode)                            |
| Affects | `docs/falsifiers.md` (new artifact); `docs/PRD.md` §12 (sealed falsifier pre-registration, new section) |

**Context**: Carlos's evidence that tools die silently maps to a specific failure mode for Cuttle: under load, operators disable strict policies and the tool reverts to advisory shape. This refutes Cuttle's bedrock thesis ("harness mechanics > advisory skills") in practice even if every gate works perfectly in isolation. The framework discipline (`framework_development_methodology.md:39-50`) requires sealed-falsifier pre-registration before v0.1 ship; Cuttle has none yet.

**Options considered**:
A. No falsifier. Trust the dogfood. Violates the framework's own pre-registration discipline.
B. F-Cuttle-DISABLE: if the operator disables ≥1 harness-mechanic gate during v0.1 dogfood week, the bedrock thesis is partially refuted (gates surviving as advisory in practice). First-draft predicate; refine in TDD.
C. Stricter version: any disable = full refutation. Brittle; does not distinguish between disabling a gate that was wrong (legitimate) from disabling a gate to bypass a correct deny (refutation).

**Decision**: B. Matches the framework's "first-draft predicates; refine in TDD against paper §C immutability discipline" pattern. Distinction between legitimate-disable and bypass-disable is a TDD-grain refinement; the v0.1 PRD captures the predicate at PRD grain.

**Consequences**:

- New artifact `docs/falsifiers.md` to be created; F-Cuttle-DISABLE is the first entry.
- PRD gains new §12 (sealed falsifier pre-registration) per session-1 drift register item 9.
- Dogfood instrumentation requirement: the audit log must capture gate-disable events with reason codes so the falsifier predicate can be evaluated mechanically post-week, not by recall.
- Sealing discipline: per `framework_development_methodology.md:39-43`, the falsifier becomes immutable at v0.1 ship; refinements between now and ship are permitted.
