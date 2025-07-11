use std::{fmt::Display, ops::Deref, path::PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Error {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Error {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Error {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: impl Into<serde_json::Value>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
    pub fn parse_error() -> Self {
        Error::new(-32700, "Parse error")
    }

    /// The JSON sent is not a valid Request object.
    pub fn invalid_request() -> Self {
        Error::new(-32600, "Invalid Request")
    }

    /// The method does not exist / is not available.
    pub fn method_not_found() -> Self {
        Error::new(-32601, "Method not found")
    }

    /// Invalid method parameter(s).
    pub fn invalid_params() -> Self {
        Error::new(-32602, "Invalid params")
    }

    /// Internal JSON-RPC error.
    pub fn internal_error() -> Self {
        Error::new(-32603, "Internal error")
    }

    pub fn into_internal_error(err: impl std::error::Error) -> Self {
        Error::internal_error().with_data(err.to_string())
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

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Method {
    pub name: &'static str,
    pub request_type: &'static str,
    pub param_payload: bool,
    pub response_type: &'static str,
    pub response_payload: bool,
}

pub trait AnyRequest: Serialize + Sized + 'static {
    type Response: Serialize + 'static;
    fn from_method_and_params(method: &str, params: &RawValue) -> Result<Self, Error>;
    fn response_from_method_and_result(
        method: &str,
        params: &RawValue,
    ) -> Result<Self::Response, Error>;
}

macro_rules! acp_peer {
    (
        $handler_trait_name:ident,
        $request_trait_name:ident,
        $request_enum_name:ident,
        $response_enum_name:ident,
        $method_map_name:ident,
        $(($request_method:ident, $request_method_string:expr, $request_name:ident, $param_payload: tt, $response_name:ident, $response_payload: tt)),*
        $(,)?
    ) => {
        macro_rules! handler_trait_call_req {
            ($self: ident, $method: ident, false, $resp_name: ident, false, $params: ident) => {
                {
                    $self.$method()
                        .await
                        .map_err(|e| Error::internal_error().with_data(e.to_string()))?;
                    Ok($response_enum_name::$resp_name($resp_name))
                }
            };
            ($self: ident, $method: ident, false, $resp_name: ident, true, $params: ident) => {
                {
                    let resp = $self.$method()
                        .await
                        .map_err(|e| Error::internal_error().with_data(e.to_string()))?;
                    Ok($response_enum_name::$resp_name(resp))
                }
            };
            ($self: ident, $method: ident, true, $resp_name: ident, false, $params: ident) => {
                {
                    $self.$method($params)
                        .await
                        .map_err(|e| Error::internal_error().with_data(e.to_string()))?;
                    Ok($response_enum_name::$resp_name($resp_name))
                }
            };
            ($self: ident, $method: ident, true, $resp_name: ident, true, $params: ident) => {
                {
                    let resp = $self.$method($params)
                        .await
                        .map_err(|e| Error::internal_error().with_data(e.to_string()))?;
                    Ok($response_enum_name::$resp_name(resp))
                }
            }
        }

        macro_rules! handler_trait_req_method {
            ($method: ident, $req: ident, false, $resp: tt, false) => {
                fn $method(&self) -> impl Future<Output = Result<(), Error>>;
            };
            ($method: ident, $req: ident, false, $resp: tt, true) => {
                fn $method(&self) -> impl Future<Output = Result<$resp, Error>>;
            };
            ($method: ident, $req: ident, true, $resp: tt, false) => {
                fn $method(&self, request: $req) -> impl Future<Output = Result<(), Error>>;
            };
            ($method: ident, $req: ident, true, $resp: tt, true) => {
                fn $method(&self, request: $req) -> impl Future<Output = Result<$resp, Error>>;
            }
        }

        pub trait $handler_trait_name {
            fn call(&self, params: $request_enum_name) -> impl Future<Output = Result<$response_enum_name, Error>> {
                async move {
                    match params {
                        $(#[allow(unused_variables)]
                        $request_enum_name::$request_name(params) => {
                            handler_trait_call_req!(self, $request_method, $param_payload, $response_name, $response_payload, params)
                        }),*
                    }
                }
            }

            $(
                handler_trait_req_method!($request_method, $request_name, $param_payload, $response_name, $response_payload);
            )*
        }

        pub trait $request_trait_name {
            type Response;
            fn into_any(self) -> $request_enum_name;
            fn response_from_any(any: $response_enum_name) -> Result<Self::Response, Error>;
        }

        #[derive(Serialize, JsonSchema)]
        #[serde(untagged)]
        pub enum $request_enum_name {
            $(
                $request_name($request_name),
            )*
        }

        #[derive(Serialize, Deserialize, JsonSchema)]
        #[serde(untagged)]
        pub enum $response_enum_name {
            $(
                $response_name($response_name),
            )*
        }

        macro_rules! request_from_method_and_params {
            ($req_name: ident, false, $params: tt) => {
                Ok($request_enum_name::$req_name($req_name))
            };
            ($req_name: ident, true, $params: tt) => {
                match serde_json::from_str($params.get()) {
                    Ok(params) => Ok($request_enum_name::$req_name(params)),
                    Err(e) => Err(Error::parse_error().with_data(e.to_string())),
                }
            };
        }

        macro_rules! response_from_method_and_result {
            ($resp_name: ident, false, $result: tt) => {
                Ok($response_enum_name::$resp_name($resp_name))
            };
            ($resp_name: ident, true, $result: tt) => {
                match serde_json::from_str($result.get()) {
                    Ok(result) => Ok($response_enum_name::$resp_name(result)),
                    Err(e) => Err(Error::parse_error().with_data(e.to_string())),
                }
            };
        }

        impl AnyRequest for $request_enum_name {
            type Response = $response_enum_name;

            fn from_method_and_params(method: &str, params: &RawValue) -> Result<Self, Error> {
                match method {
                    $(
                        $request_method_string => {
                            request_from_method_and_params!($request_name, $param_payload, params)
                        }
                    )*
                    _ => Err(Error::method_not_found()),
                }
            }

            fn response_from_method_and_result(method: &str, params: &RawValue) -> Result<Self::Response, Error> {
                match method {
                    $(
                        $request_method_string => {
                            response_from_method_and_result!($response_name, $response_payload, params)
                        }
                    )*
                    _ => Err(Error::method_not_found()),
                }
            }
        }

        impl $request_enum_name {
            pub fn method_name(&self) -> &'static str {
                match self {
                    $(
                        $request_enum_name::$request_name(_) => $request_method_string,
                    )*
                }
            }
        }



        pub static $method_map_name: &[Method] = &[
            $(
                Method {
                    name: $request_method_string,
                    request_type: stringify!($request_name),
                    param_payload: $param_payload,
                    response_type: stringify!($response_name),
                    response_payload: $response_payload,
                },
            )*
        ];

        macro_rules! req_into_any {
            ($self: ident, $req_name: ident, false) => {
                $request_enum_name::$req_name($req_name)
            };
            ($self: ident, $req_name: ident, true) => {
                $request_enum_name::$req_name($self)
            };
        }

        macro_rules! resp_type {
            ($resp_name: ident, false) => {
                ()
            };
            ($resp_name: ident, true) => {
                $resp_name
            };
        }

        macro_rules! resp_from_any {
            ($any: ident, $resp_name: ident, false) => {
                match $any {
                    $response_enum_name::$resp_name(_) => Ok(()),
                    _ => Err(Error::internal_error().with_data("Unexpected Response"))
                }
            };
            ($any: ident, $resp_name: ident, true) => {
                match $any {
                    $response_enum_name::$resp_name(this) => Ok(this),
                    _ => Err(Error::internal_error().with_data("Unexpected Response"))
                }
            };
        }

        $(
            impl $request_trait_name for $request_name {
                type Response = resp_type!($response_name, $response_payload);

                fn into_any(self) -> $request_enum_name {
                    req_into_any!(self, $request_name, $param_payload)
                }

                fn response_from_any(any: $response_enum_name) -> Result<Self::Response, Error> {
                    resp_from_any!(any, $response_name, $response_payload)
                }
            }
        )*
    };
}

// requests sent from the client (the IDE) to the agent
acp_peer!(
    Client,
    ClientRequest,
    AnyClientRequest,
    AnyClientResult,
    CLIENT_METHODS,
    (
        stream_assistant_message_chunk,
        "streamAssistantMessageChunk",
        StreamAssistantMessageChunkParams,
        true,
        StreamAssistantMessageChunkResponse,
        false
    ),
    (
        request_tool_call_confirmation,
        "requestToolCallConfirmation",
        RequestToolCallConfirmationParams,
        true,
        RequestToolCallConfirmationResponse,
        true
    ),
    (
        push_tool_call,
        "pushToolCall",
        PushToolCallParams,
        true,
        PushToolCallResponse,
        true
    ),
    (
        update_tool_call,
        "updateToolCall",
        UpdateToolCallParams,
        true,
        UpdateToolCallResponse,
        false
    ),
    (
        write_text_file,
        "writeTextFile",
        WriteTextFileParams,
        true,
        WriteTextFileResponse,
        false
    ),
    (
        read_text_file,
        "readTextFile",
        ReadTextFileParams,
        true,
        ReadTextFileResponse,
        true
    )
);

// requests sent from the agent to the client (the IDE)
acp_peer!(
    Agent,
    AgentRequest,
    AnyAgentRequest,
    AnyAgentResult,
    AGENT_METHODS,
    (
        initialize,
        "initialize",
        InitializeParams,
        false,
        InitializeResponse,
        true
    ),
    (
        authenticate,
        "authenticate",
        AuthenticateParams,
        false,
        AuthenticateResponse,
        false
    ),
    (
        send_user_message,
        "sendUserMessage",
        SendUserMessageParams,
        true,
        SendUserMessageResponse,
        false
    ),
    (
        cancel_send_message,
        "cancelSendMessage",
        CancelSendMessageParams,
        false,
        CancelSendMessageResponse,
        false
    )
);

// --- Messages sent from the client to the agent --- \\

/// Initialize sets up the agent's state. It should be called before any other method,
/// and no other methods should be called until it has completed.
///
/// If the agent is not authenticated, then the client should prompt the user to authenticate,
/// and then call the `authenticate` method.
/// Otherwise the client can send other messages to the agent.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    /// Indicates whether the agent is authenticated and
    /// ready to handle requests.
    pub is_authenticated: bool,
}

/// Triggers authentication on the agent side.
///
/// This method should only be called if the initialize response indicates the user isn't already authenticated.
/// If this succceeds then the client can send other messasges to the agent,
/// If it fails then the error message should be shown and the user prompted to authenticate.
///
/// The implementation of authentication is left up to the agent, typically an oauth
/// flow is run by opening a browser window in the background.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateResponse;

/// sendUserMessage allows the user to send a message to the agent.
/// This method should complete after the agent is finished, during
/// which time the agent may update the client by calling
/// streamAssistantMessageChunk and other methods.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendUserMessageParams {
    pub chunks: Vec<UserMessageChunk>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendUserMessageResponse;

/// A part in a user message
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged, rename_all = "camelCase")]
pub enum UserMessageChunk {
    /// A chunk of text in user message
    Text { text: String },
    /// A file path mention in a user message
    Path { path: PathBuf },
}

