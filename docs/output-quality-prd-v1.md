# Output-Quality Review: Cuttle PRD v1

| Field          | Value                                                                                         |
| -------------- | --------------------------------------------------------------------------------------------- |
| Target         | `docs/PRD.md` v1 (302 lines)                                                                  |
| Companion docs | `docs/DECISIONS.md` (189 lines), `process/carlos-arguelles-input.md` (161 lines)              |
| Standard       | `~/.claude/skills/output-quality/SKILL.md` (PRD checklist + general non-code deliverable bar) |
| Author         | output-quality skill, PRD-stage pre-check                                                     |
| Severity scale | BLOCK (commit-blocker) / FIX-BEFORE-V2 (must close before adversarial review) / NICE-TO-HAVE  |

## Top-line: PRD passes the structural checklist

PRD checklist coverage (skill SKILL.md PRDs section):

| Item                               | PRD section      | Status                                                                   |
| ---------------------------------- | ---------------- | ------------------------------------------------------------------------ |
| Who is the user / current workflow | §2.1, §2.2       | PASS. Concrete baseline (56 skills, 21 hooks, named scripts).            |
| What specific problem              | §3.1, §3.2, §3.3 | PASS. Three structural problems, each with substrate-constraint framing. |
| Why now                            | §4 (3 reasons)   | PASS. Each reason has a date or memory anchor.                           |
| Success criteria (measurable)      | §5 (SC-1..SC-6)  | PASS. Binary, owner-verifiable, audit-log measurable.                    |
| Non-goals                          | §7 + §7.1        | PASS. Explicit, anti-scope-creep.                                        |
| Edge cases (≥3 non-obvious)        | §8 (8 cases)     | PASS. Each is concrete and non-obvious.                                  |

