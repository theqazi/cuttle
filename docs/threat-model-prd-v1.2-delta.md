CONFIDENTIAL: This document contains detailed security findings. Handle per your organization's data classification policy. This is AI-assisted analysis and requires human expert review before use in security decisions or compliance.

# Threat Model Delta: PRD v1.1 -> v1.2

| Field            | Value                                                                                                                                                                                         |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Target           | `docs/PRD.md` v1.2                                                                                                                                                                            |
| Method           | Delta check on v1.1 -> v1.2 edits only. Full v1 -> v1.1 delta at `docs/threat-model-prd-v1.1-delta.md`. Wolverine + Black Panther adversarial review still queued for path #2 against PRD v2. |
| Source decisions | D-2026-04-26-10 (fitness-functions cross-ref), D-2026-04-26-11 (domain primitives), D-2026-04-26-12 (continuous-threat-modeling framing + §15 corroboration)                                  |

## v1.1 -> v1.2 edit inventory

| Edit                                                         | New attack surface?                                                                                                                         | Closes prior finding?                                                                                                                          |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| §1 paragraph 4: Mohan/Gumbley + Ford cross-references        | None. Framing only.                                                                                                                         | None directly. Provides external anchor that Wolverine review (path #2) may attack as appeal-to-authority; pre-empt by citing source artifact. |
| §6.1.5 new "Domain primitives at trust boundaries" invariant | None. **Reduces** v1.1 surface by moving runtime-only checks to type-system + runtime defense-in-depth. Strengthens T-001 + T-007 closures. | Hardens T-001, T-002, T-004, T-007 from runtime predicate to type-system primitive. CC-2 zeroization tightens (ApiKey is read-once).           |
| §12 introduction: fitness-functions cross-reference          | None. Naming only.                                                                                                                          | None directly. Adds discoverability to industry vocabulary.                                                                                    |
| §15 new "External corroboration" section                     | None. Documentation.                                                                                                                        | None directly.                                                                                                                                 |

**Net assessment:** v1.2 introduces zero new attack surface. The §6.1.5 domain-primitives invariant **strengthens** the v1.1 closures of T-001, T-002, T-004, T-007 by adding type-system enforcement to the runtime predicates. Other v1.2 edits are framing/cross-reference and carry no security implications.

## v1.1 closures: still closed?

Spot-check of v1.1 closures (verified at v1 -> v1.1 delta):

| ID    | v1.1 closure mechanism                                                   | v1.2 status                                                                                                                          |
| ----- | ------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| T-001 | §6.1.5 attestation-provenance separation + §8 case 1 critical refinement | **Hardened.** §6.1.5 now also requires `AttestationBody { provenance: Tty                                                            | Model }` domain primitive: rejection happens at type construction, not runtime string-match. |
| T-002 | §6.1.1 helper hash + sandboxed network egress                            | **Hardened.** `ApiKey` and `HelperHash` domain primitives in §6.1.5 prevent raw-bytes leakage at type level.                         |
| T-004 | §8 case 6 lockfile + process-tree detection                              | **Hardened.** `LockfilePath` domain primitive in §6.1.5 prevents arbitrary-path construction at type level.                          |
| T-007 | §6.1.1 + §6.1.5 cross-session memory quarantine                          | **Hardened.** `OperatorAuthoredText` vs `ModelAuthoredText` domain primitives in §6.1.5 enforce structural separation at type level. |
| T-009 | §6.1.5 exception-tables operator-loaded                                  | Unchanged. v1.2 does not touch this surface.                                                                                         |
| T-003 | §6.1.1 audit-log scope acknowledgment + F-Cuttle-DISABLE expansion       | Unchanged. v1.2 does not touch this surface.                                                                                         |
| T-010 | §6.1.2 L5 review queue + signed provenance                               | Unchanged. v1.2 does not touch this surface.                                                                                         |
| T-006 | §7.1 honesty + F-Cuttle-SNAPSHOT-DRIFT                                   | Unchanged.                                                                                                                           |
| T-008 | §6.1.6 ACL + privacy review                                              | Unchanged.                                                                                                                           |
| T-005 | §6.1.1 sandbox softening to OQ-3                                         | Unchanged.                                                                                                                           |
| CC-1  | §6.1.1 fail-closed gate                                                  | Unchanged.                                                                                                                           |
| CC-2  | §6.1.1 zeroization                                                       | **Hardened.** `ApiKey` domain primitive specifies read-once + zeroizable as type-level invariant.                                    |
| CC-3  | §11 step 3 privacy skill                                                 | Unchanged.                                                                                                                           |

No regressions detected. Five findings hardened by the §6.1.5 domain-primitives invariant.

## v1.2-introduced sub-surfaces (TDD-grain, not PRD-grain)

The §6.1.5 domain-primitives invariant pushes complexity into the type system. New sub-surfaces TDD must address:

- **Constructor authorization.** Each domain primitive has a constructor that enforces invariants. The constructor itself becomes a trust-boundary surface. If `ApiKey::from_keychain(...)` can be called from any module, the read-once invariant is only as strong as the discipline preventing repeat construction. TDD §2 must specify constructor-visibility scoping.
- **Serialization round-trip.** Domain primitives must round-trip through audit-log JSONL without losing the type tag. `AttestationBody { provenance: Tty }` serialized to JSON and re-deserialized must NOT become `AttestationBody { provenance: Model }` due to a missing-field default. TDD §5 specifies serialization contract.
- **FFI / native binding boundary.** If any TS surface needs to use these primitives via a Rust native binding (e.g., for the zeroizable API key), the FFI boundary becomes a domain-primitive-leak surface. TDD §2 (OQ-1 resolution) should account for this if TS is selected.

These are TDD-grain, not PRD-grain. Logged here so TDD inherits them.

## Delta-check verdict

PRD v1.2 introduces **zero new PRD-grain attack surface**. The Fowler-pass edits are net-positive: §6.1.5 domain-primitives invariant strengthens 5 of v1.1's CRITICAL/cross-cut closures by adding type-system defense-in-depth. §1 + §15 corroboration framing is documentation only. §12 fitness-functions cross-reference is naming only.

Wolverine + Black Panther adversarial review (handoff path #2 against PRD v2) inherits a tighter starting point than v1.1, with a clean type-system narrative for the trust-boundary primitives.
