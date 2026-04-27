//! `cuttle ask` subcommand handler. The first daily-driver-shape
//! Cuttle command — exercises the full streaming path:
//!
//!   `cuttle_credential::ApiKey::from_env_var("ANTHROPIC_API_KEY")` →
//!   build `AnthropicClient` →
//!   build `Request` from prompt →
//!   `messages_stream()` →
//!   write `text_delta` bytes to stdout as they arrive.
//!
//! v0.0.12 scope: single-turn ask. NO conversation history. NO tool
//! dispatch (the model can describe what it would do but cannot
//! execute). NO audit-log writes (audit events for conversation turns
//! land in v0.0.13 with new variant additions to cuttle-audit).
//!
//! Tokio runtime: built on demand inside `run()` via
//! `tokio::runtime::Builder::new_current_thread()`. v0.0.12 has only one
//! async path; restructuring all of cuttle-cli to be async-throughout
//! waits for v0.0.13 (when both `ask` and `session start` need the
//! runtime).

use crate::args::{AskArgs, PromptSource};
use cuttle_anthropic::{AnthropicClient, ClientConfig, KnownModel, Message, Model, Request};
use cuttle_credential::{ApiKey, ApiKeyEnvError};
use futures_util::StreamExt;
use std::io::{Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AskCmdError {
    #[error("could not load API key: {0}")]
    ApiKey(#[from] ApiKeyEnvError),

    #[error("could not read prompt from stdin: {0}")]
    StdinRead(std::io::Error),

    #[error("prompt from stdin was empty")]
    EmptyStdinPrompt,

    #[error("could not build Anthropic client: {0}")]
    ClientBuild(cuttle_anthropic::AnthropicError),

    #[error("Anthropic API call failed: {0}")]
    Api(#[from] cuttle_anthropic::AnthropicError),

    #[error("could not build tokio runtime: {0}")]
    Runtime(std::io::Error),

    #[error("writing model output to stdout failed: {0}")]
    WriteFailed(std::io::Error),
}

/// Synchronous entry point. Builds a tokio runtime, runs the streaming
/// ask, returns once the model emits `MessageStop` (or an error).
pub fn run<W: Write>(args: &AskArgs, out: &mut W) -> Result<(), AskCmdError> {
    let prompt = match &args.source {
        PromptSource::Inline(p) => p.clone(),
        PromptSource::Stdin => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(AskCmdError::StdinRead)?;
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                return Err(AskCmdError::EmptyStdinPrompt);
            }
            trimmed.to_string()
        }
    };

    let api_key = ApiKey::from_env_var(&args.api_key_env)?;

    let model = parse_model(&args.model);
    let request = Request::new(model, vec![Message::user_text(prompt)], args.max_tokens);

    let client = AnthropicClient::new(ClientConfig::default()).map_err(AskCmdError::ClientBuild)?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(AskCmdError::Runtime)?;

    rt.block_on(stream_to_writer(&client, &api_key, &request, out))
}

async fn stream_to_writer<W: Write>(
    client: &AnthropicClient,
    api_key: &ApiKey,
    request: &Request,
    out: &mut W,
) -> Result<(), AskCmdError> {
    let mut stream = client.messages_stream(api_key, request).await?;
    while let Some(item) = stream.next().await {
        let event = item?;
        if let Some(text) = event.text_delta() {
            out.write_all(text.as_bytes())
                .map_err(AskCmdError::WriteFailed)?;
            // Flush after every delta so the operator sees streaming
            // text in real time, not in a single batch at end-of-stream.
            out.flush().map_err(AskCmdError::WriteFailed)?;
        }
        if event.is_message_stop() {
            // Final newline so the shell prompt doesn't share a line
            // with model output.
            writeln!(out).map_err(AskCmdError::WriteFailed)?;
            return Ok(());
        }
    }
    // Stream ended without explicit MessageStop. Add a newline anyway.
    writeln!(out).map_err(AskCmdError::WriteFailed)?;
    Ok(())
}

/// Map a model-id string into the `Model` enum. Known ids deserialize
/// to `KnownModel`; everything else lands in `Custom` (operator can
/// target preview models without a recompile).
fn parse_model(s: &str) -> Model {
    match s {
        "claude-opus-4-7" => Model::Known(KnownModel::ClaudeOpus47),
        "claude-sonnet-4-6" => Model::Known(KnownModel::ClaudeSonnet46),
        "claude-haiku-4-5-20251001" => Model::Known(KnownModel::ClaudeHaiku45),
        other => Model::Custom(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_model_recognizes_known_ids() {
        assert_eq!(
            parse_model("claude-sonnet-4-6"),
            Model::Known(KnownModel::ClaudeSonnet46)
        );
        assert_eq!(
            parse_model("claude-opus-4-7"),
            Model::Known(KnownModel::ClaudeOpus47)
        );
    }

    #[test]
    fn parse_model_falls_back_to_custom_for_unknown() {
        assert_eq!(
            parse_model("claude-future-3000"),
            Model::Custom("claude-future-3000".to_string())
        );
    }

    /// We can't run a live API call in unit tests (no key, no network).
    /// Validate the wiring shape: the run() function fails cleanly on
    /// missing API key with a specific, named error.
    #[test]
    fn run_errors_when_api_key_env_unset() {
        // Use a name that's almost certainly not in the test env.
        let args = AskArgs {
            api_key_env: "CUTTLE_TEST_DEFINITELY_UNSET_KEY_VAR".to_string(),
            source: PromptSource::Inline("hello".to_string()),
            ..AskArgs::default()
        };
        let mut out = Vec::new();
        let err = run(&args, &mut out).unwrap_err();
        match err {
            AskCmdError::ApiKey(ApiKeyEnvError::NotSet { var }) => {
                assert_eq!(var, "CUTTLE_TEST_DEFINITELY_UNSET_KEY_VAR");
            }
            other => panic!("expected ApiKey(NotSet), got {other:?}"),
        }
    }
}
