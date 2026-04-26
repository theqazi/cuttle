# Martin Fowler input: source artifact

| Field      | Value                                                                                                                                                                                                                                |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Captured   | 2026-04-26 (session 4 part 3)                                                                                                                                                                                                        |
| Lens       | Code development principles + threat modeling, security-focused. Per Mo's 2026-04-26 directive.                                                                                                                                      |
| Discipline | Three-layer artifact discipline: this file is the source. DECISIONS.md (D-10/11/12) records what was decided. PRD records what shipped. Framework-doc-update task (handoff path #4) inherits any cross-cutting framework-side notes. |
| Anti-goal  | Do NOT compare Cuttle to Fowler's tooling or to the Secure by Design example codebase. Single-target lens per `feedback_review_as_lens_not_comparator.md`.                                                                           |

## Selection criteria

Mo's directive named "Martin Fowler's principles on code development and threat modeling. Code development especially focused on security." Mo's signal `Secure by Design` (the canonical Johnsson/Deogun/Sawano book, Manning 2019, Fowler foreword) reads as a request to fold the secure-by-design pattern language into Cuttle, not just generic Fowler bibliography.

Selected three sources by load-bearingness for Cuttle's bedrock thesis:

1. **Mohan & Gumbley, "Threat Modeling Guide for Software Teams" (2025), hosted on martinfowler.com.** Direct corroboration of the substrate-vs-advisory thesis at the threat-modeling layer.
2. **Ford, Parsons, Kua, "Building Evolutionary Architectures" (O'Reilly, 2nd ed 2022), foreword by Fowler.** Architectural fitness functions concept; direct synonym for Cuttle's sealed-falsifier predicates.
3. **Johnsson, Deogun, Sawano, "Secure by Design" (Manning, 2019), foreword by Fowler.** Domain primitives + read-once objects + invariants-at-construction. Direct fit for Cuttle's credential vault, attestation bodies, helper hashes, lockfile paths.

Set aside (not load-bearing for Cuttle's current scope): Refactoring 2nd ed code-smell vocabulary (Cuttle's reward-loop registry already speaks AP/VP); Strangler Fig pattern (already implicit in v0.1 → v0.2 → v0.N path); Tolerant Reader (already cross-referenced in PRD §6.1.1 transcript-schema bullet); Microservices articles (Cuttle is single-process v0.1; not relevant until the agent surface scales).

---

## Source 1: Mohan & Gumbley, "Threat Modeling Guide for Software Teams" (May 2025)

URL: `https://martinfowler.com/articles/agile-threat-modelling.html`

Verbatim quotes (load-bearing for Cuttle's substrate thesis):

- "Once you get the hang of identifying threats, it's tempting to organize a full-day workshop to 'threat model' every dataflow in your entire system at once. This big-bang approach often overwhelms teams and rarely sticks as a consistent practice."
- "Instead, integrate threat modeling regularly, like continuous integration for security. The most effective threat modeling happens in bite-sized chunks, closely tied to what your team is working on right now."
- "Spending fifteen minutes examining the security implications of a new feature can yield more practical value than hours analyzing hypothetical scenarios for code that isn't written yet."
- "These small sessions fit naturally into your existing rhythms ... threat modeling becomes a natural part of how your team thinks about and delivers software, rather than a separate security activity."
- Article frames security as building "security into their application and infrastructure" and emphasizes that "effective threat modeling should start simple and grow incrementally," positioning it as foundational rather than advisory.

What this corroborates for Cuttle:

- The substrate-vs-advisory thesis at the threat-modeling level. Mohan/Gumbley make the same argument for human dev teams that Cuttle makes for LLM agents: threat modeling has to be embedded in the loop, not bolted on.
- The "like continuous integration for security" phrase is the cleanest external articulation of bedrock 1. Cuttle's policy gate IS continuous-threat-modeling at the per-tool-call grain. The substrate constraint Carlos Arguelles names at the CI/CD layer (D-2026-04-26-01) and Mohan/Gumbley name at the threat-modeling layer are the same constraint shape: ceremony loses, embedded primitive wins.
- Convergent independent corroboration. Carlos (Amazon Senior Principal, ex-Google) and Mohan/Gumbley (Thoughtworks, Fowler-hosted) reach the same architectural conclusion via different domains (CI cost, threat-model practice). Cuttle's claim narrows from "novel architecture" to "substrate-vs-advisory IS the lesson; Cuttle is its substrate-native form for LLM agents."

What was set aside:

- The article's specific tooling recommendations (cards, sprint integration). Cuttle is single-operator; multi-operator workshop dynamics don't apply.
- The full STRIDE-by-element walkthrough. Cuttle uses the threat-model skill (`~/.claude/skills/threat-model/`) with its own STRIDE+ extensions; the Fowler article would be redundant.

---

## Source 2: Ford, Parsons, Kua, "Building Evolutionary Architectures" (O'Reilly, 2017 / 2nd ed 2022)

URL: `https://martinfowler.com/articles/evo-arch-forward.html` (Fowler's foreword); book at `https://nealford.com/books/buildingevolutionaryarchitectures.html`.

Quotes (paraphrased from search results; verify against book before quoting in publishable copy):

- "An architectural fitness function provides an objective integrity assessment of some architectural characteristic(s)."
- "Fitness functions employ a wide variety of implementation mechanisms: tests, metrics, monitoring, logging" to protect architectural dimensions.
- The authors use fitness functions to monitor the state of the architecture continuously.
- Fowler's foreword: the rise of Continuous Delivery has been a crucial enabling factor in making evolutionary architecture practical.

What this gives Cuttle:

- A widely-recognized industry term for what Cuttle calls sealed-falsifier predicates. Both name the same idea: testable, machine-evaluable conditions that, if violated, falsify a load-bearing architectural claim.
- Inheriting the "fitness functions" framing for §12 lets Cuttle's predicates land in the same conceptual slot as `archunit` rules, contract tests, performance budgets, etc.: primitives an audience already knows.
- Operationally: Cuttle's audit-log + telemetry surface (§6.1.6) is the substrate that lets fitness functions evaluate continuously, not at one-off review.

What was set aside:

- The book's typology of fitness functions (atomic vs holistic, triggered vs continuous, static vs dynamic, etc.). May matter at TDD; doesn't change the PRD-grain framing.
- Microservices examples that dominate the book.

---

## Source 3: Johnsson, Deogun, Sawano, "Secure by Design" (Manning, 2019)

URL: `https://www.manning.com/books/secure-by-design`; chapter 5 (Domain primitives) at `https://livebook.manning.com/book/secure-by-design/chapter-5/`.

Pattern language that applies to Cuttle (verbatim where searched):

- **Domain primitives**: "similar to value objects in Domain-Driven Design, with key differences being that invariants must exist and be enforced at the point of creation, and the use of simple language primitives or generic types as representations of domain concepts is prohibited."
- "Each concept should be modeled as a domain primitive to carry meaning when passed around, and to uphold its invariants."
- "if every argument and return value of a method is valid by definition, you'll have input and output validation in every single method without extra effort."
- **Read-once object**: design pattern used to mitigate sensitive data leaks (e.g., a value that, once read, becomes inaccessible; useful for credentials).
- **Immutable / Secure Entity**: how entities benefit from being constructed from domain primitives.
- **Taint analysis**: domain primitives work well with taint analysis because untainted values are constructible only via validated paths.

What this gives Cuttle:

The most direct PRD-grain application is in §6.1.5 cross-cutting harness invariants. Today the section says "intent + rationale fields are required," "context-budget enforcement is automatic," "escape hatches are themselves a Cuttle harness primitive," and (added in v1.1) "attestation provenance separation," "cross-session memory promotion." A Secure-by-Design-style invariant would commit Cuttle to: trust-boundary-crossing values are constructed only through domain-primitive types that enforce invariants at construction; raw `String` / `[u8]` / `int` are forbidden at trust-boundary surfaces.

Concrete v1.2 candidates for domain primitives:

- `ApiKey`: read-once + zeroizable + content-hashed. Constructible only from a verified credential-vault fetch. Cannot be implicit-string-cast. Maps to T-002, CC-2.
- `AttestationBody { provenance: Tty | Model, content: String }`: a value that explicitly tags whether the bytes came from operator TTY input or from model emission. Predicate evaluation rejects model-provenance variants by type, not by runtime string-match. Maps to T-001, the §6.1.5 attestation-provenance-separation invariant.
- `HelperHash`: not a raw `[u8; 32]`; constructible only by hashing the resolved helper script content. Recorded in `CredentialRecord` as `HelperHash`, not as raw bytes. Maps to T-002.
- `LockfilePath`: not a raw `String` / `Path`; constructible only at the canonical `~/.cuttle/run/<session-id>.lock` location. Prevents accidental construction with arbitrary paths. Maps to T-004.
- `TierClassification`: enum `Patch | Feature | Refactor | System`, not string. Tier-mismatch becomes a type error, not a runtime string compare. Maps to L2 mechanic.
- `OperatorAuthoredText` vs `ModelAuthoredText`: type-distinct wrappers around `String`, with no implicit cross-cast. Memory-quarantine logic, attestation-provenance logic, and skill-content-load logic all consume the appropriate variant by type. Maps to T-001, T-007.

This also constrains OQ-1 (language choice): TypeScript's structural typing makes domain-primitive enforcement weaker (any `{ provenance, content }` shape passes); Rust newtypes / Go named types enforce by nominal typing. Adds a security argument to the OQ-1 deliberation that v1.1 already started (CC-2 zeroization).

What was set aside:

- Specific Java/Spring examples in the book.
- The book's chapters on legacy-code migration. Cuttle is greenfield; not relevant.

---

## Convergent thesis (Carlos + Fowler/Romeo + Mohan/Gumbley)

Three independent industry voices converge on the same shape:

| Source                               | Domain                    | Conclusion                                                                                             |
| ------------------------------------ | ------------------------- | ------------------------------------------------------------------------------------------------------ |
| Carlos Arguelles (Amazon, ex-Google) | CI/CD blast radius        | Pre-submit verification at $100M/yr is justified by per-commit blast radius. Substrate beats post-hoc. |
| Mohan & Gumbley (Thoughtworks)       | Threat modeling           | "Continuous integration for security." Bite-sized embedded threat modeling beats one-off ceremony.     |
| Ford, Parsons, Kua (O'Reilly)        | Evolutionary architecture | Fitness functions in the build beat one-off architectural review. Architecture defended continuously.  |

Cuttle's bedrock thesis is the substrate-native form of all three for LLM agents: the policy gate IS continuous threat modeling, IS pre-submit verification, IS the fitness-function evaluator, all at the per-tool-call grain. The convergence narrows Cuttle's contribution claim from "novel architecture" to "novel application of an industry-converged principle to the LLM-agent case where blast radius is per-call and the no-human-in-loop differentiator (D-2026-04-26-02) makes ceremony-based defense unviable."

This matters for the framework-side update (handoff path #4): the framework_external_corroboration.md sidecar should track all three independent voices, not just Carlos.
