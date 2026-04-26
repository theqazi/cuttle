# Output-Quality Delta: PRD v1.2 -> v1.3

| Field     | Value                                                                                                                   |
| --------- | ----------------------------------------------------------------------------------------------------------------------- |
| Target    | `docs/PRD.md` v1.3                                                                                                      |
| Method    | Delta check on v1.3 edits. Verifies adversarial-review wording closes findings without introducing filler/hedge issues. |
| Companion | `docs/threat-model-prd-v1.3-delta.md`                                                                                   |
| Source    | D-2026-04-26-13.                                                                                                        |

## v1.3 edit-by-edit checklist

| Edit                                                | Output-quality status                                                                                                                                                                                                                                                            |
| --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Status field bump v1.2 → v1.3                       | PASS. Cites adversarial-review artifact + commit hash chain.                                                                                                                                                                                                                     |
| §6.1.1 audit-log `secret_bearing` taint annotation  | PASS. Names safe-by-default posture; cites WV-03 and D-13 explicitly. Two-clause final sentence ("not anti-Sybil ... not a cleartext side-channel") symmetrically discloses both limitations.                                                                                    |
| §6.1.1 skills-loader allowlist + fail-closed        | PASS. Names the failure mode (Best-of-N novel injection); names the mechanism (allowlist not denylist); cross-references TDD §2 for the maintenance contract.                                                                                                                    |
| §6.1.1 Keychain prompt-fatigue                      | PASS. Names the cross-purpose explicitly (CC-1 and Keychain at odds); concrete TDD scope (rate-budget + alternative).                                                                                                                                                            |
| §6.1.2 row L5 signing-key disclaimer                | PASS. Inherits the T-003 audit-log language symmetrically; explicit "anti-forgetfulness/anti-drift, NOT anti-Sybil" framing.                                                                                                                                                     |
| §6.1.5 attestation-provenance limitation disclaimer | PASS. The most load-bearing v1.3 edit. Explicitly distinguishes operator-INTENT from operator-FATIGUE-KEYPRESS, names the §15 corroboration as warning-not-validation, names F-Cuttle-FATIGUE as the falsifier. Honest framing per `framework_development_methodology.md:18-30`. |
| §6.1.5 constructor authorization extension          | PASS. Names module-private + capability-scoping; states the failure if absent ("collapses to anyone can call the constructor"); cross-references TDD §2.                                                                                                                         |
| §8 case 9 backup/restore                            | PASS. Names the affected subsystems (audit-log, lockfile, registry, helper-hash, telemetry); names the mechanism (state-coherence file); names the operator-acknowledgment override.                                                                                             |
| §10 OQ-11 process isolation                         | PASS. Names the trade explicitly (cleaner OS boundary vs latency + IPC complexity); cites BP-01 and D-13.                                                                                                                                                                        |
| §10 OQ-12 PII posture                               | PASS. Three explicit options (record-as-is, redact-at-write, refuse-tools); cites BP-06 and D-13; names privacy-skill review involvement.                                                                                                                                        |
| §12 fitness-functions hedge                         | PASS. Explicitly distinguishes "data only at v0.1 ship" from "operationally true at v0.2+." Names the AUTOMATED EVALUATOR as TDD-grade scope; cites BP-02 and D-13.                                                                                                              |
| §12 F-Cuttle-FATIGUE                                | PASS. Operationally measurable predicate (substring-match rate, threshold R_F); names a probable starting threshold (≥30% trigram-matchable); names the privacy-sensitive sub-surface (per-attestation model-context logging) with cross-link to OQ-12.                          |
| §15 dissenting-precedents paragraph                 | PASS. Names the cross-domain extrapolation explicitly ("fifth domain ... not been tested at industry scale"); names two specific dissenters (Lampson; Mickens, Geer); names what F-Cuttle-FATIGUE captures.                                                                      |

## Hedging calibration on v1.3

- §6.1.5 attestation disclaimer is the right level of hedge: it does not over-promise and does not under-promise. v0.1 commits to bytes-provenance separation; v0.1 disclaims operator-intent separation. Honest.
- §15 new paragraph reframes the corroboration table from "validates Cuttle" to "convergent precedent." This is a meaningful narrowing and Wolverine-defensible.
- §12 fitness-functions hedge: the v1.2 D-10 framing was "Cuttle's predicates ARE fitness functions." v1.3 says "predicates are data; evaluator is TDD." This is an honest correction; the D-10 cross-reference still holds because the framing aspires to be a fitness function, but v0.1 does not deliver the eval machinery.

## Vague-language scan on v1.3

Spot-checked the v1.3 additions for filler.

- §6.1.1 audit-log `secret_bearing` bullet: "for tagged tools, only metadata (length, type, success/failure) recorded, NOT a content sha256." Specific list; specific exclusion. OK.
- §6.1.1 Keychain bullet: "rate-budget and an alternative when the budget is exceeded (e.g., session-scoped Keychain handle with explicit re-auth on N>threshold)." Specific candidate mechanism. OK.
- §6.1.5 attestation disclaimer: "TDD §3 may explore UX-grain primitives ... these are not v0.1 commitments." Right level of hedge: names the exploration without committing.
- §8 case 9: "state-coherence.json", "mtime / chain-head digest", "--restored-from-backup". Specific files, specific predicates, specific flag.
- §12 F-Cuttle-FATIGUE: "≥30% of attestation tokens trigram-matchable to immediately-prior model output". Specific threshold candidate.

No filler flagged.

## Cross-doc consistency on v1.3

- Every PRD v1.3 edit citing D-2026-04-26-13 resolves to the new umbrella DECISIONS entry.
- Every PRD v1.3 edit citing WV-XX or BP-XX resolves to a real adversarial-review entry.
- DECISIONS D-13 §"Specific guarantees added in v1.3" enumerates 12 items; all 12 are present in PRD v1.3 with the cross-reference signal.
- F-Cuttle-FATIGUE in §12 is cited in the §6.1.5 disclaimer; cross-link complete.
- §15 dissenting-precedents paragraph cites BP-03 + BP-07 jointly.

## v1.3 verdict

PRD v1.3 closes the adversarial-review punchlist with no BLOCK or FIX-BEFORE-V2 items of its own. The §6.1.5 attestation disclaimer is the most consequential change (narrows v1.2's "type system does the work" claim); the rest are precise technical-requirement additions. Doc is ready for pruning to v3 (handoff path #3): "third version is shorter and trustworthy" per `framework_methodology_document.md:34, 72`.
