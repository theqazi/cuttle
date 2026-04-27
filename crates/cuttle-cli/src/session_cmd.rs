//! `cuttle session start` subcommand handler. Multi-turn streaming REPL.
//!
//! Per session-6 design decisions:
//! - **Session id**: UTC timestamp + 8 random hex chars
//!   (`2026-04-26T15-30-45Z-a1b2c3d4`). Sortable, unique, no collision
//!   risk for a single operator.
//! - **Session directory**: `<cuttle_home>/sessions/<id>/`. Created with
//!   default umask; per-file modes are tightened below.
//! - **Chain key**: 32 bytes from `rand::random()`, written to
//!   `chain.key` in the session dir with mode 0600 (owner-only). The
//!   operator points `cuttle audit verify --chain-key-file <PATH>` at
//!   this file post-session.
//! - **Audit log**: `audit.jsonl` in the session dir, opened with the
//!   freshly-minted chain key. Each turn appends `UserPrompt` then
//!   `AssistantResponse` events; content stays in the transcript.
//! - **Transcript**: `transcript.jsonl` in the session dir, mode 0600.
//!   Each line is `{"role": "user"|"assistant", "content": "...",
//!   "timestamp_utc": "..."}`. The audit log carries digest + length;
//!   the transcript carries the actual text the operator typed and the
//!   model returned.
//!
//! Conversation history (Vec<Message>) is in-memory only for v0.0.14.
//! Each turn appends both the user message and the assistant message to
//! the history, then sends the full history on the next turn.
//!
//! REPL exit: EOF (Ctrl+D) or `/quit` / `/exit` typed at the prompt.
//! Ctrl+C terminates the process; SIGINT-handling for graceful mid-stream
//! abort is v0.0.15+.

