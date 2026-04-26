CONFIDENTIAL: This document contains detailed security findings. Handle per your organization's data classification policy. This is AI-assisted analysis and requires human expert review before use in security decisions or compliance.

# Threat Model Delta: PRD v1.2 -> v1.3

| Field  | Value                                                                                                                                                 |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Target | `docs/PRD.md` v1.3                                                                                                                                    |
| Method | Delta check of v1.3 against the adversarial-review artifact `docs/adversarial-review-prd-v1.2.md`. Verifies the 10 PRD-grain findings landed cleanly. |
| Source | D-2026-04-26-13 (umbrella).                                                                                                                           |

## Adversarial finding -> v1.3 status

| ID    | Title                                                     | Disposition                 | v1.3 status                                                                                                               |
| ----- | --------------------------------------------------------- | --------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| WV-01 | Domain-primitive constructor authorization                | TDD-deferred (cross-linked) | **Cross-linked.** §6.1.5 domain-primitives bullet adds capability-scoping requirement; TDD §2 specifies.                  |
| WV-02 | Lockfile TOCTOU                                           | TDD-deferred                | **Deferred.** §8 case 6 already names lockfile detection; authentication mechanism left to TDD §3.                        |
| WV-03 | Audit-log content digest as fingerprint side-channel      | CRITICAL                    | **CLOSED at PRD-grain.** §6.1.1 audit-log bullet: per-tool `secret_bearing` flag; metadata-only logging for tagged tools. |
| WV-04 | Reward-loop registry signing-key disclaimer               | High                        | **CLOSED at PRD-grain.** §6.1.2 row L5: signing key is operator-owned; chain anti-forgetfulness/anti-drift only.          |
| WV-05 | Skills-loader strip-list maintenance contract             | High                        | **CLOSED at PRD-grain.** §6.1.1 skills-loader bullet: allowlist-shaped + fail-closed on unknown Unicode categories.       |
| WV-06 | TTY-provenance reduces to operator-fatigue-keypress       | CRITICAL                    | **DISCLAIMED at PRD-grain.** §6.1.5 attestation-provenance limitation disclaimer + F-Cuttle-FATIGUE in §12.               |
| WV-07 | Memory-quarantine social-engineering bypass               | CRITICAL                    | **DISCLAIMED at PRD-grain.** Same §6.1.5 disclaimer + F-Cuttle-FATIGUE.                                                   |
| BP-01 | Single-process load-bearing assumption                    | Structural                  | **OPENED.** New OQ-11 in §10. v0.1 may ship single-process; revisit at v0.2 latest.                                       |
| BP-02 | Fitness functions aspirational at v0.1                    | Structural                  | **HEDGED.** §12 introduction: predicates are data only at v0.1; evaluator is TDD-grade.                                   |
| BP-03 | Cherry-picked corroboration                               | Framing                     | **CLOSED.** §15 new paragraph names dissenting precedents (Lampson, Mickens/Geer).                                        |
| BP-04 | Backup/restore as trust boundary unmodeled                | PRD-grain                   | **CLOSED.** New §8 case 9: state-coherence file + `--restored-from-backup` operator acknowledgment.                       |
| BP-05 | Keychain prompt-fatigue interaction with fail-closed gate | PRD-grain                   | **CLOSED at PRD-grain.** §6.1.1 credential-vault bullet: rate-budget + alternative + named cross-purposes.                |
| BP-06 | PII in audit log scope unaddressed                        | Structural                  | **OPENED.** New OQ-12 in §10. Resolved by TDD §5 + privacy review.                                                        |
| BP-07 | Cross-domain framing                                      | Framing                     | **CLOSED.** §15 corroboration recast as "convergent precedent, not direct validation."                                    |

Summary: 10 PRD-grain findings closed or appropriately disposed. 2 TDD-deferred with cross-link. 2 CRITICAL re-opens (WV-06, WV-07) addressed by HONEST DISCLAIMER + falsifier rather than by claimed-closure (which would be dishonest).

## v1.3 attack surface delta

| Edit                                   | New surface?                                                                                                                                                            |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| §6.1.1 audit-log `secret_bearing` flag | Surfaces tool-registration tagging as a load-bearing trust boundary. TDD §5 must specify safe-by-default posture for unknown tools (treated as secret-bearing).         |
| §6.1.1 skills-loader Unicode allowlist | Surfaces the allowlist itself as a maintenance burden; fail-closed default is conservative.                                                                             |
| §6.1.1 Keychain rate-budget            | Surfaces a new operator-fatigue surface (Keychain prompt rate); v0.1 mitigation TBD.                                                                                    |
| §6.1.5 attestation disclaimer          | NO new surface; reduces over-claimed scope.                                                                                                                             |
| §6.1.5 constructor capability-scoping  | Surfaces the capability-grant graph as an audit target; TDD §2 specifies.                                                                                               |
| §8 case 9 state-coherence file         | Surfaces a new file `~/.cuttle/state-coherence.json` whose own integrity matters. Recursion warning: TDD must specify how the state-coherence file is itself protected. |
| §10 OQ-11 process isolation            | NO new surface; opens design question.                                                                                                                                  |
| §10 OQ-12 PII posture                  | Opens scope question that will produce new design constraints in TDD.                                                                                                   |
| §12 F-Cuttle-FATIGUE                   | Surfaces per-attestation logging of model context (privacy-sensitive; OQ-12 must address).                                                                              |
| §12 fitness-functions hedge            | NO new surface; corrects prior over-claim.                                                                                                                              |
| §15 dissent note                       | NO new surface; corrects prior framing.                                                                                                                                 |

3 net-new TDD-grain sub-surfaces flagged: tool-registration tagging contract (new), state-coherence file integrity (new, recursive), per-attestation model-context logging (new, privacy-sensitive). Logged for TDD inheritance.

## v1.3 verdict

PRD v1.3 closes or appropriately disposes 12 of 14 adversarial findings (10 PRD-grain closures + 2 TDD-deferred with cross-link). The 2 CRITICAL re-opens (WV-06, WV-07) are HONESTLY DISCLAIMED rather than claimed-closed; the v0.1 ship narrows to "operator-fatigue-keypress at the per-attestation grain is acknowledged unsolved; F-Cuttle-FATIGUE makes it empirically falsifiable; TDD §3 may explore UX-grain primitives but they are not v0.1 commitments." This is the most honest framing available under v1.2's design constraints.

PRD v1.3 is now ready for pruning to v3 (handoff path #3) per the framework's "third version is shorter and trustworthy" discipline. Adversarial review's own Step 8 methodology has been applied; further adversarial-attack passes (Codex/Gemini external duel) remain available before v3 seal but are no longer required for v0.1-grade defensibility.
