//! `cuttle audit verify` subcommand handler.
//!
//! Composes `cuttle_audit::verify_chain(path, key)` and reports the
//! verified head HMAC (or the specific entry that failed). The chain
//! key is read from a separate keyfile path rather than the command
//! line so the secret never enters shell history or `ps aux` output.
//!
//! Keyfile formats accepted:
//! - 32 raw bytes (file is exactly 32 bytes; common case for an
//!   operator who used `head -c 32 /dev/urandom > chain.key`).
//! - 64 hex chars (with or without surrounding whitespace; common
//!   case for a key copied from a Keychain export or a docs example).
//!
//! Anything else fails closed with a specific error rather than trying
//! to guess. v0.1 will gain a `cuttle audit gen-key` helper once the
//! session-engine wires Keychain integration.

use crate::args::AuditVerifyArgs;
use cuttle_audit::{verify_chain, AuditChainKey};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuditCmdError {
    #[error("audit log not found at {0}")]
    AuditLogMissing(PathBuf),

    #[error("chain key file not found at {0}")]
    ChainKeyFileMissing(PathBuf),

    #[error("chain key file read failed: {0}")]
    ChainKeyRead(std::io::Error),

    #[error(
        "chain key file at {path} is {got} bytes; expected exactly 32 raw bytes \
         or 64 hex characters"
    )]
    ChainKeyWrongLength { path: PathBuf, got: usize },

    #[error("chain key file at {path} contains invalid hex: {detail}")]
    ChainKeyInvalidHex { path: PathBuf, detail: String },

    #[error("chain verification failed at entry seq={seq}: HMAC mismatch")]
    HmacMismatch { seq: u64 },

    #[error("audit log read failed: {0}")]
    AuditRead(cuttle_audit::AuditError),

    #[error("writing report to stdout failed: {0}")]
    WriteFailed(#[from] std::io::Error),
}

pub fn run<W: Write>(args: &AuditVerifyArgs, out: &mut W) -> Result<(), AuditCmdError> {
    if !args.audit_log.exists() {
        return Err(AuditCmdError::AuditLogMissing(args.audit_log.clone()));
    }
    if !args.chain_key_file.exists() {
        return Err(AuditCmdError::ChainKeyFileMissing(
            args.chain_key_file.clone(),
        ));
    }

    let key_bytes = std::fs::read(&args.chain_key_file).map_err(AuditCmdError::ChainKeyRead)?;
    let key_array = parse_chain_key_bytes(&key_bytes, &args.chain_key_file)?;
    let key = AuditChainKey::from_bytes(key_array);

    match verify_chain(&args.audit_log, &key) {
        Ok(head) => {
            writeln!(out, "audit log verified.")?;
            writeln!(out, "chain head: {}", hex_encode(&head))?;
            Ok(())
        }
        Err(cuttle_audit::AuditError::HmacMismatch { seq }) => {
            Err(AuditCmdError::HmacMismatch { seq })
        }
        Err(other) => Err(AuditCmdError::AuditRead(other)),
    }
}

