use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use futures::future::LocalBoxFuture;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ContentBlock, Error, Plan, SessionId, ToolCall, ToolCallUpdate};

pub trait Agent {
    fn new_session(
        &self,
        arguments: NewSessionArguments,
    ) -> LocalBoxFuture<'static, Result<NewSessionOutput, Error>>;

    fn load_session(
        &self,
        arguments: LoadSessionArguments,
    ) -> LocalBoxFuture<'static, Result<LoadSessionOutput, Error>>;

    fn prompt(&self, arguments: PromptArguments) -> LocalBoxFuture<'static, Result<(), Error>>;
}

pub const NEW_SESSION_METHOD_NAME: &'static str = "newSession";
pub const LOAD_SESSION_METHOD_NAME: &'static str = "loadSession";
pub const PROMPT_METHOD_NAME: &'static str = "prompt";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum AgentRequest {
    NewSession(NewSessionArguments),
    LoadSession(LoadSessionArguments),
    Prompt(PromptArguments),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AgentResponse {
    NewSession(NewSessionOutput),
    LoadSession(LoadSessionOutput),
    Prompt,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum AgentNotification {
    SessionUpdate(SessionNotification),
}

// Authenticatication

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateArguments {
    pub method_id: AuthMethodId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct AuthMethodId(pub Arc<str>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethod {
    pub id: AuthMethodId,
    pub label: String,
    pub description: Option<String>,
}

// New session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionArguments {
    pub mcp_servers: Vec<McpServer>,
    pub cwd: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionOutput {
    /// The session id if one was created, or null if authentication is required
    // Note: It'd be nicer to use an enum here, but MCP requires the output schema
    // to be a non-union object and adding another level seemed impractical.
    pub session_id: Option<SessionId>,
    #[serde(default)]
    pub auth_methods: Vec<AuthMethod>,
}

// Load session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionArguments {
    pub mcp_servers: Vec<McpServer>,
    pub cwd: PathBuf,
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionOutput {
    pub auth_required: bool,
    #[serde(default)]
    pub auth_methods: Vec<AuthMethod>,
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
pub struct PromptArguments {
    pub session_id: SessionId,
    pub prompt: Vec<ContentBlock>,
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
