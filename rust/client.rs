//! Methods and notifications the client handles/receives.
//!
//! This module defines the Client trait and all associated types for implementing
//! a client that interacts with AI coding agents via the Agent Client Protocol (ACP).

use std::{fmt, path::PathBuf, sync::Arc};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ContentBlock, EnvVariable, Error, Plan, SessionId, ToolCall, ToolCallUpdate};

/// Defines the interface that ACP-compliant clients must implement.
///
/// Clients are typically code editors (IDEs, text editors) that provide the interface
/// between users and AI agents. They manage the environment, handle user interactions,
/// and control access to resources.
pub trait Client {
    /// Requests permission from the user for a tool call operation.
    ///
    /// Called by the agent when it needs user authorization before executing
    /// a potentially sensitive operation. The client should present the options
    /// to the user and return their decision.
    ///
    /// If the client cancels the prompt turn via `session/cancel`, it MUST
    /// respond to this request with `RequestPermissionOutcome::Cancelled`.
    ///
    /// See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
    fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> impl Future<Output = Result<RequestPermissionResponse, Error>>;

    /// Writes content to a text file in the client's file system.
    ///
    /// Only available if the client advertises the `fs.writeTextFile` capability.
    /// Allows the agent to create or modify files within the client's environment.
    ///
    /// See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
    fn write_text_file(
        &self,
        args: WriteTextFileRequest,
    ) -> impl Future<Output = Result<(), Error>>;

    /// Reads content from a text file in the client's file system.
    ///
    /// Only available if the client advertises the `fs.readTextFile` capability.
    /// Allows the agent to access file contents within the client's environment.
    ///
    /// See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
    fn read_text_file(
        &self,
        args: ReadTextFileRequest,
    ) -> impl Future<Output = Result<ReadTextFileResponse, Error>>;

    fn create_terminal(
        &self,
        args: CreateTerminalRequest,
    ) -> impl Future<Output = Result<CreateTerminalResponse, Error>>;

    fn terminal_output(
        &self,
        args: TerminalOutputRequest,
    ) -> impl Future<Output = Result<TerminalOutputResponse, Error>>;

    fn release_terminal(
        &self,
        args: ReleaseTerminalRequest,
    ) -> impl Future<Output = Result<(), Error>>;

    fn wait_for_terminal_exit(
        &self,
        args: WaitForTerminalExitRequest,
    ) -> impl Future<Output = Result<WaitForTerminalExitResponse, Error>>;

    /// Handles session update notifications from the agent.
    ///
    /// This is a notification endpoint (no response expected) that receives
    /// real-time updates about session progress, including message chunks,
    /// tool calls, and execution plans.
    ///
    /// Note: Clients SHOULD continue accepting tool call updates even after
    /// sending a `session/cancel` notification, as the agent may send final
    /// updates before responding with the cancelled stop reason.
    ///
    /// See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
    fn session_notification(
        &self,
        args: SessionNotification,
    ) -> impl Future<Output = Result<(), Error>>;
}

// Session updates

/// Notification containing a session update from the agent.
///
/// Used to stream real-time progress and results during prompt processing.
///
/// See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "session/update"))]
#[serde(rename_all = "camelCase")]
pub struct SessionNotification {
    /// The ID of the session this update pertains to.
    pub session_id: SessionId,
    /// The actual update content.
    pub update: SessionUpdate,
}

/// Different types of updates that can be sent during session processing.
///
/// These updates provide real-time feedback about the agent's progress.
///
/// See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "sessionUpdate")]
pub enum SessionUpdate {
    /// A chunk of the user's message being streamed.
    UserMessageChunk { content: ContentBlock },
    /// A chunk of the agent's response being streamed.
    AgentMessageChunk { content: ContentBlock },
    /// A chunk of the agent's internal reasoning being streamed.
    AgentThoughtChunk { content: ContentBlock },
    /// Notification that a new tool call has been initiated.
    ToolCall(ToolCall),
    /// Update on the status or results of a tool call.
    ToolCallUpdate(ToolCallUpdate),
    /// The agent's execution plan for complex tasks.
    /// See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
    Plan(Plan),
}

// Permission

/// Request for user permission to execute a tool call.
///
/// Sent when the agent needs authorization before performing a sensitive operation.
///
/// See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "session/request_permission"))]
#[serde(rename_all = "camelCase")]
pub struct RequestPermissionRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// Details about the tool call requiring permission.
    pub tool_call: ToolCallUpdate,
    /// Available permission options for the user to choose from.
    pub options: Vec<PermissionOption>,
}

