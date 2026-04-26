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

---

## D-2026-04-26-07: §6.1.1 implementation-detail commitments softened to defer-to-TDD

| Field   | Value                                                                                                                              |
| ------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Date    | 2026-04-26                                                                                                                         |
| Status  | Accepted                                                                                                                           |
| Source  | `docs/output-quality-prd-v1.md` OQ-FIX-1; `docs/threat-model-prd-v1.md` T-005 (sandbox-exec deprecation); T-003 (audit-log scheme) |
| Affects | `docs/PRD.md` §6.1.1 (Substrate), §10 (Open questions: OQ-3, OQ-4)                                                                 |

**Context**: PRD v1 §6.1.1 silently committed to specific implementation primitives (HMAC chain for audit-log tamper-evidence; `sandbox-exec` for bash sandbox) while §10 still listed both as open questions (OQ-3, OQ-4). Output-quality flagged the contradiction as PRD-vs-TDD scope discipline failure. Threat-model flagged sandbox-exec as on Apple's deprecation path; HMAC chain as operator-keyed (anti-Sybil-weak in single-operator scope).

**Options considered**:
A. Resolve OQ-3 and OQ-4 in §10, removing them as open. Locks the choices early; closes options before TDD has explored them.
B. Soften §6.1.1 to defer-to-TDD-with-leading-candidates. Keeps OQ-3 and OQ-4 open; PRD declares the requirement (tamper-evident chain; macOS process-isolation primitive) and TDD picks the scheme.
C. Strike the bullets entirely. Leaves the substrate underspecified; loses the "v0.1 ships THIS surface" clarity that SC-3 / SC-4 / SC-5 depend on.

**Decision**: B. PRD declares what surface ships and what invariant it satisfies; TDD picks the primitive. The PRD wording becomes "tamper-evident chain (specific scheme is OQ-4)" and "macOS process-isolation primitive scoped to project working directory; primitive choice is OQ-3." Leading candidates remain named so TDD has a starting point.

**Consequences**:

- §6.1.1 audit-log bullet softened; OQ-4 stays open in §10.
- §6.1.1 sandbox bullet softened; OQ-3 stays open in §10. T-005 (Apple deprecation) explicitly cross-referenced; TDD §4 must produce a contingency.
- §9 Constraints text already names sandbox-exec; left as-is because it's framed as the rationale for "macOS-first" not as a v0.1 commitment. (Re-evaluate at PRD v1.2.)
- This is a pure scope-discipline correction; no security guarantee changes hands.

---

## D-2026-04-26-08: Trust-boundary tightening from threat-model PRD-grain CRITICAL findings

| Field   | Value                                                                                                                                                                                                  |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Date    | 2026-04-26                                                                                                                                                                                             |
| Status  | Accepted                                                                                                                                                                                               |
| Source  | `docs/threat-model-prd-v1.md` (4 CRITICAL: T-001, T-002, T-007, T-009; T-004; T-010; CC-1; CC-2)                                                                                                       |
| Affects | `docs/PRD.md` §6.1.1 (Substrate), §6.1.2 row L5 (registry), §6.1.5 (cross-cutting invariants), §8 case 1 (skill prompt-injection), §8 case 6 (nested harness); `docs/falsifiers.md` indirectly via §12 |

**Context**: Threat-model produced 4 PRD-grain CRITICAL findings plus 3 cross-cutting requirements gaps that v1 did not declare. Each finding traces to a load-bearing security property the v1 PRD assumed but did not specify, allowing implementation to pick silently. Umbrella entry rather than 8 micro-entries because the findings share a structural shape: "v1 declared a boundary; v1.1 specifies what enforces the boundary."

**Options considered**:
A. One DECISIONS entry per finding (T-001..T-010, CC-1, CC-2 = 8 entries). Each has full context/options/decision/consequences. Highest provenance fidelity; substantial doc bloat.
B. Single umbrella entry referencing the threat-model artifact for the per-finding detail (D-08 covers all 8). Provenance preserved by cross-link to threat-model doc; DECISIONS stays scannable.
C. Group by trust boundary (credential vault entry, policy gate entry, memory entry, attestation entry). Middle-ground; harder to pattern-match against the threat-model artifact.

**Decision**: B. The threat-model doc is the authoritative per-finding record; DECISIONS captures the umbrella architectural commitment that the v1 was missing those guarantees and v1.1 adds them.

**Specific guarantees added in v1.1** (each cited in PRD with `(per T-XXX, D-2026-04-26-08)`):

