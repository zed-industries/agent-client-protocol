//! Methods and notifications the agent handles/receives.
//!
//! This module defines the Agent trait and all associated types for implementing
//! an AI coding agent that follows the Agent Client Protocol (ACP).

use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::ext::ExtRequest;
use crate::{
    ClientCapabilities, ContentBlock, Error, ExtNotification, ExtResponse, ProtocolVersion,
    SessionId,
};

/// Defines the interface that all ACP-compliant agents must implement.
///
/// Agents are programs that use generative AI to autonomously modify code. They handle
/// requests from clients and execute tasks using language models and tools.
#[async_trait::async_trait(?Send)]
pub trait Agent {
    /// Establishes the connection with a client and negotiates protocol capabilities.
    ///
    /// This method is called once at the beginning of the connection to:
    /// - Negotiate the protocol version to use
    /// - Exchange capability information between client and agent
    /// - Determine available authentication methods
    ///
    /// The agent should respond with its supported protocol version and capabilities.
    ///
    /// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
    async fn initialize(&self, args: InitializeRequest) -> Result<InitializeResponse, Error>;

    /// Authenticates the client using the specified authentication method.
    ///
    /// Called when the agent requires authentication before allowing session creation.
    /// The client provides the authentication method ID that was advertised during initialization.
    ///
    /// After successful authentication, the client can proceed to create sessions with
    /// `new_session` without receiving an `auth_required` error.
    ///
    /// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
    async fn authenticate(&self, args: AuthenticateRequest) -> Result<AuthenticateResponse, Error>;

    /// Creates a new conversation session with the agent.
    ///
    /// Sessions represent independent conversation contexts with their own history and state.
    ///
    /// The agent should:
    /// - Create a new session context
    /// - Connect to any specified MCP servers
    /// - Return a unique session ID for future requests
    ///
    /// May return an `auth_required` error if the agent requires authentication.
    ///
    /// See protocol docs: [Session Setup](https://agentclientprotocol.com/protocol/session-setup)
    async fn new_session(&self, args: NewSessionRequest) -> Result<NewSessionResponse, Error>;

    /// Loads an existing session to resume a previous conversation.
    ///
    /// This method is only available if the agent advertises the `loadSession` capability.
    ///
    /// The agent should:
    /// - Restore the session context and conversation history
    /// - Connect to the specified MCP servers
    /// - Stream the entire conversation history back to the client via notifications
    ///
    /// See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
    async fn load_session(&self, args: LoadSessionRequest) -> Result<LoadSessionResponse, Error>;

    /// Sets the current mode for a session.
    ///
    /// Allows switching between different agent modes (e.g., "ask", "architect", "code")
    /// that affect system prompts, tool availability, and permission behaviors.
    ///
    /// The mode must be one of the modes advertised in `availableModes` during session
    /// creation or loading. Agents may also change modes autonomously and notify the
    /// client via `current_mode_update` notifications.
    ///
    /// This method can be called at any time during a session, whether the Agent is
    /// idle or actively generating a response.
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    async fn set_session_mode(
        &self,
        args: SetSessionModeRequest,
    ) -> Result<SetSessionModeResponse, Error>;

    /// Processes a user prompt within a session.
    ///
    /// This method handles the whole lifecycle of a prompt:
    /// - Receives user messages with optional context (files, images, etc.)
    /// - Processes the prompt using language models
    /// - Reports language model content and tool calls to the Clients
    /// - Requests permission to run tools
    /// - Executes any requested tool calls
    /// - Returns when the turn is complete with a stop reason
    ///
    /// See protocol docs: [Prompt Turn](https://agentclientprotocol.com/protocol/prompt-turn)
    async fn prompt(&self, args: PromptRequest) -> Result<PromptResponse, Error>;

