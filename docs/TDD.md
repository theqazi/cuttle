# TDD: Cuttle (working codename)

| Field        | Value                                                                                                                                                                           |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Status       | Draft v0 (§1 only). Scope: resolve OQ-1 through OQ-6, OQ-9, OQ-11, OQ-12 from PRD v3 §10. Pending §2 through §6.                                                                |
| Owner        | Mohammed Qazi (sirajuddin.qazi@gmail.com)                                                                                                                                       |
| Created      | 2026-04-26                                                                                                                                                                      |
| Tier         | SYSTEM (per global CLAUDE.md). Inherits PRD v3 SYSTEM-tier ceremony.                                                                                                            |
| Source PRD   | `docs/PRD.md` v3 (committed `4f0ffbb`)                                                                                                                                          |
| Sealed       | `docs/falsifiers.md` (pre-seal draft; thresholds refined in this TDD §3 + §5)                                                                                                   |
| Decision log | `docs/DECISIONS.md` (entries from 2026-04-26 onward)                                                                                                                            |
| TDD scope    | Inherits sub-surfaces flagged across PRD v1.1, v1.2, v1.3, v3 deltas (per handoff v0.8 path #1 "TDD scope inheritance" cross-cutting reference)                                 |
| Pipeline     | After TDD lands: REVIEW-1 (`code-review`) → REVIEW-2 (`legal-review` + `threat-model` + `privacy`) → FIX-DOCS → DESIGN (`system-design`) → API (`api-design`) → Implementation. |

---

## Conventions

- **Section numbering**: TDD §N maps to PRD v3 §10 OQs as documented in each section header. PRD-grain decisions are NOT re-derived here; this doc resolves OQs and adds implementation-grain commitments.
- **Cross-references**: PRD v3 §X = `docs/PRD.md` v3 §X. WV-XX, BP-XX, T-XXX, CC-X, D-XX = adversarial-review / threat-model / cross-cut / DECISIONS entries from session 4.
- **TDD-grain decisions get DECISIONS entries** (D-15 onward) using the same ADR-lite format.
- **Code samples are illustrative**, not normative. The implementation lives in source after this TDD lands; samples here exist to make the design concrete enough to attack.

---

## §1. Language choice (resolves OQ-1)

### Decision: **Rust** for v0.1.

### Constraint inventory

PRD v3 has the following load-bearing constraints that the language choice must satisfy:

1. **In-memory zeroization** (§6.1.1 credential vault, CC-2 / D-2026-04-26-08). API key bytes zeroized on session end and uncaught panic. PRD §6.1.1 explicitly notes: "Rust `zeroize`, Go via mlock + explicit overwrite, TS not viable for this surface without native bindings."
2. **Domain primitives at trust boundaries** (§6.1.5, D-2026-04-26-11). Nominal typing required. PRD §6.1.5: "TS structural typing weakens domain-primitive enforcement; Rust newtypes / Go named types enforce nominally."
3. **Constructor authorization** (§6.1.5, WV-01 / D-2026-04-26-13). Each domain primitive's constructor must be module-private with capability scoping. Untrusted modules pass raw bytes through validating boundary functions.
4. **Sandboxed core tool set** (§6.1.1). Bash sandboxed via macOS process-isolation primitive. Process isolation, fork/exec management, sandbox-exec invocation, optional future-replacement primitives (Endpoint Security framework, hypervisor-based isolation). Native systems-language affinity desirable.
5. **Policy gate fail-closed on process death** (§6.1.1, CC-1 / D-2026-04-26-08). Supervisor pattern (parent watches child, restarts on death, halts dispatch). Native systems-language affinity desirable.
6. **Process isolation candidate at v0.2** (OQ-11, BP-01 / D-2026-04-26-13). Policy gate as separate OS process supervising the model client over typed IPC. Cross-process typed-IPC primitives need to be cheap to write correctly.
7. **Multi-platform substrate**: macOS-first for v0.1; Linux for v0.2; Windows for v0.N. Cross-platform without losing native systems primitives.
8. **Operator dogfood velocity**: v0.1 needs to ship before the operator's velocity-vs-Claude-Code comparison loses meaning (target: ~1 month from PRD seal).

### Candidates

#### Rust

- **Strengths.** Native `zeroize` crate (CC-2 ✓). Newtype pattern with `pub(crate)` constructors for module-private capability scoping (WV-01 + §6.1.5 ✓). Strong type system enforces nominal typing without escape hatches (§6.1.5 ✓). First-class systems-language for fork/exec/sandbox-exec via `nix`, `caps`, `command-fds` crates (constraints 4, 5 ✓). Cheap typed IPC via Unix sockets + `serde` + `bincode` or `rkyv` (constraint 6 ✓). Cross-platform via `cfg(target_os)` (constraint 7 ✓). Memory safety + no GC means panics are deterministic and zeroization on panic is reliable.
- **Weaknesses.** Compile times slow operator iteration (constraint 8 risk). Async story (Tokio vs `async-std` vs raw threads) is a v0.1 design decision in itself. Crate ecosystem for Anthropic SDK is community-maintained (no official Anthropic Rust SDK as of 2026-04); requires writing or maintaining a thin client.
- **Velocity mitigation.** Use `cargo-watch` + incremental compilation. Pin to a small dependency tree (audit cost per Carlos predicate-maintenance constraint, PRD v3 §9). Async via Tokio (mature, well-known).

#### Go

- **Strengths.** Native systems-language affinity (constraints 4, 5 ✓). Cross-platform `os/exec` + `syscall` (constraint 7 ✓). Cheap typed IPC via Unix sockets + Protobuf (constraint 6 ✓). Fast compile times (constraint 8 ✓). Goroutines and channels make supervisor patterns ergonomic.
- **Weaknesses.** Zeroization requires `mlock` + explicit overwrite, more error-prone than Rust `zeroize`. GC is non-deterministic; zeroization on panic relies on `defer` discipline. Named types enforce nominal typing but type assertions and `interface{}` give escape hatches that an untrusted-or-low-trust module can use to bypass capability scoping (WV-01 partial coverage; not as strong as Rust). Anthropic SDK status: also community-maintained.
- **Net.** Acceptable but weaker than Rust on the security-critical constraints (CC-2, WV-01, §6.1.5). Strictly better on velocity.

#### TypeScript

- **Strengths.** Operator already deeply familiar with TS via Claude Code itself. Fastest iteration. Best Anthropic SDK ecosystem (official `@anthropic-ai/sdk`).
- **Weaknesses.** Structural typing means `AttestationBody { provenance: Tty | Model, content }` is not nominally distinguishable from any other `{ provenance, content }` object. WV-01 constructor authorization collapses (§6.1.5 explicitly says so). Zeroization requires native bindings (FFI surface = new attack surface per v1.2 delta TDD sub-surface #3). Process-isolation primitives via `child_process` are weaker (no easy `sandbox-exec` invocation; no first-class fork). GC + V8 means zeroization-on-panic is unreliable.
- **Net.** Disqualified for v0.1 on PRD v3 §9 grounds. v0.2+ may add a TS surface if there's a need (e.g., a thin TS CLI wrapper around the Rust core), but the load-bearing security primitives must be Rust.

### Decision rationale

PRD v3 §9 leans Rust > Go > TS on security grounds. Rust uniquely satisfies CC-2 zeroization with `zeroize` crate's deterministic on-drop semantics, WV-01 constructor authorization with `pub(crate)` module privacy, and §6.1.5 nominal-type domain primitives with newtypes that have no structural equivalent.

Go is operationally acceptable but trades multiple security-load-bearing constraints for velocity. The trade does not pay for itself: PRD v3 SC-5 requires zero CRITICAL findings on v0.1 implementation; Go's escape hatches (interface{}, type assertion) make WV-01 closure operationally fragile in a way that would surface as REVIEW-2 findings.

TS is disqualified per PRD §6.1.1 + §6.1.5 explicit constraint statements.

### Consequences

- **Toolchain**: Rust 2024 edition. Stable channel (avoid nightly per dogfood-stability requirement). Pin via `rust-toolchain.toml`.
- **Async runtime**: Tokio 1.x (multi-threaded, mature, expected by most ecosystem crates).
- **Anthropic client**: write a thin client over `reqwest` + `serde` + `eventsource-stream` (for streaming). No third-party Anthropic SDK dependency in v0.1; this avoids supply-chain risk per PRD §9 constraint and per CamoLeak / postmark-mcp incident class. Move to community SDK in v0.2 only after `legal-review` + `sbom-license` pass.
- **Crate dependency budget**: PRD §9 predicate-maintenance constraint applies. Target ≤30 direct dependencies in v0.1 `Cargo.toml`. Each direct dependency requires explicit operator approval (logged in DECISIONS as needed). Indirect dependencies tracked via `cargo audit` + `cargo deny` (configured in CI).
- **Build profile**: release profile with `panic = "abort"` (deterministic abort on panic; required for zeroization-on-panic invariant per CC-2 to hold; otherwise unwinding may run destructors in non-deterministic order). Debug profile preserved for development.
- **Testing**: standard `cargo test`. Property tests via `proptest` for domain-primitive invariants. No async test framework dependency in v0.1 (use `#[tokio::test]` directly).
- **CI/CD scope**: out of v0.1 scope (single-operator dogfood; `git commit` is the deploy event). Re-evaluate at v0.2.

### Cross-references

- PRD v3 §6.1.1 credential vault In-memory zeroization invariant (CC-2 / D-08): satisfied by Rust `zeroize` crate, `panic = "abort"` build profile.
- PRD v3 §6.1.5 Domain primitives at trust boundaries (D-11): satisfied by Rust newtype pattern with `pub(crate)` constructors.
- PRD v3 §6.1.5 Constructor authorization (WV-01 / D-13): satisfied by Rust module privacy + capability tokens (TDD §2 specifies pattern).
- PRD v3 §6.1.1 Policy gate failure mode (CC-1 / D-08): satisfied by Tokio supervisor pattern + child-process management (TDD §3 specifies).
- PRD v3 §10 OQ-11 Process-isolation model (BP-01 / D-13): typed IPC over Unix sockets is cheap in Rust (TDD §3 revisits at v0.2 latest).

### What this section does NOT decide

- TDD §2 (config, data model, domain primitive enumeration): how each primitive is implemented as a Rust type.
- TDD §3 (policy gate): supervisor-restart contract, allow/warn/deny graduation, IPC schema.
- TDD §4 (sandbox): sandbox-exec invocation crate vs FFI-direct; deprecation contingency.
- TDD §5 (audit log): HMAC vs Merkle scheme.
- TDD §6 (memory): quarantine layout.
- Anthropic API client implementation details: streaming, retry, prompt-cache integration.

---

## §2 through §6: PENDING

To be written in subsequent commits.
