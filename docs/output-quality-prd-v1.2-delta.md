# Output-Quality Delta: PRD v1.1 -> v1.2

| Field            | Value                                                                                                                                    |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Target           | `docs/PRD.md` v1.2                                                                                                                       |
| Method           | Delta check on v1.1 -> v1.2 edits only. Full v1 -> v1.1 delta at `docs/output-quality-prd-v1.1-delta.md`.                                |
| Companion        | `docs/threat-model-prd-v1.2-delta.md`                                                                                                    |
| Source decisions | D-2026-04-26-10 (fitness-functions cross-ref), D-2026-04-26-11 (domain primitives), D-2026-04-26-12 (continuous-threat-modeling framing) |

## v1.1 -> v1.2 edit-by-edit checklist

| Edit                                                         | Output-quality status                                                                                                                                                                                   |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| §1 paragraph 4: Mohan/Gumbley + Ford cross-references        | PASS. Carries provenance to specific quotes in source artifact. Forward-refs §15 and D-12 explicitly. Risk: appeal-to-authority feel; mitigated by cross-link to source artifact verbatim quotes.       |
| §6.1.5 new "Domain primitives at trust boundaries" invariant | PASS. Names six concrete v0.1 candidates with T-### / CC-# trace. Cites Manning book + foreword. Adds OQ-1 security argument explicitly. Density appropriate for PRD-grain; TDD will instantiate types. |
| §12 introduction: fitness-functions cross-reference          | PASS. Single short paragraph; preserves "sealed falsifier" primary name; cites D-10 + source artifact.                                                                                                  |
| §15 new "External corroboration" section                     | PASS. Four-row table with provenance trace. Explicit narrowing of contribution claim ("substrate-native form of converged principle"). Honest framing per `framework_development_methodology.md:18-30`. |

## NICE-TO-HAVE items from v1 review: status check

These were carried forward at v1.1 delta. v1.2 partially addresses some.

| ID       | v1 finding                                               | v1.2 status                                                                                                                                                                       |
| -------- | -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| OQ-NTH-1 | "implementation existence proof" lands cold in §1        | NOT-ADDRESSED. Carries forward.                                                                                                                                                   |
| OQ-NTH-2 | "structurally lower-risk" weak in §4 reason 2            | NOT-ADDRESSED. Carries forward.                                                                                                                                                   |
| OQ-NTH-3 | OQ-7 (public name) is meta-question                      | NOT-ADDRESSED. Carries forward.                                                                                                                                                   |
| OQ-NTH-4 | Asymmetric Option C source-link in §6.1.3 vs DECISIONS   | NOT-ADDRESSED. Carries forward.                                                                                                                                                   |
| OQ-NTH-5 | §12 falsifier predicates have identical rhetorical shape | PARTIAL. Two new falsifiers (added v1.1) carry distinct shape. v1.2 §12 introduction adds fitness-functions framing that recontextualizes the predicates but doesn't reword them. |

## Cross-doc consistency on v1.2

- Every PRD v1.2 edit that cites D-10 / D-11 / D-12 resolves to a real DECISIONS entry.
- Every cross-reference to `process/martin-fowler-input.md` resolves (Source 1, 2, 3 sections present).
- §15 source attributions match `process/martin-fowler-input.md` "Convergent thesis" table.
- §6.1.5 domain-primitive enumeration matches the source-artifact concrete-candidates list.
- §12 introduction's "process/martin-fowler-input.md Source 2" anchor resolves.

## Vague-language scan on v1.2 additions

- §1 added sentence is dense but every clause carries weight (industry/year/source for each citation, then the LLM-agent-substrate-native framing). OK.
- §6.1.5 new bullet names six concrete primitives with T-### maps. Specific.
- §12 new paragraph: two sentences, both load-bearing. OK.
- §15 four-row table: each row has source/domain/claim/cross-ref. No filler.

No filler flagged.

## Hedging calibration on v1.2

- §1 narrowed claim from "novel architecture" -> "substrate-native form of converged principle" is the right direction (sharper, more defensible).
- §15 closing sentence "does not weaken Cuttle's contribution, it sharpens it" earns the assertion: the table demonstrates the convergence is real.
- §6.1.5 domain-primitives invariant uses strong language ("forbidden at trust-boundary surfaces") with the type-system mechanism backing it.

## Verdict

PRD v1.2 closes none of the carried-forward NICE-TO-HAVE items but introduces zero BLOCK or FIX-BEFORE-V2 items of its own. The Fowler-pass edits are additive and well-cross-referenced. Adversarial review (handoff path #2 against PRD v2) inherits a doc that's now externally-anchored against three converging industry voices.
