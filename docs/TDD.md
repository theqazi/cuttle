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

## §3. Policy gate (resolves OQ-2, OQ-9, OQ-11; specifies CC-1, T-001, WV-02, BP-05; refines falsifier thresholds)

### 3.1 Workspace structure

v0.1 ships as a Cargo workspace with the following crates. Each crate's name encodes its trust-boundary role per D-17.

```text
cuttle/
├── Cargo.toml                    # workspace root; members below
├── rust-toolchain.toml           # pinned per D-15
├── deny.toml                     # cargo-deny config (D-15 dependency budget)
└── crates/
    ├── cuttle-cli/               # bin: `cuttle` entry point; argument parsing only
    ├── cuttle-runtime/           # session orchestration; owns LockfilePath, TierClassification
    ├── cuttle-gate/              # POLICY GATE; owns AttestationBody + capability tokens
    ├── cuttle-input/             # TTY input handler; mints TtyInputCap
    ├── cuttle-anthropic/         # Anthropic API client; thin reqwest wrapper
    ├── cuttle-credential/        # credential vault; owns ApiKey + HelperHash
    ├── cuttle-sandbox/           # macOS sandbox-exec invocation (TDD §4)
    ├── cuttle-audit/             # audit-log writer/reader (TDD §5)
    ├── cuttle-telemetry/         # cuttle telemetry surface (PRD §6.1.6)
    ├── cuttle-memory/            # auto-memory; owns OperatorAuthoredText / ModelAuthoredText
    ├── cuttle-skills/            # skills loader with Unicode allowlist (per WV-05)
    ├── cuttle-rewardloop/        # AP/VP registry (L5 mechanic)
    └── cuttle-falsifiers/        # falsifier predicate evaluator (TDD §3.9 stub for v0.1)
```

The dependency graph is acyclic and deliberately constrained so that **`cuttle-gate` is the choke point for every tool dispatch**. `cuttle-cli` depends on `cuttle-runtime`; `cuttle-runtime` depends on `cuttle-gate` + `cuttle-input` + `cuttle-anthropic` + `cuttle-credential` + `cuttle-audit` + `cuttle-skills` + `cuttle-memory` + `cuttle-rewardloop` + `cuttle-sandbox` + `cuttle-telemetry`. `cuttle-gate` depends only on primitives (no model client, no I/O). `cuttle-anthropic` and `cuttle-skills` depend on `cuttle-gate` to dispatch tool calls; they cannot bypass it because the dispatch API is the only way to request a tool execution.

### 3.2 Supervisor + restart contract (CC-1 fail-closed gate)

PRD v3 §6.1.1 commits Cuttle to fail closed on gate-process death. v0.1 ships **single-process** (in-process gate) with the supervisor pattern; OQ-11 (process isolation) is revisited at v0.2 latest. Single-process v0.1 means "fail closed on gate-process death" reduces to "fail closed on Cuttle process death," which is trivially true (the process is gone). The interesting case is **gate-task panic** within the Cuttle process.

**Mechanism**: `cuttle-gate` exposes a single `dispatch()` async function. All tool calls go through it. The function holds a `GateState` (in-memory; not shared across threads except via `Arc<Mutex<>>` for the audit-log writer handle). If `dispatch()` panics, `panic = "abort"` (D-15) terminates the entire process. There is no per-call recovery; the operator restarts Cuttle via the CLI.

```rust
// crates/cuttle-gate/src/lib.rs
pub struct GateState {
    audit: Arc<AuditLogWriter>,
    policy_db: PolicyDatabase,  // §3.5 imperative plug-ins
    cap_registry: CapabilityRegistry,
    telemetry: Arc<TelemetryWriter>,
}

impl GateState {
    pub async fn dispatch(
        &self,
        tool_call: ToolCall,
        attestation: Option<AttestationBody>,
    ) -> Result<ToolResult, GateError> {
        // 1. Pre-decision: policy lookup + attestation provenance check
        let decision = self.policy_db.evaluate(&tool_call, attestation.as_ref())?;
        // 2. Audit-log the decision (with attestation provenance)
        self.audit.record_decision(&tool_call, &decision, attestation.as_ref()).await?;
        // 3. Telemetry update (gate-fire counters)
        self.telemetry.increment_gate_fire(&tool_call.tool_name, &decision);
        // 4. Execute or deny
        match decision {
            Decision::Allow => self.execute_tool(tool_call).await,
            Decision::Warn { reason } => {
                eprintln!("[cuttle:warn] {}: {}", tool_call.tool_name, reason);
                self.execute_tool(tool_call).await
            }
            Decision::Deny { reason, suggested_exception } => {
                Err(GateError::Denied { reason, suggested_exception })
            }
            Decision::Prompt { .. } => unreachable!("Prompt collapses to Deny in non-interactive mode; interactive mode is OQ-9 follow-up"),
        }
    }
}
```

