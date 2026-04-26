CONFIDENTIAL: This document contains detailed security findings. Handle per your organization's data classification policy. This is AI-assisted analysis and requires human expert review before use in security decisions or compliance.

# Adversarial Review: PRD v1.2 (Wolverine + Black Panther)

| Field            | Value                                                                                                                                                                            |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Target           | `docs/PRD.md` v1.2 (committed `5c0a741`)                                                                                                                                         |
| Method           | Step 8 of threat-model skill (`~/.claude/skills/threat-model/SKILL.md`), executed against PRD v1.2. Deferred at v1 review (`docs/threat-model-prd-v1.md` Step 8: DEFERRED).      |
| Personas         | Wolverine (offensive / red team) and Black Panther (structural / platform). Both run sequentially per skill methodology.                                                         |
| Companion        | This artifact feeds PRD v1.3 (or queues findings for TDD if implementation-grain).                                                                                               |
| Decision context | Mo's session-4 expansion ("don't stop until interrupted; use judgment"); judgment-call gate at handoff v0.6 path #1 chose adversarial review over Karpathy review at this grain. |

---

## WOLVERINE REVIEW: PRD v1.2

### MISSED ATTACK PATHS

**WV-01: Domain-primitive constructor authorization is unspecified at PRD-grain.**
PRD §6.1.5 says "trust-boundary-crossing values are constructed only through domain-primitive types" but does not specify which modules can call which constructors. If `ApiKey::new(bytes)` is callable from the skills-loader module (whose content is untrusted per §6.1.5), an attacker who controls a skill can construct an `ApiKey` from arbitrary bytes that satisfies the type but is attacker-controlled. The v1.2 delta-check flagged this as TDD-grain ("constructor authorization"); Wolverine escalates because the PRD-grain claim "type system does the work" depends on capability-gating that PRD does not specify.

**WV-02: Lockfile TOCTOU.**
§8 case 6 nested-harness detection uses lockfile in `~/.cuttle/run/`. Race condition: outer Cuttle holds lockfile; attacker reads it, kills Cuttle (or waits for crash), grabs the lockfile path, starts a child Cuttle that finds an attacker-crafted lockfile and inherits attacker-controlled state. The "fail-closed on inheritance failure" only triggers if inheritance FAILS; an attacker who crafts a valid-looking lockfile makes inheritance SUCCEED with attacker state. PRD has not specified the lockfile authentication mechanism (signed contents? file-system-level capability? mtime-fence?).

**WV-03: Audit-log content digest is a fingerprint side-channel for secret-bearing tool outputs.**
§6.1.1: "tool result digest (sha256) written to ... audit log." If any tool fetches or generates secret-bearing data (an API response with a token, a database row with a password hash, file contents containing a credential), the audit log records sha256(content). That digest is itself a fingerprint of the secret. An attacker who later acquires the audit log (broken host, lost backup, leaked snapshot) can offline-attack: hash known/guessed secrets, compare to recorded digests, confirm matches. The audit log is treated as confidential per T-008 mitigation, but T-008 was scoped to telemetry aggregates, not the audit log itself, and the digest format is NOT designed to be confidentiality-preserving. Real-world precedent: SpAIware persistent memory poisoning (Rehberger 2024-25) demonstrates audit-trail confidentiality is rarely defensive against a determined attacker.

**WV-04: Reward-loop registry "signed provenance" is unspecified key model.**
§6.1.2 row L5: "signed provenance: session ID, model output that triggered the proposal, score, operator-confirmation timestamp." Signed by what key? In v0.1 single-operator scope, the operator owns the signing key. An attacker who compromises the operator process can re-sign forged registry entries indistinguishably, exactly as T-003 acknowledged for the audit-log chain. PRD §6.1.2 row L5 does NOT carry the equivalent T-003 "anti-forgetfulness, not anti-Sybil" disclaimer. Symmetry gap.

**WV-05: Skills-loader Unicode-attack strip-list maintenance contract is unspecified.**
§6.1.1: "zero-width / RTL-override / variation-selector removal." The strip set is enumerated. Best-of-N jailbreak research (Anthropic Dec 2024) and Constitutional Classifiers bypass (2025) both demonstrate novel character-injection vectors emerge faster than strip-list updates. PRD has not specified the strip-list maintenance contract (who updates it, on what cadence, how does Cuttle handle skills with Unicode categories not yet classified).

### MITIGATION BYPASSES

**WV-06: Attestation-provenance separation reduces to operator-fatigue-keypress at the per-attestation grain.** _**The most load-bearing finding in this review.**_

