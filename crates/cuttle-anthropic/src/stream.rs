//! `StreamEvent` + `messages_stream()`: SSE streaming for `/v1/messages`.
//!
//! Anthropic's `/v1/messages` with `stream: true` emits an SSE byte stream
//! (RFC 8895-shaped: `event: <name>\ndata: <json>\n\n`). The events of
//! interest:
//!
//! | event                  | meaning                                            |
//! | ---------------------- | -------------------------------------------------- |
//! | `message_start`        | message metadata (id, model, role, usage)          |
//! | `content_block_start`  | new content block (text, tool_use, etc.)           |
//! | `content_block_delta`  | partial content (text_delta with `text` field)     |
//! | `content_block_stop`   | content block ended                                |
//! | `message_delta`        | final usage update + stop_reason                   |
//! | `message_stop`         | stream is complete                                 |
//! | `ping`                 | keepalive; no payload to expose to caller          |
//! | `error`                | upstream error inside the stream                   |
//!
//! Retry safety (per `retry.rs` module doc): once `messages_stream()`
//! returns `Ok(stream)`, the HTTP request has been ack'd and the response
//! has begun. From that moment forward the request is NOT idempotent —
//! retrying would replay the prompt and double-bill output tokens. The
//! retry loop in `client.rs::messages()` handles this by ONLY wrapping
//! the connection-establish phase; once we hand a stream back, the
//! caller owns recovery.
//!
//! For v0.0.9 the stream itself does no retry. Mid-stream failures
//! propagate as `Err(AnthropicError::PartialStream { ... })` items, and
//! the caller decides whether to surface to the operator (typical
//! daily-driver UX) or to discard partial output and re-prompt manually
//! (operator-initiated, never automatic).

use crate::error::AnthropicError;
use crate::model::{Role, StopReason, Usage};
use eventsource_stream::Eventsource;
use futures_util::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};