**Restart contract**: there is no in-process restart. Operator-visible behavior on panic: Cuttle aborts; operator sees the panic message; operator runs `cuttle` again. `state-coherence.json` (per PRD v3 §8 case 9) is written at clean shutdown only; an aborted process leaves the file stale, which the next-startup invariant detects and refuses to start without `--restored-from-backup` (or `--ignore-stale-state` if the operator confirms the prior abort was their own).

**v0.2 promotion criterion (OQ-11 revisit)**: if F-Cuttle-DISABLE fires due to operator routinely invoking `--ignore-stale-state` (proxy for "the gate is too brittle"), promote to OS-process isolation: `cuttle-gate` becomes a separate `cuttle-gated` daemon process supervising `cuttle-cli` over Unix sockets. Cost: typed IPC schema, IPC latency budget. Benefit: gate panics no longer abort the operator's session; gate restarts independently.

### 3.3 IPC schema for OQ-11 (forward-compatible v0.2 sketch)

For v0.1 the gate is in-process; for v0.2 it becomes out-of-process. To make the v0.2 promotion cheap, v0.1 already structures the gate API as a serializable message-passing surface even though all calls are local function calls today.

```rust
// Conceptual v0.2 IPC; v0.1 is the trait shape.
#[derive(Serialize, Deserialize)]
pub enum GateRequest {
    Dispatch { tool_call: ToolCall, attestation: Option<AttestationBody> },
    Heartbeat,
    Shutdown { reason: String },
}

#[derive(Serialize, Deserialize)]
pub enum GateResponse {
    Allowed { result: ToolResult },
    Denied { reason: String, suggested_exception: Option<ExceptionSuggestion> },
    Error { kind: GateErrorKind, message: String },
}
```

v0.1 implements the dispatch trait directly without serialization. v0.2 wraps the same trait behind a Unix-socket transport (likely `tokio::net::UnixStream` + `serde_json` or `bincode` for the wire format). The trait shape stays identical so consumers (`cuttle-runtime`) don't change.

### 3.4 Policy gate API: Allow / Warn / Deny graduation (resolves OQ-9)

PRD v3 §10 OQ-9 left this open per Carlos's "configurable risk dial" lens. v0.1 ships the **graduated** model:

```rust
#[derive(Serialize, Deserialize, Clone)]
pub enum Decision {
    Allow,
    Warn { reason: String },
    Deny {
        reason: String,
        suggested_exception: Option<ExceptionSuggestion>,
    },
    Prompt { question: PromptQuestion },  // collapses to Deny in non-interactive mode (PRD §8 case 3)
}
```

- **Allow**: tool executes; audit-log records the call.
- **Warn**: tool executes; audit-log records both the call AND the warning reason; `cuttle telemetry` counts warns separately (per PRD §6.1.6).
- **Deny**: tool does NOT execute; audit-log records the deny reason + the optional `suggested_exception` (so the deny path surfaces the operator option per PRD §8 case 8).
- **Prompt**: ask the operator at TTY; non-interactive mode collapses to Deny.

**Per-rule configuration** (lives in `~/.cuttle/config.toml` `[gate.rules]` section + per-rule TOML files in a future `~/.cuttle/policies/` directory at v0.2). Default for unmatched tool dispatches is `default_decision = "deny"` (PRD `[gate]` config). Rules promote to `Warn` only when the operator explicitly opts in for a specific tool/policy combination.

### 3.5 Policy DSL vs imperative plug-ins (resolves OQ-2)

**Decision: imperative Rust plug-ins for v0.1; DSL deferred to v0.N.**

OQ-2 framed this as "declarative DSL or imperative plug-ins." Imperative plug-ins (Rust functions registered against the gate at compile time) win for v0.1 because:

- Operator's 21 hooks (PRD v3 §2.1 baseline) are bash scripts; each one's logic is concrete. A DSL needs to express the union of those logics, which is design work that hasn't shipped at any complexity tier the operator already trusts.
- v0.1 single-operator scope means "ship policies in the binary, recompile to update" is acceptable. The dogfood week's gate-fire patterns inform what the v0.2 DSL needs to express.
- DSL adds a parser + interpreter as a trust-boundary surface. Better to defer until the v0.1 audit log shows what the DSL actually needs.

