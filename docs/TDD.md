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

## §2. Config + data model + domain primitives (resolves OQ-6; specifies §6.1.5 trust-boundary primitives)

### 2.1 Filesystem layout (resolves OQ-6)

**Decision: `~/.cuttle/` (operator-home, NOT XDG).** PRD v3 §10 OQ-6 listed three options (`~/.cuttle/config.toml`, XDG `~/.config/cuttle/`, owner-pickable). v0.1 ships `~/.cuttle/`:

- Single discoverable root: state-coherence file (§8 case 9), audit log, telemetry, lockfile, credential vault all under one directory; backup/restore boundary (BP-04) operates on one path.
- Operator already has `~/.claude/`; `~/.cuttle/` is the obvious sibling for migration ergonomics.
- XDG split (`~/.config/cuttle/` for config + `~/.local/share/cuttle/` for data) doubles the backup/restore surface and complicates the state-coherence-file invariant.

```text
~/.cuttle/
├── config.toml                    # operator-editable; loaded via serde
├── credentials/                   # CredentialRecord storage; keychain-handle indirected
│   └── default.json               # CredentialRecord; api key NOT in plaintext (Keychain ref)
├── audit/
│   ├── 2026-04-26.jsonl           # tamper-evident chain (scheme TBD §5)
│   └── chain-head.json            # current chain head digest (read at startup)
├── projects/<project-key>/
│   └── <session-id>.jsonl         # session transcript
├── memory/
│   ├── canonical/                 # operator-promoted (per §6.1.5)
│   └── quarantine/                # model-authored, pending operator promotion
├── exception-tables/              # operator-controlled escape-hatch tables (§6.1.5)
│   └── <rule-id>.toml             # ship empty by default
├── telemetry/
│   └── aggregates.json            # operator-only ACL (§6.1.6)
├── run/
│   └── <session-id>.lock          # lockfile for nested-harness detection (§8 case 6)
└── state-coherence.json           # written at clean shutdown; checked at startup (§8 case 9)
```

**Permissions:** operator-only (mode 0700 on the directory; 0600 on regular files; 0700 on `run/` for the lockfile-check-then-create race window). TDD §5 specifies the audit-log permission contract in detail.

### 2.2 `config.toml` schema (v0.1)

```toml
# ~/.cuttle/config.toml
[cuttle]
version = "0.1"

[anthropic]
# Credential reference (NOT the key itself); resolved via credentials/default.json
credential_id = "default"
# Default model preferences; per PRD v3 §6.1.1
default_model_sonnet = "claude-sonnet-4-6"
default_model_opus   = "claude-opus-4-7"
prompt_cache = true

[gate]
# Allow / Warn / Deny graduation per OQ-9 (resolved in TDD §3)
default_decision = "deny"

[sandbox]
# Sandbox primitive choice per OQ-3 (resolved in TDD §4)
primitive = "sandbox-exec"   # leading candidate; contingency in §4

[audit]
# Tamper-evident chain scheme per OQ-4 (resolved in TDD §5)
scheme = "hmac-chain"        # leading candidate

[telemetry]
# Local-only per PRD v3 §6.1.6; remote forbidden in v0.1
enabled = true
remote = false               # ALWAYS false in v0.1; redundant guard
```

Config is loaded once at startup into a `Config` struct (`serde` derive). Config mutations require operator-typed TTY edit; Cuttle does NOT write to `config.toml` from the running process (operator-source-of-truth invariant).

### 2.3 `CredentialRecord` schema (extends `auth_mode_decision.md:21-28` with `helper_hash`)

PRD v3 §6.1.1 + D-08 require helper-hash-binding. Schema:

```rust
// crates/cuttle-credential/src/record.rs
use serde::{Serialize, Deserialize};
use crate::primitives::HelperHash;  // §2.4

#[derive(Serialize, Deserialize)]
pub struct CredentialRecord {
    pub id: String,                          // "default", or operator-chosen
    pub backend: CredentialBackend,
    pub helper_hash: Option<HelperHash>,     // Some when backend = ApiKeyHelper
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_refreshed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CredentialBackend {
    /// macOS Keychain (default in v0.1).
    Keychain { service: String, account: String },
    /// Encrypted file fallback (opt-in; per-session operator confirmation).
    EncryptedFile { path: std::path::PathBuf },
    /// apiKeyHelper-style indirection (opt-in only; helper_hash REQUIRED).
    ApiKeyHelper { helper_path: std::path::PathBuf, refresh_ttl_secs: u64 },
}
```

`helper_hash` is `Option<HelperHash>` not `Option<[u8; 32]>` per the §2.4 domain-primitive invariant (raw bytes forbidden at trust-boundary surfaces).

### 2.4 Domain primitive enumeration (specifies PRD v3 §6.1.5 candidates)

Each primitive is a Rust newtype in a dedicated module with `pub(crate)` constructor, public read accessors, and either `Zeroize`/`Drop` (for secret-bearing types) or `serde::{Serialize, Deserialize}` (for serializable types). Only the validating boundary functions in the same crate hold the capability to construct.

