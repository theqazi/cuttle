# Cuttle: Sealed Falsifier Pre-Registration

| Field            | Value                                                                                                                                                                                               |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Status           | **PRE-SEAL DRAFT.** Refinements (threshold values N, M, R, R_F, X) permitted up to v0.1 ship; structural changes to predicate identity or measurement source forbidden after seal.                  |
| Seal date        | TBD (triggered at v0.1 ship per `framework_development_methodology.md:39-43`)                                                                                                                       |
| Author           | Mohammed Qazi (sirajuddin.qazi@gmail.com)                                                                                                                                                           |
| Source PRD       | `docs/PRD.md` v3 §12 (D-2026-04-26-14)                                                                                                                                                              |
| Source decisions | D-2026-04-26-06 (F-Cuttle-DISABLE seed), D-09 (F-Cuttle-DISABLE expanded + F-Cuttle-SNAPSHOT-DRIFT + F-Cuttle-MEMORY-DRIFT seed), D-13 (F-Cuttle-FATIGUE seed)                                      |
| Methodology      | `framework_development_methodology.md:39-48`. Each load-bearing claim has an associated operationalized predicate that, if observed, forces public retraction.                                      |
| Cross-reference  | Each predicate is also a Ford / Parsons / Kua _architectural fitness function_ (per D-2026-04-26-10). v0.1 ships predicates as data; the automated evaluator is TDD-grade scope (per BP-02 / D-13). |

---

## Pre-registration discipline

Per the framework's sealed-falsifier discipline (paper §C immutability):

- Each predicate names a load-bearing claim, an operationalized predicate evaluable from the audit log + telemetry, and the consequence of observation (which claim is refuted, fully or partially).
- Threshold values (N, M, R, R_F, X) are first-draft. They are refined in TDD §3 against operational baselines from v0.1 dogfood week's audit log.
- After v0.1 ship: predicates become immutable. Threshold tightening forbidden (would let a failing claim be retroactively rescued by raising the bar). Threshold loosening permitted only with public retraction notice attached.
- Detection mechanism: the audit log + telemetry surfaces capture the events. v0.1 evaluation is operator-manual post-week; v0.2+ ships the automated evaluator.

## Falsifier predicates

### F-Cuttle-DISABLE

| Field              | Value                                                                                                                                                                                                                                                                                               |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Load-bearing claim | Bedrock 1: harness mechanics > advisory skills (substrate-vs-ceremony thesis).                                                                                                                                                                                                                      |
| Predicate          | During v0.1 dogfood week, ANY of: (a) operator disables ≥1 harness-mechanic gate; OR (b) operator re-keys the audit-log tamper-evident chain; OR (c) operator invokes `--restored-from-backup` resetting chain heads (per §8 case 9); OR (d) operator invokes `--override-snapshot-block` ≥3 times. |
| Measurement        | Audit-log entries: gate-disable events, chain-rotation events, `--restored-from-backup` high-trust events, `--override-snapshot-block` events.                                                                                                                                                      |
| Consequence        | Bedrock thesis partially refuted: gates surviving as advisory in practice. Cuttle's v0.1 ship narrative on bedrock 1 must add operator-discipline-dependence disclaimer. F-Cuttle-FATIGUE may also fire and compound.                                                                               |
| First-draft N      | 1 event of (a), (b), or (c). 3 events of (d) within dogfood week.                                                                                                                                                                                                                                   |
| TDD refinement     | TDD §3 sets the (d) override-frequency threshold against operational baseline.                                                                                                                                                                                                                      |

### F-Cuttle-BEDROCK

| Field              | Value                                                                                                                                                                                                                            |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Load-bearing claim | Bedrock 1 + 2 collectively: substrate-native form delivers measurably different operator behavior from advisory-skill form.                                                                                                      |
| Predicate          | Across N≥20 SYSTEM-tier sessions, operator skip rate on Cuttle's harness mechanics is statistically indistinguishable from operator skip rate on toolkit's skills/hooks.                                                         |
| Measurement        | Cuttle audit-log gate-fire vs gate-bypass rates; toolkit's `~/.claude/reward-loop/session-scores.md` skill-skip events. Two-sample comparison (Mann-Whitney U or equivalent; TDD §5 picks).                                      |
| Consequence        | Bedrock thesis rejected outright (not partial). The substrate-native form fails to change operator behavior; the v0.1 implementation existence proof retracts to "the substrate exists; whether it matters is empirically open." |
| First-draft N      | N=20 SYSTEM-tier sessions; significance threshold p<0.05.                                                                                                                                                                        |
| TDD refinement     | TDD §3 specifies sample-size justification + significance threshold per `framework_development_methodology.md:45`.                                                                                                               |

