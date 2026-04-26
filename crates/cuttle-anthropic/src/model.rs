//! Anthropic API domain types. Wire-format mirrors the public `/v1/messages`
//! contract; field names use serde rename to match the over-the-wire JSON.
//!
//! Scope minimisation: v0.0.8 implements only the field set Cuttle's v0.1
//! daily-driver path actually exercises. Tool-use blocks, multi-modal image
//! content, and `cache_control` are stub-typed (carried as raw JSON where
//! present, or deferred entirely) so `Request`/`Response` round-trip cleanly
//! against the live API without claiming feature coverage we do not yet test.

use serde::{Deserialize, Serialize};

/// Claude model identifier. Strings match the public model catalog so the
/// `cuttle config` surface and the wire format agree without translation.
/// Unknown identifiers deserialize via the `Custom` variant rather than
/// failing — operators can target preview models by string without a
/// crate release.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Model {
    Known(KnownModel),
    Custom(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum KnownModel {
    #[serde(rename = "claude-opus-4-7")]
    ClaudeOpus47,
    #[serde(rename = "claude-sonnet-4-6")]
    ClaudeSonnet46,
    #[serde(rename = "claude-haiku-4-5-20251001")]
    ClaudeHaiku45,
}

impl Model {
    /// Wire string for this model identifier.
    pub fn as_str(&self) -> &str {
        match self {
            Model::Known(KnownModel::ClaudeOpus47) => "claude-opus-4-7",
            Model::Known(KnownModel::ClaudeSonnet46) => "claude-sonnet-4-6",
            Model::Known(KnownModel::ClaudeHaiku45) => "claude-haiku-4-5-20251001",
            Model::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// Content block. v0.0.8 covers the text-only path. Tool-use blocks deserialize
/// into `Other` (raw JSON) so a Response carrying a tool_use block does not
/// crash deserialization while we wait for v0.0.9 to model them properly.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Content {
    Text {
        text: String,
    },
    #[serde(other)]
    Other,
}

impl Content {
    pub fn text<S: Into<String>>(s: S) -> Self {
        Content::Text { text: s.into() }
    }

    /// Returns the text payload if this is a `Text` block.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text { text } => Some(text.as_str()),
            Content::Other => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: Vec<Content>,
}

impl Message {
    pub fn user_text<S: Into<String>>(s: S) -> Self {
        Message {
            role: Role::User,
            content: vec![Content::text(s)],
        }
    }

    pub fn assistant_text<S: Into<String>>(s: S) -> Self {
        Message {
            role: Role::Assistant,
            content: vec![Content::text(s)],
        }
    }
}

/// `/v1/messages` request body. Required fields per Anthropic spec:
/// `model`, `messages`, `max_tokens`. Optional fields are skipped when
/// `None` to keep the wire payload identical to a hand-written curl.
///
/// `Eq` is deliberately NOT derived: `temperature` / `top_p` are `f32` and
/// `f32` is not totally ordered (NaN). `PartialEq` is enough for our needs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Request {
    pub model: Model,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Always `false` (or omitted) for non-streaming `messages()`.
    /// Streaming uses a separate code path in v0.0.9.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl Request {
    pub fn new(model: Model, messages: Vec<Message>, max_tokens: u32) -> Self {
        Request {
            model,
            messages,
            max_tokens,
            system: None,
            temperature: None,
            top_p: None,
            stop_sequences: None,
            stream: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    #[serde(other)]
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    /// Prompt-cache fields are reported by the API even when not requested;
    /// optional for forward-compat without breaking deserialization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub role: Role,
    pub model: String,
    pub content: Vec<Content>,
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

impl Response {
    /// Concatenate all `Text` blocks into one string. `Other` blocks are
    /// skipped — callers that need tool-use blocks should match on
    /// `self.content` directly.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for block in &self.content {
            if let Some(t) = block.as_text() {
                out.push_str(t);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_known_serializes_to_wire_string() {
        let m = Model::Known(KnownModel::ClaudeSonnet46);
        let s = serde_json::to_string(&m).unwrap();
        assert_eq!(s, "\"claude-sonnet-4-6\"");
    }

    #[test]
    fn model_custom_serializes_to_raw_string() {
        let m = Model::Custom("claude-preview-9".into());
        let s = serde_json::to_string(&m).unwrap();
        assert_eq!(s, "\"claude-preview-9\"");
    }

    #[test]
    fn model_unknown_string_deserializes_as_custom() {
        let m: Model = serde_json::from_str("\"claude-future-3000\"").unwrap();
        assert_eq!(m, Model::Custom("claude-future-3000".into()));
    }

    #[test]
    fn model_known_string_deserializes_as_known() {
        let m: Model = serde_json::from_str("\"claude-opus-4-7\"").unwrap();
        assert_eq!(m, Model::Known(KnownModel::ClaudeOpus47));
    }

    #[test]
    fn request_skips_none_fields() {
        let req = Request::new(
            Model::Known(KnownModel::ClaudeHaiku45),
            vec![Message::user_text("hello")],
            64,
        );
        let s = serde_json::to_string(&req).unwrap();
        // Optional fields must be absent, not "null".
        assert!(!s.contains("temperature"));
        assert!(!s.contains("top_p"));
        assert!(!s.contains("stream"));
        assert!(!s.contains("system"));
        assert!(s.contains("\"max_tokens\":64"));
    }

    #[test]
    fn response_round_trip_includes_text_concatenation() {
        let json = r#"{
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-6",
            "content": [
                {"type": "text", "text": "hello "},
                {"type": "text", "text": "world"}
            ],
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {"input_tokens": 5, "output_tokens": 2}
        }"#;
        let resp: Response = serde_json::from_str(json).unwrap();
        assert_eq!(resp.text(), "hello world");
        assert_eq!(resp.usage.input_tokens, 5);
        assert_eq!(resp.stop_reason, Some(StopReason::EndTurn));
    }

    #[test]
    fn response_with_unknown_content_block_deserializes_as_other() {
        let json = r#"{
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-6",
            "content": [
                {"type": "tool_use", "id": "tu_1", "name": "x", "input": {}}
            ],
            "stop_reason": "tool_use",
            "stop_sequence": null,
            "usage": {"input_tokens": 5, "output_tokens": 2}
        }"#;
        let resp: Response = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content.len(), 1);
        assert_eq!(resp.content[0], Content::Other);
        assert_eq!(resp.text(), "");
    }

    #[test]
    fn usage_round_trip_includes_optional_cache_fields() {
        let json = r#"{
            "input_tokens": 10,
            "output_tokens": 5,
            "cache_creation_input_tokens": 100,
            "cache_read_input_tokens": 50
        }"#;
        let u: Usage = serde_json::from_str(json).unwrap();
        assert_eq!(u.cache_creation_input_tokens, Some(100));
        assert_eq!(u.cache_read_input_tokens, Some(50));
    }

    #[test]
    fn usage_omits_none_cache_fields() {
        let u = Usage {
            input_tokens: 1,
            output_tokens: 2,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };
        let s = serde_json::to_string(&u).unwrap();
        assert!(!s.contains("cache_creation"));
        assert!(!s.contains("cache_read"));
    }

    #[test]
    fn stop_reason_unknown_deserializes_as_other() {
        let r: StopReason = serde_json::from_str("\"future_reason\"").unwrap();
        assert_eq!(r, StopReason::Other);
    }
}
