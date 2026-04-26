//! Capability tokens for constructor authorization.
//!
//! Per `docs/TDD.md` §2.5 and D-2026-04-26-17. Capability tokens are unforgeable
//! marker types; only specific modules can mint them. Other modules can hold and
//! pass references but cannot construct.
//!
//! [`TtyInputCap`] is minted by `cuttle-input` (via the `pub` re-export of the
//! `__internal_input_cap_factory` module, which is documented as a load-bearing
//! trust-boundary surface in TDD §3 and is the ONLY allowed minter). Other
//! crates construct the cap only by holding a reference returned from
//! `cuttle-input::Session::tty_input_cap()`.

/// Capability proving the holder has TTY-input authority.
///
/// Constructed once per session by the input-handler crate. Does not implement
/// `Clone`, `Copy`, `Serialize`, `Deserialize`, or any other trait that would
/// allow forging or persisting. Holding a reference is the only legitimate
/// way to prove TTY-input authority.
pub struct TtyInputCap {
    // Private field: cannot be constructed via struct literal outside this module.
    _private: (),
}

/// Hidden re-export module: `cuttle-input` calls
/// `cuttle_gate::capabilities::__internal_input_cap_factory::issue()` to mint
/// a `TtyInputCap` at session start. Documented as a load-bearing
/// trust-boundary surface; reviewing this module is part of REVIEW-1.
///
/// `#[doc(hidden)]` keeps it out of public docs but visible at `pub` for
/// cross-crate use.
#[doc(hidden)]
pub mod __internal_input_cap_factory {
    use super::TtyInputCap;

    /// Mint a `TtyInputCap`. ONLY callable from `cuttle-input::Session::start`
    /// per the documented trust contract. REVIEW-1 verifies no other call sites.
    pub fn issue() -> TtyInputCap {
        TtyInputCap { _private: () }
    }
}
