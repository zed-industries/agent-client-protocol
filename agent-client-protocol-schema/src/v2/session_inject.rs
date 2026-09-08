//! Unstable ACP v2 session injection types.

use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as, skip_serializing_none};

#[cfg(feature = "schemars")]
use schemars::Schema;

use super::{ContentBlock, MessageId, Meta, SessionId};
use crate::IntoOption;

/// Method name for injecting a user message into a session.
pub(crate) const SESSION_INJECT_METHOD_NAME: &str = "session/inject";
/// Method name for revoking a pending injected message.
pub(crate) const SESSION_REVOKE_INJECT_METHOD_NAME: &str = "session/revoke_inject";
/// Method name for replacing a pending injected message.
pub(crate) const SESSION_REPLACE_INJECT_METHOD_NAME: &str = "session/replace_inject";

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Delivery mode for a message injected into a session.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SessionInjectMode {
    /// Deliver after the current turn returns to idle.
    Queue,
    /// Deliver at the next safe breakpoint in the current turn.
    Steer,
    /// Custom or future injection mode.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    #[serde(untagged)]
    Other(String),
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// How an agent handles steering that arrives during an LLM stream.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SessionInjectSteerInStream {
    /// Truncate the in-flight stream and re-prompt with the injected message.
    Interrupt,
    /// Finish the in-flight stream before delivering the injected message.
    Finish,
    /// Custom or future stream-steering behavior.
    ///
    /// Values beginning with `_` are reserved for implementation-specific
    /// extensions. Unknown values that do not begin with `_` are reserved for
    /// future ACP variants.
    #[serde(untagged)]
    Other(String),
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for injecting a user message into a session.
///
/// The response acknowledges pending delivery. The agent later delivers the
/// message by emitting a `user_message` session update with the returned
/// [`MessageId`].
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "agent", "x-method" = SESSION_INJECT_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct InjectSessionRequest {
    /// The ID of the session that receives the injected message.
    pub session_id: SessionId,
    /// When the agent should deliver the message.
    pub mode: SessionInjectMode,
    /// The content blocks that compose the injected user message.
    pub content: Vec<ContentBlock>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl InjectSessionRequest {
    /// Builds an injection request with the required fields set.
    #[must_use]
    pub fn new(
        session_id: impl Into<SessionId>,
        mode: SessionInjectMode,
        content: Vec<ContentBlock>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            mode,
            content,
            meta: None,
        }
    }

    /// Attaches protocol extension metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response acknowledging that an injected message is pending delivery.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "agent", "x-method" = SESSION_INJECT_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct InjectSessionResponse {
    /// Agent-assigned ID for the pending message.
    pub message_id: MessageId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl InjectSessionResponse {
    /// Builds an injection response with the assigned message ID.
    #[must_use]
    pub fn new(message_id: impl Into<MessageId>) -> Self {
        Self {
            message_id: message_id.into(),
            meta: None,
        }
    }

    /// Attaches protocol extension metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for revoking a pending injected message.
///
/// Every agent that advertises session injection must support this method. A
/// successful response guarantees that no future `user_message` update will be
/// emitted for this message ID. If delivery wins the race, the agent returns
/// `-32010` with `data: { reason: "already_delivered", messageId }`. An unknown
/// message ID returns `-32002` with
/// `data: { reason: "unknown_message_id", messageId }`.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "agent", "x-method" = SESSION_REVOKE_INJECT_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RevokeInjectSessionRequest {
    /// The ID of the session that owns the pending message.
    pub session_id: SessionId,
    /// The ID returned by `session/inject`.
    pub message_id: MessageId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl RevokeInjectSessionRequest {
    /// Builds a revoke request for a pending injected message.
    #[must_use]
    pub fn new(session_id: impl Into<SessionId>, message_id: impl Into<MessageId>) -> Self {
        Self {
            session_id: session_id.into(),
            message_id: message_id.into(),
            meta: None,
        }
    }

    /// Attaches protocol extension metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response confirming that a pending injected message was revoked.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "agent", "x-method" = SESSION_REVOKE_INJECT_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RevokeInjectSessionResponse {
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl RevokeInjectSessionResponse {
    /// Builds an empty successful revoke response.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attaches protocol extension metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for replacing the content of a pending injected message.
///
/// Agents support this optional method only when they advertise
/// `session.inject.pending.replace`. Replacement preserves the message ID, mode,
/// and pending-order position. Already-delivered messages return `-32010` with
/// `data: { reason: "already_delivered", messageId }`; unknown message IDs return
/// `-32002` with `data: { reason: "unknown_message_id", messageId }`. If replace
/// was not advertised, the agent returns method-not-found or `-32010` with
/// `data: { reason: "replace_not_supported", messageId }`.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "agent", "x-method" = SESSION_REPLACE_INJECT_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ReplaceInjectSessionRequest {
    /// The ID of the session that owns the pending message.
    pub session_id: SessionId,
    /// The ID returned by `session/inject`.
    pub message_id: MessageId,
    /// Complete replacement content for the pending user message.
    pub content: Vec<ContentBlock>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ReplaceInjectSessionRequest {
    /// Builds a replacement request with the required fields set.
    #[must_use]
    pub fn new(
        session_id: impl Into<SessionId>,
        message_id: impl Into<MessageId>,
        content: Vec<ContentBlock>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            message_id: message_id.into(),
            content,
            meta: None,
        }
    }

    /// Attaches protocol extension metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response confirming replacement of a pending injected message.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", schemars(extend("x-side" = "agent", "x-method" = SESSION_REPLACE_INJECT_METHOD_NAME)))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ReplaceInjectSessionResponse {
    /// The unchanged ID of the pending message.
    pub message_id: MessageId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ReplaceInjectSessionResponse {
    /// Builds a replacement response for the pending message ID.
    #[must_use]
    pub fn new(message_id: impl Into<MessageId>) -> Self {
        Self {
            message_id: message_id.into(),
            meta: None,
        }
    }

    /// Attaches protocol extension metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Capabilities for mid-turn user-message injection.
///
/// `modes` must be non-empty. When it contains `steer`, `steerInStream` is
/// required and must also be non-empty. Advertising this capability makes
/// `session/revoke_inject` mandatory.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(transform = session_inject_capabilities_transform))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionInjectCapabilities {
    /// Supported injection delivery modes. Must be non-empty.
    #[cfg_attr(feature = "schemars", schemars(length(min = 1)))]
    pub modes: Vec<SessionInjectMode>,
    /// Supported handling for steering during an LLM stream.
    ///
    /// Required and non-empty when `modes` contains `steer`; otherwise optional.
    #[cfg_attr(feature = "schemars", schemars(length(min = 1)))]
    pub steer_in_stream: Option<Vec<SessionInjectSteerInStream>>,
    /// Capabilities for editing pending injected messages.
    ///
    /// Omitted or `null` means pending replacement is not supported.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub pending: Option<SessionInjectPendingCapabilities>,
    /// The _meta property is reserved by ACP for extension metadata.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionInjectCapabilities {
    /// Builds injection capabilities with the required supported modes.
    #[must_use]
    pub fn new(modes: Vec<SessionInjectMode>) -> Self {
        Self {
            modes,
            steer_in_stream: None,
            pending: None,
            meta: None,
        }
    }

    /// Declares handling supported for steering during an LLM stream.
    #[must_use]
    pub fn steer_in_stream(
        mut self,
        steer_in_stream: impl IntoOption<Vec<SessionInjectSteerInStream>>,
    ) -> Self {
        self.steer_in_stream = steer_in_stream.into_option();
        self
    }

    /// Declares capabilities for pending injected messages.
    #[must_use]
    pub fn pending(mut self, pending: impl IntoOption<SessionInjectPendingCapabilities>) -> Self {
        self.pending = pending.into_option();
        self
    }

    /// Attaches protocol extension metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

#[cfg(feature = "schemars")]
fn session_inject_capabilities_transform(schema: &mut Schema) {
    let condition = serde_json::json!({
        "if": {
            "properties": {
                "modes": {
                    "contains": { "const": "steer" }
                }
            },
            "required": ["modes"]
        },
        "then": {
            "properties": {
                "steerInStream": {
                    "type": "array",
                    "minItems": 1
                }
            },
            "required": ["steerInStream"]
        }
    });
    schema.insert("allOf".into(), serde_json::json!([condition]));
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Capabilities for pending injected messages.
#[serde_as]
#[skip_serializing_none]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SessionInjectPendingCapabilities {
    /// Whether `session/replace_inject` is supported.
    ///
    /// Optional. Omitted or `null` defaults to `false`.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    pub replace: Option<bool>,
    /// The _meta property is reserved by ACP for extension metadata.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[cfg_attr(feature = "schemars", schemars(extend("x-deserialize-default-on-error" = true)))]
    #[serde(default)]
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl SessionInjectPendingCapabilities {
    /// Builds empty pending-message capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether pending content replacement is supported.
    #[must_use]
    pub fn replace(mut self, replace: impl IntoOption<bool>) -> Self {
        self.replace = replace.into_option();
        self
    }

    /// Attaches protocol extension metadata.
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::v2::TextContent;

    #[test]
    fn serializes_inject_requests_and_capabilities() {
        let content = vec![ContentBlock::Text(TextContent::new(
            "correct the auth path",
        ))];
        let request =
            InjectSessionRequest::new("session-1", SessionInjectMode::Steer, content.clone());
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            json!({
                "sessionId": "session-1",
                "mode": "steer",
                "content": [{"type": "text", "text": "correct the auth path"}]
            })
        );

        let replace =
            ReplaceInjectSessionRequest::new("session-1", MessageId::new("message-1"), content);
        assert_eq!(
            serde_json::to_value(replace).unwrap()["messageId"],
            json!("message-1")
        );

        let capabilities = SessionInjectCapabilities::new(vec![
            SessionInjectMode::Queue,
            SessionInjectMode::Steer,
        ])
        .steer_in_stream(vec![
            SessionInjectSteerInStream::Interrupt,
            SessionInjectSteerInStream::Finish,
        ])
        .pending(SessionInjectPendingCapabilities::new().replace(true));
        assert_eq!(
            serde_json::to_value(capabilities).unwrap(),
            json!({
                "modes": ["queue", "steer"],
                "steerInStream": ["interrupt", "finish"],
                "pending": {"replace": true}
            })
        );
    }

    #[test]
    fn routes_injection_methods_and_error_code() {
        let content = vec![ContentBlock::Text(TextContent::new("new context"))];
        let inject = crate::v2::ClientRequest::InjectSessionRequest(Box::new(
            InjectSessionRequest::new("session-1", SessionInjectMode::Queue, content.clone()),
        ));
        let revoke = crate::v2::ClientRequest::RevokeInjectSessionRequest(Box::new(
            RevokeInjectSessionRequest::new("session-1", "message-1"),
        ));
        let replace = crate::v2::ClientRequest::ReplaceInjectSessionRequest(Box::new(
            ReplaceInjectSessionRequest::new("session-1", "message-1", content),
        ));

        assert_eq!(inject.method(), "session/inject");
        assert_eq!(revoke.method(), "session/revoke_inject");
        assert_eq!(replace.method(), "session/replace_inject");
        assert_eq!(
            i32::from(crate::v2::ErrorCode::InjectPreconditionFailed),
            -32010
        );
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn capability_schema_enforces_non_empty_and_steer_requirements() {
        let schema =
            serde_json::to_value(schemars::schema_for!(SessionInjectCapabilities)).unwrap();
        assert_eq!(schema["properties"]["modes"]["minItems"], json!(1));
        assert_eq!(schema["properties"]["steerInStream"]["minItems"], json!(1));
        assert_eq!(
            schema["allOf"][0]["then"]["required"],
            json!(["steerInStream"])
        );
        assert_eq!(
            schema["allOf"][0]["then"]["properties"]["steerInStream"]["type"],
            json!("array")
        );
        assert_eq!(
            schema["allOf"][0]["then"]["properties"]["steerInStream"]["minItems"],
            json!(1)
        );
    }
}