/// Parse a chain key from the keyfile bytes. Accepts:
/// - exactly 32 raw bytes → returned as-is
/// - 64 hex chars (after stripping ASCII whitespace) → decoded
///
/// Anything else fails with `ChainKeyWrongLength`.
fn parse_chain_key_bytes(bytes: &[u8], path: &Path) -> Result<[u8; 32], AuditCmdError> {
    if bytes.len() == 32 {
        let mut out = [0u8; 32];
        out.copy_from_slice(bytes);
        return Ok(out);
    }
    // Try hex parse: strip ASCII whitespace, expect exactly 64 hex chars.
    let stripped: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if stripped.len() != 64 {
        return Err(AuditCmdError::ChainKeyWrongLength {
            path: path.to_path_buf(),
            got: bytes.len(),
        });
    }
    let mut out = [0u8; 32];
    for (i, pair) in stripped.chunks_exact(2).enumerate() {
        let hi = hex_nibble(pair[0]).ok_or_else(|| AuditCmdError::ChainKeyInvalidHex {
            path: path.to_path_buf(),
            detail: format!("non-hex char at position {}", i * 2),
        })?;
        let lo = hex_nibble(pair[1]).ok_or_else(|| AuditCmdError::ChainKeyInvalidHex {
            path: path.to_path_buf(),
            detail: format!("non-hex char at position {}", i * 2 + 1),
        })?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn hex_encode(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttle_audit::{AuditChain, AuditEvent};
    use std::path::Path;
    use tempfile::TempDir;

    fn write_audit_log(path: &Path, key_bytes: [u8; 32], events: Vec<AuditEvent>) {
        let key = AuditChainKey::from_bytes(key_bytes);
        let mut chain = AuditChain::open(path.to_path_buf(), key).unwrap();
        for ev in events {
            chain.append(ev).unwrap();
        }
    }

    fn dispatch(tool: &str) -> AuditEvent {
        AuditEvent::ToolDispatch {
            tool_name: tool.into(),
            argument_summary: "x".into(),
        }
    }

    #[test]
    fn parse_chain_key_accepts_raw_32_bytes() {
        let bytes = [7u8; 32];
        let parsed = parse_chain_key_bytes(&bytes, &PathBuf::from("/x")).unwrap();
        assert_eq!(parsed, bytes);
    }

    #[test]
    fn parse_chain_key_accepts_64_hex_chars() {
        let hex = b"00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let parsed = parse_chain_key_bytes(hex, &PathBuf::from("/x")).unwrap();
        assert_eq!(parsed[0], 0x00);
        assert_eq!(parsed[1], 0x11);
        assert_eq!(parsed[15], 0xff);
    }

    #[test]
    fn parse_chain_key_strips_surrounding_whitespace_in_hex() {
        let hex = b"  00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff\n";
        let parsed = parse_chain_key_bytes(hex, &PathBuf::from("/x")).unwrap();
        assert_eq!(parsed[0], 0x00);
    }

    #[test]
    fn parse_chain_key_rejects_wrong_length() {
        let bytes = [7u8; 16];
        let err = parse_chain_key_bytes(&bytes, &PathBuf::from("/x")).unwrap_err();
        assert!(matches!(
            err,
            AuditCmdError::ChainKeyWrongLength { got: 16, .. }
        ));
    }

    #[test]
    fn parse_chain_key_rejects_invalid_hex() {
        let hex = b"zzz12233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let err = parse_chain_key_bytes(hex, &PathBuf::from("/x")).unwrap_err();
        assert!(matches!(err, AuditCmdError::ChainKeyInvalidHex { .. }));
    }

    #[test]
    fn run_verifies_clean_chain() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("audit.jsonl");
        let key_path = tmp.path().join("chain.key");
        let key_bytes = [9u8; 32];
        write_audit_log(&log, key_bytes, vec![dispatch("bash"), dispatch("read")]);
        std::fs::write(&key_path, key_bytes).unwrap();

        let args = AuditVerifyArgs {
            audit_log: log,
            chain_key_file: key_path,
        };
        let mut out = Vec::new();
        run(&args, &mut out).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("verified"));
    }

    #[test]
    fn run_detects_hmac_mismatch_on_wrong_key() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("audit.jsonl");
        let key_path = tmp.path().join("wrong.key");
        write_audit_log(&log, [9u8; 32], vec![dispatch("bash")]);
        std::fs::write(&key_path, [11u8; 32]).unwrap();

        let args = AuditVerifyArgs {
            audit_log: log,
            chain_key_file: key_path,
        };
        let mut out = Vec::new();
        let err = run(&args, &mut out).unwrap_err();
        assert!(matches!(err, AuditCmdError::HmacMismatch { .. }));
    }

    #[test]
    fn run_errors_when_audit_log_missing() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("nope.jsonl");
        let key_path = tmp.path().join("key");
        std::fs::write(&key_path, [9u8; 32]).unwrap();
        let args = AuditVerifyArgs {
            audit_log: log,
            chain_key_file: key_path,
        };
        let mut out = Vec::new();
        let err = run(&args, &mut out).unwrap_err();
        assert!(matches!(err, AuditCmdError::AuditLogMissing(_)));
    }

    #[test]
    fn run_errors_when_keyfile_missing() {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("audit.jsonl");
        write_audit_log(&log, [9u8; 32], vec![dispatch("bash")]);
        let key_path = tmp.path().join("missing.key");
        let args = AuditVerifyArgs {
            audit_log: log,
            chain_key_file: key_path,
        };
        let mut out = Vec::new();
        let err = run(&args, &mut out).unwrap_err();
        assert!(matches!(err, AuditCmdError::ChainKeyFileMissing(_)));
    }
}