### F-Cuttle-SUBSTRATE

| Field              | Value                                                                                                                                                                                                                                      |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Load-bearing claim | Substrate-constraint thesis (D-01): the framework's audit-log-async default was a Claude Code workaround, NOT a chosen architecture. Cuttle's pre-execution gate is strictly preferable.                                                   |
| Predicate          | Cuttle's deterministic-pre-execution-gate produces operator workflow disruption rates ≥X×toolkit's audit-log-async rates (measured by abandon-point telemetry from §6.1.6).                                                                |
| Measurement        | Telemetry §6.1.6: abandon-point counts per session for Cuttle. Toolkit baseline: comparable abandon-point recording from session transcripts.                                                                                              |
| Consequence        | Substrate-constraint claim incomplete: the gate IS the bottleneck, not the substrate. The pre-execution-vs-post-execution architectural difference reduces to "trade ergonomics for security" rather than "remove a substrate constraint." |
| First-draft X      | X=2.0 (Cuttle disrupts at least 2× toolkit's rate to falsify; below 2× is consistent with the substrate-constraint claim because some additional friction is expected from per-call gating vs per-session post-hoc audit).                 |
| TDD refinement     | TDD §3 specifies abandon-point definition (every operator action that is started but not completed within session); X recalibrated against v0.1 baseline.                                                                                  |

### F-Cuttle-OPTION-C

| Field              | Value                                                                                                                                                                                                                                                  |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Load-bearing claim | Option C deterministic harness review (§6.1.3): each enumerated rule terminates in machine-checkable predicate plus operator self-attestation; bypass rate is materially lower than the framework's §10.2 default.                                     |
| Predicate          | Across the v0.1 enumerated set (4 rules per `cuttle_v01_option_c_enumeration.md`), attestation-terminating predicate bypass rates are statistically indistinguishable from §10.2 default audit-log-only rates over N≥10 dogfood-week samples per rule. |
| Measurement        | Audit log: per-rule bypass count, per-rule deny count, per-rule attestation acceptance count. Compared against operator's `~/.claude/reward-loop/` skill-skip baseline for the closest-mapping toolkit rule.                                           |
| Consequence        | Option C reduces to §10.2 default; the synchronous-deny framing is theatre. The "machine-checkable predicate plus operator self-attestation" model adds ceremony without changing behavior.                                                            |
| First-draft N      | N=10 dogfood-week samples per rule; significance threshold p<0.05.                                                                                                                                                                                     |
| TDD refinement     | TDD §3 specifies per-rule sampling cadence + bypass-rate normalization across rules with different fire-frequencies.                                                                                                                                   |

### F-Cuttle-SNAPSHOT-DRIFT

| Field              | Value                                                                                                                                                                                |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Load-bearing claim | L1 operator-snapshot mechanic (§6.1.2 row L1): refusing SYSTEM-tier classification on poor-state snapshot is load-bearing for catching low-quality SYSTEM-tier work before it ships. |
| Predicate          | During v0.1 dogfood week: `--override-snapshot-block` is invoked > N times AND post-hoc operator review concludes the override was wrong > M of those times.                         |
| Measurement        | Audit-log `--override-snapshot-block` events; post-week manual operator review (rated wrong / right / debatable per override).                                                       |
| Consequence        | L1 mechanic delivering theatre, not protection. The snapshot prompt prevents nothing; the operator routinely overrides AND those overrides correlate with low-quality outcomes.      |
| First-draft N, M   | N=3 invocations in dogfood week. M=2 wrong of those 3.                                                                                                                               |
| TDD refinement     | TDD §3 specifies operator-review rubric (what counts as "wrong" override) + N + M against operational baseline.                                                                      |

### F-Cuttle-MEMORY-DRIFT

| Field              | Value                                                                                                                                                                                                   |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Load-bearing claim | §6.1.5 cross-session memory promotion invariant: model-authored memory writes land in quarantine; operator promotes via TTY confirmation; cross-session context is untrusted-by-default until promoted. |
| Predicate          | During v0.1 dogfood week, the ratio of (model-authored memory promotions accepted via TTY) to (model-authored memory writes deny+quarantine retained) ≥ R.                                              |
| Measurement        | Audit-log: memory-write events tagged by provenance; promotion-via-TTY events; quarantine-retention events.                                                                                             |
| Consequence        | Cross-session memory promotion mechanic functioning as auto-promote in practice. The untrusted-by-default invariant refuted; T-007 reopens at v0.1-ship-grade.                                          |
| First-draft R      | R=0.8 (operator accepts 80%+ of proposed promotions, suggesting the TTY confirmation has degenerated to muscle-memory `y`).                                                                             |
| TDD refinement     | TDD §3 specifies R against operational baseline. May need normalization against the operator's overall acceptance rate for genuine canonical-memory edits.                                              |

### F-Cuttle-FATIGUE

| Field              | Value                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Load-bearing claim | §6.1.5 attestation-provenance separation invariant: operator-typed attestation reflects operator-INTENT, not just operator-keystrokes-of-model-suggested-text. Per §6.1.5 limitation disclaimer, this is the predicate that empirically falsifies the "type system does the work" claim.                                                                                                                                      |
| Predicate          | During v0.1 dogfood week, the rate of attestation bodies operator-typed AND containing trigram-matchable substrings against model-emitted text in the same conversation turn ≥ R_F.                                                                                                                                                                                                                                           |
| Measurement        | Per-attestation logging at gate-evaluation time captures the model context window snapshot. Trigram tokenizer over both the attestation body and the model output; Jaccard or substring overlap above threshold counts as a "matched" attestation. **Privacy-sensitive surface: the model context window may contain PII per OQ-12; storage and access scoped accordingly.**                                                  |
| Consequence        | Attestation-provenance separation defeated by operator-fatigue-keypress. Option C / escape-hatch evidence pathways collapse to model-driven bypass at the operator-fatigue grain. The §6.1.5 invariant is honest but operationally insufficient; v0.2+ requires UX-grain primitives that extract operator-intent from operator-keystrokes (dynamic prompts; predicate-specific evidence shapes the model cannot pre-suggest). |
| First-draft R_F    | R_F=0.30 (≥30% of attestation tokens trigram-matchable to immediately-prior model output).                                                                                                                                                                                                                                                                                                                                    |
| TDD refinement     | TDD §3 specifies trigram-tokenizer parameters, "same conversation turn" temporal window, and the privacy-sensitive logging contract per OQ-12.                                                                                                                                                                                                                                                                                |

---

## Sealing protocol

Per `framework_development_methodology.md:39-43`:

1. PRD §12 references this file as the canonical predicate registry. ✓ (PRD v3 §12 cross-references `docs/falsifiers.md`.)
2. Each predicate is operationally measurable from declared sources (audit log, telemetry, manual operator review). ✓ (above tables.)
3. Threshold values are first-draft and refined in TDD §3 against operational baselines. ✓ (each predicate names TDD-refinement scope.)
4. At v0.1 ship: the file is digitally signed-and-dated by the operator; subsequent edits require explicit retraction notice on threshold loosens; predicate identity / measurement source / consequence freeze.
5. Post-v0.1 dogfood week: each predicate evaluated; results published in `docs/falsifier-eval-2026-MM-DD.md` (one per dogfood cycle); any predicate that fired triggers the consequence (public retraction of the named claim).

Pre-seal status: **DRAFT**. Threshold refinements expected during TDD pass. Structural shape (7 predicates, 7 load-bearing claims) frozen at v0.1 PRD seal per D-2026-04-26-14.

## What this file does NOT contain

- The automated evaluator (TDD §5; per BP-02 / D-13, v0.1 ships predicates as data, not as live monitors).
- Per-week evaluation results (those land in `docs/falsifier-eval-YYYY-MM-DD.md` artifacts, post-dogfood).
- Predicates that are TDD-grain or operational rather than load-bearing-claim-grain (those live in TDD §5 and are not seal-blocking).
