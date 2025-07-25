pub mod mcp_types;
pub use mcp_types::*;

use std::{collections::HashMap, fmt::Display, path::PathBuf, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// New session

pub const NEW_SESSION_TOOL_NAME: &str = "acp/new_session";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionToolArguments {
    pub mcp_servers: HashMap<String, McpServerConfig>,
    pub client_tools: ClientTools,
    pub cwd: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionToolResult {
    pub session_id: SessionId,
}

// Load session

pub const LOAD_SESSION_TOOL_NAME: &str = "acp/load_session";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionToolArguments {
    pub mcp_servers: HashMap<String, McpServerConfig>,
    pub client_tools: ClientTools,
    pub cwd: PathBuf,
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SessionId(pub Arc<str>);

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    pub command: PathBuf,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    /// If provided, only the specified tools are enabled
    #[serde(default)]
    pub enabled_tools: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpToolId {
    pub mcp_server: String,
    pub tool_name: String,
}

// Prompt

pub const PROMPT_TOOL_NAME: &str = "acp/prompt";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PromptToolArguments {
    pub session_id: SessionId,
    pub prompt: Vec<ContentBlock>,
}

// Session updates

pub const SESSION_UPDATE_METHOD_NAME: &str = "acp/session_update";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionNotification {
    pub session_id: SessionId,
    #[serde(flatten)]
    pub update: SessionUpdate,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "sessionUpdate", rename_all = "camelCase")]
pub enum SessionUpdate {
    Started,
    UserMessage(ContentBlock),
    AgentMessageChunk(ContentBlock),
    AgentThoughtChunk(ContentBlock),
    ToolCall(ToolCall),
    ToolCallUpdate(ToolCallUpdate),
    Plan(Plan),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    #[serde(rename = "toolCallId")]
    pub id: ToolCallId,
    pub label: String,
    pub kind: ToolKind,
    pub status: ToolCallStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ToolCallContent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<ToolCallLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallUpdate {
    #[serde(rename = "toolCallId")]
    pub id: ToolCallId,
    #[serde(flatten)]
    pub fields: ToolCallUpdateFields,
}

#[derive(Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallUpdateFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<ToolKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<ToolCallStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ToolCallContent>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<ToolCallLocation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ToolCallId(pub Arc<str>);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ToolKind {
    Read,
    Edit,
    Search,
    Execute,
    Think,
    Fetch,
    Other,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ToolCallStatus {
    /// The tool call is currently running
    InProgress,
    /// The tool call completed successfully
    Completed,
    /// The tool call failed
    Failed,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged, rename_all = "camelCase")]
pub enum ToolCallContent {
    ContentBlock(ContentBlock),
    Diff { diff: Diff },
}

impl<T: Into<ContentBlock>> From<T> for ToolCallContent {
    fn from(content: T) -> Self {
        ToolCallContent::ContentBlock(content.into())
    }
}

impl From<Diff> for ToolCallContent {
    fn from(diff: Diff) -> Self {
        ToolCallContent::Diff { diff }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Diff {
    pub path: PathBuf,
    pub old_text: Option<String>,
    pub new_text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ToolCallLocation {
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub entries: Vec<PlanEntry>,
}

/// A single entry in the execution plan.
///
/// Represents a task or goal that the assistant intends to accomplish
/// as part of fulfilling the user's request.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanEntry {
    /// Description of what this task aims to accomplish
    pub content: String,
    /// Relative importance of this task
    pub priority: PlanEntryPriority,
    /// Current progress of this task
    pub status: PlanEntryStatus,
}

/// Priority levels for plan entries.
///
/// Used to indicate the relative importance or urgency of different
/// tasks in the execution plan.
#[derive(Deserialize, Serialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PlanEntryPriority {
    High,
    Medium,
    Low,
}

/// Status of a plan entry in the execution flow.
///
/// Tracks the lifecycle of each task from planning through completion.
#[derive(Deserialize, Serialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PlanEntryStatus {
    Pending,
    InProgress,
    Completed,
}

// Client tools

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClientTools {
    pub request_permission: Option<McpToolId>,
    pub write_text_file: Option<McpToolId>,
    pub read_text_file: Option<McpToolId>,
}

// Permission

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PermissionToolArguments {
    pub session_id: SessionId,
    pub tool_call: ToolCall,
    pub options: Vec<PermissionOption>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PermissionOption {
    #[serde(rename = "optionId")]
    pub id: PermissionOptionId,
    pub label: String,
    pub kind: PermissionOptionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PermissionOptionId(pub Arc<str>);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PermissionOptionKind {
    AllowOnce,
    AllowAlways,
    RejectOnce,
    RejectAlways,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum PermissionOutcome {
    Canceled,
    #[serde(rename_all = "camelCase")]
    Selected {
        option_id: PermissionOptionId,
    },
}

// Write text file

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WriteTextFileToolArguments {
    pub session_id: SessionId,
    pub path: PathBuf,
    pub content: String,
}

// Read text file

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileArguments {
    pub session_id: SessionId,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}