```rust
// crates/cuttle-gate/src/policies/mod.rs

/// Each policy implements this trait. PolicyDatabase holds Vec<Box<dyn Policy>>
/// and dispatches in priority order.
pub trait Policy: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, tool_call: &ToolCall, attestation: Option<&AttestationBody>) -> Option<Decision>;
}

// Example: bash destructive-shell policy (port of bash-guard.sh)
pub struct BashDestructiveShell;

impl Policy for BashDestructiveShell {
    fn name(&self) -> &str { "bash-destructive-shell" }
    fn evaluate(&self, tool_call: &ToolCall, attestation: Option<&AttestationBody>) -> Option<Decision> {
        if tool_call.tool_name != "Bash" { return None; }
        let cmd = tool_call.argument_str("command")?;
        if !is_destructive(cmd) { return None; }
        match attestation {
            // Per Option C §6.1.3 case 2: destructive shell needs explicit
            // operator-typed evidence with target enumeration + path allowlist + WHY.
            Some(att) if att.provenance() == &Provenance::Tty
                && validate_destructive_attestation(att.content()).is_ok() =>
            {
                Some(Decision::Allow)
            }
            _ => Some(Decision::Deny {
                reason: format!("Destructive shell: {}", cmd),
                suggested_exception: Some(ExceptionSuggestion::OptionC {
                    rule: "bash-destructive-shell",
                    required_fields: vec!["target_enumeration", "system_path_allowlist", "why"],
                }),
            }),
        }
    }
}
```

Per WV-06 / WV-07 disclaimer (PRD v3 §6.1.5): the type system enforces `provenance() == &Provenance::Tty` rejects model-emitted text. It does NOT enforce operator-INTENT vs operator-FATIGUE-KEYPRESS; F-Cuttle-FATIGUE measures the residual.

### 3.6 Predicate maintenance subsection (PRD v3 §9 constraint)

Per Carlos $100M/yr Google pre-submit anchor: pre-execution gating relocates cost from post-hoc audit to predicate engineering. v0.1 commits to:

- **Per-policy maintainer log**: each `Policy` impl carries a doc comment `/// Maintainer: <name> | Source: <PRD-or-DECISIONS-anchor> | Last-reviewed: <date>`. Reviewed at v0.1 ship and on every Anthropic API surface change.
- **Anthropic API surface change cadence**: weekly check (manual via `cuttle audit` of upstream Anthropic-SDK changelog; automated in v0.2). Any change that affects tool-call shape (e.g., new tool, parameter schema change) triggers a per-policy review.
- **Policy-set audit at REVIEW-1**: `code-review` skill on the policy set. Output: `docs/policy-coverage-v0.1.md` mapping the operator's 21 hooks to v0.1 policies (per SC-3).
- **Policy-fire telemetry**: each policy's gate-fire / gate-bypass / deny counts surface in `cuttle telemetry`. Policies with zero fires after dogfood week are flagged for review (either remove or strengthen).

### 3.7 Lockfile authentication mechanism (WV-02 closure)

PRD v3 §8 case 6 + WV-02: lockfile in `~/.cuttle/run/<session-id>.lock` is the nested-harness detection signal. WV-02 raised TOCTOU concern: an attacker who can craft a fake lockfile defeats the inheritance check.

**Mechanism**: lockfile contents include (a) the parent Cuttle process PID, (b) a 32-byte random session token, (c) a SHA-256 HMAC of the contents using a per-session signing key derived at session start. The signing key lives in process memory only (never written to disk); a child process inheriting the file descriptor can read the lockfile but cannot regenerate a valid HMAC because the signing key is not exported.

```rust
// crates/cuttle-runtime/src/lockfile.rs (sketch)

pub struct LockfileContents {
    pub parent_pid: u32,
    pub session_token: [u8; 32],
    pub hmac: [u8; 32],
}

pub fn write_lockfile(path: &LockfilePath, signing_key: &SigningKey) -> Result<(), LockfileError> {
    let contents = LockfileContents {
        parent_pid: std::process::id(),
        session_token: rand::thread_rng().gen(),
        hmac: [0u8; 32],
    };
    let serialized = bincode::serialize(&contents)?;
    let hmac = compute_hmac(signing_key, &serialized);
    // ... write atomically with O_CREAT | O_EXCL ...
}

pub fn verify_lockfile(path: &LockfilePath, expected_signing_key: &SigningKey)
    -> Result<LockfileContents, LockfileError>
{
    // Read; recompute HMAC; constant-time compare via subtle crate.
    // Failure modes:
    //   - File missing -> no nested harness; allow start.
    //   - File present + valid HMAC + parent_pid alive -> nested; refuse start unless CUTTLE_NESTED=allow.
    //   - File present + invalid HMAC -> attacker-crafted lockfile OR stale + key rotation.
    //                                    Per PRD §8 case 6: fail closed. Operator must manually inspect.
    //   - File present + valid HMAC + parent_pid not alive -> stale; require operator
    //                                                           confirmation to clear.
}
```

The `O_CREAT | O_EXCL` flag closes the create-then-fsync TOCTOU window for the create operation. The HMAC closes the read-then-trust TOCTOU window. The signing key in process memory closes the inherit-then-forge window.

### 3.8 Keychain prompt-rate budget (BP-05 closure)

PRD v3 §6.1.1 acknowledges the cross-purpose between CC-1 fail-closed gate and macOS Keychain per-process prompt-fatigue. v0.1 mechanism:

- **Session-scoped Keychain handle**: `cuttle-credential` opens the Keychain item once at session start, holds the handle in memory, and reuses it across in-session credential reads. The handle is in `ApiKey` zeroize-on-drop scope.
- **Restart-budget**: `~/.cuttle/state-coherence.json` (PRD §8 case 9 file) tracks Keychain-prompt count for the current dogfood week (rolling 7-day window). Budget: ≤5 prompts per week. If exceeded, Cuttle prints a warning to stderr at startup recommending the operator review whether the gate-restart frequency is high enough to warrant OQ-11 process-isolation promotion.
- **Override**: operator can set `[anthropic] keychain_always_allow = true` in `config.toml` to suppress the warning (acknowledging the per-process-isolation degradation). The setting is logged as a high-trust event in the audit log; it counts as evidence for F-Cuttle-DISABLE per `docs/falsifiers.md`.

### 3.9 Falsifier threshold refinement (per `docs/falsifiers.md`)

`docs/falsifiers.md` ships in pre-seal-draft state with first-draft thresholds. TDD §3.9 refines:

- **F-Cuttle-DISABLE**: `N=1` event of (a)-(c), `N=3` events of (d) override-snapshot-block per dogfood week. Refinement: also count `keychain_always_allow = true` (per §3.8) as a (d)-class event. **Updated v0.1 trigger**: any of {gate-disable, chain-rotation, --restored-from-backup, ≥3 override-snapshot, ≥1 keychain_always_allow}.
- **F-Cuttle-BEDROCK**: `N=20` SYSTEM-tier sessions, `p<0.05`. Statistical test: Mann-Whitney U two-sample (Cuttle gate-fire-vs-bypass rates vs toolkit skill-skip rates). v0.1 ships the data-collection mechanism (`cuttle telemetry --bedrock-export` produces a CSV); the statistical test runs in operator's R / Python post-week.
- **F-Cuttle-SUBSTRATE**: `X=2.0` ratio. Refinement: abandon-point definition is **any operator action that initiates a tool dispatch but does not see the tool result within 60 seconds AND is not followed by a deny/warn audit-log entry**. Captures both "operator killed Cuttle mid-dispatch" and "operator gave up waiting."
- **F-Cuttle-OPTION-C**: `N=10` per rule. Refinement: per-rule normalization formula = `bypass_count / (bypass_count + deny_count + accept_count)` (a fraction in [0,1]). Compare against toolkit's closest-mapping skill skip rate.
- **F-Cuttle-SNAPSHOT-DRIFT**: `N=3` overrides, `M=2` wrong. Operator-review rubric: a "wrong" override is one where the SYSTEM-tier task subsequently fails REVIEW-1 / REVIEW-2 / FIX-DOCS at v0.1's PRD-pipeline grain. Operator manually rates each override post-week.
- **F-Cuttle-MEMORY-DRIFT**: `R=0.8` (operator accepts ≥80% of proposed promotions). Refinement: normalize against operator's overall acceptance rate for genuine canonical-memory edits (proxy: track the operator's own MEMORY.md commits; compare promotion-acceptance rate to commit-creation rate).
- **F-Cuttle-FATIGUE**: `R_F=0.30` (≥30% trigram-matchable). Refinement: trigram tokenizer over both attestation body and the model's last 5 turns of context (not just immediately-prior turn; the operator may type 2-3 turns later). Temporal window = 5 conversation turns. **Privacy-sensitive logging contract** (per OQ-12 cross-link, resolved in TDD §5): the model context window snapshot is stored in a separate `~/.cuttle/audit/fatigue/<session-id>.jsonl` file with even tighter ACL than the regular audit log (mode 0600, owner-only, no group). The fatigue-detection job runs locally (no model call) at end-of-session.

### Cross-references and what §3 does NOT decide

- TDD §4: sandbox primitive crate impl.
- TDD §5: audit-log scheme + tool-registration tagging contract for `secret_bearing` flag (per WV-03).
- TDD §6: memory quarantine layout.
- The `~/.cuttle/policies/` directory for per-rule TOML config is NOT a v0.1 surface; v0.1 hardcodes policies in the binary.
- The DSL design is NOT v0.1 scope; the v0.2 follow-up takes operator's dogfood-week audit-log patterns as input.

### DECISIONS entries from §3

- **D-2026-04-26-19**: workspace structure (12 v0.1 crates) + crate-as-trust-boundary mapping.
- **D-2026-04-26-20**: in-process gate for v0.1 with `panic = "abort"` restart-via-cli; OQ-11 forward-compat trait shape preserved for v0.2 IPC promotion.
- **D-2026-04-26-21**: Allow/Warn/Deny/Prompt graduation (resolves OQ-9).
- **D-2026-04-26-22**: imperative Rust plug-ins for v0.1 policy expression (resolves OQ-2); DSL deferred to v0.N pending dogfood-week audit-log patterns.
- **D-2026-04-26-23**: lockfile HMAC + parent-PID + signing-key-in-memory (closes WV-02).
- **D-2026-04-26-24**: Keychain prompt-rate budget (5/week) + session-scoped handle + override-as-falsifier-evidence (closes BP-05).
- **D-2026-04-26-25**: falsifier threshold refinements (per `docs/falsifiers.md`); fatigue-detection privacy-sensitive logging contract (cross-links OQ-12 resolution in §5).