1. **Policy gate fails closed on gate-process death** (CC-1). PRD §6.1.1 new bullet. Gate panic / crash halts all tool dispatch until restart; no fallback to "execute without gate."
2. **API key in-memory zeroization on session end and panic** (CC-2). PRD §6.1.1 credential-vault bullet. Constrains OQ-1 language choice (Rust `zeroize` viable; TS not viable for this surface without native bindings).
3. **apiKeyHelper script content-hash pinned, sandboxed network egress** (T-002). PRD §6.1.1 credential-vault bullet. Helper hash recorded in `CredentialRecord`; mismatch refuses invocation. Helper runs under sandbox-exec profile denying network egress except to documented credential endpoints. Helper is opt-in only.
4. **Cross-session memory presented as untrusted-by-default; quarantine workflow** (T-007). PRD §6.1.1 auto-memory bullet + §6.1.5 new invariant. Model writes land in quarantine; promotion to canonical sidecar requires TTY operator confirmation.
5. **Exception tables operator-runtime-loaded, not distributed, not model-readable** (T-009). PRD §6.1.5 escape-hatch invariant updated. Ship empty by default; operator populates per-project.
6. **Attestation provenance separation: TTY input vs model emit** (T-001). PRD §6.1.5 new invariant + §8 case 1 critical refinement. Gate-bypass predicates require operator-typed attestation; model-emitted text rejected as evidence.
7. **Nested-harness detection uses out-of-band signals, fail-closed on inheritance failure** (T-004). PRD §8 case 6 refined. Lockfile + process-tree walk, not env-vars alone.
8. **Reward-loop registry writes go through operator review queue with signed provenance** (T-010). PRD §6.1.2 row L5 updated. LEARN proposes; promotion requires TTY confirmation; each mutation carries session ID, model-output trigger, score, confirmation timestamp.

Audit-log integrity acknowledgment (T-003): the audit log is anti-forgetfulness and anti-drift, NOT anti-Sybil against the operator-as-adversary in v0.1 single-operator scope. PRD §6.1.1 audit-log bullet says so explicitly. F-Cuttle-DISABLE expanded (per D-2026-04-26-09) to count chain-rotation events.

**Consequences**:

- 8 distinct PRD requirements added or sharpened in v1.1.
- TDD scope grows: §2 (config) gains apiKeyHelper hash extension; §3 (policy gate) gains TTY-vs-model-emit primitive, supervisor/restart contract, registry review queue, nested-harness lockfile; §4 (sandbox) gains apiKeyHelper sandbox profile; §5 (audit log) gains attestation-provenance field, tamper-chain scheme decision; §6 (memory) gains quarantine layout.
- v0.1 implementation language constraint tightens (CC-2 zeroization).
- Some of these guarantees are operationally observable; F-Cuttle-DISABLE / F-Cuttle-MEMORY-DRIFT (per D-2026-04-26-09) become live monitors.

---

## D-2026-04-26-09: Pipeline expansion (privacy skill in REVIEW-2; falsifier set additions)

| Field   | Value                                                                                                                       |
| ------- | --------------------------------------------------------------------------------------------------------------------------- |
| Date    | 2026-04-26                                                                                                                  |
| Status  | Accepted                                                                                                                    |
| Source  | `docs/threat-model-prd-v1.md` CC-3, T-006, T-007; `docs/output-quality-prd-v1.md` OQ-FIX-5 (telemetry-ACL forward-ref)      |
| Affects | `docs/PRD.md` §11 (Pipeline), §12 (Sealed falsifier pre-registration), §6.1.6 (telemetry surface ACL + privacy forward-ref) |

**Context**: Two strands collapse into one entry: (a) the v1 §11 pipeline omitted `privacy` skill, but v1 §6.1.6 ships `cuttle telemetry` (workflow-shape data) and §6.1.1 audit log captures content digests, both of which trigger `privacy` per global CLAUDE.md mandatory-skills table. (b) The v1 §12 falsifier set covered the bedrock thesis (F-Cuttle-DISABLE, F-Cuttle-BEDROCK, F-Cuttle-SUBSTRATE, F-Cuttle-OPTION-C) but did not cover the L1 snapshot mechanic or the cross-session memory promotion mechanic, both of which are load-bearing for v1.1's new invariants.

**Options considered**:
A. Add `privacy` to §11 only; leave falsifier set unchanged. Closes the pipeline gap but leaves the new v1.1 invariants without falsifier predicates.
B. Add `privacy` to §11 + add F-Cuttle-SNAPSHOT-DRIFT and F-Cuttle-MEMORY-DRIFT to §12 + expand F-Cuttle-DISABLE to cover audit-log chain re-keying. Closes both gaps in one revision pass.
C. Defer falsifier additions to TDD. Violates the framework's pre-seal-before-v0.1-ship discipline; falsifiers should be PRD-grain so they can be sealed at the end of the PRD pipeline.

