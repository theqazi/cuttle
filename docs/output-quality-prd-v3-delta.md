# Output-Quality Delta: PRD v1.3 -> v3 (pruning pass)

| Field  | Value                                                                                                                                                       |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Target | `docs/PRD.md` v3                                                                                                                                            |
| Method | Delta check on pruning. Verifies the rewrite did not introduce hedge-piling, did not lose information density, and tightened the §11 versioning convention. |
| Source | D-2026-04-26-14.                                                                                                                                            |

## Pruning targets vs output-quality bar

| Pruning target                      | Output-quality status                                                                                                                                                                                                                       |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| §0 Status field                     | PASS. Single line conveys version + pending-step + pipeline position. Prior-version commit chain accessible via handoff + git history.                                                                                                      |
| §1 paragraph 4 corroboration cite   | PASS. Four-voice convergence is named (one phrase) with forward-ref to §15 table. Reader hits §1 paragraph 4 with a teaser, not the full citation; §15 carries the load. Cleaner narrative arc.                                             |
| §11 pipeline step list              | PASS. Removed the now-completed "adversarial review → v2" and "PRD pruning → v3" steps. Pipeline reflects what's left, not what's done. v3 reader sees the forward path: TDD → review-cycle → DESIGN → API → Implementation.                |
| §11 versioning convention           | PASS. The new convention paragraph explicitly walks through the actual session-4 history (v1.x → v2 → v3) so a future reader does not get confused that the doc is labeled v3 while §11 originally predicted "v2 after adversarial review". |
| §12 falsifier cross-references      | PASS. Predicate parentheticals tightened (D-XX cited once not twice). Each predicate still cites source (T-XXX, BP-XX, or D-XX) for full provenance.                                                                                        |
| §15 dissenting-precedents paragraph | PASS. Tightened from 4 sentences to 3; the load-bearing claim ("convergent-precedent, not direct-validation") preserved verbatim.                                                                                                           |

## Hedging calibration on v3

- §1 paragraph 4: pruning replaced "Mohan & Gumbley's 'integrate threat modeling regularly, like continuous integration for security' (Thoughtworks, hosted on martinfowler.com 2025), and at the architecture-governance layer in Ford / Parsons / Kua's architectural fitness functions (Building Evolutionary Architectures, foreword by Fowler)" with "four independent industry voices converge on the substrate-vs-ceremony shape (full table at §15)." This is the right level of hedge: §1 reader gets the existence claim, §15 reader gets the evidence. Less is more credible because §1 is no longer name-dropping; it's making a structural claim and offering provenance one click away.
- §11 versioning convention: the new paragraph honestly describes the actual revision history rather than the original prediction. This is a small hedging upgrade; the original convention sentence was correct-in-spirit but wrong-in-fact (the file is labeled v3, not "v3 after pruning per the original list"). Honest framing.
- All other sections: hedging unchanged. v3 preserves the v1.3 hedges (attestation-provenance limitation disclaimer, §15 dissenting-precedents, fitness-functions aspirational hedge) verbatim.

## Vague-language scan on v3

The pruning pass did NOT introduce new prose; it removed redundant prose. So the v3 vague-language scan inherits the v1.3 verdict (no filler flagged) modulo:

- §0 status field new sentence: "Sealed-falsifier pre-registration at `docs/falsifiers.md`. Pending FIX-DOCS + system-design + api-design before implementation. Karpathy review (handoff path #2) parallel-stream." Three concrete next-steps named. No filler.
- §1 paragraph 4 new phrase: "four independent industry voices converge on the substrate-vs-ceremony shape (full table at §15)." Specific count + named comparator + forward-ref. No filler.
- §11 versioning convention paragraph: explicitly traces the actual revision history. No filler.

## Cross-doc consistency on v3

- All v1.3 D-XX cross-references resolve to DECISIONS entries (D-01..14). ✓
- §12 cross-references `docs/falsifiers.md` which now exists. ✓
- §15 cross-references `process/carlos-arguelles-input.md` and `process/martin-fowler-input.md`. ✓
- §11 versioning-convention paragraph cites session-4 actual history; v1, v1.1, v1.2, v1.3 commits all present in git log. ✓
- D-14 in DECISIONS cross-references `framework_methodology_document.md:34, 72` (the discipline anchor) + handoff v0.7 path #1 (where pruning was scoped). Both resolve. ✓

## Verdict

PRD v3 passes the output-quality bar. Pruning was conservative (rewording, not rescoping), preserves all PRD-grain commitments, tightens §11 versioning convention to match actual revision history, and creates `docs/falsifiers.md` per SC-6.

The doc is ready for FIX-DOCS at end of REVIEW-1 + REVIEW-2 (which run against PRD + TDD).