(D-19/20/21/22/23/24/25 to be added to DECISIONS.md when §3 commits.)

---

## §4. Sandbox primitive (resolves OQ-3; specifies T-005 contingency)

### 4.1 v0.1 primitive: `sandbox-exec` via Rust FFI

**Decision: `sandbox-exec(1)` invoked via Rust process spawning, with the App Sandbox profile language generated programmatically per project working directory.** Hybrid via `posix_spawn` rejected for v0.1 (additional engineering surface; sandbox-exec alone covers the v0.1 attack surface).

```rust
// crates/cuttle-sandbox/src/lib.rs

pub struct SandboxProfile {
    project_root: PathBuf,
    allowed_subprocess_paths: Vec<PathBuf>,  // /bin, /usr/bin, etc.; explicitly enumerated
    cpu_limit_secs: u32,
    mem_limit_mb: u32,
    max_open_fds: u32,
    max_subprocesses: u32,
}

impl SandboxProfile {
    pub fn for_project(project_root: PathBuf) -> Self {
        Self {
            project_root,
            allowed_subprocess_paths: default_allowed_binaries(),
            cpu_limit_secs: 60,        // operator-configurable in config.toml
            mem_limit_mb: 1024,
            max_open_fds: 256,
            max_subprocesses: 16,
        }
    }

    /// Renders the App Sandbox profile language (Apple's TinyScheme variant).
    pub fn render_sbpl(&self) -> String {
        format!(r#"
(version 1)
(deny default)
(allow process-fork)
(allow file-read*
  (subpath "{project}")
  (subpath "/usr/lib")
  (subpath "/System/Library"))
(allow file-write*
  (subpath "{project}"))
(allow process-exec
  {allowed_paths})
(deny network*)
(allow network* (remote ip "127.0.0.1:*"))
"#,
            project = self.project_root.display(),
            allowed_paths = self.allowed_subprocess_paths
                .iter()
                .map(|p| format!("(literal \"{}\")", p.display()))
                .collect::<Vec<_>>()
                .join(" "),
        )
    }

    /// Spawn `sandbox-exec -p <profile> <command>` and apply rlimits in
    /// the spawned program via tokio::process::Command::pre_exec.
    pub async fn spawn(&self, command: &str, args: &[&str]) -> Result<TokioChild, SandboxError> {
        let profile = self.render_sbpl();
        // ... write profile to temp file (tmpfs mount; auto-cleanup on Drop),
        // ... build Command with sandbox-exec wrapping,
        // ... apply rlimits via pre_exec hook,
        // ... spawn.
    }
}
```

**Why generate-the-profile-per-call rather than ship-static-profiles**: project root differs per session; the profile string must be regenerated. Caching by project-root hash is a v0.2 optimization.

**Network policy**: `(deny network*)` plus `(allow network* (remote ip "127.0.0.1:*"))` permits MCP-server-on-localhost (deferred to v0.2 anyway) and blocks all other network. Anthropic API client runs **outside** the sandbox (only sandboxed subprocesses are bash invocations), so the API connection is not affected.

### 4.2 Deprecation contingency (T-005)

Apple has deprecated the App Sandbox profile language documentation, but `sandbox-exec(1)` remains shipped and functional in macOS 14 / 15 / 16 (verified 2026-04). Cuttle's contingency plan for the day Apple withdraws `sandbox-exec`:

| Order | Candidate                                                    | Engineering cost | Coverage equivalence                                                                           |
| ----- | ------------------------------------------------------------ | ---------------- | ---------------------------------------------------------------------------------------------- |
| 1st   | **Endpoint Security framework**                              | Medium-high      | Better; per-syscall hooks; requires entitlement + system-extension installation.               |
| 2nd   | **Hypervisor framework + microVM**                           | High             | Strongest isolation; significant per-call latency. Use only if Endpoint Security is also gone. |
| 3rd   | **Apple Virtualization framework + lightweight Linux guest** | Highest          | Strongest; loses macOS-native filesystem semantics; operator confusion.                        |

The contingency triggers when (a) Apple announces removal in a beta release, OR (b) `sandbox-exec` returns ENOSYS / EPERM on a stable macOS release. v0.1 ships sandbox-exec; the contingency stays as a TDD-grade plan, not a v0.1 commitment.

### 4.3 Failure modes