    /// Cancels ongoing operations for a session.
    ///
    /// This is a notification sent by the client to cancel an ongoing prompt turn.
    ///
    /// Upon receiving this notification, the Agent SHOULD:
    /// - Stop all language model requests as soon as possible
    /// - Abort all tool call invocations in progress
    /// - Send any pending `session/update` notifications
    /// - Respond to the original `session/prompt` request with `StopReason::Cancelled`
    ///
    /// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
    async fn cancel(&self, args: CancelNotification) -> Result<(), Error>;

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Select a model for a given session.
    #[cfg(feature = "unstable")]
    async fn set_session_model(
        &self,
        args: SetSessionModelRequest,
    ) -> Result<SetSessionModelResponse, Error>;

    /// Handles extension method requests from the client.
    ///
    /// Extension methods provide a way to add custom functionality while maintaining
    /// protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, Error>;

    /// Handles extension notifications from the client.
    ///
    /// Extension notifications provide a way to send one-way messages for custom functionality
    /// while maintaining protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    async fn ext_notification(&self, args: ExtNotification) -> Result<(), Error>;
}

// Initialize

/// Request parameters for the initialize method.
///
/// Sent by the client to establish connection and negotiate capabilities.
///
/// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = INITIALIZE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequest {
    /// The latest protocol version supported by the client.
    pub protocol_version: ProtocolVersion,
    /// Capabilities supported by the client.
    #[serde(default)]
    pub client_capabilities: ClientCapabilities,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Response from the initialize method.
///
/// Contains the negotiated protocol version and agent capabilities.
///
/// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = INITIALIZE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    /// The protocol version the client specified if supported by the agent,
    /// or the latest protocol version supported by the agent.
    ///
    /// The client should disconnect, if it doesn't support this version.
    pub protocol_version: ProtocolVersion,
    /// Capabilities supported by the agent.
    #[serde(default)]
    pub agent_capabilities: AgentCapabilities,
    /// Authentication methods supported by the agent.
    #[serde(default)]
    pub auth_methods: Vec<AuthMethod>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

// Authentication