#### `ApiKey` (T-002 + CC-2)

```rust
// crates/cuttle-credential/src/primitives/api_key.rs
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct ApiKey {
    bytes: Vec<u8>,
    consumed: std::cell::Cell<bool>,
}

impl ApiKey {
    /// Constructor is `pub(crate)`: only credential-vault module can mint.
    pub(crate) fn from_keychain_fetch(bytes: Vec<u8>) -> Self {
        Self { bytes, consumed: std::cell::Cell::new(false) }
    }

    /// Read-once consumer. Subsequent calls panic (release) or return error (debug).
    /// Used by the Anthropic API client during request construction.
    pub fn consume(&self) -> &[u8] {
        if self.consumed.replace(true) {
            panic!("ApiKey::consume called twice; this is a bug, not a runtime condition");
        }
        &self.bytes
    }
}

// No Display, Debug, Clone, Serialize derives. ApiKey leaves the credential
// vault crate ONLY through ApiKey::consume(), which the Anthropic client uses
// once per request.
```

The `panic!` is intentional: misuse is a programmer bug; `panic = "abort"` build profile (per §1) makes the abort deterministic and zeroization runs on-drop before unwinding can leak.

#### `AttestationBody { provenance: Tty | Model, content: String }` (T-001)

```rust
// crates/cuttle-gate/src/primitives/attestation.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Provenance {
    Tty,
    Model,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttestationBody {
    provenance: Provenance,
    content: String,
}

impl AttestationBody {
    /// Constructor for TTY-input events. ONLY the input-handler module
    /// (`cuttle-input`) holds the capability to call this.
    pub(crate) fn from_tty_input(content: String) -> Self {
        Self { provenance: Provenance::Tty, content }
    }

    /// Constructor for model-emitted text. ONLY the model-client module
    /// (`cuttle-anthropic`) holds the capability to call this.
    pub(crate) fn from_model_output(content: String) -> Self {
        Self { provenance: Provenance::Model, content }
    }

    pub fn provenance(&self) -> &Provenance { &self.provenance }
    pub fn content(&self) -> &str { &self.content }
}
```

Predicates that consume attestations as evidence (Option C §6.1.3, escape-hatch evidence per §6.1.5) match on `Provenance::Tty` and reject `Provenance::Model` at the type level. Per WV-06 disclaimer: this distinguishes bytes-typed-by-operator from bytes-emitted-by-model, NOT operator-INTENT from operator-FATIGUE-KEYPRESS; F-Cuttle-FATIGUE per `docs/falsifiers.md` measures the residual.

#### `HelperHash` (T-002)

```rust
// crates/cuttle-credential/src/primitives/helper_hash.rs
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct HelperHash([u8; 32]);

impl HelperHash {
    /// Constructor: only the credential-vault module can mint, and only by
    /// hashing a resolved helper-script content. Raw bytes from anywhere else
    /// CANNOT be coerced into a HelperHash.
    pub(crate) fn compute_from(helper_script_bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(helper_script_bytes);
        Self(hasher.finalize().into())
    }

    pub fn matches(&self, helper_script_bytes: &[u8]) -> bool {
        let computed = Self::compute_from(helper_script_bytes);
        // Constant-time comparison via `subtle` crate (avoid timing side-channel).
        use subtle::ConstantTimeEq;
        self.0.ct_eq(&computed.0).into()
    }
}
```

#### `LockfilePath` (T-004)

```rust
// crates/cuttle-runtime/src/primitives/lockfile_path.rs
use std::path::PathBuf;

pub struct LockfilePath(PathBuf);

impl LockfilePath {
    /// Constructor: only the runtime module can mint. Validates the path is
    /// canonical to ~/.cuttle/run/<session-id>.lock; arbitrary paths rejected.
    pub(crate) fn for_session(session_id: &SessionId) -> Result<Self, LockfilePathError> {
        let home = dirs::home_dir().ok_or(LockfilePathError::NoHomeDir)?;
        let path = home.join(".cuttle").join("run").join(format!("{}.lock", session_id));
        // Canonicalize: prevents `..` traversal and symlink shenanigans.
        // (TDD §3 specifies the create-then-fsync contract.)
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &std::path::Path { &self.0 }
}

#[derive(Debug)]
pub enum LockfilePathError {
    NoHomeDir,
}
```

#### `TierClassification` (L2 mechanic)

```rust
// crates/cuttle-runtime/src/primitives/tier.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TierClassification {
    Patch,
    Feature,
    Refactor,
    System,
}
```

The model proposes a tier as a string; the parser rejects anything outside the four variants at deserialization time. Mismatch becomes a parse error, not a runtime string compare per PRD v3 §6.1.2 row L2.

#### `OperatorAuthoredText` vs `ModelAuthoredText` (T-001 + T-007)

