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

/// `system` field shape: either a plain string (no caching) or a list
/// of typed blocks each with optional `cache_control`. The Anthropic
/// API accepts either form on the wire, untagged. Callers wanting prompt
/// caching MUST use `Blocks(...)` and tag the long-stable prefix with
/// `CacheControl::ephemeral()`. v0.0.10 adds the `Blocks` variant; the
/// `Plain` variant preserves the v0.0.8/v0.0.9 single-string ergonomic.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum SystemContent {
    Plain(String),
    Blocks(Vec<SystemBlock>),
}

impl From<String> for SystemContent {
    fn from(s: String) -> Self {
        SystemContent::Plain(s)
    }
}

impl From<&str> for SystemContent {
    fn from(s: &str) -> Self {
        SystemContent::Plain(s.to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SystemBlock {
    /// Block kind. Anthropic v1 accepts only `"text"`; left as a free
    /// String for forward-compat with future block kinds (image, etc.)
    /// without a recompile.
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl SystemBlock {
    pub fn text<S: Into<String>>(text: S) -> Self {
        SystemBlock {
            kind: "text".to_string(),
            text: text.into(),
            cache_control: None,
        }
    }

    pub fn with_cache_control(mut self, cc: CacheControl) -> Self {
        self.cache_control = Some(cc);
        self
    }
}

/// Prompt-cache control. Currently `ephemeral` is the only Anthropic-
/// supported variant (5-minute TTL). Per Anthropic docs the cache breakpoint
/// must be placed at a stable prefix boundary; the operator decides where.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub kind: String,
}

impl CacheControl {
    pub fn ephemeral() -> Self {
        CacheControl {
            kind: "ephemeral".to_string(),
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
    pub system: Option<SystemContent>,
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

    #[test]
    fn system_content_plain_serializes_as_json_string() {
        let sc = SystemContent::Plain("hello".to_string());
        let s = serde_json::to_string(&sc).unwrap();
        assert_eq!(s, "\"hello\"");
    }

    #[test]
    fn system_content_blocks_serializes_as_json_array() {
        let sc = SystemContent::Blocks(vec![SystemBlock::text("hello")]);
        let s = serde_json::to_string(&sc).unwrap();
        // Array form, with type=text and no cache_control field present.
        assert!(s.starts_with('['), "expected array, got: {s}");
        assert!(s.contains("\"type\":\"text\""));
        assert!(s.contains("\"text\":\"hello\""));
        assert!(!s.contains("cache_control"));
    }

    #[test]
    fn system_block_with_ephemeral_cache_control_serializes() {
        let block =
            SystemBlock::text("long stable prefix").with_cache_control(CacheControl::ephemeral());
        let s = serde_json::to_string(&block).unwrap();
        assert!(s.contains("\"cache_control\":{\"type\":\"ephemeral\"}"));
    }

    #[test]
    fn system_content_round_trips_as_blocks() {
        let original = SystemContent::Blocks(vec![
            SystemBlock::text("short header"),
            SystemBlock::text("long doc").with_cache_control(CacheControl::ephemeral()),
        ]);
        let s = serde_json::to_string(&original).unwrap();
        let restored: SystemContent = serde_json::from_str(&s).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn system_content_round_trips_as_plain() {
        let original = SystemContent::Plain("just a string".to_string());
        let s = serde_json::to_string(&original).unwrap();
        let restored: SystemContent = serde_json::from_str(&s).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn system_content_from_str_yields_plain() {
        let sc: SystemContent = "x".into();
        assert_eq!(sc, SystemContent::Plain("x".to_string()));
    }

    #[test]
    fn request_with_cached_system_blocks_serializes_correctly() {
        let mut req = Request::new(
            Model::Known(KnownModel::ClaudeHaiku45),
            vec![Message::user_text("hi")],
            64,
        );
        req.system = Some(SystemContent::Blocks(vec![SystemBlock::text(
            "cached prefix",
        )
        .with_cache_control(CacheControl::ephemeral())]));
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("\"system\":["));
        assert!(s.contains("\"cache_control\":{\"type\":\"ephemeral\"}"));
    }

    #[test]
    fn request_with_plain_string_system_serializes_unchanged() {
        // Backward-compat smoke: setting system to a plain string still
        // produces the simple `"system": "text"` wire shape (no array,
        // no per-block `cache_control`). Substring assertion targets the
        // exact `"system":"..."` form to avoid coincidental matches in
        // other fields (e.g. message Content blocks also use type=text).
        let mut req = Request::new(
            Model::Known(KnownModel::ClaudeHaiku45),
            vec![Message::user_text("hi")],
            64,
        );
        req.system = Some(SystemContent::Plain("be helpful".to_string()));
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("\"system\":\"be helpful\""));
        assert!(!s.contains("\"system\":["));
        assert!(!s.contains("cache_control"));
    }
}