- `sandbox-exec` not found: Cuttle fails closed at startup with structured error; operator instructed to verify macOS install.
- Profile compilation error: Cuttle fails closed; logs the rendered SBPL to `~/.cuttle/run/sandbox-debug.sbpl` (operator-readable, no secrets); operator inspects.
- Sandboxed program exceeds rlimit: program killed; audit log records the resource exhaustion; gate returns `Decision::Deny { reason: "resource limit exceeded", ... }` for the parent tool call.

### DECISIONS entry from §4

- **D-2026-04-26-26**: `sandbox-exec` via Rust FFI for v0.1 (resolves OQ-3); per-project SBPL generation; explicit allowed-subprocess-paths enumeration; tiered T-005 deprecation contingency (Endpoint Security → Hypervisor → Apple Virtualization).

---

## §5. Audit log + tamper-evident chain (resolves OQ-4 + OQ-12; specifies WV-03, BP-02 evaluator scope)

### 5.1 Tamper-evident chain scheme: HMAC chain (resolves OQ-4)

**Decision: HMAC chain for v0.1; Merkle tree with periodic root publication deferred to v0.2.** PRD v3 §6.1.1 already disclaims the chain is anti-forgetfulness/anti-drift, NOT anti-Sybil against operator-as-adversary (per T-003); the simpler scheme is honest given that scope.

```rust
// crates/cuttle-audit/src/chain.rs
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize, Deserialize)]
pub struct AuditEntry {
    pub seq: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event: AuditEvent,
    pub prev_hmac: [u8; 32],   // HMAC of the previous entry; 0u8;32 for genesis
    pub hmac: [u8; 32],         // HMAC over (seq || timestamp || event || prev_hmac) using session key
}

pub struct AuditChain {
    session_key: SecretKey,    // per-session; derived from per-day rotating key (TDD-grade key-management subsection deferred)
    last_seq: u64,
    last_hmac: [u8; 32],
    writer: tokio::fs::File,   // append-only; ~/.cuttle/audit/<yyyy-mm-dd>.jsonl
}

impl AuditChain {
    pub async fn append(&mut self, event: AuditEvent) -> Result<(), AuditError> {
        let next_seq = self.last_seq + 1;
        let timestamp = chrono::Utc::now();
        let mut mac = HmacSha256::new_from_slice(&self.session_key.0)?;
        mac.update(&next_seq.to_le_bytes());
        mac.update(&timestamp.timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        mac.update(&serde_json::to_vec(&event)?);
        mac.update(&self.last_hmac);
        let hmac: [u8; 32] = mac.finalize().into_bytes().into();
        let entry = AuditEntry { seq: next_seq, timestamp, event, prev_hmac: self.last_hmac, hmac };
        let line = serde_json::to_string(&entry)? + "\n";
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.sync_data().await?;  // flush to disk per entry; latency cost accepted
        self.last_seq = next_seq;
        self.last_hmac = hmac;
        // Update chain-head digest file for state-coherence (per PRD §8 case 9)
        self.update_chain_head().await?;
        Ok(())
    }
}
```

**Sync-per-entry**: yes, despite latency cost. Audit log is the evidence chain; un-flushed entries violate the "audit catches drift" claim if Cuttle crashes between `write` and `sync`.

### 5.2 Tool-registration tagging contract for `secret_bearing` flag (WV-03 closure)

```rust
// crates/cuttle-audit/src/tagging.rs
#[derive(Serialize, Deserialize)]
pub struct ToolTag {
    pub tool_name: String,
    pub secret_bearing: bool,
    pub pii_bearing: PiiPosture,        // §5.4
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub enum PiiPosture {
    Refused,                            // tool refused at registration (§5.4 option C)
    RedactAtWrite,                      // §5.4 option B
    RecordAsIs,                         // §5.4 option A; requires explicit operator opt-in per tool
}

/// Tool registry; populated at session start. Unknown tools default to:
/// secret_bearing = true, pii_bearing = RedactAtWrite. Safe-by-default per WV-03.
pub struct ToolRegistry {
    tools: HashMap<String, ToolTag>,
}

impl ToolRegistry {
    pub fn lookup_or_safe_default(&self, tool_name: &str) -> ToolTag {
        self.tools.get(tool_name).cloned().unwrap_or(ToolTag {
            tool_name: tool_name.to_string(),
            secret_bearing: true,
            pii_bearing: PiiPosture::RedactAtWrite,
            registered_at: chrono::Utc::now(),
        })
    }
}
```

**Audit-log writer behavior** for tool-result events:

```rust
match registry.lookup_or_safe_default(&tool_call.tool_name) {
    ToolTag { secret_bearing: false, pii_bearing: _, .. } => {
        // Safe to record full content digest
        AuditEvent::ToolResult {
            tool_name: tool_call.tool_name,
            length: result.len(),
            content_sha256: Some(sha256(result.as_bytes())),
            success: true,
        }
    }
    ToolTag { secret_bearing: true, .. } => {
        // Per WV-03: metadata only, no content digest.
        AuditEvent::ToolResult {
            tool_name: tool_call.tool_name,
            length: result.len(),
            content_sha256: None,
            success: true,
        }
    }
}
```

