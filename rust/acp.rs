pub mod mcp_types;
pub use mcp_types::*;

use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AgentMethods {
    pub new_session: &'static str,
    pub load_session: &'static str,
    pub prompt: &'static str,
    pub session_update: &'static str,
}

pub const AGENT_METHODS: AgentMethods = AgentMethods {
    new_session: "acp/new_session",
    load_session: "acp/load_session",
    prompt: "acp/prompt",
    session_update: "acp/session_update",
};

// New session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionArguments {
    pub mcp_servers: HashMap<String, McpServerConfig>,
    pub client_tools: ClientTools,
    pub cwd: PathBuf,
}

impl NewSessionArguments {
    pub fn schema() -> serde_json::Value {
        schema_for::<Self>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionOutput {
    pub session_id: SessionId,
}

impl NewSessionOutput {
    pub fn schema() -> serde_json::Value {
        schema_for::<Self>()
    }
}
// Load session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionArguments {
    pub mcp_servers: HashMap<String, McpServerConfig>,
    pub client_tools: ClientTools,
    pub cwd: PathBuf,
    pub session_id: SessionId,
}

impl LoadSessionArguments {
    pub fn schema() -> serde_json::Value {
        schema_for::<Self>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SessionId(pub Arc<str>);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    pub command: PathBuf,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct McpToolId {
    pub mcp_server: String,
    pub tool_name: String,
}

// Prompt

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PromptArguments {
    pub session_id: SessionId,
    pub prompt: Vec<ContentBlock>,
}

impl PromptArguments {
    pub fn schema() -> serde_json::Value {
        schema_for::<Self>()
    }
}

// Session updates

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionNotification {
    pub session_id: SessionId,
    #[serde(flatten)]
    pub update: SessionUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "sessionUpdate", rename_all = "camelCase")]
pub enum SessionUpdate {
    UserMessageChunk { content: ContentBlock },
    AgentMessageChunk { content: ContentBlock },
    AgentThoughtChunk { content: ContentBlock },
    ToolCall(ToolCall),
    ToolCallUpdate(ToolCallUpdate),
    Plan(Plan),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub raw_input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallUpdate {
    #[serde(rename = "toolCallId")]
    pub id: ToolCallId,
    #[serde(flatten)]
    pub fields: ToolCallUpdateFields,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub raw_input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ToolCallId(pub Arc<str>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ToolKind {
    Read,
    Edit,
    Delete,
    Move,
    Search,
    Execute,
    Think,
    Fetch,
    Other,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ToolCallStatus {
    /// The tool call is currently running
    Pending,
    /// The tool call is currently running
    InProgress,
    /// The tool call completed successfully
    Completed,
    /// The tool call failed
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolCallContent {
    Content {
        content: ContentBlock,
    },
    Diff {
        #[serde(flatten)]
        diff: Diff,
    },
}

impl<T: Into<ContentBlock>> From<T> for ToolCallContent {
    fn from(content: T) -> Self {
        ToolCallContent::Content {
            content: content.into(),
        }
    }
}

impl From<Diff> for ToolCallContent {
    fn from(diff: Diff) -> Self {
        ToolCallContent::Diff { diff }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub entries: Vec<PlanEntry>,
}

/// A single entry in the execution plan.
///
/// Represents a task or goal that the assistant intends to accomplish
/// as part of fulfilling the user's request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PlanEntryPriority {
    High,
    Medium,
    Low,
}

/// Status of a plan entry in the execution flow.
///
/// Tracks the lifecycle of each task from planning through completion.
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PlanEntryStatus {
    Pending,
    InProgress,
    Completed,
}

// Client tools

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClientTools {
    pub request_permission: Option<McpToolId>,
    pub write_text_file: Option<McpToolId>,
    pub read_text_file: Option<McpToolId>,
}

// Permission

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestPermissionArguments {
    pub session_id: SessionId,
    pub tool_call: ToolCall,
    pub options: Vec<PermissionOption>,
}

impl RequestPermissionArguments {
    pub fn schema() -> serde_json::Value {
        schema_for::<Self>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PermissionOption {
    #[serde(rename = "optionId")]
    pub id: PermissionOptionId,
    pub label: String,
    pub kind: PermissionOptionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PermissionOptionId(pub Arc<str>);

impl fmt::Display for PermissionOptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PermissionOptionKind {
    AllowOnce,
    AllowAlways,
    RejectOnce,
    RejectAlways,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestPermissionOutput {
    // This extra-level is unfortunately needed because the output must be an object
    pub outcome: RequestPermissionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum RequestPermissionOutcome {
    Canceled,
    #[serde(rename_all = "camelCase")]
    Selected {
        option_id: PermissionOptionId,
    },
}

// Write text file

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WriteTextFileArguments {
    pub session_id: SessionId,
    pub path: PathBuf,
    pub content: String,
}

impl WriteTextFileArguments {
    pub fn schema() -> serde_json::Value {
        schema_for::<Self>()
    }
}

// Read text file

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileArguments {
    pub session_id: SessionId,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl ReadTextFileArguments {
    pub fn schema() -> serde_json::Value {
        schema_for::<Self>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileOutput {
    pub content: String,
}

impl ReadTextFileOutput {
    pub fn schema() -> serde_json::Value {
        schema_for::<Self>()
    }
}

fn schema_for<T: JsonSchema>() -> serde_json::Value {
    let mut settings = SchemaSettings::draft2020_12();
    settings.inline_subschemas = true;
    settings.into_generator().into_root_schema_for::<T>().into()
}