The PRD also exceeds the checklist in two structurally good ways: §12 sealed falsifier pre-registration (the framework's own discipline applied to itself), and §13 + §14 (self-aware framing of toolkit-content vs substrate-coupled-guidance + dual use of framework). These are not standard PRD sections; they earn their place by closing a specific drift item from session-1 and by making the methodology-vs-content distinction explicit before adversarial review.

## Findings (sorted by severity)

### BLOCK (commit-acceptance gating)

**None.** The session-3 commit is not blocked by output-quality. The PRD is publishable as v1; the items below are for v1.1 / v2 progression, not for the seed commit.

### FIX-BEFORE-V2 (must close before path #3 adversarial review)

**OQ-FIX-1: Implementation-level commitments inside §6.1.1 contradict open questions in §10.**

Two specific contradictions:

- §6.1.1 says "Audit log: every tool call ... written to ~/.cuttle/audit/<yyyy-mm-dd>.jsonl with HMAC chain for tamper-evidence." OQ-4 (§10) leaves audit-log tamper-evidence open: "HMAC chain (simpler) or Merkle tree with periodic root publication (stronger, more complex)?". The §6.1.1 commitment closes OQ-4 silently. Either OQ-4 is resolved (then say so and remove the OQ), or §6.1.1 should soften to "tamper-evident chain (scheme TBD per OQ-4)."
- §6.1.1 says "Bash sandboxed via macOS sandbox-exec with deny-by-default profile." OQ-3 leaves the macOS sandbox primitive open: "sandbox-exec ... or a custom seatbelt over posix_spawn + sandbox-exec hybrid?". Same pattern: the §6.1.1 commitment closes part of OQ-3 silently.

Both are PRD-vs-TDD scope discipline failures: PRD over-specifies implementation in §6.1.1 while §10 still lists the question as open. Fix: either move the specifics to TDD and soften §6.1.1, or resolve the OQs in §10 and cross-reference.

**OQ-FIX-2: Threat-model just produced CC-1, CC-2, CC-3 (see `docs/threat-model-prd-v1.md`). All three are PRD-grain requirements gaps, not implementation gaps.**

- CC-1: PRD does not declare what happens when the policy gate process crashes. Fail-open vs fail-closed is bedrock-load-bearing.
- CC-2: PRD does not declare in-memory key zeroization on session end / panic.
- CC-3: PRD §11 pipeline does not list `privacy` skill, but §6.1.6 telemetry surface and §6.1.1 audit log content digests trigger it per global CLAUDE.md mandatory-skills table.

These are requirements the PRD must declare; deferring them to TDD lets implementation choose silently.

**OQ-FIX-3: §3.2 contains unhedged universal claim that adversarial review will attack.**

Quote: "per-call blast radius is unbounded by intent in a way that human-authored commits in enterprise CI never are."

"Never" is the unhedged word. A human author can absolutely write `rm -rf $HOME` and commit it; the difference is the _commit-to-execute_ cycle gives the human the chance to notice. Sharpen to: "...in a way that human-authored commits _with intervening human review_ (the dominant case in enterprise CI) reduce in expectation." Less catchy but defensible against Wolverine-style critique.

**OQ-FIX-4: §6.1.2 v0.N column items lack explicit promotion criteria.**

Each L# row has "v0.1 mechanic | Deferred to v0.N" but no criterion for what triggers the v0.N port. This is the same shape as Carlos's "tools die silently because second milestones never get scoped" failure mode (per `process/carlos-arguelles-input.md` §3). At minimum, each Deferred-to-v0.N item should name (a) the falsifier signal that would force its promotion (e.g., "L1 snapshot decay model promoted when F-Cuttle-SNAPSHOT-DRIFT fires"), or (b) the SC-equivalent that would trigger it. This connects §6.1.2 to §12 falsifiers, which is currently a thin link.

**OQ-FIX-5: §6.1.6 telemetry surface (introduced session 3) under-specifies privacy posture.**

D-2026-04-26-04 in DECISIONS.md says: "PRD §6.1 v0.1 scope gains `cuttle telemetry` surface" + "TDD §5 (audit log design) gains an aggregation requirement." PRD §6.1.6 lands the first half but does not explicitly forward to TDD §5 for the second half, and does not declare ACL / file-permission requirements on the local telemetry store. Threat-model T-008 surfaces this same gap. Fix: add a sentence to §6.1.6 binding it to TDD §5 (aggregation requirement) and to §11 step 3 (privacy skill review).

### NICE-TO-HAVE (will not block adversarial review but improve doc clarity)

**OQ-NTH-1: §1 paragraph 4 is the first place "implementation existence proof" appears; reader has not yet encountered the framework's no-effect-claims discipline.**

The phrase is precise but lands cold. Either forward-ref the §7.1 non-goal, or move the phrase definition (currently in §7.1) up to §1's first occurrence. This is paragraph-flow, not correctness.

**OQ-NTH-2: §4 reason 2 says "structurally lower-risk than building on a pre-review version that might still contain claims that fail to survive scrutiny."**

"Structurally lower-risk" is a weak phrase by what's-the-comparison. Stronger: "lower-risk by the same metric the framework's own adversarial-defense methodology measures: number of claims that fail post-rectification." If the comparison metric is named, the sentence becomes auditable.

**OQ-NTH-3: §10 OQ-7 (public name) is a meta-question, not a technical OQ.**

Most OQs in the table block TDD progress; OQ-7 blocks first public release. Either flag OQ-7 differently in the table column ("Resolves in: Phase 2 prep" already does this loosely; consider a separate "blocking" column), or move OQ-7 out of §10 and into a new §10.1 "Non-blocking open questions."

**OQ-NTH-4: Cross-doc consistency between PRD §6.1.3 and DECISIONS.md is asymmetric.**

DECISIONS.md cites Carlos for D-01 through D-06, all from the same source artifact. PRD §6.1.3 cites `cuttle_v01_option_c_enumeration.md` (memory file), not `process/carlos-arguelles-input.md` (source). This is correct because Option C predates Carlos, but a one-sentence forward-link would make the dependency graph cleaner: "Per `cuttle_v01_option_c_enumeration.md` (originated session 2) and refined per Carlos's risk-dial framing (see D-2026-04-26-05)."

**OQ-NTH-5: §12 falsifier set has 4 first-draft predicates, all begin with "if X happens, the bedrock thesis is partially refuted."**

The pattern is right but the phrasing is identical across all four. Consider varying the rhetorical shape so each predicate carries information about what specifically is refuted (the bedrock thesis is multi-claim; each falsifier should pin down which sub-claim). Threat-model also queues F-Cuttle-SNAPSHOT-DRIFT (T-006) and F-Cuttle-MEMORY-DRIFT (T-007) for v1.1.

## Vague-language scan

Checked for filler / generic claims that could have been written without doing the work. Most sentences in this PRD pass; a few flagged for trim:

- §1 paragraph 1: "Cuttle is a security-first, terminal-native AI coding harness that connects to Anthropic's Claude API via the user's own API key (BYOK). It targets feature parity with Claude Code's current agent surface (tool use, subagent dispatch, skills, MCP, hooks, plan mode, memory) but is built on two co-equal bedrocks rather than one." OK; the BYOK + parity-list + bedrock framing are all load-bearing.
- §3.4 "These are not reference implementations of pre-execution gating", sharp, anti-comparator-failure-mode. Keep.
- §6.1.1 "schema-compatible with Claude Code's transcript format where reasonable", "where reasonable" is filler. Fix to "where the schema does not introduce policy-gate bypass vectors" or remove the qualifier and commit to compat.
- §6.1.2 row L3 "Worklog auto-captured per tool call." Underspecified, what schema, what storage, what retention? PRD-acceptable as a one-liner if forwarded to TDD; otherwise sharpen.

## Hedging calibration summary

Across the doc:

- Over-hedging risk: low. The PRD takes positions and labels OQs explicitly when it doesn't.
- Under-hedging risk: medium. OQ-FIX-3 is the clearest case. §6.1.2 row L4 "Multi-persona verification dispatches the same output through N personas as parallel agents" is also under-hedged; what does N mean, who picks N, what if personas disagree? PRD-acceptable if forwarded to TDD; tag for forward-ref.

## Cross-doc consistency check

Compared PRD against DECISIONS.md and process/carlos-arguelles-input.md.

- Every Carlos-driven decision (D-01..D-06) lands somewhere in the PRD. ✓
- Every PRD section that cites a D-### number resolves to a real DECISIONS entry. ✓
- Every PRD section that cites a memory file by line range (e.g., `framework_components.md:96-109`), spot-checked three; all resolve. ✓
- Reverse direction: PRD §6.1.6 (telemetry) anchors in D-2026-04-26-04. ✓
- One asymmetry noted in OQ-NTH-4 above (Option C provenance link).

## Action summary for PRD v1.1 author

Proposed sequence (matches threat-model's required-edits list, deduplicated):

1. Resolve OQ-3 and OQ-4 in §10, OR soften §6.1.1 commitments. (OQ-FIX-1)
2. Add CC-1, CC-2, CC-3 from threat-model into §6.1.1 + §11. (OQ-FIX-2 + threat-model T-### list)
3. Sharpen §3.2 unhedged "never" claim. (OQ-FIX-3)
4. Add v0.N promotion criteria to §6.1.2 table. (OQ-FIX-4)
5. Tighten §6.1.6 forward-refs to TDD §5 + privacy skill. (OQ-FIX-5)
6. Apply NICE-TO-HAVE 1-5 in same pass if cheap.

Quantitative summary: 5 FIX-BEFORE-V2 items, 5 NICE-TO-HAVE items, 0 BLOCK items. Doc is commit-ready as v1; v1.1 has a clear punchlist for path #2-#3 adversarial review.