### 5.3 State-coherence file integrity (recursive, per BP-04)

`~/.cuttle/state-coherence.json` itself is a target for the same backup/restore failure mode. v0.1 mechanism:

```rust
#[derive(Serialize, Deserialize)]
pub struct StateCoherence {
    pub last_clean_shutdown: chrono::DateTime<chrono::Utc>,
    pub audit_chain_head: [u8; 32],         // current HMAC head per §5.1
    pub registry_chain_head: Option<[u8; 32]>,  // L5 reward-loop chain head per PRD v3 §6.1.2 row L5
    pub keychain_prompt_count_7day: u32,    // per BP-05 / D-24 budget
    pub state_coherence_self_hmac: [u8; 32], // HMAC of THIS file using a key derived from operator-machine binding (e.g., HMAC-SHA256(machine-uuid || cuttle-install-id))
}
```

The `state_coherence_self_hmac` covers the file's own contents using a key the operator owns. A backup-restore that includes `state-coherence.json` from another machine fails the self-HMAC check, triggering the `--restored-from-backup` requirement. A backup-restore from the same machine passes the self-HMAC check, but the `audit_chain_head` mismatch against the actual audit log triggers the requirement anyway. Two-fence design: even if one fence is bypassed, the other fires.

### 5.4 Audit-log PII posture (resolves OQ-12)

PRD v3 §10 OQ-12 listed three options: record-as-is, redact-at-write, refuse-tools-that-may-emit-PII.

**Decision: hybrid; RedactAtWrite is the safe default; tools opt-in to RecordAsIs (operator approval per tool); Refused is reserved for v0.2+ if a tool turns out to be unredactable.**

Redaction at write happens at the audit-log writer boundary using a `Redactor` trait per tool category. Default redactor masks email-shaped strings, SSN-shaped digits, IP-address-shaped tokens, and operator-configurable regexes from `~/.cuttle/config.toml` `[audit.redact]`. Redacted content digest is computed on the redacted text, NOT the original.

```rust
trait Redactor: Send + Sync {
    fn redact(&self, content: &str) -> String;
}

struct DefaultRedactor;

impl Redactor for DefaultRedactor {
    fn redact(&self, content: &str) -> String {
        let email_re = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        let mut redacted = email_re.replace_all(content, "<email-redacted>").to_string();
        // ... additional PII regexes ...
        redacted
    }
}
```

**Privacy-skill review** (per PRD v3 §11 step 3, D-09 / CC-3) audits the redactor coverage; recommendations land in REVIEW-2 findings registry.

### 5.5 Aggregation contract for telemetry (per D-04, D-09)

`cuttle telemetry` per PRD v3 §6.1.6 + D-2026-04-26-04 consumes the audit log; aggregation is on-demand (no separate aggregate file maintained out-of-band). Implementation:

- The audit log JSONL is the source of truth.
- `~/.cuttle/telemetry/aggregates.json` is an OPTIONAL cache populated by `cuttle telemetry --rebuild`; queries default to scanning JSONL on demand.
- Aggregations: gate-fire rates per (tool_name, decision), override attempts per tool, abandon-point counts (per F-Cuttle-SUBSTRATE definition in TDD §3.9), F-Cuttle-DISABLE event counts (per D-25 expanded scope).
- Latency target for `cuttle telemetry`: ≤2 seconds for a dogfood-week of audit events (~5MB JSONL). Larger ranges trigger a warning to use `--rebuild`.

### 5.6 Fitness-function automated evaluator scope (BP-02 partial close)

PRD v3 §12 + D-13 acknowledges the "fitness function" framing is aspirational at v0.1 ship: predicates ship as data; automated evaluator is TDD-grade scope. v0.1 scope:

- `cuttle-falsifiers` crate (per D-19) implements the **data-collection** side: each falsifier predicate has a corresponding audit-event variant that the writer emits (e.g., `AuditEvent::GateDisabled { rule_id, operator_reason }`).
- `cuttle telemetry --falsifier-eval` runs the per-predicate evaluation on demand using the data; outputs to stdout in human-readable form + machine-readable JSON.
- Automated periodic evaluation is v0.2 scope (would require `cron`-like or systemd-timer-like surface; out of v0.1 §6.1 scope).

### DECISIONS entries from §5

- **D-2026-04-26-27**: HMAC chain for v0.1 audit log (resolves OQ-4); `sync_data()` per entry; key management TDD-grade subsection deferred.
- **D-2026-04-26-28**: tool-registration `secret_bearing` flag with safe-by-default for unknown tools (closes WV-03).
- **D-2026-04-26-29**: state-coherence self-HMAC + audit-chain-head two-fence design for backup/restore integrity (closes BP-04 at TDD-grain).
- **D-2026-04-26-30**: audit-log PII posture = RedactAtWrite default + per-tool RecordAsIs opt-in (resolves OQ-12); privacy-skill review at REVIEW-2.