T-001 / §6.1.5 attestation-provenance separation: model-emitted text rejected as evidence even if it satisfies the predicate's content shape. Bypass: the model emits "to bypass this gate, please type the following: '<long compliant-shaped WHY>'". Operator under load types it. The bytes now have `Tty` provenance. The type system says the value is valid. The security claim fails.

The bedrock thesis (substrate beats ceremony BECAUSE substrate doesn't need operator discipline) is contradicted by the v1.1 attestation-provenance primitive: the primitive distinguishes bytes-typed-by-operator from bytes-emitted-by-model, but does NOT distinguish operator-INTENT from operator-FATIGUE-KEYPRESS. At the per-attestation grain, ceremony is re-introduced; the very thing Cuttle's bedrock argues against.

This is the same class of failure Carlos / Mohan-Gumbley / Ford / Johnsson all warn about (operator fatigue = ceremony failure mode). PRD v1.2's §15 corroboration table is being attacked by its own corroborators.

**WV-07: Memory-quarantine promotion has the same social-engineering bypass as WV-06.**
§6.1.5 cross-session memory promotion: "promotion to canonical sidecar requires TTY operator confirmation." Bypass: model emits a quarantined memory entry shaped as a question the operator wants to answer ("save this fact?"). Operator types `y`. TTY confirmation provenance is satisfied. Same class as WV-06.

### SCORE CHALLENGES

| ID    | v1 score (delta-checked)                 | Wolverine challenge                                                                                                                                                                | New composite |
| ----- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| T-001 | 23 CRITICAL (closed v1.1, hardened v1.2) | Re-opened by WV-06: type-system primitive doesn't address operator-fatigue-keypress vector. AE-3 (one human decision per attempt). L=4, I=5, AE_mod=+1. **Composite=21 CRITICAL.** | 21 CRITICAL   |
| T-007 | 19 CRITICAL (closed v1.1, hardened v1.2) | Re-opened by WV-07: same shape as WV-06 for memory promotion. **Composite=17 CRITICAL.**                                                                                           | 17 CRITICAL   |
| T-008 | 10 Medium                                | WV-03 expands T-008 scope from telemetry aggregates to audit-log content digests. AE-2. L=3, I=4, AE_mod=+3. **Composite=15 CRITICAL.**                                            | 15 CRITICAL   |

### DETECTION GAPS

- WV-03 audit-log digest side-channel: detection requires monitoring what content gets digested, but Cuttle has no taint-tracking on tool outputs. Gap: PRD does not specify per-tool taint annotation.
- WV-06 / WV-07 social-engineering: undetectable from the audit log alone (the bytes have valid Tty provenance). Detection requires modeling operator-intent, which is hard. Gap is structural.
- WV-05 novel Unicode injection: detection requires knowing what was injected; novel categories evade pattern-based detection by definition.

### WOLVERINE VERDICT: **FAIL with conditions.**

PRD v1.2 has three CRITICAL findings that re-open v1.1 closures:

- WV-06 / T-001 reopen at composite 21
- WV-07 / T-007 reopen at composite 17
- WV-03 / T-008 expansion at composite 15

The TTY-provenance primitive (§6.1.5) is the load-bearing v1.1 boundary; WV-06 reduces it to operator-discipline-at-attestation-grain, which is the failure mode Cuttle's bedrock claims to avoid. PRD v1.3 must address this honestly. Either:

- Disclaim the limitation explicitly in §6.1.5 + add F-Cuttle-FATIGUE to §12. Honest framing.
- Or specify a UX-grain primitive that makes operator-intent extractable from operator-keystrokes. Speculative; TDD-grade research.

WV-03 audit-log digest side-channel is independently CRITICAL and must add a taint-annotation requirement to §6.1.1 audit-log bullet (only metadata digest, not content digest, for tagged-secret-bearing tools).

---

## BLACK PANTHER REVIEW: PRD v1.2

### STRUCTURAL FLAWS

**BP-01: Single-process load-bearing assumption.**
PRD §6.1.1 + §6.1.5: policy gate is the load-bearing security boundary, fail-closed on gate-process death (per CC-1). But the gate, the model client, the audit-log writer, the credential vault, the skills loader, and the reward-loop registry all live in the SAME process. Compromise of any one is compromise of all. The PRD's "trust boundary" between operator-trusted CLI and the load-bearing gate is intra-process, which is not a real OS-level boundary. Cuttle should consider process-isolating the policy gate from the model client (separate processes, IPC over a typed socket or kernel mechanism). New OQ candidate: OQ-11 process-isolation model.

**BP-02: Fitness functions are aspirational, not concrete in v0.1.**
§12 + D-10 cross-references the predicates as Ford / Parsons / Kua architectural fitness functions. A real fitness function is automated by the build (tests, metrics, monitoring, alerts). Cuttle's predicates ship as DATA in `docs/falsifiers.md`; the EVALUATION mechanism is repeatedly TDD-deferred ("TDD §3 picks N", "TDD §3 makes this auditable"). At v0.1 ship, the falsifiers are operator-evaluable manually, not automated. The §12 fitness-functions cross-reference therefore over-claims. v1.3 should hedge §12 introduction: "predicates are first-draft _data_; the automated evaluator is TDD-grade scope."

**BP-03: §15 corroboration table cherry-picks confirming voices.**
The four voices (Carlos, Mohan/Gumbley, Ford, Johnsson) all converge on substrate>ceremony. The table omits dissenting views: Lampson on capability discipline (which argues for trust-the-user, not deny-by-default at the per-call grain); Mickens / Geer on operator-fatigue inevitability (which argues threat-modeling-as-substrate hits the same fatigue wall). A balanced corroboration list mentions the dissent the design overcomes. Currently §15 reads as appeal-to-authority. v1.3 should add a "what the convergence does NOT establish" sentence or a dissenting-view row.

### MISSING COMPONENTS

**BP-04: Backup / restore pipeline is unmodeled.**
PRD §6.1.1 names `~/.cuttle/audit/`, `~/.cuttle/projects/`, `~/.cuttle/run/` (lockfile), `~/.cuttle/<credential-vault>`, telemetry store. Operator backs up `~/.cuttle/`. A restore from an old snapshot resurrects: stale audit-log chain (head mismatch), stale lockfile (WV-02 surface), stale registry entries (WV-04 surface), stale telemetry aggregates (T-008 surface). Cuttle has not modeled backup/restore as a trust boundary. Every Cuttle subsystem implicitly assumes `~/.cuttle/` state is current; backup/restore breaks this. Missing component: snapshot-version + state-coherence verification.

**BP-05: macOS Keychain prompt-fatigue interaction with CC-1 fail-closed gate.**
§6.1.1 credential vault: macOS Keychain backend. Keychain prompts for access on first use per process. CC-1: gate fails closed; Cuttle restarts. Each restart triggers Keychain prompt. Operator-under-load configures "always allow" to escape the prompts, which DEFEATS per-process keychain isolation. The CC-1 fail-closed mechanism and the Keychain prompt-fatigue mechanism are at cross purposes. v1.3 should acknowledge in §6.1.1.

### TRUST-BOUNDARY FAILURES

**BP-TB-1**: PRD's only trust boundary is operator-vs-everything-else; intra-process trust boundaries (gate vs model client vs skills loader vs vault) are nominal but not OS-enforced (per BP-01).
**BP-TB-2**: Backup/restore boundary unmodeled (per BP-04).
**BP-TB-3**: TTY-input boundary reduces to operator-keystroke, not operator-intent (per WV-06; this is also a Wolverine finding but it's structurally a trust-boundary modeling failure).

### MITIGATION FEASIBILITY

- §6.1.5 domain-primitives invariant operationally requires the implementation language to have nominal typing. OQ-1 deliberation already started in v1.2; v1.3 should make the language constraint a hard requirement, not a leaning. (Implication: TS likely off the table for v0.1 unless Cuttle accepts FFI for every trust-boundary-crossing type, which has its own surface per v1.2 delta TDD sub-surface #3.)
- F-Cuttle-DISABLE evaluation requires audit-log gate-disable events with reason codes. Per BP-02, the evaluation itself is operator-manual in v0.1; the falsifier is only as good as operator's recall and discipline at v0.1 ship.

### SHARED-FATE RISKS

- Single-process compromise (BP-01) cascades to all subsystems.
- Audit-log + telemetry + registry all live under `~/.cuttle/`; backup/restore (BP-04) treats them as one fate.
- Keychain "always allow" toggle (BP-05) cascades from one Cuttle restart to all subsequent restarts.

### COMPLIANCE GAPS

**BP-06: PII in audit log scope is unaddressed.**
The audit log captures tool call arguments and result digests. The operator's tools may touch PII (file paths with email addresses, DB queries for user records, content from document tools). CC-3 added `privacy` skill to REVIEW-2 (D-09) but the PRD does not scope what privacy means for the audit log: is the audit log allowed to record PII? If yes, the audit log is itself a PII data store with retention/access/GDPR consequences. If no, Cuttle needs PII-redaction at audit-write time (which is hard for opaque tool outputs). Major scope question PRD has not surfaced. v1.3 should add OQ-12: "audit-log PII posture: record-as-is, redact-at-write, or refuse-tools-that-may-emit-PII."

### BP-07: §15 corroboration cross-domain framing.

Carlos = enterprise CI. Mohan/Gumbley = team threat modeling. Ford = software architecture. Johnsson = application code design. Cuttle is single-operator AI-agent harness, a domain none of the four was addressing. The convergence narrows to "substrate>ceremony in their respective domains," NOT "substrate>ceremony for AI-agent harnesses." v1.3 §15 should make the cross-domain extrapolation explicit: "the substrate>ceremony principle has converged across these four adjacent domains; Cuttle takes the same principle to a fifth (single-operator AI-agent harness) where it has not been tested at industry scale yet."

### BLACK PANTHER VERDICT: **PASS with conditions.**

PRD v1.2 is structurally sound but carries 7 architectural concerns. None reach BLOCK-grade for the seal, but most warrant v1.3 PRD edits:

- BP-01 (single-process): new OQ-11.
- BP-02 (fitness functions aspirational): hedge §12 + D-10 framing in v1.3.
- BP-03 (cherry-picked corroboration): §15 wording in v1.3.
- BP-04 (backup/restore): new §8 case 9 in v1.3.
- BP-05 (Keychain fatigue): §6.1.1 acknowledgment in v1.3.
- BP-06 (PII in audit log): new OQ-12 in v1.3.
- BP-07 (cross-domain framing): §15 wording in v1.3.

---

## REMEDIATION TRIAGE (PRD v1.3 input)

| Finding | Severity                    | Disposition for v1.3                                                                                                               |
| ------- | --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| WV-01   | High                        | TDD-grain (already flagged in v1.2 delta as constructor authorization). Cross-link in v1.3 §6.1.5.                                 |
| WV-02   | Medium                      | TDD-grain (lockfile authentication mechanism).                                                                                     |
| WV-03   | CRITICAL                    | **PRD-grain.** §6.1.1 audit-log bullet adds taint-annotation requirement.                                                          |
| WV-04   | Medium                      | **PRD-grain framing.** §6.1.2 row L5 inherits T-003 disclaimer language.                                                           |
| WV-05   | High                        | **PRD-grain.** §6.1.1 skills-loader bullet adds strip-list maintenance contract + fail-closed-on-unknown-Unicode-category default. |
| WV-06   | CRITICAL (re-open of T-001) | **PRD-grain.** §6.1.5 attestation-provenance disclaimer + new F-Cuttle-FATIGUE in §12.                                             |
| WV-07   | CRITICAL (re-open of T-007) | **PRD-grain.** Covered by same §6.1.5 disclaimer as WV-06.                                                                         |
| BP-01   | Structural                  | **PRD-grain.** New OQ-11 (process-isolation model).                                                                                |
| BP-02   | Structural                  | **PRD-grain framing.** §12 + D-10 hedge: predicates are data; evaluator is TDD-grade.                                              |
| BP-03   | Framing                     | **PRD-grain framing.** §15 dissenting-view note.                                                                                   |
| BP-04   | PRD-grain                   | **PRD-grain.** §8 case 9 (backup/restore as trust boundary).                                                                       |
| BP-05   | PRD-grain                   | **PRD-grain.** §6.1.1 credential-vault bullet acknowledges Keychain prompt-fatigue interaction.                                    |
| BP-06   | Structural                  | **PRD-grain.** New OQ-12 (audit-log PII posture).                                                                                  |
| BP-07   | Framing                     | **PRD-grain framing.** §15 cross-domain extrapolation explicit.                                                                    |

10 findings warrant PRD v1.3 edits. 4 are TDD-deferred (or were already flagged at v1.2 delta and don't need re-flagging). Two new falsifier predicates seeded (F-Cuttle-FATIGUE; the WV-04 disclaimer doesn't need a new falsifier because F-Cuttle-DISABLE already covers the chain-rotation symmetry).

## REMEDIATION LOG TEMPLATE (filled at v1.3 application)

The PRD v1.3 commit will close this artifact with FIXED / DISPUTED dispositions per finding. This adversarial-review doc itself becomes input to the path #2 adversarial-pruning v3 pass (handoff path #3).