**Decision**: B. Both gaps are PRD-grain; both close cleanly in v1.1. Privacy skill added to REVIEW-2 step in §11 step 3, scoped to telemetry surface (§6.1.6), audit-log content digests (§6.1.1), and cross-session memory promotion (§6.1.5). Falsifier set in §12 grows by 2 (F-Cuttle-SNAPSHOT-DRIFT, F-Cuttle-MEMORY-DRIFT) and F-Cuttle-DISABLE expands per D-2026-04-26-08.

**Consequences**:

- PRD §11 step 3 lists privacy alongside legal-review and threat-model.
- PRD §12 grows to 6 first-draft falsifier predicates (was 4).
- PRD §6.1.6 forward-refs to §11 step 3 (privacy review) and to TDD §5 (aggregation requirement).
- Privacy skill becomes a Cuttle-pipeline obligation, not just a global mandatory-skill trigger.
- N, M, R thresholds for the new falsifiers set in TDD §3.

---

## D-2026-04-26-10: Falsifier predicates cross-referenced as architectural fitness functions (Ford / Parsons / Kua)

| Field   | Value                                                                                               |
| ------- | --------------------------------------------------------------------------------------------------- |
| Date    | 2026-04-26                                                                                          |
| Status  | Accepted                                                                                            |
| Source  | `process/martin-fowler-input.md` Source 2 (Building Evolutionary Architectures, foreword by Fowler) |
| Affects | `docs/PRD.md` §12 (Sealed falsifier pre-registration)                                               |

**Context**: Cuttle's §12 sealed falsifier predicates inherit terminology from research methodology (`framework_development_methodology.md:39-48`). Ford / Parsons / Kua's "architectural fitness function" concept names the same idea in industry-standard vocabulary: an objective, machine-evaluable assessment of an architectural characteristic, evaluated continuously through tests/metrics/monitoring/logging in the build. Adopting the cross-reference does not change Cuttle's discipline; it lets the predicates land in a conceptual slot that engineers reading the PRD already know.

**Options considered**:
A. Rename §12 falsifiers to fitness functions throughout. Loses the framework-methodology lineage; breaks the cross-reference to `framework_development_methodology.md`.
B. Cross-reference. Keep "sealed falsifier" as the primary name (preserves framework-methodology lineage), add "fitness function" as the parenthetical industry equivalent in §12 introduction.
C. Use both names interchangeably. Confusing.

**Decision**: B. §12 introduction adds: "Each predicate is a Ford / Parsons / Kua _architectural fitness function_ (`process/martin-fowler-input.md` Source 2): an objective integrity assessment of an architectural characteristic, evaluated continuously rather than at one-off review."

**Consequences**:

