pub mod mcp_types;
pub use mcp_types::*;

pub mod rpc;

use futures::{FutureExt, future::LocalBoxFuture};

use std::{
    fmt::{self, Display},
    ops::Deref as _,
    path::PathBuf,
    sync::Arc,
};

use anyhow::Result;
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};

pub trait Agent {
    fn new_session(
        &self,
        arguments: NewSessionArguments,
    ) -> impl Future<Output = Result<NewSessionOutput>>;

    fn load_session(
        &self,
        arguments: LoadSessionArguments,
    ) -> impl Future<Output = Result<LoadSessionOutput>>;

    fn prompt(&self, arguments: PromptArguments) -> impl Future<Output = Result<()>>;
}

// impl<T: Agent> LocalPeer for T {
//     type Request = AgentRequest;
//     type Response = AgentResponse;
//     type IncomingNotification = AgentNotification;

//     fn dispatch_request<'a>(
//         &'a self,
//         request: Self::Request,
//     ) -> LocalBoxFuture<'a, Result<Self::Response>> {
//         async move {
//             match request {
//                 AgentRequest::NewSession(args) => {
//                     self.new_session(args).await.map(AgentResponse::NewSession)
//                 }
//                 AgentRequest::LoadSession(args) => self
//                     .load_session(args)
//                     .await
//                     .map(AgentResponse::LoadSession),
//                 AgentRequest::Prompt(args) => {
//                     self.prompt(args).await.map(|()| AgentResponse::Prompt)
//                 }
//             }
//         }
//         .boxed_local()
//     }

//     fn dispatch_notification<'a>(
//         &'a self,
//         notification: Self::IncomingNotification,
//     ) -> LocalBoxFuture<'a, Result<()>> {
//         todo!();
//     }
// }

pub trait Client {
    fn write_text_file(
        &self,
        arguments: WriteTextFileArguments,
    ) -> impl Future<Output = Result<()>>;

    fn read_text_file(
        &self,
        arguments: ReadTextFileArguments,
    ) -> impl Future<Output = Result<ReadTextFileOutput>>;

    fn request_permission(
        &self,
        arguments: RequestPermissionArguments,
    ) -> impl Future<Output = Result<RequestPermissionOutput>>;

    fn dispatch<'a>(
        &'a self,
        request: ClientRequest,
    ) -> LocalBoxFuture<'a, Result<ClientResponse>> {
        async move {
            match request {
                ClientRequest::WriteTextFile(args) => self
                    .write_text_file(args)
                    .await
                    .map(|()| ClientResponse::WriteTextFile),
                ClientRequest::ReadTextFile(args) => self
                    .read_text_file(args)
                    .await
                    .map(ClientResponse::ReadTextFile),
                ClientRequest::RequestPermission(args) => self
                    .request_permission(args)
                    .await
                    .map(ClientResponse::RequestPermission),
            }
        }
        .boxed_local()
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum ClientRequest {
    WriteTextFile(WriteTextFileArguments),
    ReadTextFile(ReadTextFileArguments),
    RequestPermission(RequestPermissionArguments),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ClientResponse {
    WriteTextFile,
    ReadTextFile(ReadTextFileOutput),
    RequestPermission(RequestPermissionOutput),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum ClientNotification {
    Cancelled { request_id: u64 },
}

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
    SessionUpdate(SessionUpdate),
}

#[derive(Serialize)]
pub struct AgentMethods {
    pub authenticate: &'static str,
    pub new_session: &'static str,
    pub load_session: &'static str,
    pub prompt: &'static str,
    pub session_update: &'static str,
}

pub const AGENT_METHODS: AgentMethods = AgentMethods {
    authenticate: "acp/authenticate",
    new_session: "acp/new_session",
    load_session: "acp/load_session",
    prompt: "acp/prompt",
    session_update: "acp/session_update",
};

// New session

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionArguments {
    pub mcp_servers: Vec<McpServer>,
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
    /// The session id if one was created, or null if authentication is required
    // Note: It'd be nicer to use an enum here, but MCP requires the output schema
    // to be a non-union object and adding another level seemed impractical.
    pub session_id: Option<SessionId>,
    #[serde(default)]
    pub auth_methods: Vec<AuthMethod>,
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
    pub mcp_servers: Vec<McpServer>,
    pub client_tools: ClientTools,
    pub cwd: PathBuf,
    pub session_id: SessionId,
}

impl LoadSessionArguments {
    pub fn schema() -> serde_json::Value {
        schema_for::<Self>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoadSessionOutput {
    pub auth_required: bool,
    #[serde(default)]
    pub auth_methods: Vec<AuthMethod>,
}

impl LoadSessionOutput {
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct McpToolId {
    pub mcp_server: String,
    pub tool_name: String,
}

// Agent state

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentState {
    pub auth_methods: Vec<AuthMethod>,
    // pub models: Vec<Model>,
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

// #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
// #[serde(transparent)]
// pub struct ModelId(pub Arc<str>);

// #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
// #[serde(rename_all = "camelCase")]
// pub struct Model {
//     pub id: ModelId,
//     pub label: String,
// }

// Authenticatication

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateArguments {
    pub method_id: AuthMethodId,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Error {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Error {
    pub fn new(code: impl Into<(i32, String)>) -> Self {
        let (code, message) = code.into();
        Error {
            code,
            message,
            data: None,
        }
    }

    pub fn with_data(mut self, data: impl Into<serde_json::Value>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
    pub fn parse_error() -> Self {
        Error::new(ErrorCode::PARSE_ERROR)
    }

    /// The JSON sent is not a valid Request object.
    pub fn invalid_request() -> Self {
        Error::new(ErrorCode::INVALID_REQUEST)
    }

    /// The method does not exist / is not available.
    pub fn method_not_found() -> Self {
        Error::new(ErrorCode::METHOD_NOT_FOUND)
    }

    /// Invalid method parameter(s).
    pub fn invalid_params() -> Self {
        Error::new(ErrorCode::INVALID_PARAMS)
    }

    /// Internal JSON-RPC error.
    pub fn internal_error() -> Self {
        Error::new(ErrorCode::INTERNAL_ERROR)
    }

    pub fn into_internal_error(err: impl std::error::Error) -> Self {
        Error::internal_error().with_data(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorCode {
    code: i32,
    message: &'static str,
}

impl ErrorCode {
    pub const PARSE_ERROR: ErrorCode = ErrorCode {
        code: -32700,
        message: "Parse error",
    };

    pub const INVALID_REQUEST: ErrorCode = ErrorCode {
        code: -32600,
        message: "Invalid Request",
    };

    pub const METHOD_NOT_FOUND: ErrorCode = ErrorCode {
        code: -32601,
        message: "Method not found",
    };

    pub const INVALID_PARAMS: ErrorCode = ErrorCode {
        code: -32602,
        message: "Invalid params",
    };

    pub const INTERNAL_ERROR: ErrorCode = ErrorCode {
        code: -32603,
        message: "Internal error",
    };
}

impl From<ErrorCode> for (i32, String) {
    fn from(error_code: ErrorCode) -> Self {
        (error_code.code, error_code.message.to_string())
    }
}

impl From<ErrorCode> for Error {
    fn from(error_code: ErrorCode) -> Self {
        Error::new(error_code)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.message.is_empty() {
            write!(f, "{}", self.code)?;
        } else {
            write!(f, "{}", self.message)?;
        }

        if let Some(data) = &self.data {
            write!(f, ": {data}")?;
        }

        Ok(())
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::into_internal_error(error.deref())
    }
}
