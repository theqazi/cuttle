# Output-Quality Delta: PRD v1 -> v1.1

| Field            | Value                                                                                         |
| ---------------- | --------------------------------------------------------------------------------------------- |
| Target           | `docs/PRD.md` v1.1 (delta against `docs/output-quality-prd-v1.md`)                            |
| Method           | Delta check. Verifies v1.1 closes the 5 FIX-BEFORE-V2 items + reviews new wording introduced. |
| Companion        | `docs/threat-model-prd-v1.1-delta.md`                                                         |
| Source decisions | D-2026-04-26-07, D-2026-04-26-08, D-2026-04-26-09                                             |

## v1 finding -> v1.1 status

| ID       | v1 title                                                                     | v1.1 status                                                                                                                                                                                                                                                                                   |
| -------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| OQ-FIX-1 | Implementation-level commitments inside §6.1.1 contradict OQ-3 / OQ-4 in §10 | **CLOSED.** Per D-2026-04-26-07. §6.1.1 audit-log bullet softened to "tamper-evident chain (specific scheme is OQ-4: HMAC chain vs Merkle tree, resolved in TDD §5)." §6.1.1 sandbox bullet softened to "macOS process-isolation primitive ... primitive choice is OQ-3." Both OQs stay open. |
| OQ-FIX-2 | Threat-model CC-1, CC-2, CC-3 are PRD-grain requirements gaps                | **CLOSED.** §6.1.1 new "Policy gate failure mode" bullet (CC-1). §6.1.1 credential-vault bullet adds in-memory zeroization invariant (CC-2). §11 step 3 lists privacy alongside legal-review and threat-model (CC-3).                                                                         |
| OQ-FIX-3 | §3.2 unhedged "never" claim                                                  | **CLOSED.** §3.2 paragraph 1 reads "...human-authored commits _with intervening human review_ (the dominant case in enterprise CI) reduce in expectation: the human can author the same destructive sequence, but the commit-to-execute cycle gives them the opportunity to notice."          |
| OQ-FIX-4 | §6.1.2 v0.N column items lack promotion criteria                             | **CLOSED.** §6.1.2 table grew a third column "Promotion trigger (per OQ-FIX-4)". Each v0.N item names a falsifier signal or operationally-measurable threshold (TDD picks N).                                                                                                                 |
| OQ-FIX-5 | §6.1.6 telemetry under-specifies privacy posture                             | **CLOSED.** §6.1.6 paragraph 2 specifies ACL inheritance from audit log, forward-ref to TDD §5 aggregation contract, forward-ref to §11 step 3 privacy review.                                                                                                                                |
| OQ-NTH-1 | "implementation existence proof" lands cold in §1                            | NOT-ADDRESSED-IN-V1.1. NICE-TO-HAVE; not blocking adversarial review.                                                                                                                                                                                                                         |
| OQ-NTH-2 | "structurally lower-risk" weak in §4 reason 2                                | NOT-ADDRESSED-IN-V1.1. NICE-TO-HAVE; carries forward.                                                                                                                                                                                                                                         |
| OQ-NTH-3 | OQ-7 (public name) is meta-question                                          | NOT-ADDRESSED-IN-V1.1. NICE-TO-HAVE; carries forward.                                                                                                                                                                                                                                         |
| OQ-NTH-4 | Asymmetric Option C source-link in §6.1.3 vs DECISIONS                       | NOT-ADDRESSED-IN-V1.1. NICE-TO-HAVE; carries forward.                                                                                                                                                                                                                                         |
| OQ-NTH-5 | §12 falsifier predicates have identical rhetorical shape                     | PARTIALLY ADDRESSED. Two new falsifiers added (F-Cuttle-SNAPSHOT-DRIFT, F-Cuttle-MEMORY-DRIFT) carry distinct sub-claim-pinning language. The original four are unchanged in shape.                                                                                                           |

Summary: all 5 FIX-BEFORE-V2 items closed. 4 of 5 NICE-TO-HAVE items carry forward (acceptable per their severity).

## Vague-language re-scan on v1.1 edits

Spot-checked the v1.1 additions for filler.

- §6.1.1 credential-vault: "TDD §2 (OQ-1 language) constrains the implementation choice to languages with deterministic zeroization or wrapped equivalents (Rust `zeroize`, Go via mlock + explicit overwrite, TS not viable for this surface without native bindings)." Specific. Names actual implementations.
- §6.1.1 sandbox: "primitive choice is OQ-3: `sandbox-exec` is the leading candidate but is on Apple's deprecation path (per T-005); TDD §4 must produce a contingency (Endpoint Security framework, hypervisor-based isolation, or container-via-Apple-Virtualization)." Specific.
- §6.1.5 attestation provenance separation: "model-emitted strings are rejected as evidence even if they would satisfy the predicate's content shape." Sharp.
- §6.1.5 cross-session memory promotion: phrasing is dense but every clause carries weight. OK.
- §8 case 1 "Critical refinement (per T-001, D-2026-04-26-08)": good provenance signaling.
- §8 case 6 "Even with `CUTTLE_NESTED=allow`, if the outer session's policy state cannot be inherited (e.g., lockfile present but unreadable), the inner instance fails closed." Specific edge case named.
- §6.1.2 row L4 added "(N defaults to 2; operator-configurable; persona disagreement surfaces both reports)." Closes the vague-N flag from v1 output-quality.

No filler flagged. v1.1 wording maintains the v1 information-density bar.

## Hedging calibration on v1.1 edits

- §3.2 went from unhedged "never" to specific "with intervening human review (the dominant case)". Sharper, defensible. CLOSED.
- §6.1.1 went from over-committed (HMAC, sandbox-exec) to honestly-deferred (OQ-4, OQ-3 with leading candidates named). Closes silent-decision-making.
- §6.1.5 escape-hatch invariant carries strong "are NOT embedded in the distributed binary, NOT readable by the model, and ship empty by default" language. Confidence tracks evidence (operator-controlled storage is the mechanism that earns the strong language). OK.
- §6.1.1 audit log "anti-forgetfulness and anti-drift; it is **not** anti-Sybil against the operator-as-adversary in v0.1 single-operator scope" is the correct calibration: the original silent assumption that the audit log was anti-Sybil is now explicitly disclaimed. Honest.

## Cross-doc consistency on v1.1

- Every PRD v1.1 edit that cites a D-2026-04-26-07/08/09 number resolves to a real DECISIONS entry now. ✓
- Every PRD v1.1 edit that cites a T-### or CC-# number resolves to a real threat-model entry. ✓
- DECISIONS D-08 §"Specific guarantees added in v1.1" enumerates 8 items; all 8 are present in PRD v1.1 with the cross-reference signal `(per T-XXX, D-2026-04-26-08)`. ✓
- New falsifiers F-Cuttle-SNAPSHOT-DRIFT and F-Cuttle-MEMORY-DRIFT cited in §6.1.2 promotion-trigger column resolve to §12 entries. ✓
- F-Cuttle-DISABLE cited in §6.1.2 row L1 promotion column resolves to the expanded §12 entry. ✓

## v1.1 verdict

PRD v1.1 closes all 5 FIX-BEFORE-V2 items from the v1 output-quality review. New wording is dense, specific, and well-cross-referenced against DECISIONS and threat-model. NICE-TO-HAVE items carry forward without consequence.

Adversarial review (handoff path #3 against PRD v2) inherits a tighter starting point than v1 would have given it.
