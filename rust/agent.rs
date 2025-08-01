use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use futures::future::LocalBoxFuture;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ContentBlock, Error, Plan, SessionId, ToolCall, ToolCallUpdate};

pub trait Agent {
    fn authenticate(
        &self,
        arguments: AuthenticateRequest,
    ) -> LocalBoxFuture<'static, Result<(), Error>>;

    fn new_session(
        &self,
        arguments: NewSessionRequest,
    ) -> LocalBoxFuture<'static, Result<NewSessionResponse, Error>>;

    fn load_session(
        &self,
        arguments: LoadSessionRequest,
    ) -> LocalBoxFuture<'static, Result<LoadSessionResponse, Error>>;

    fn prompt(&self, arguments: PromptRequest) -> LocalBoxFuture<'static, Result<(), Error>>;
}

// Authenticatication

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
    pub label: String,
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
pub struct LoadSessionRequest {
    pub mcp_servers: Vec<McpServer>,
    pub cwd: PathBuf,
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionResponse {
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
pub struct PromptRequest {
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

// Method schema

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMethodNames {
    pub authenticate: &'static str,
    pub session_new: &'static str,
    pub session_load: &'static str,
    pub session_prompt: &'static str,
    pub session_update: &'static str,
}

pub const AGENT_METHOD_NAMES: AgentMethodNames = AgentMethodNames {
    authenticate: AUTHENTICATE_METHOD_NAME,
    session_new: SESSION_NEW_METHOD_NAME,
    session_load: SESSION_LOAD_METHOD_NAME,
    session_prompt: SESSION_PROMPT_METHOD_NAME,
    session_update: SESSION_UPDATE_NOTIFICATION,
};

pub const AUTHENTICATE_METHOD_NAME: &str = "authenticate";
pub const SESSION_NEW_METHOD_NAME: &str = "session/new";
pub const SESSION_LOAD_METHOD_NAME: &str = "session/load";
pub const SESSION_PROMPT_METHOD_NAME: &str = "session/prompt";
pub const SESSION_UPDATE_NOTIFICATION: &str = "session/update";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AgentRequest {
    AuthenticateRequest(AuthenticateRequest),
    NewSessionRequest(NewSessionRequest),
    LoadSessionRequest(LoadSessionRequest),
    PromptRequest(PromptRequest),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AgentResponse {
    AuthenticateResponse,
    NewSessionResponse(NewSessionResponse),
    LoadSessionResponse(LoadSessionResponse),
    PromptResponse,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AgentNotification {
    SessionNotification(SessionNotification),
}