use crate::args::SessionStartArgs;
use crate::banner;
use crate::paths;
use chrono::Utc;
use cuttle_anthropic::{
    AnthropicClient, AnthropicError, ClientConfig, KnownModel, Message, Model, Request,
    SystemContent,
};
use cuttle_audit::{AuditChain, AuditChainKey, AuditEvent};
use cuttle_credential::{ApiKey, ResolveError};
use futures_util::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionCmdError {
    #[error("could not load API key: {0}")]
    ApiKey(#[from] ResolveError),

    #[error("could not resolve cuttle home directory; set CUTTLE_HOME or HOME")]
    NoCuttleHome,

    #[error("could not create session directory at {path}: {source}")]
    SessionDirCreate {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("could not write chain key file at {path}: {source}")]
    ChainKeyWrite {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("could not open audit log at {path}: {source}")]
    AuditOpen {
        path: PathBuf,
        source: cuttle_audit::AuditError,
    },

    #[error("audit log write failed: {0}")]
    AuditAppend(cuttle_audit::AuditError),

    #[error("could not open transcript at {path}: {source}")]
    TranscriptOpen {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("transcript write failed: {0}")]
    TranscriptWrite(std::io::Error),

    #[error("could not build Anthropic client: {0}")]
    ClientBuild(AnthropicError),

    #[error("Anthropic API call failed: {0}")]
    Api(#[from] AnthropicError),

    #[error("could not build tokio runtime: {0}")]
    Runtime(std::io::Error),

    #[error("stdin read failed: {0}")]
    StdinRead(std::io::Error),

    #[error("writing to stdout failed: {0}")]
    Stdout(std::io::Error),
}

#[derive(Serialize)]
struct TranscriptEntry<'a> {
    role: &'a str,
    content: &'a str,
    timestamp_utc: String,
}

/// Synchronous entry point. Mints session, runs REPL until EOF/quit,
/// returns. Conversation history is dropped on exit (no resume in
/// v0.0.14; the audit log + transcript are the durable record).
pub fn run(args: &SessionStartArgs) -> Result<(), SessionCmdError> {
    // Print the wordmark up front so the operator gets instant visual
    // feedback (and so the banner is visible even if a subsequent
    // resolve step fails). Per-session paths print after the directory
    // is minted, just below.
    print!("{}", banner::render());
    use std::io::Write as _;
    let _ = std::io::stdout().flush();

    // Resolve session paths up front so any failure here aborts before
    // we mint a key or open the model client.
    let cuttle_home = paths::cuttle_home().ok_or(SessionCmdError::NoCuttleHome)?;
    let session_id = mint_session_id();
    let session_dir = cuttle_home.join("sessions").join(&session_id);
    let audit_log_path = session_dir.join("audit.jsonl");
    let chain_key_path = session_dir.join("chain.key");
    let transcript_path = session_dir.join("transcript.jsonl");

    std::fs::create_dir_all(&session_dir).map_err(|source| SessionCmdError::SessionDirCreate {
        path: session_dir.clone(),
        source,
    })?;

    // Mint chain key, write 0600.
    let chain_key_bytes: [u8; 32] = rand::random();
    write_secret_file(&chain_key_path, &chain_key_bytes).map_err(|source| {
        SessionCmdError::ChainKeyWrite {
            path: chain_key_path.clone(),
            source,
        }
    })?;
    let chain_key = AuditChainKey::from_bytes(chain_key_bytes);

    // Open the audit chain + transcript.
    let audit_chain = AuditChain::open(audit_log_path.clone(), chain_key).map_err(|source| {
        SessionCmdError::AuditOpen {
            path: audit_log_path.clone(),
            source,
        }
    })?;
    let transcript = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode_secret_if_unix()
        .open(&transcript_path)
        .map_err(|source| SessionCmdError::TranscriptOpen {
            path: transcript_path.clone(),
            source,
        })?;

    // API key + Anthropic client.
    let api_key = ApiKey::resolve(&args.api_key_env)?;
    let client =
        AnthropicClient::new(ClientConfig::default()).map_err(SessionCmdError::ClientBuild)?;

    // Print per-session paths (banner already printed above).
    print_session_paths(&session_id, &session_dir);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(SessionCmdError::Runtime)?;

    rt.block_on(repl_loop(
        &client,
        &api_key,
        &args.model,
        args.max_tokens,
        args.system.as_deref(),
        audit_chain,
        transcript,
    ))
}

fn mint_session_id() -> String {
    let now = Utc::now();
    let stamp = now.format("%Y-%m-%dT%H-%M-%SZ").to_string();
    let suffix: u32 = rand::random();
    format!("{stamp}-{suffix:08x}")
}

fn print_session_paths(session_id: &str, session_dir: &std::path::Path) {
    println!("session: {session_id}");
    println!("  audit log:  {}/audit.jsonl", session_dir.display());
    println!("  chain key:  {}/chain.key", session_dir.display());
    println!("  transcript: {}/transcript.jsonl", session_dir.display());
    println!();
}

/// REPL: read prompt → audit + transcript → stream response → audit +
/// transcript → loop. Returns Ok on clean exit (EOF / /quit), Err on
/// any underlying failure.
async fn repl_loop(
    client: &AnthropicClient,
    api_key: &ApiKey,
    model_name: &str,
    max_tokens: u32,
    system: Option<&str>,
    mut audit: AuditChain,
    mut transcript: std::fs::File,
) -> Result<(), SessionCmdError> {
    let model = parse_model(model_name);
    let mut history: Vec<Message> = Vec::new();
    let stdin = std::io::stdin();
    let stdin_lock = stdin.lock();
    let mut lines = stdin_lock.lines();

    loop {
        // Prompt indicator.
        print!("you> ");
        std::io::stdout().flush().map_err(SessionCmdError::Stdout)?;

        // Read one line of operator input. EOF → clean exit.
        let raw = match lines.next() {
            None => {
                println!();
                return Ok(());
            }
            Some(Ok(line)) => line,
            Some(Err(e)) => return Err(SessionCmdError::StdinRead(e)),
        };
        let user_prompt = raw.trim();
        if user_prompt.is_empty() {
            // Empty line: skip the turn rather than burn an API call.
            continue;
        }
        if matches!(user_prompt, "/quit" | "/exit") {
            return Ok(());
        }

        // Append UserPrompt audit event + transcript line.
        let user_digest = sha256(user_prompt.as_bytes());
        let user_event = AuditEvent::UserPrompt {
            content_sha256: user_digest,
            length: user_prompt.len(),
        };
        audit
            .append(user_event)
            .map_err(SessionCmdError::AuditAppend)?;
        write_transcript_line(&mut transcript, "user", user_prompt)?;

        history.push(Message::user_text(user_prompt));

        // Build request from full history.
        let mut request = Request::new(model.clone(), history.clone(), max_tokens);
        if let Some(s) = system {
            request.system = Some(SystemContent::Plain(s.to_string()));
        }

        // Stream the response. Accumulate the assistant text + token
        // usage as deltas arrive so we can audit + transcribe at end.
        print!("claude> ");
        std::io::stdout().flush().map_err(SessionCmdError::Stdout)?;

        let mut stream = client.messages_stream(api_key, &request).await?;
        let mut assistant_text = String::new();
        let mut last_input_tokens: u32 = 0;
        let mut last_output_tokens: u32 = 0;

        while let Some(item) = stream.next().await {
            let event = item?;
            if let Some(text) = event.text_delta() {
                assistant_text.push_str(text);
                print!("{text}");
                std::io::stdout().flush().map_err(SessionCmdError::Stdout)?;
            }
            // MessageStart and MessageDelta both carry usage info we
            // want; prefer MessageDelta's value (final accounting).
            if let cuttle_anthropic::StreamEvent::MessageStart { message } = &event {
                last_input_tokens = message.usage.input_tokens;
                last_output_tokens = message.usage.output_tokens;
            }
            if let cuttle_anthropic::StreamEvent::MessageDelta { usage, .. } = &event {
                if usage.input_tokens > 0 {
                    last_input_tokens = usage.input_tokens;
                }
                if usage.output_tokens > 0 {
                    last_output_tokens = usage.output_tokens;
                }
            }
            if event.is_message_stop() {
                break;
            }
        }
        println!();

        // Append AssistantResponse audit event + transcript line.
        let asst_digest = sha256(assistant_text.as_bytes());
        let asst_event = AuditEvent::AssistantResponse {
            content_sha256: asst_digest,
            length: assistant_text.len(),
            input_tokens: last_input_tokens,
            output_tokens: last_output_tokens,
        };
        audit
            .append(asst_event)
            .map_err(SessionCmdError::AuditAppend)?;
        write_transcript_line(&mut transcript, "assistant", &assistant_text)?;

        history.push(Message::assistant_text(assistant_text));
    }
}

fn parse_model(s: &str) -> Model {
    match s {
        "claude-opus-4-7" => Model::Known(KnownModel::ClaudeOpus47),
        "claude-sonnet-4-6" => Model::Known(KnownModel::ClaudeSonnet46),
        "claude-haiku-4-5-20251001" => Model::Known(KnownModel::ClaudeHaiku45),
        other => Model::Custom(other.to_string()),
    }
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

fn write_transcript_line(
    transcript: &mut std::fs::File,
    role: &str,
    content: &str,
) -> Result<(), SessionCmdError> {
    let entry = TranscriptEntry {
        role,
        content,
        timestamp_utc: Utc::now().to_rfc3339(),
    };
    let line = serde_json::to_string(&entry)
        .map_err(|e| SessionCmdError::TranscriptWrite(std::io::Error::other(e)))?;
    writeln!(transcript, "{line}").map_err(SessionCmdError::TranscriptWrite)?;
    transcript
        .sync_data()
        .map_err(SessionCmdError::TranscriptWrite)?;
    Ok(())
}

/// Write `bytes` to `path` with restrictive owner-only permissions on
/// Unix (0600). On non-Unix platforms, falls back to default permissions
/// — v0.1 is macOS-only so this branch is never taken in production.
fn write_secret_file(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(path)?;
        f.write_all(bytes)?;
        f.sync_data()?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let mut f = OpenOptions::new().create_new(true).write(true).open(path)?;
        f.write_all(bytes)?;
        f.sync_data()?;
        Ok(())
    }
}

/// Extension trait that adds `mode_secret_if_unix()` to `OpenOptions`.
/// On Unix sets mode 0600 on file creation; on other platforms is a no-op.
trait OpenOptionsSecretMode {
    fn mode_secret_if_unix(&mut self) -> &mut Self;
}

impl OpenOptionsSecretMode for std::fs::OpenOptions {
    #[cfg(unix)]
    fn mode_secret_if_unix(&mut self) -> &mut Self {
        use std::os::unix::fs::OpenOptionsExt;
        self.mode(0o600)
    }
    #[cfg(not(unix))]
    fn mode_secret_if_unix(&mut self) -> &mut Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint_session_id_is_unique_per_call() {
        let a = mint_session_id();
        let b = mint_session_id();
        assert_ne!(a, b);
    }

    #[test]
    fn mint_session_id_starts_with_iso_date() {
        let id = mint_session_id();
        // ISO 8601 prefix `YYYY-MM-DD`.
        assert!(
            id.chars().nth(4) == Some('-') && id.chars().nth(7) == Some('-'),
            "expected iso date prefix, got {id}"
        );
    }

    #[test]
    fn parse_model_recognizes_known_ids() {
        assert_eq!(
            parse_model("claude-sonnet-4-6"),
            Model::Known(KnownModel::ClaudeSonnet46)
        );
    }

    #[test]
    fn parse_model_falls_back_to_custom() {
        assert_eq!(
            parse_model("claude-future-9000"),
            Model::Custom("claude-future-9000".to_string())
        );
    }

    #[test]
    fn sha256_matches_known_value() {
        // SHA-256 of "abc" per FIPS 180-4 spec.
        let d = sha256(b"abc");
        let expected = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ];
        assert_eq!(d, expected);
    }

    #[cfg(unix)]
    #[test]
    fn write_secret_file_uses_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("secret.bin");
        write_secret_file(&path, b"abc").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        // Mode bitmask: only the low 9 bits matter.
        assert_eq!(mode & 0o777, 0o600, "expected 0600, got {mode:o}");
    }

    #[test]
    fn write_secret_file_refuses_to_clobber_existing_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("secret.bin");
        std::fs::write(&path, b"existing").unwrap();
        // Second write must refuse via create_new.
        let r = write_secret_file(&path, b"new");
        assert!(r.is_err());
    }

    #[test]
    fn run_errors_when_no_credential_anywhere() {
        // Runs the path up to ApiKey::resolve which fails fast.
        // This validates that:
        // (a) mint_session_id + create_dir succeed,
        // (b) chain key file gets written under a temp CUTTLE_HOME,
        // (c) ApiKey::resolve failure surfaces cleanly when neither env
        //     nor Keychain holds a credential.
        let tmp = tempfile::TempDir::new().unwrap();
        let prev = std::env::var("CUTTLE_HOME").ok();
        unsafe { std::env::set_var("CUTTLE_HOME", tmp.path()) };
        let account = "CUTTLE_TEST_NEVER_SET_KEY_VAR";
        let args = SessionStartArgs {
            api_key_env: account.to_string(),
            ..SessionStartArgs::default()
        };
        let err = run(&args).unwrap_err();
        // Restore env before any assertion that might panic.
        match prev {
            Some(p) => unsafe { std::env::set_var("CUTTLE_HOME", p) },
            None => unsafe { std::env::remove_var("CUTTLE_HOME") },
        }
        match err {
            SessionCmdError::ApiKey(ResolveError::NoCredentialFound { account: a }) => {
                assert_eq!(a, account);
            }
            other => panic!("expected ApiKey(NoCredentialFound), got {other:?}"),
        }
    }
}