/// One parsed SSE event from the `/v1/messages` stream. Variants mirror
/// the Anthropic event names (`type` field on the JSON payload).
///
/// Forward-compat: unknown event types deserialize as `Unknown` so the
/// stream does not abort if Anthropic adds a new event in a future API
/// revision. Callers ignore `Unknown` by default.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart {
        message: MessageStartPayload,
    },
    ContentBlockStart {
        index: u32,
        content_block: ContentBlockStartPayload,
    },
    ContentBlockDelta {
        index: u32,
        delta: ContentBlockDelta,
    },
    ContentBlockStop {
        index: u32,
    },
    MessageDelta {
        delta: MessageDeltaPayload,
        usage: Usage,
    },
    MessageStop,
    Ping,
    Error {
        error: ErrorPayload,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MessageStartPayload {
    pub id: String,
    pub role: Role,
    pub model: String,
    pub usage: Usage,
}

/// `content_block_start.content_block`. Modeled minimally for v0.0.9:
/// the `Text` variant (with empty initial text) covers daily-driver
/// streaming. Tool-use blocks deserialize into `Other` rather than
/// failing the stream.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockStartPayload {
    Text {
        text: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockDelta {
    TextDelta {
        text: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MessageDeltaPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ErrorPayload {
    #[serde(rename = "type")]
    pub kind: String,
    pub message: String,
}

impl StreamEvent {
    /// If this event carries a text delta, return the new text bytes.
    /// Convenience helper for the common daily-driver path: print
    /// streaming text to the terminal as the model produces it.
    pub fn text_delta(&self) -> Option<&str> {
        match self {
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::TextDelta { text },
                ..
            } => Some(text.as_str()),
            _ => None,
        }
    }

    /// Whether this event signals the end of the stream. Callers should
    /// continue consuming until they see `MessageStop` (the SSE
    /// connection may still send `Ping` events afterward; the stream
    /// ends naturally with EOF).
    pub fn is_message_stop(&self) -> bool {
        matches!(self, StreamEvent::MessageStop)
    }
}

/// Convert a reqwest `Response` body into a `Stream<StreamEvent>`.
/// Public so tests + future advanced callers can pipe their own bytes
/// through (e.g., a transport-level recorder for replay).
pub fn parse_response_stream(
    resp: reqwest::Response,
) -> impl Stream<Item = Result<StreamEvent, AnthropicError>> {
    resp.bytes_stream().eventsource().map(|item| match item {
        Ok(ev) => parse_event_data(&ev.data),
        Err(e) => Err(AnthropicError::PartialStream {
            reason: format!("SSE transport error: {e}"),
        }),
    })
}

/// Parse one SSE `data:` payload as a `StreamEvent`. JSON deserialize
/// failure surfaces as `AnthropicError::Deserialize` (terminal; the
/// retry policy never retries mid-stream anyway).
fn parse_event_data(data: &str) -> Result<StreamEvent, AnthropicError> {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        // Empty data lines are SSE-legal (keepalive shape); map to Ping.
        return Ok(StreamEvent::Ping);
    }
    Ok(serde_json::from_str(trimmed)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_start_event_deserializes() {
        let json = r#"{
            "type": "message_start",
            "message": {
                "id": "msg_1",
                "role": "assistant",
                "model": "claude-sonnet-4-6",
                "usage": {"input_tokens": 10, "output_tokens": 0}
            }
        }"#;
        let ev = parse_event_data(json).unwrap();
        match ev {
            StreamEvent::MessageStart { message } => {
                assert_eq!(message.id, "msg_1");
                assert_eq!(message.usage.input_tokens, 10);
            }
            other => panic!("expected MessageStart, got {other:?}"),
        }
    }

    #[test]
    fn content_block_delta_text_event_deserializes() {
        let json = r#"{
            "type": "content_block_delta",
            "index": 0,
            "delta": {"type": "text_delta", "text": "Hello"}
        }"#;
        let ev = parse_event_data(json).unwrap();
        assert_eq!(ev.text_delta(), Some("Hello"));
    }

    #[test]
    fn message_stop_event_is_recognized() {
        let json = r#"{"type": "message_stop"}"#;
        let ev = parse_event_data(json).unwrap();
        assert!(ev.is_message_stop());
    }

    #[test]
    fn message_delta_event_carries_stop_reason_and_usage() {
        let json = r#"{
            "type": "message_delta",
            "delta": {"stop_reason": "end_turn"},
            "usage": {"input_tokens": 0, "output_tokens": 5}
        }"#;
        let ev = parse_event_data(json).unwrap();
        match ev {
            StreamEvent::MessageDelta { delta, usage } => {
                assert_eq!(delta.stop_reason, Some(StopReason::EndTurn));
                assert_eq!(usage.output_tokens, 5);
            }
            other => panic!("expected MessageDelta, got {other:?}"),
        }
    }

    #[test]
    fn error_event_carries_kind_and_message() {
        let json = r#"{
            "type": "error",
            "error": {"type": "overloaded_error", "message": "try again"}
        }"#;
        let ev = parse_event_data(json).unwrap();
        match ev {
            StreamEvent::Error { error } => {
                assert_eq!(error.kind, "overloaded_error");
                assert_eq!(error.message, "try again");
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn unknown_event_type_deserializes_as_unknown_not_error() {
        let json = r#"{"type": "future_event_type"}"#;
        let ev = parse_event_data(json).unwrap();
        assert_eq!(ev, StreamEvent::Unknown);
    }

    #[test]
    fn empty_data_payload_maps_to_ping() {
        assert_eq!(parse_event_data("").unwrap(), StreamEvent::Ping);
        assert_eq!(parse_event_data("   ").unwrap(), StreamEvent::Ping);
    }

    #[test]
    fn malformed_json_surfaces_as_deserialize_error() {
        let r = parse_event_data("not json at all");
        assert!(matches!(r, Err(AnthropicError::Deserialize(_))));
    }

    #[test]
    fn ping_event_is_recognized() {
        let json = r#"{"type": "ping"}"#;
        let ev = parse_event_data(json).unwrap();
        assert_eq!(ev, StreamEvent::Ping);
    }

    #[test]
    fn content_block_start_text_block_deserializes() {
        let json = r#"{
            "type": "content_block_start",
            "index": 0,
            "content_block": {"type": "text", "text": ""}
        }"#;
        let ev = parse_event_data(json).unwrap();
        match ev {
            StreamEvent::ContentBlockStart {
                index,
                content_block,
            } => {
                assert_eq!(index, 0);
                assert!(matches!(
                    content_block,
                    ContentBlockStartPayload::Text { .. }
                ));
            }
            other => panic!("expected ContentBlockStart, got {other:?}"),
        }
    }

    #[test]
    fn tool_use_content_block_deserializes_as_other_not_failure() {
        let json = r#"{
            "type": "content_block_start",
            "index": 0,
            "content_block": {"type": "tool_use", "id": "tu_1", "name": "x", "input": {}}
        }"#;
        let ev = parse_event_data(json).unwrap();
        match ev {
            StreamEvent::ContentBlockStart { content_block, .. } => {
                assert_eq!(content_block, ContentBlockStartPayload::Other);
            }
            other => panic!("expected ContentBlockStart, got {other:?}"),
        }
    }
}