---

## §6. Memory + quarantine layout (specifies §6.1.5 cross-session memory invariant)

### 6.1 Filesystem layout

```text
~/.cuttle/memory/
├── canonical/                       # operator-promoted; loaded as trusted
│   ├── MEMORY.md                    # operator-authored index
│   └── <topic>.md                   # operator-promoted sidecars
└── quarantine/                      # model-authored; loaded with untrusted-by-default framing
    ├── pending/                     # awaiting operator review
    │   └── <session>-<seq>.md
    └── rejected/                    # operator rejected (kept N=30 days for re-review)
        └── <session>-<seq>.md
```

Mode: `canonical/` 0700 (operator owns); `quarantine/` 0700; both on the operator-only ACL inherited from `~/.cuttle/`.

### 6.2 Promotion workflow

```rust
// crates/cuttle-memory/src/promotion.rs
pub enum PromotionDecision {
    Promote { canonical_path: PathBuf },
    Reject { reason: String },
    Defer,  // operator chose to leave in quarantine for now
}

pub async fn prompt_promotion(
    quarantine_entry: &Path,
    cap: &TtyInputCap,    // capability-token witness per D-17
) -> Result<PromotionDecision, MemoryError> {
    // 1. Display quarantine entry contents to TTY.
    // 2. Display proposed canonical path.
    // 3. Prompt operator (via the input-handler crate which holds TtyInputCap).
    // 4. Operator response is OperatorAuthoredText (per §2.4).
    // 5. Audit-log the decision with full provenance.
}
```

**Cross-session loading** (per PRD v3 §6.1.5):

- At session start, `cuttle-memory` loads `canonical/MEMORY.md` and its sidecars as `OperatorAuthoredText` (trusted).
- It ALSO scans `quarantine/pending/` and surfaces each entry to the model with `<untrusted-pending-promotion>` tags in the system prompt.
- The model can REFERENCE quarantine entries but cannot promote them; promotion requires `prompt_promotion()` which requires `&TtyInputCap`.

### 6.3 Per-session vs per-project scope

v0.1 scope: **per-project** memory at `~/.cuttle/memory/projects/<project-key>/` mirrors the layout above; **global** memory at `~/.cuttle/memory/` (no `projects/` prefix). Loading order: global first, then per-project (per-project overrides).

### 6.4 Quarantine retention

- `quarantine/pending/`: kept indefinitely until promoted or rejected.
- `quarantine/rejected/`: kept N=30 days then auto-purged. Operator can reset via `cuttle memory --rescan-rejected` to recover within the window.
- Audit log records every quarantine write, every promotion, every rejection (per F-Cuttle-MEMORY-DRIFT measurement source per `docs/falsifiers.md`).

### DECISIONS entry from §6

- **D-2026-04-26-31**: memory filesystem layout (canonical/ + quarantine/{pending,rejected}/); promotion requires TtyInputCap; cross-session loading distinguishes provenance via type-system primitives (per D-17). Quarantine retention 30-day rejected-window.

---

## TDD complete (v0)

§1 through §6 cover OQ-1 (language), OQ-2 (DSL vs imperative), OQ-3 (sandbox primitive), OQ-4 (audit-log scheme), OQ-6 (filesystem), OQ-9 (Allow/Warn/Deny graduation), OQ-11 (process isolation forward-compat), OQ-12 (PII posture). OQ-5 (skill-trust model) is threat-model-skill scope per PRD v3 §10. OQ-7 (public name) is Phase 2 prep. OQ-8 (telemetry posture) is privacy-skill scope. OQ-10 (integration-vs-ablation) is Phase-1-equivalent-validation scope.

Closes 8 of 12 OQs at TDD-grain; the remaining 4 (OQ-5, 7, 8, 10) are correctly not-TDD-scope per PRD v3 §10.

Pipeline next: REVIEW-1 (`code-review` skill on PRD v3 + TDD v0) → REVIEW-2 (`legal-review` + `threat-model` + `privacy`) → FIX-DOCS → DESIGN (`system-design` skill) → API (`api-design` skill) → Implementation begins.

DECISIONS added in this TDD: D-15 (Rust), D-16 (filesystem), D-17 (domain primitives + capability tokens), D-18 (serialization round-trip), D-19 (workspace), D-20 (in-process gate v0.1), D-21 (Allow/Warn/Deny graduation), D-22 (imperative plug-ins), D-23 (lockfile HMAC), D-24 (Keychain rate-budget), D-25 (falsifier thresholds + fatigue logging), D-26 (sandbox-exec), D-27 (HMAC audit chain), D-28 (tool tagging), D-29 (state-coherence self-HMAC), D-30 (PII posture), D-31 (memory layout). Total: 17 TDD-grade decisions on top of 14 PRD-grade decisions = 31 decisions in DECISIONS.md at TDD complete.