```rust
// crates/cuttle-memory/src/primitives/text.rs

/// Bytes the operator typed in the TTY (or wrote to operator-controlled
/// surfaces like ~/.claude/CLAUDE.md). Trusted-by-default at the boundary.
pub struct OperatorAuthoredText(String);

impl OperatorAuthoredText {
    /// Constructor: only the input-handler / file-loader modules can mint
    /// from the operator-trust surface.
    pub(crate) fn from_tty(s: String) -> Self { Self(s) }
    pub(crate) fn from_operator_file(s: String) -> Self { Self(s) }
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Bytes the model emitted. Untrusted-by-default; quarantined per
/// §6.1.5 cross-session memory promotion invariant.
pub struct ModelAuthoredText(String);

impl ModelAuthoredText {
    /// Constructor: only the model-client module can mint.
    pub(crate) fn from_model_output(s: String) -> Self { Self(s) }
    pub fn as_str(&self) -> &str { &self.0 }
}

// NO `From<String>` / `Into<String>` impls. NO cross-cast between
// OperatorAuthoredText and ModelAuthoredText. The type system enforces the
// memory-quarantine boundary.
```

### 2.5 Capability-token pattern for constructor authorization (WV-01 / D-13)

`pub(crate)` constructors enforce module boundaries, but the policy gate often needs to authorize the constructor invocation conditionally (e.g., "this read-loop holds the capability to construct `AttestationBody::from_tty_input` because it owns the TTY file descriptor"). The capability-token pattern:

```rust
// crates/cuttle-gate/src/capabilities.rs

/// Marker that proves the holder has TTY-input authority.
/// Created ONCE at session start by the input-handler crate's owner.
/// Cannot be cloned, serialized, or constructed outside the input-handler.
pub struct TtyInputCap(());

impl TtyInputCap {
    /// `pub(super)`: only the input-handler crate's parent (cuttle-input
    /// crate root) can mint. Skills loader, model client, etc. cannot.
    pub(super) fn issue() -> Self { Self(()) }
}

// Then the AttestationBody constructor takes the capability as a witness:
impl AttestationBody {
    pub(crate) fn from_tty_input(_cap: &TtyInputCap, content: String) -> Self {
        Self { provenance: Provenance::Tty, content }
    }
}
```

A skill's code (running inside its own sub-module / sub-crate) cannot acquire a `TtyInputCap` because the constructor is not visible to it. The capability is unforgeable at the type-system layer, satisfying WV-01.

### 2.6 Serialization round-trip contract (v1.2 delta TDD sub-surface #2)

Domain primitives round-trip through audit-log JSONL, transcript JSONL, and config TOML without losing the type tag. Specifically: `AttestationBody { provenance: Tty }` MUST NOT become `AttestationBody { provenance: Model }` due to a missing-field default.

Contract:

```rust
// In tests:
#[test]
fn attestation_provenance_round_trip() {
    let original = AttestationBody::from_tty_input(&cap, "operator's reason".to_string());
    let json = serde_json::to_string(&original).unwrap();
    let restored: AttestationBody = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.provenance(), &Provenance::Tty);
}

#[test]
fn attestation_rejects_unknown_provenance() {
    let bad_json = r#"{"provenance":"Spoofed","content":"..."}"#;
    let result: Result<AttestationBody, _> = serde_json::from_str(bad_json);
    assert!(result.is_err()); // serde rejects unknown variant
}

#[test]
fn attestation_rejects_missing_provenance() {
    let bad_json = r#"{"content":"missing provenance field"}"#;
    let result: Result<AttestationBody, _> = serde_json::from_str(bad_json);
    assert!(result.is_err()); // serde requires the field; no default
}
```

`serde` derives MUST NOT use `#[serde(default)]` or `#[serde(other)]` on `Provenance` enum. `proptest`-driven round-trip tests in `crates/cuttle-gate/tests/` verify the invariant for every domain primitive.

### Cross-references and what §2 does NOT decide

- TDD §3 specifies how `TtyInputCap` is issued at session start (input-handler crate initialization), how the capability flows through the gate, and the supervisor / restart contract that determines what happens to capabilities on gate-process death.
- TDD §4 (sandbox) does not interact with §2 directly; sandbox-exec invocation lives in `cuttle-sandbox` crate with its own primitives.
- TDD §5 (audit log) consumes `AttestationBody` for provenance recording; the per-tool `secret_bearing` flag (per WV-03) is a separate primitive defined in §5.
- TDD §6 (memory) specifies the canonical-vs-quarantine filesystem layout; §2.4 already provides the type-level distinction.
- The `cuttle-` crate names above are illustrative; the workspace layout commitment lives in TDD §3 (workspace structure subsection).

### DECISIONS entries from §2

- **D-2026-04-26-16**: filesystem layout `~/.cuttle/` (resolves OQ-6); operator-only mode 0700.
- **D-2026-04-26-17**: domain primitive enumeration with capability-token pattern for constructor authorization (WV-01); 6 primitives shipped in v0.1.
- **D-2026-04-26-18**: serialization round-trip contract; `serde` derives forbidden from using `default` / `other` on enums representing trust-boundary discriminants.

(D-16/17/18 to be added to DECISIONS.md when §2 commits.)

---

## §3 through §6: PENDING

To be written in subsequent commits.
