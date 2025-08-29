//! Methods and notifications the agent handles/receives.
//!
//! This module defines the Agent trait and all associated types for implementing
//! an AI coding agent that follows the Agent Client Protocol (ACP).

use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ClientCapabilities, ContentBlock, Error, ProtocolVersion, SessionId};

/// Defines the interface that all ACP-compliant agents must implement.
///
/// Agents are programs that use generative AI to autonomously modify code. They handle
/// requests from clients and execute tasks using language models and tools.
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
    fn initialize(
        &self,
        arguments: InitializeRequest,
    ) -> impl Future<Output = Result<InitializeResponse, Error>>;

    /// Authenticates the client using the specified authentication method.
    ///
    /// Called when the agent requires authentication before allowing session creation.
    /// The client provides the authentication method ID that was advertised during initialization.
    ///
    /// After successful authentication, the client can proceed to create sessions with
    /// `new_session` without receiving an `auth_required` error.
    ///
    /// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
    fn authenticate(
        &self,
        arguments: AuthenticateRequest,
    ) -> impl Future<Output = Result<(), Error>>;

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
    fn new_session(
        &self,
        arguments: NewSessionRequest,
    ) -> impl Future<Output = Result<NewSessionResponse, Error>>;

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
    fn load_session(
        &self,
        arguments: LoadSessionRequest,
    ) -> impl Future<Output = Result<(), Error>>;

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
    fn prompt(
        &self,
        arguments: PromptRequest,
    ) -> impl Future<Output = Result<PromptResponse, Error>>;

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
    fn cancel(&self, args: CancelNotification) -> impl Future<Output = Result<(), Error>>;

    /// Lists available custom commands for a session.
    ///
    /// Returns all commands available in the agent's `.claude/commands` directory
    /// or equivalent command registry. Commands can be executed via `run_command`.
    fn list_commands(
        &self,
        arguments: ListCommandsRequest,
    ) -> impl Future<Output = Result<ListCommandsResponse, Error>>;

    /// Executes a custom command within a session.
    ///
    /// Runs the specified command with optional arguments. The agent should
    /// stream results back via session update notifications.
    fn run_command(&self, arguments: RunCommandRequest) -> impl Future<Output = Result<(), Error>>;
}

// Initialize

/// Request parameters for the initialize method.
///
/// Sent by the client to establish connection and negotiate capabilities.
///
/// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "initialize"))]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequest {
    /// The latest protocol version supported by the client.
    pub protocol_version: ProtocolVersion,
    /// Capabilities supported by the client.
    #[serde(default)]
    pub client_capabilities: ClientCapabilities,
}

/// Response from the initialize method.
///
/// Contains the negotiated protocol version and agent capabilities.
///
/// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "initialize"))]
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

/// Request parameters for the authenticate method.
///
/// Specifies which authentication method to use.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "authenticate"))]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    /// The ID of the authentication method to use.
    /// Must be one of the methods advertised in the initialize response.
    pub method_id: AuthMethodId,
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
}

// New session

/// Request parameters for creating a new session.
///
/// See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "session/new"))]
#[serde(rename_all = "camelCase")]
pub struct NewSessionRequest {
    /// The working directory for this session. Must be an absolute path.
    pub cwd: PathBuf,
    /// List of MCP (Model Context Protocol) servers the agent should connect to.
    pub mcp_servers: Vec<McpServer>,
}

/// Response from creating a new session.
///
/// See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "session/new"))]
#[serde(rename_all = "camelCase")]
pub struct NewSessionResponse {
    /// Unique identifier for the created session.
    ///
    /// Used in all subsequent requests for this conversation.
    pub session_id: SessionId,
}

// Load session

/// Request parameters for loading an existing session.
///
/// Only available if the agent supports the `loadSession` capability.
///
/// See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "session/load"))]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionRequest {
    /// List of MCP servers to connect to for this session.
    pub mcp_servers: Vec<McpServer>,
    /// The working directory for this session.
    pub cwd: PathBuf,
    /// The ID of the session to load.
    pub session_id: SessionId,
}

// MCP

/// Configuration for connecting to an MCP (Model Context Protocol) server.
///
/// MCP servers provide tools and context that the agent can use when
/// processing prompts.
///
/// See protocol docs: [MCP Servers](https://agentclientprotocol.com/protocol/session-setup#mcp-servers)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    /// Human-readable name identifying this MCP server.
    pub name: String,
    /// Path to the MCP server executable.
    pub command: PathBuf,
    /// Command-line arguments to pass to the MCP server.
    pub args: Vec<String>,
    /// Environment variables to set when launching the MCP server.
    pub env: Vec<EnvVariable>,
}

/// An environment variable to set when launching an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnvVariable {
    /// The name of the environment variable.
    pub name: String,
    /// The value to set for the environment variable.
    pub value: String,
}