/// An option presented to the user when requesting permission.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PermissionOption {
    /// Unique identifier for this permission option.
    #[serde(rename = "optionId")]
    pub id: PermissionOptionId,
    /// Human-readable label to display to the user.
    pub name: String,
    /// Hint about the nature of this permission option.
    pub kind: PermissionOptionKind,
}

/// Unique identifier for a permission option.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PermissionOptionId(pub Arc<str>);

impl fmt::Display for PermissionOptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The type of permission option being presented to the user.
///
/// Helps clients choose appropriate icons and UI treatment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionOptionKind {
    /// Allow this operation only this time.
    AllowOnce,
    /// Allow this operation and remember the choice.
    AllowAlways,
    /// Reject this operation only this time.
    RejectOnce,
    /// Reject this operation and remember the choice.
    RejectAlways,
}

/// Response to a permission request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "session/request_permission"))]
#[serde(rename_all = "camelCase")]
pub struct RequestPermissionResponse {
    /// The user's decision on the permission request.
    // This extra-level is unfortunately needed because the output must be an object
    pub outcome: RequestPermissionOutcome,
}

/// The outcome of a permission request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum RequestPermissionOutcome {
    /// The prompt turn was cancelled before the user responded.
    ///
    /// When a client sends a `session/cancel` notification to cancel an ongoing
    /// prompt turn, it MUST respond to all pending `session/request_permission`
    /// requests with this `Cancelled` outcome.
    ///
    /// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
    Cancelled,
    /// The user selected one of the provided options.
    #[serde(rename_all = "camelCase")]
    Selected {
        /// The ID of the option the user selected.
        option_id: PermissionOptionId,
    },
}

// Write text file

/// Request to write content to a text file.
///
/// Only available if the client supports the `fs.writeTextFile` capability.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "fs/write_text_file"))]
#[serde(rename_all = "camelCase")]
pub struct WriteTextFileRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// Absolute path to the file to write.
    pub path: PathBuf,
    /// The text content to write to the file.
    pub content: String,
}

// Read text file

/// Request to read content from a text file.
///
/// Only available if the client supports the `fs.readTextFile` capability.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "fs/read_text_file"))]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileRequest {
    /// The session ID for this request.
    pub session_id: SessionId,
    /// Absolute path to the file to read.
    pub path: PathBuf,
    /// Optional line number to start reading from (1-based).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Optional maximum number of lines to read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Response containing the contents of a text file.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileResponse {
    pub content: String,
}

// Terminals

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TerminalId(pub Arc<str>);

impl std::fmt::Display for TerminalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "terminal/create"))]
#[serde(rename_all = "camelCase")]
pub struct CreateTerminalRequest {
    pub session_id: SessionId,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvVariable>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_byte_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "terminal/create"))]
#[serde(rename_all = "camelCase")]
pub struct CreateTerminalResponse {
    pub terminal_id: TerminalId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "terminal/output"))]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutputRequest {
    pub session_id: SessionId,
    pub terminal_id: TerminalId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "terminal/output"))]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutputResponse {
    pub output: String,
    pub truncated: bool,
    pub exit_status: Option<TerminalExitStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "terminal/release"))]
#[serde(rename_all = "camelCase")]
pub struct ReleaseTerminalRequest {
    pub session_id: SessionId,
    pub terminal_id: TerminalId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "terminal/wait_for_exit"))]
#[serde(rename_all = "camelCase")]
pub struct WaitForTerminalExitRequest {
    pub session_id: SessionId,
    pub terminal_id: TerminalId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-side" = "client", "x-method" = "terminal/wait_for_exit"))]
#[serde(rename_all = "camelCase")]
pub struct WaitForTerminalExitResponse {
    #[serde(flatten)]
    pub exit_status: TerminalExitStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalExitStatus {
    pub exit_code: Option<u32>,
    pub signal: Option<String>,
}

// Capabilities

/// Capabilities supported by the client.
///
/// Advertised during initialization to inform the agent about
/// available features and methods.
///
/// See protocol docs: [Client Capabilities](https://agentclientprotocol.com/protocol/initialization#client-capabilities)
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// File system capabilities supported by the client.
    /// Determines which file operations the agent can request.
    #[serde(default)]
    pub fs: FileSystemCapability,
    #[serde(default)]
    pub terminal: bool,
}

/// File system capabilities that a client may support.
///
/// See protocol docs: [FileSystem](https://agentclientprotocol.com/protocol/initialization#filesystem)
#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemCapability {
    /// Whether the Client supports `fs/read_text_file` requests.
    #[serde(default)]
    pub read_text_file: bool,
    /// Whether the Client supports `fs/write_text_file` requests.
    #[serde(default)]
    pub write_text_file: bool,
}