/// Request parameters for the authenticate method.
///
/// Specifies which authentication method to use.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = AUTHENTICATE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    /// The ID of the authentication method to use.
    /// Must be one of the methods advertised in the initialize response.
    pub method_id: AuthMethodId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Response to authenticate method
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(extend("x-side" = "agent", "x-method" = AUTHENTICATE_METHOD_NAME))]
pub struct AuthenticateResponse {
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Unique identifier for an authentication method.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct AuthMethodId(pub Arc<str>);

/// Describes an available authentication method.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethod {
    /// Unique identifier for this authentication method.
    pub id: AuthMethodId,
    /// Human-readable name of the authentication method.
    pub name: String,
    /// Optional description providing more details about this authentication method.
    pub description: Option<String>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

// New session

/// Request parameters for creating a new session.
///
/// See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_NEW_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct NewSessionRequest {
    /// The working directory for this session. Must be an absolute path.
    pub cwd: PathBuf,
    /// List of MCP (Model Context Protocol) servers the agent should connect to.
    pub mcp_servers: Vec<McpServer>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Response from creating a new session.
///
/// See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_NEW_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct NewSessionResponse {
    /// Unique identifier for the created session.
    ///
    /// Used in all subsequent requests for this conversation.
    pub session_id: SessionId,
    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modes: Option<SessionModeState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<SessionModelState>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

// Load session

/// Request parameters for loading an existing session.
///
/// Only available if the Agent supports the `loadSession` capability.
///
/// See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_LOAD_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionRequest {
    /// List of MCP servers to connect to for this session.
    pub mcp_servers: Vec<McpServer>,
    /// The working directory for this session.
    pub cwd: PathBuf,
    /// The ID of the session to load.
    pub session_id: SessionId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Response from loading an existing session.
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_LOAD_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionResponse {
    /// Initial mode state if supported by the Agent
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modes: Option<SessionModeState>,
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Initial model state if supported by the Agent
    #[cfg(feature = "unstable")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<SessionModelState>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

// Session modes

/// The set of modes and the one currently active.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionModeState {
    /// The current mode the Agent is in.
    pub current_mode_id: SessionModeId,
    /// The set of modes that the Agent can operate in
    pub available_modes: Vec<SessionMode>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// A mode the agent can operate in.
///
/// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionMode {
    pub id: SessionModeId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Unique identifier for a Session Mode.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SessionModeId(pub Arc<str>);

impl std::fmt::Display for SessionModeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Request parameters for setting a session mode.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_MODE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct SetSessionModeRequest {
    /// The ID of the session to set the mode for.
    pub session_id: SessionId,
    /// The ID of the mode to set.
    pub mode_id: SessionModeId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Response to `session/set_mode` method.
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_MODE_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct SetSessionModeResponse {
    pub meta: Option<serde_json::Value>,
}

// MCP

/// Configuration for connecting to an MCP (Model Context Protocol) server.
///
/// MCP servers provide tools and context that the agent can use when
/// processing prompts.
///
/// See protocol docs: [MCP Servers](https://agentclientprotocol.com/protocol/session-setup#mcp-servers)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpServer {
    /// HTTP transport configuration
    ///
    /// Only available when the Agent capabilities indicate `mcp_capabilities.http` is `true`.
    #[serde(rename_all = "camelCase")]
    Http {
        /// Human-readable name identifying this MCP server.
        name: String,
        /// URL to the MCP server.
        url: String,
        /// HTTP headers to set when making requests to the MCP server.
        headers: Vec<HttpHeader>,
    },
    /// SSE transport configuration
    ///
    /// Only available when the Agent capabilities indicate `mcp_capabilities.sse` is `true`.
    #[serde(rename_all = "camelCase")]
    Sse {
        /// Human-readable name identifying this MCP server.
        name: String,
        /// URL to the MCP server.
        url: String,
        /// HTTP headers to set when making requests to the MCP server.
        headers: Vec<HttpHeader>,
    },
    /// Stdio transport configuration
    ///
    /// All Agents MUST support this transport.
    #[serde(untagged, rename_all = "camelCase")]
    Stdio {
        /// Human-readable name identifying this MCP server.
        name: String,
        /// Path to the MCP server executable.
        command: PathBuf,
        /// Command-line arguments to pass to the MCP server.
        args: Vec<String>,
        /// Environment variables to set when launching the MCP server.
        env: Vec<EnvVariable>,
    },
}

/// An environment variable to set when launching an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnvVariable {
    /// The name of the environment variable.
    pub name: String,
    /// The value to set for the environment variable.
    pub value: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// An HTTP header to set when making requests to the MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader {
    /// The name of the HTTP header.
    pub name: String,
    /// The value to set for the HTTP header.
    pub value: String,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

// Prompt

/// Request parameters for sending a user prompt to the agent.
///
/// Contains the user's message and any additional context.
///
/// See protocol docs: [User Message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_PROMPT_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct PromptRequest {
    /// The ID of the session to send this user message to
    pub session_id: SessionId,
    /// The blocks of content that compose the user's message.
    ///
    /// As a baseline, the Agent MUST support [`ContentBlock::Text`] and [`ContentBlock::ResourceLink`],
    /// while other variants are optionally enabled via [`PromptCapabilities`].
    ///
    /// The Client MUST adapt its interface according to [`PromptCapabilities`].
    ///
    /// The client MAY include referenced pieces of context as either
    /// [`ContentBlock::Resource`] or [`ContentBlock::ResourceLink`].
    ///
    /// When available, [`ContentBlock::Resource`] is preferred
    /// as it avoids extra round-trips and allows the message to include
    /// pieces of context from sources the agent may not have access to.
    pub prompt: Vec<ContentBlock>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Response from processing a user prompt.
///
/// See protocol docs: [Check for Completion](https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_PROMPT_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct PromptResponse {
    /// Indicates why the agent stopped processing the turn.
    pub stop_reason: StopReason,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Reasons why an agent stops processing a prompt turn.
///
/// See protocol docs: [Stop Reasons](https://agentclientprotocol.com/protocol/prompt-turn#stop-reasons)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// The turn ended successfully.
    EndTurn,
    /// The turn ended because the agent reached the maximum number of tokens.
    MaxTokens,
    /// The turn ended because the agent reached the maximum number of allowed
    /// agent requests between user turns.
    MaxTurnRequests,
    /// The turn ended because the agent refused to continue. The user prompt
    /// and everything that comes after it won't be included in the next
    /// prompt, so this should be reflected in the UI.
    Refusal,
    /// The turn was cancelled by the client via `session/cancel`.
    ///
    /// This stop reason MUST be returned when the client sends a `session/cancel`
    /// notification, even if the cancellation causes exceptions in underlying operations.
    /// Agents should catch these exceptions and return this semantically meaningful
    /// response to confirm successful cancellation.
    Cancelled,
}

// Model

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// The set of models and the one currently active.
#[cfg(feature = "unstable")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionModelState {
    /// The current model the Agent is in.
    pub current_model_id: ModelId,
    /// The set of models that the Agent can use
    pub available_models: Vec<ModelInfo>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// A unique identifier for a model.
#[cfg(feature = "unstable")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ModelId(pub Arc<str>);

#[cfg(feature = "unstable")]
impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Information about a selectable model.
#[cfg(feature = "unstable")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    /// Unique identifier for the model.
    pub model_id: ModelId,
    /// Human-readable name of the model.
    pub name: String,
    /// Optional description of the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Request parameters for setting a session model.
#[cfg(feature = "unstable")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_MODEL_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct SetSessionModelRequest {
    /// The ID of the session to set the model for.
    pub session_id: SessionId,
    /// The ID of the model to set.
    pub model_id: ModelId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Response to `session/set_model` method.
#[cfg(feature = "unstable")]
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_SET_MODEL_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct SetSessionModelResponse {
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

// Capabilities

/// Capabilities supported by the agent.
///
/// Advertised during initialization to inform the client about
/// available features and content types.
///
/// See protocol docs: [Agent Capabilities](https://agentclientprotocol.com/protocol/initialization#agent-capabilities)
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    /// Whether the agent supports `session/load`.
    #[serde(default)]
    pub load_session: bool,
    /// Prompt capabilities supported by the agent.
    #[serde(default)]
    pub prompt_capabilities: PromptCapabilities,
    /// MCP capabilities supported by the agent.
    #[serde(default)]
    pub mcp_capabilities: McpCapabilities,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// Prompt capabilities supported by the agent in `session/prompt` requests.
///
/// Baseline agent functionality requires support for [`ContentBlock::Text`]
/// and [`ContentBlock::ResourceLink`] in prompt requests.
///
/// Other variants must be explicitly opted in to.
/// Capabilities for different types of content in prompt requests.
///
/// Indicates which content types beyond the baseline (text and resource links)
/// the agent can process.
///
/// See protocol docs: [Prompt Capabilities](https://agentclientprotocol.com/protocol/initialization#prompt-capabilities)
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PromptCapabilities {
    /// Agent supports [`ContentBlock::Image`].
    #[serde(default)]
    pub image: bool,
    /// Agent supports [`ContentBlock::Audio`].
    #[serde(default)]
    pub audio: bool,
    /// Agent supports embedded context in `session/prompt` requests.
    ///
    /// When enabled, the Client is allowed to include [`ContentBlock::Resource`]
    /// in prompt requests for pieces of context that are referenced in the message.
    #[serde(default)]
    pub embedded_context: bool,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

/// MCP capabilities supported by the agent
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpCapabilities {
    /// Agent supports [`McpServer::Http`].
    #[serde(default)]
    pub http: bool,
    /// Agent supports [`McpServer::Sse`].
    #[serde(default)]
    pub sse: bool,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

// Method schema

/// Names of all methods that agents handle.
///
/// Provides a centralized definition of method names used in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMethodNames {
    /// Method for initializing the connection.
    pub initialize: &'static str,
    /// Method for authenticating with the agent.
    pub authenticate: &'static str,
    /// Method for creating a new session.
    pub session_new: &'static str,
    /// Method for loading an existing session.
    pub session_load: &'static str,
    /// Method for setting the mode for a session.
    pub session_set_mode: &'static str,
    /// Method for sending a prompt to the agent.
    pub session_prompt: &'static str,
    /// Notification for cancelling operations.
    pub session_cancel: &'static str,
    /// Method for selecting a model for a given session.
    #[cfg(feature = "unstable")]
    pub model_select: &'static str,
}

/// Constant containing all agent method names.
pub const AGENT_METHOD_NAMES: AgentMethodNames = AgentMethodNames {
    initialize: INITIALIZE_METHOD_NAME,
    authenticate: AUTHENTICATE_METHOD_NAME,
    session_new: SESSION_NEW_METHOD_NAME,
    session_load: SESSION_LOAD_METHOD_NAME,
    session_set_mode: SESSION_SET_MODE_METHOD_NAME,
    session_prompt: SESSION_PROMPT_METHOD_NAME,
    session_cancel: SESSION_CANCEL_METHOD_NAME,
    #[cfg(feature = "unstable")]
    model_select: SESSION_SET_MODEL_METHOD_NAME,
};

/// Method name for the initialize request.
pub(crate) const INITIALIZE_METHOD_NAME: &str = "initialize";
/// Method name for the authenticate request.
pub(crate) const AUTHENTICATE_METHOD_NAME: &str = "authenticate";
/// Method name for creating a new session.
pub(crate) const SESSION_NEW_METHOD_NAME: &str = "session/new";
/// Method name for loading an existing session.
pub(crate) const SESSION_LOAD_METHOD_NAME: &str = "session/load";
/// Method name for setting the mode for a session.
pub(crate) const SESSION_SET_MODE_METHOD_NAME: &str = "session/set_mode";
/// Method name for sending a prompt.
pub(crate) const SESSION_PROMPT_METHOD_NAME: &str = "session/prompt";
/// Method name for the cancel notification.
pub(crate) const SESSION_CANCEL_METHOD_NAME: &str = "session/cancel";
/// Method name for selecting a model for a given session.
#[cfg(feature = "unstable")]
pub(crate) const SESSION_SET_MODEL_METHOD_NAME: &str = "session/set_model";

/// All possible requests that a client can send to an agent.
///
/// This enum is used internally for routing RPC requests. You typically won't need
/// to use this directly - instead, use the methods on the [`Agent`] trait.
///
/// This enum encompasses all method calls from client to agent.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-docs-ignore" = true))]
pub enum ClientRequest {
    InitializeRequest(InitializeRequest),
    AuthenticateRequest(AuthenticateRequest),
    NewSessionRequest(NewSessionRequest),
    LoadSessionRequest(LoadSessionRequest),
    SetSessionModeRequest(SetSessionModeRequest),
    PromptRequest(PromptRequest),
    #[cfg(feature = "unstable")]
    SetSessionModelRequest(SetSessionModelRequest),
    ExtMethodRequest(ExtRequest),
}

/// All possible responses that an agent can send to a client.
///
/// This enum is used internally for routing RPC responses. You typically won't need
/// to use this directly - the responses are handled automatically by the connection.
///
/// These are responses to the corresponding `ClientRequest` variants.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-docs-ignore" = true))]
pub enum AgentResponse {
    InitializeResponse(InitializeResponse),
    AuthenticateResponse(#[serde(default)] AuthenticateResponse),
    NewSessionResponse(NewSessionResponse),
    LoadSessionResponse(#[serde(default)] LoadSessionResponse),
    SetSessionModeResponse(#[serde(default)] SetSessionModeResponse),
    PromptResponse(PromptResponse),
    #[cfg(feature = "unstable")]
    SetSessionModelResponse(SetSessionModelResponse),
    ExtMethodResponse(#[schemars(with = "serde_json::Value")] Arc<RawValue>),
}

/// All possible notifications that a client can send to an agent.
///
/// This enum is used internally for routing RPC notifications. You typically won't need
/// to use this directly - use the notification methods on the [`Agent`] trait instead.
///
/// Notifications do not expect a response.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-docs-ignore" = true))]
pub enum ClientNotification {
    CancelNotification(CancelNotification),
    ExtNotification(ExtNotification),
}

/// Notification to cancel ongoing operations for a session.
///
/// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = SESSION_CANCEL_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
pub struct CancelNotification {
    /// The ID of the session to cancel operations for.
    pub session_id: SessionId,
    /// Extension point for implementations
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

#[cfg(test)]
mod test_serialization {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_server_stdio_serialization() {
        let server = McpServer::Stdio {
            name: "test-server".to_string(),
            command: PathBuf::from("/usr/bin/server"),
            args: vec!["--port".to_string(), "3000".to_string()],
            env: vec![EnvVariable {
                name: "API_KEY".to_string(),
                value: "secret123".to_string(),
                meta: None,
            }],
        };

        let json = serde_json::to_value(&server).unwrap();
        assert_eq!(
            json,
            json!({
                "name": "test-server",
                "command": "/usr/bin/server",
                "args": ["--port", "3000"],
                "env": [
                    {
                        "name": "API_KEY",
                        "value": "secret123"
                    }
                ]
            })
        );

        let deserialized: McpServer = serde_json::from_value(json).unwrap();
        match deserialized {
            McpServer::Stdio {
                name,
                command,
                args,
                env,
            } => {
                assert_eq!(name, "test-server");
                assert_eq!(command, PathBuf::from("/usr/bin/server"));
                assert_eq!(args, vec!["--port", "3000"]);
                assert_eq!(env.len(), 1);
                assert_eq!(env[0].name, "API_KEY");
                assert_eq!(env[0].value, "secret123");
            }
            _ => panic!("Expected Stdio variant"),
        }
    }

    #[test]
    fn test_mcp_server_http_serialization() {
        let server = McpServer::Http {
            name: "http-server".to_string(),
            url: "https://api.example.com".to_string(),
            headers: vec![
                HttpHeader {
                    name: "Authorization".to_string(),
                    value: "Bearer token123".to_string(),
                    meta: None,
                },
                HttpHeader {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                    meta: None,
                },
            ],
        };

        let json = serde_json::to_value(&server).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "http",
                "name": "http-server",
                "url": "https://api.example.com",
                "headers": [
                    {
                        "name": "Authorization",
                        "value": "Bearer token123"
                    },
                    {
                        "name": "Content-Type",
                        "value": "application/json"
                    }
                ]
            })
        );

        let deserialized: McpServer = serde_json::from_value(json).unwrap();
        match deserialized {
            McpServer::Http { name, url, headers } => {
                assert_eq!(name, "http-server");
                assert_eq!(url, "https://api.example.com");
                assert_eq!(headers.len(), 2);
                assert_eq!(headers[0].name, "Authorization");
                assert_eq!(headers[0].value, "Bearer token123");
                assert_eq!(headers[1].name, "Content-Type");
                assert_eq!(headers[1].value, "application/json");
            }
            _ => panic!("Expected Http variant"),
        }
    }

    #[test]
    fn test_mcp_server_sse_serialization() {
        let server = McpServer::Sse {
            name: "sse-server".to_string(),
            url: "https://sse.example.com/events".to_string(),
            headers: vec![HttpHeader {
                name: "X-API-Key".to_string(),
                value: "apikey456".to_string(),
                meta: None,
            }],
        };

        let json = serde_json::to_value(&server).unwrap();
        assert_eq!(
            json,
            json!({
                "type": "sse",
                "name": "sse-server",
                "url": "https://sse.example.com/events",
                "headers": [
                    {
                        "name": "X-API-Key",
                        "value": "apikey456"
                    }
                ]
            })
        );

        let deserialized: McpServer = serde_json::from_value(json).unwrap();
        match deserialized {
            McpServer::Sse { name, url, headers } => {
                assert_eq!(name, "sse-server");
                assert_eq!(url, "https://sse.example.com/events");
                assert_eq!(headers.len(), 1);
                assert_eq!(headers[0].name, "X-API-Key");
                assert_eq!(headers[0].value, "apikey456");
            }
            _ => panic!("Expected Sse variant"),
        }
    }
}