- §12 gains a one-line cross-reference; the per-predicate entries are unchanged.
- Cuttle's §12 predicates become discoverable as fitness functions to readers who arrive from the evolutionary-architecture literature.
- Framework-side note (handoff path #4): the framework's §10.3 sealed-falsifier discipline is one instance of a broader convergence (CI-style continuous evaluation of architecture). Logged for `framework_external_corroboration.md`.

---

## D-2026-04-26-11: Domain-primitives invariant added to §6.1.5 (Johnsson / Deogun / Sawano, "Secure by Design")

| Field   | Value                                                                                                                                  |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Date    | 2026-04-26                                                                                                                             |
| Status  | Accepted                                                                                                                               |
| Source  | `process/martin-fowler-input.md` Source 3 (Secure by Design, Manning 2019, foreword by Fowler)                                         |
| Affects | `docs/PRD.md` §6.1.5 (Cross-cutting harness invariants), §10 OQ-1 (language choice security argument), `docs/falsifiers.md` indirectly |

**Context**: PRD v1.1 already declared two structural-typing invariants in §6.1.5: attestation-provenance separation (T-001) and cross-session memory promotion (T-007). Both implicitly need a type-system primitive to enforce: today the PRD says "the gate distinguishes TTY-input from model-emit" but does not constrain how. Secure-by-Design's domain-primitives pattern is the canonical answer: wrap raw types (String, [u8], Path) in domain-meaningful types whose construction enforces invariants. This catches the v1.1 attestation-provenance and memory-quarantine invariants at the type system, not at runtime.

**Options considered**:
A. Leave the invariants as runtime predicates the gate evaluates. Works at v0.1 grain but pushes correctness onto careful coding rather than into the type system. Drift risk: a future TDD or implementation choice may quietly drop the runtime check.
B. Adopt Secure-by-Design domain primitives as a §6.1.5 invariant. Trust-boundary-crossing values are constructed only through domain-primitive types whose constructors enforce invariants. Raw `String` / `[u8]` / `int` forbidden at trust-boundary surfaces. Concrete v0.1 primitives enumerated in source artifact.
C. Defer to TDD §2 (data model). PRD-grain decision is: "the type system does the work, not the runtime." TDD picks the specific types.

**Decision**: B + C. PRD-grain commitment: trust-boundary-crossing values are constructed through domain primitives (Johnsson / Deogun / Sawano pattern) that enforce invariants at construction. TDD enumerates the types. PRD §6.1.5 cites the source artifact for v1.2 candidates (`ApiKey`, `AttestationBody`, `HelperHash`, `LockfilePath`, `TierClassification`, `OperatorAuthoredText` vs `ModelAuthoredText`).

**Consequences**:

- §6.1.5 gains a new "Domain primitives at trust boundaries" invariant.
- OQ-1 (language choice) gains an explicit security argument: TS structural typing makes domain-primitive enforcement weaker than Rust newtypes / Go named types. Combined with v1.1 CC-2 zeroization argument, the OQ-1 deliberation now leans Rust > Go > TS for security-load-bearing reasons (separate from velocity considerations).
- TDD §2 (config / data model) gains a "domain primitive enumeration" subsection.
- T-001 and T-007 closures move from "runtime predicate" to "type-system + runtime predicate." Defense-in-depth at the type layer.
- Implementation cost flagged: domain primitives push complexity into the type system; PRD §9 Constraints already names "predicate maintenance cost" (per Carlos $100M/yr anchor); analogous "type system maintenance cost" applies. Acceptable trade.

---

## D-2026-04-26-12: Continuous-threat-modeling framing made explicit in §1 bedrock 1 (Mohan / Gumbley)

| Field   | Value                                                                                                                                     |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Date    | 2026-04-26                                                                                                                                |
| Status  | Accepted                                                                                                                                  |
| Source  | `process/martin-fowler-input.md` Source 1 (Mohan & Gumbley, "Threat Modeling Guide for Software Teams", 2025, on Fowler's site)           |
| Affects | `docs/PRD.md` §1 (Summary), §13 / §14 (framework framing); `framework_external_corroboration.md` (sidecar to be created, handoff path #4) |

**Context**: PRD v1 §1 framed bedrock 1 as "deterministic security: every tool invocation routes through a deny-by-default policy gate. No model is in the policy loop." The Mohan/Gumbley 2025 article hosted on Fowler's site articulates the same architectural argument at the threat-modeling layer for human dev teams: "integrate threat modeling regularly, like continuous integration for security ... bite-sized, closely tied to what your team is working on right now." Cuttle's policy gate IS this argument applied at the per-tool-call grain for LLM agents. Naming this convergence explicitly in §1 narrows Cuttle's contribution claim from "novel architecture" to "novel application of an industry-converged principle" (the same shape as D-2026-04-26-01 with Carlos at the CI layer).

**Options considered**:
A. Leave §1 as-is. Convergence implicit; reader has to construct the cross-reference.
B. Add a §1 sentence cross-referencing Mohan/Gumbley. Narrows the contribution claim; mirrors D-2026-04-26-01's Carlos cross-reference.
C. Build a new §15 "External corroboration" listing all three independent voices (Carlos, Mohan/Gumbley, Ford/Parsons/Kua). More structural; lets future additions land cleanly.

**Decision**: B + C. Add Mohan/Gumbley cross-reference to §1 paragraph 4 (where post-rectification framing already names Carlos). Add new §15 "External corroboration" enumerating the three independent voices and the convergent thesis (substrate beats ceremony). The convergence is now load-bearing for Cuttle's contribution claim and deserves its own structural home.

**Consequences**:

- §1 paragraph 4 grows by one sentence; Carlos and Mohan/Gumbley now cited together.
- New PRD §15 "External corroboration" lists Carlos, Mohan/Gumbley, Ford/Parsons/Kua with one-line summary each.
- Framework-side: `framework_external_corroboration.md` sidecar (handoff path #4) becomes the canonical home for this list as it grows. Cuttle's §15 is a snapshot; the framework sidecar is the live record.
- Cuttle's claim register: "novel substrate-native form of an industry-converged principle." Even narrower than D-01's "novel application." Defensibility increases.