// Method schema

/// Names of all methods that clients handle.
///
/// Provides a centralized definition of method names used in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMethodNames {
    /// Method for requesting permission from the user.
    pub session_request_permission: &'static str,
    /// Notification for session updates.
    pub session_update: &'static str,
    /// Method for writing text files.
    pub fs_write_text_file: &'static str,
    /// Method for reading text files.
    pub fs_read_text_file: &'static str,
    /// Method for creating new terminals.
    pub terminal_create: &'static str,
    /// Method for getting terminals output.
    pub terminal_output: &'static str,
    /// Method for releasing a terminal.
    pub terminal_release: &'static str,
    /// Method for waiting for a terminal to finish.
    pub terminal_wait_for_exit: &'static str,
}

/// Constant containing all client method names.
pub const CLIENT_METHOD_NAMES: ClientMethodNames = ClientMethodNames {
    session_update: SESSION_UPDATE_NOTIFICATION,
    session_request_permission: SESSION_REQUEST_PERMISSION_METHOD_NAME,
    fs_write_text_file: FS_WRITE_TEXT_FILE_METHOD_NAME,
    fs_read_text_file: FS_READ_TEXT_FILE_METHOD_NAME,
    terminal_create: TERMINAL_CREATE_METHOD_NAME,
    terminal_output: TERMINAL_OUTPUT_METHOD_NAME,
    terminal_release: TERMINAL_RELEASE_METHOD_NAME,
    terminal_wait_for_exit: TERMINAL_WAIT_FOR_EXIT_METHOD_NAME,
};

/// Notification name for session updates.
pub(crate) const SESSION_UPDATE_NOTIFICATION: &str = "session/update";
/// Method name for requesting user permission.
pub(crate) const SESSION_REQUEST_PERMISSION_METHOD_NAME: &str = "session/request_permission";
/// Method name for writing text files.
pub(crate) const FS_WRITE_TEXT_FILE_METHOD_NAME: &str = "fs/write_text_file";
/// Method name for reading text files.
pub(crate) const FS_READ_TEXT_FILE_METHOD_NAME: &str = "fs/read_text_file";
/// Method name for creating a new terminal.
pub(crate) const TERMINAL_CREATE_METHOD_NAME: &str = "terminal/create";
/// Method for getting terminals output.
pub(crate) const TERMINAL_OUTPUT_METHOD_NAME: &str = "terminal/output";
/// Method for releasing a terminal.
pub(crate) const TERMINAL_RELEASE_METHOD_NAME: &str = "terminal/release";
/// Method for waiting for a terminal to finish.
pub(crate) const TERMINAL_WAIT_FOR_EXIT_METHOD_NAME: &str = "terminal/wait_for_exit";

/// All possible requests that an agent can send to a client.
///
/// This enum is used internally for routing RPC requests. You typically won't need
/// to use this directly - instead, use the methods on the [`Client`] trait.
///
/// This enum encompasses all method calls from agent to client.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-docs-ignore" = true))]
pub enum AgentRequest {
    WriteTextFileRequest(WriteTextFileRequest),
    ReadTextFileRequest(ReadTextFileRequest),
    RequestPermissionRequest(RequestPermissionRequest),
    CreateTerminalRequest(CreateTerminalRequest),
    TerminalOutputRequest(TerminalOutputRequest),
    ReleaseTerminalRequest(ReleaseTerminalRequest),
    WaitForTerminalExitRequest(WaitForTerminalExitRequest),
}

/// All possible responses that a client can send to an agent.
///
/// This enum is used internally for routing RPC responses. You typically won't need
/// to use this directly - the responses are handled automatically by the connection.
///
/// These are responses to the corresponding AgentRequest variants.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-docs-ignore" = true))]
pub enum ClientResponse {
    WriteTextFileResponse,
    ReadTextFileResponse(ReadTextFileResponse),
    RequestPermissionResponse(RequestPermissionResponse),
    CreateTerminalResponse(CreateTerminalResponse),
    TerminalOutputResponse(TerminalOutputResponse),
    ReleaseTerminalResponse,
    WaitForTerminalExitResponse(WaitForTerminalExitResponse),
}

/// All possible notifications that an agent can send to a client.
///
/// This enum is used internally for routing RPC notifications. You typically won't need
/// to use this directly - use the notification methods on the [`Client`] trait instead.
///
/// Notifications do not expect a response.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-docs-ignore" = true))]
pub enum AgentNotification {
    SessionNotification(SessionNotification),
}