/// cancelSendMessage allows the client to request that the agent
/// stop running. The agent should resolve or reject the current sendUserMessage call.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CancelSendMessageParams;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CancelSendMessageResponse;

// --- Messages sent from the agent to the client --- \\

/// Streams part of an assistant response to the client
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamAssistantMessageChunkParams {
    pub chunk: AssistantMessageChunk,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamAssistantMessageChunkResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged, rename_all = "camelCase")]
pub enum AssistantMessageChunk {
    Text { text: String },
    Thought { thought: String },
}

/// Request confirmation before running a tool
///
/// When allowed, the client returns a [`ToolCallId`] which can be used
/// to update the tool call's `status` and `content` as it runs.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RequestToolCallConfirmationParams {
    #[serde(flatten)]
    pub tool_call: PushToolCallParams,
    pub confirmation: ToolCallConfirmation,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
// todo? make this `pub enum ToolKind { Edit, Search, Read, Fetch, ...}?`
// avoids being to UI centric.
pub enum Icon {
    FileSearch,
    Folder,
    Globe,
    Hammer,
    LightBulb,
    Pencil,
    Regex,
    Terminal,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolCallConfirmation {
    #[serde(rename_all = "camelCase")]
    Edit {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Execute {
        command: String,
        root_command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Mcp {
        server_name: String,
        tool_name: String,
        tool_display_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Fetch {
        urls: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Other { description: String },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct RequestToolCallConfirmationResponse {
    pub id: ToolCallId,
    pub outcome: ToolCallConfirmationOutcome,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ToolCallConfirmationOutcome {
    /// Allow this one call
    Allow,
    /// Always allow this kind of operation
    AlwaysAllow,
    /// Always allow any tool from this MCP server
    AlwaysAllowMcpServer,
    /// Always allow this tool from this MCP server
    AlwaysAllowTool,
    /// Reject this tool call
    Reject,
    /// The generation was canceled before a confirming
    Cancel,
}

/// pushToolCall allows the agent to start a tool call
/// when it does not need to request permission to do so.
///
/// The returned id can be used to update the UI for the tool
/// call as needed.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PushToolCallParams {
    pub label: String,
    pub icon: Icon,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolCallContent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<ToolCallLocation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ToolCallLocation {
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct PushToolCallResponse {
    pub id: ToolCallId,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallId(pub u64);

/// updateToolCall allows the agent to update the content and status of the tool call.
///
/// The new content replaces what is currently displayed in the UI.
///
/// The [`ToolCallId`] is included in the response of
/// `pushToolCall` or `requestToolCallConfirmation` respectively.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateToolCallParams {
    pub tool_call_id: ToolCallId,
    pub status: ToolCallStatus,
    pub content: Option<ToolCallContent>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateToolCallResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ToolCallStatus {
    /// The tool call is currently running
    Running,
    /// The tool call completed successfully
    Finished,
    /// The tool call failed
    Error,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolCallContent {
    #[serde(rename_all = "camelCase")]
    Markdown { markdown: String },
    #[serde(rename_all = "camelCase")]
    Diff {
        #[serde(flatten)]
        diff: Diff,
    },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Diff {
    pub path: PathBuf,
    pub old_text: Option<String>,
    pub new_text: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WriteTextFileParams {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WriteTextFileResponse;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadTextFileParams {
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadTextFileResponse {
    pub content: String,
}
