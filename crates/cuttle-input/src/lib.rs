//! Cuttle TTY input handler.
//!
//! Mints [`cuttle_gate::TtyInputCap`] once per session via the
//! `__internal_input_cap_factory` documented surface (per `docs/TDD.md` §2.5).
//! `cuttle-input` is the ONLY crate authorized to mint the cap. REVIEW-1
//! verifies no other crate calls the factory.

use cuttle_gate::{capabilities::__internal_input_cap_factory, TtyInputCap};

/// Per-session input handler. Holds the [`TtyInputCap`] and exposes it to
/// the runtime via `tty_input_cap()`.
pub struct Session {
    cap: TtyInputCap,
}

impl Session {
    /// Start a new session and mint the per-session capability token.
    /// Called exactly once per Cuttle process lifetime by `cuttle-runtime`.
    pub fn start() -> Self {
        Self {
            cap: __internal_input_cap_factory::issue(),
        }
    }

    /// Borrow the TTY input capability. Pass to APIs (e.g.,
    /// [`cuttle_gate::AttestationBody::from_tty_input`]) that require proof
    /// of TTY-input authority.
    pub fn tty_input_cap(&self) -> &TtyInputCap {
        &self.cap
    }

    /// Read a line from the TTY and wrap it in an `AttestationBody` with
    /// `Provenance::Tty`. The actual TTY read is stubbed for now; v0.1
    /// implementation in TDD §3 specifies the readline + termios contract.
    pub fn read_attestation_line(&self, _prompt: &str) -> cuttle_gate::AttestationBody {
        // STUB: real implementation reads from stdin with raw-mode termios.
        // For v0.0.1 scaffolding we return a placeholder; TDD §3 + §6 specify
        // the interactive behavior.
        let line = String::new();
        cuttle_gate::AttestationBody::from_tty_input(&self.cap, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttle_gate::{AttestationBody, Provenance};

    #[test]
    fn session_mints_cap() {
        let s = Session::start();
        let cap = s.tty_input_cap();
        let a = AttestationBody::from_tty_input(cap, "op reason".to_string());
        assert_eq!(a.provenance(), &Provenance::Tty);
    }
}
