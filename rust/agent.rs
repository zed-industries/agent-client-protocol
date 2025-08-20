//! Methods and notifications the agent handles/receives

use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ClientCapabilities, ContentBlock, Error, ProtocolVersion, SessionId};

pub trait Agent {
    fn initialize(
        &self,
        arguments: InitializeRequest,
    ) -> impl Future<Output = Result<InitializeResponse, Error>>;

    fn authenticate(
        &self,
        arguments: AuthenticateRequest,
    ) -> impl Future<Output = Result<(), Error>>;

    fn new_session(
        &self,
        arguments: NewSessionRequest,
    ) -> impl Future<Output = Result<NewSessionResponse, Error>>;

    fn load_session(
        &self,
        arguments: LoadSessionRequest,
    ) -> impl Future<Output = Result<(), Error>>;

    fn prompt(
        &self,
        arguments: PromptRequest,
    ) -> impl Future<Output = Result<PromptResponse, Error>>;

    fn cancel(&self, args: CancelNotification) -> impl Future<Output = Result<(), Error>>;
}

// Initialize

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequest {
    /// The latest protocol version supported by the client.
    pub protocol_version: ProtocolVersion,
    /// Capabilities supported by the client.
    #[serde(default)]
    pub client_capabilities: ClientCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
}

// Authentication

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    pub method_id: AuthMethodId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct AuthMethodId(pub Arc<str>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethod {
    pub id: AuthMethodId,
    pub name: String,
    pub description: Option<String>,
}

// New session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionRequest {
    pub mcp_servers: Vec<McpServer>,
    pub cwd: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionResponse {
    pub session_id: SessionId,
}

// Load session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionRequest {
    pub mcp_servers: Vec<McpServer>,
    pub cwd: PathBuf,
    pub session_id: SessionId,
}

// MCP

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub name: String,
    pub command: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<EnvVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnvVariable {
    pub name: String,
    pub value: String,
}

// Prompt

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    /// ## Context
    ///
    /// The client MAY include referenced pieces of context as either
    /// [`ContentBlock::EmbeddedResource`] or [`ContentBlock::ResourceLink`].
    ///
    /// When available, [`ContentBlock::EmbeddedResource`] is preferred
    /// as it avoids extra round-trips and allows the message to include
    /// pieces of context from sources the agent may not have access to.
    pub prompt: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PromptResponse {
    pub stop_reason: StopReason,
}

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
    /// The turn was cancelled by the client.
    Cancelled,
}

// Capabilities

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    /// Whether the agent supports `session/load`.
    #[serde(default)]
    pub load_session: bool,
    /// Prompt capabilities supported by the agent.
    #[serde(default)]
    pub prompt_capabilities: PromptCapabilities,
}

/// Prompt capabilities supported by the agent in `session/prompt` requests.
///
/// Baseline agent functionality requires support for [`ContentBlock::Text`]
/// and [`ContentBlock::ResourceLink`] in prompt requests.
///
/// Other variants must be explicitly opted in to.
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
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
    /// When enabled, the Client is allowed to include [`ContentBlock::EmbeddedResource`]
    /// in prompt requests for pieces of context that are referenced in the message.
    #[serde(default)]
    pub embedded_context: bool,
}

// Method schema

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMethodNames {
    pub initialize: &'static str,
    pub authenticate: &'static str,
    pub session_new: &'static str,
    pub session_load: &'static str,
    pub session_prompt: &'static str,
    pub session_cancel: &'static str,
}

pub const AGENT_METHOD_NAMES: AgentMethodNames = AgentMethodNames {
    initialize: INITIALIZE_METHOD_NAME,
    authenticate: AUTHENTICATE_METHOD_NAME,
    session_new: SESSION_NEW_METHOD_NAME,
    session_load: SESSION_LOAD_METHOD_NAME,
    session_prompt: SESSION_PROMPT_METHOD_NAME,
    session_cancel: SESSION_CANCEL_METHOD_NAME,
};

pub const INITIALIZE_METHOD_NAME: &str = "initialize";
pub const AUTHENTICATE_METHOD_NAME: &str = "authenticate";
pub const SESSION_NEW_METHOD_NAME: &str = "session/new";
pub const SESSION_LOAD_METHOD_NAME: &str = "session/load";
pub const SESSION_PROMPT_METHOD_NAME: &str = "session/prompt";
pub const SESSION_CANCEL_METHOD_NAME: &str = "session/cancel";

/// Requests the client sends to the agent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ClientRequest {
    InitializeRequest(InitializeRequest),
    AuthenticateRequest(AuthenticateRequest),
    NewSessionRequest(NewSessionRequest),
    LoadSessionRequest(LoadSessionRequest),
    PromptRequest(PromptRequest),
}

/// Responses the agent sends to the client
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AgentResponse {
    InitializeResponse(InitializeResponse),
    AuthenticateResponse,
    NewSessionResponse(NewSessionResponse),
    LoadSessionResponse,
    PromptResponse(PromptResponse),
}

/// Notifications the client sends to the agent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ClientNotification {
    CancelNotification(CancelNotification),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CancelNotification {
    pub session_id: SessionId,
}