// Prompt

/// Request parameters for sending a user prompt to the agent.
///
/// Contains the user's message and any additional context.
///
/// See protocol docs: [User Message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "session/prompt"))]
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
}

/// Response from processing a user prompt.
///
/// See protocol docs: [Check for Completion](https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "session/prompt"))]
#[serde(rename_all = "camelCase")]
pub struct PromptResponse {
    /// Indicates why the agent stopped processing the turn.
    pub stop_reason: StopReason,
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
    /// When enabled, the Client is allowed to include [`ContentBlock::Resource`]
    /// in prompt requests for pieces of context that are referenced in the message.
    #[serde(default)]
    pub embedded_context: bool,
    /// Agent supports commands via `list_commands` and `run_command`.
    #[serde(default)]
    pub supports_commands: bool,
}

// Slash commands

/// Request parameters for listing available commands.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "session/list_commands"))]
#[serde(rename_all = "camelCase")]
pub struct ListCommandsRequest {
    /// The session ID to list commands for.
    pub session_id: SessionId,
}

/// Response containing available commands.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "session/list_commands"))]
#[serde(rename_all = "camelCase")]
pub struct ListCommandsResponse {
    /// List of available commands.
    pub commands: Vec<CommandInfo>,
}

/// Information about a custom command.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandInfo {
    /// Command name (e.g., "create_plan", "research_codebase").
    pub name: String,
    /// Human-readable description of what the command does.
    pub description: String,
    /// Whether this command requires arguments from the user.
    pub requires_argument: bool,
}

/// Request parameters for executing a command.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "session/run_command"))]
#[serde(rename_all = "camelCase")]
pub struct RunCommandRequest {
    /// The session ID to execute the command in.
    pub session_id: SessionId,
    /// Name of the command to execute.
    pub command: String,
    /// Optional arguments for the command.
    pub args: Option<String>,
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
    /// Method for sending a prompt to the agent.
    pub session_prompt: &'static str,
    /// Notification for cancelling operations.
    pub session_cancel: &'static str,
    /// Method for listing available commands.
    pub session_list_commands: &'static str,
    /// Method for running a command.
    pub session_run_command: &'static str,
}

/// Constant containing all agent method names.
pub const AGENT_METHOD_NAMES: AgentMethodNames = AgentMethodNames {
    initialize: INITIALIZE_METHOD_NAME,
    authenticate: AUTHENTICATE_METHOD_NAME,
    session_new: SESSION_NEW_METHOD_NAME,
    session_load: SESSION_LOAD_METHOD_NAME,
    session_prompt: SESSION_PROMPT_METHOD_NAME,
    session_cancel: SESSION_CANCEL_METHOD_NAME,
    session_list_commands: SESSION_LIST_COMMANDS,
    session_run_command: SESSION_RUN_COMMAND,
};

/// Method name for the initialize request.
pub(crate) const INITIALIZE_METHOD_NAME: &str = "initialize";
/// Method name for the authenticate request.
pub(crate) const AUTHENTICATE_METHOD_NAME: &str = "authenticate";
/// Method name for creating a new session.
pub(crate) const SESSION_NEW_METHOD_NAME: &str = "session/new";
/// Method name for loading an existing session.
pub(crate) const SESSION_LOAD_METHOD_NAME: &str = "session/load";
/// Method name for sending a prompt.
pub(crate) const SESSION_PROMPT_METHOD_NAME: &str = "session/prompt";
/// Method name for the cancel notification.
pub(crate) const SESSION_CANCEL_METHOD_NAME: &str = "session/cancel";
/// Method name for listing custom commands in a session.
pub const SESSION_LIST_COMMANDS: &str = "session/list_commands";
/// Method name for running a custom command in a session.
pub const SESSION_RUN_COMMAND: &str = "session/run_command";

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
    PromptRequest(PromptRequest),
    ListCommandsRequest(ListCommandsRequest),
    RunCommandRequest(RunCommandRequest),
}

/// All possible responses that an agent can send to a client.
///
/// This enum is used internally for routing RPC responses. You typically won't need
/// to use this directly - the responses are handled automatically by the connection.
///
/// These are responses to the corresponding ClientRequest variants.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-docs-ignore" = true))]
pub enum AgentResponse {
    InitializeResponse(InitializeResponse),
    AuthenticateResponse,
    NewSessionResponse(NewSessionResponse),
    LoadSessionResponse,
    PromptResponse(PromptResponse),
    ListCommandsResponse(ListCommandsResponse),
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
}

/// Notification to cancel ongoing operations for a session.
///
/// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "agent", "x-method" = "session/cancel"))]
#[serde(rename_all = "camelCase")]
pub struct CancelNotification {
    /// The ID of the session to cancel operations for.
    pub session_id: SessionId,
}
