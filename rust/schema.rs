use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::Error;

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Method {
    pub name: &'static str,
    pub request_type: &'static str,
    pub param_payload: bool,
    pub response_type: &'static str,
    pub response_payload: bool,
    pub error_type: &'static str,
}

pub trait AnyRequest: Serialize + Sized + 'static {
    type Response: Serialize + 'static;
    type Error: std::error::Error + std::fmt::Display + From<Error>;
    fn from_method_and_params(method: &str, params: &RawValue) -> Result<Self, Self::Error>;
    fn response_from_method_and_result(
        method: &str,
        params: &RawValue,
    ) -> Result<Self::Response, Self::Error>;
}

macro_rules! acp_peer {
    (
        $handler_trait_name:ident,
        $request_trait_name:ident,
        $request_enum_name:ident,
        $response_enum_name:ident,
        $error_name:ident,
        $method_map_name:ident,
        $(($request_method:ident, $request_method_string:expr, $request_name:ident, $param_payload: tt, $response_name:ident, $response_payload: tt, $request_error_name:ident)),*
        $(,)?
    ) => {
        #[non_exhaustive]
        #[derive(Clone, Debug, thiserror::Error, Serialize, JsonSchema)]
        #[serde(untagged)]
        pub enum $error_name {
            $(
                #[error(transparent)]
                $request_error_name($request_error_name),
            )*
            #[error(transparent)]
            Other(Error),
        }

        impl From<Error> for $error_name {
            fn from(err: Error) -> Self {
                $(
                   if let Ok(err) = $request_error_name::try_from(&err) {
                       return Self::$request_error_name(err);
                   }
                )*
                Self::Other(err)
            }
        }

        $(
            impl From<$request_error_name> for $error_name {
                fn from(err: $request_error_name) -> Self {
                    Self::$request_error_name(err)
                }
            }
        )*

        impl From<$error_name> for Error {
            fn from(err: $error_name) -> Self {
                match err {
                    $(
                        $error_name::$request_error_name(err) => err.into(),
                    )*
                    $error_name::Other(err) => err,
                }
            }
        }

        macro_rules! handler_trait_call_req {
            ($self: ident, $method: ident, false, $resp_name: ident, false, $params: ident) => {
                {
                    $self.$method().await?;
                    Ok($response_enum_name::$resp_name($resp_name))
                }
            };
            ($self: ident, $method: ident, false, $resp_name: ident, true, $params: ident) => {
                {
                    let resp = $self.$method().await?;
                    Ok($response_enum_name::$resp_name(resp))
                }
            };
            ($self: ident, $method: ident, true, $resp_name: ident, false, $params: ident) => {
                {
                    $self.$method($params).await?;
                    Ok($response_enum_name::$resp_name($resp_name))
                }
            };
            ($self: ident, $method: ident, true, $resp_name: ident, true, $params: ident) => {
                {
                    let resp = $self.$method($params).await?;
                    Ok($response_enum_name::$resp_name(resp))
                }
            }
        }

        macro_rules! handler_trait_req_method {
            ($method: ident, $req: ident, false, $resp: tt, false) => {
                fn $method(&self) -> impl Future<Output = Result<(), <$req as $request_trait_name>::Error>>;
            };
            ($method: ident, $req: ident, false, $resp: tt, true) => {
                fn $method(&self) -> impl Future<Output = Result<$resp, <$req as $request_trait_name>::Error>>;
            };
            ($method: ident, $req: ident, true, $resp: tt, false) => {
                fn $method(&self, request: $req) -> impl Future<Output = Result<(), <$req as $request_trait_name>::Error>>;
            };
            ($method: ident, $req: ident, true, $resp: tt, true) => {
                fn $method(&self, request: $req) -> impl Future<Output = Result<$resp, <$req as $request_trait_name>::Error>>;
            }
        }

        pub trait $handler_trait_name {
            fn call(&self, params: $request_enum_name) -> impl Future<Output = Result<$response_enum_name, $error_name>> {
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
            type Error: std::error::Error + Into<Error> + for<'a> TryFrom<&'a Error>;
            fn into_any(self) -> $request_enum_name;
            fn response_from_any(any: $response_enum_name) -> Result<Self::Response, Self::Error>;
            fn error_from_any(any: $error_name) -> Self::Error;
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
                    Err(e) => Err(Self::Error::Other(Error::parse_error().with_data(e.to_string()))),
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
                    Err(e) => Err(Self::Error::Other(Error::parse_error().with_data(e.to_string()))),
                }
            };
        }

        impl AnyRequest for $request_enum_name {
            type Response = $response_enum_name;
            type Error = $error_name;

            fn from_method_and_params(method: &str, params: &RawValue) -> Result<Self, Self::Error> {
                match method {
                    $(
                        $request_method_string => {
                            request_from_method_and_params!($request_name, $param_payload, params)
                        }
                    )*
                    _ => Err(Self::Error::Other(Error::method_not_found())),
                }
            }

            fn response_from_method_and_result(method: &str, params: &RawValue) -> Result<Self::Response, Self::Error> {
                match method {
                    $(
                        $request_method_string => {
                            response_from_method_and_result!($response_name, $response_payload, params)
                        }
                    )*
                    _ => Err(Self::Error::Other(Error::method_not_found())),
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
                    error_type: stringify!($request_error_name),
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
            ($any: ident, $resp_name: ident, false, $error: ident) => {
                match $any {
                    $response_enum_name::$resp_name(_) => Ok(()),
                    _ => Err($error::Other(Error::internal_error().with_data("Unexpected Response")))
                }
            };
            ($any: ident, $resp_name: ident, true, $error: ident) => {
                match $any {
                    $response_enum_name::$resp_name(this) => Ok(this),
                    _ => Err($error::Other(Error::internal_error().with_data("Unexpected Response")))
                }
            };
        }

        $(
            impl $request_trait_name for $request_name {
                type Response = resp_type!($response_name, $response_payload);
                type Error = $request_error_name;

                fn into_any(self) -> $request_enum_name {
                    req_into_any!(self, $request_name, $param_payload)
                }

                fn response_from_any(any: $response_enum_name) -> Result<Self::Response, Self::Error> {
                    resp_from_any!(any, $response_name, $response_payload, $request_error_name)
                }

                fn error_from_any(any: $error_name) -> Self::Error {
                    match any {
                        $error_name::$request_error_name(err) => err,
                        any => $request_error_name::Other(Error::from(any)),
                    }
                }
            }
        )*
    };
}

macro_rules! request_error {
    ($error_name:ident $(,)? $(($variant:ident, $code:literal, $message:literal)),* $(,)?) => {
        paste::paste! {
            #[non_exhaustive]
            #[derive(Clone, Debug, thiserror::Error, Serialize, JsonSchema)]
            #[serde(into = "Error")]
            #[schemars(!into, tag = "message")]
            pub enum $error_name {
                $(
                    #[schemars(with = "Error", rename = $message, transform = Self::[<transform_code_ $code>])]
                        #[error($message)]
                        $variant,
                )*
                #[schemars(with = "Error", untagged)]
                #[error(transparent)]
                Other(Error),
            }
        }

        impl $error_name {
            pub fn other(err: impl std::error::Error) -> Self {
                Self::Other(Error::internal_error().with_data(err.to_string()))
            }

            $(
                paste::paste! {
                    fn [<transform_code_ $code>](schema: &mut schemars::Schema) {
                        let Some(schema) = schema.pointer_mut("/properties/code") else {
                            return;
                        };
                        schema["const"] = serde_json::json!($code);
                    }
                }
            )*
        }

        impl<E> From<E> for $error_name where E: AsRef<dyn std::error::Error> {
            fn from(err: E) -> Self {
                Self::other(err.as_ref())
            }
        }

        impl From<$error_name> for Error {
            fn from(err: $error_name) -> Self {
                match err {
                    $($error_name::$variant => Self::new($code, $message),)*
                    $error_name::Other(err) => err,
                }
            }
        }

        impl TryFrom<&Error> for $error_name {
            type Error = ();

            fn try_from(value: &Error) -> Result<Self, Self::Error> {
                match value.code {
                    $($code => {
                        Ok(Self::$variant)
                    })*
                    _ => Err(()),
                }
            }
        }
    };
}

// requests sent from the client (the IDE) to the agent
acp_peer!(
    Client,
    ClientRequest,
    AnyClientRequest,
    AnyClientResult,
    AnyClientError,
    CLIENT_METHODS,
    (
        stream_assistant_message_chunk,
        "streamAssistantMessageChunk",
        StreamAssistantMessageChunkParams,
        true,
        StreamAssistantMessageChunkResponse,
        false,
        StreamAssistantMessageChunkError
    ),
    (
        request_tool_call_confirmation,
        "requestToolCallConfirmation",
        RequestToolCallConfirmationParams,
        true,
        RequestToolCallConfirmationResponse,
        true,
        RequestToolCallConfirmationError
    ),
    (
        push_tool_call,
        "pushToolCall",
        PushToolCallParams,
        true,
        PushToolCallResponse,
        true,
        PushToolCallError
    ),
    (
        update_tool_call,
        "updateToolCall",
        UpdateToolCallParams,
        true,
        UpdateToolCallResponse,
        false,
        UpdateToolCallError
    ),
);

// requests sent from the agent to the client (the IDE)
acp_peer!(
    Agent,
    AgentRequest,
    AnyAgentRequest,
    AnyAgentResult,
    AnyAgentError,
    AGENT_METHODS,
    (
        initialize,
        "initialize",
        InitializeParams,
        false,
        InitializeResponse,
        true,
        InitializeError
    ),
    (
        authenticate,
        "authenticate",
        AuthenticateParams,
        false,
        AuthenticateResponse,
        false,
        AuthenticateError
    ),
    (
        send_user_message,
        "sendUserMessage",
        SendUserMessageParams,
        true,
        SendUserMessageResponse,
        false,
        SendUserMessageError
    ),
    (
        cancel_send_message,
        "cancelSendMessage",
        CancelSendMessageParams,
        false,
        CancelSendMessageResponse,
        false,
        CancelSendMessageError
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

request_error!(InitializeError);

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

request_error!(AuthenticateError);

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

request_error!(
    SendUserMessageError,
    (RateLimitExceeded, 429, "Rate limit exceeded"),
);

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

request_error!(CancelSendMessageError);

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

request_error!(StreamAssistantMessageChunkError);

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
    pub label: String,
    pub icon: Icon,
    pub confirmation: ToolCallConfirmation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolCallContent>,
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

request_error!(RequestToolCallConfirmationError);

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
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct PushToolCallResponse {
    pub id: ToolCallId,
}

request_error!(PushToolCallError);

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

request_error!(
    UpdateToolCallError,
    (
        WaitingForConfirmation,
        1000,
        "Tool call waiting for confirmation"
    ),
    (Rejected, 1001, "Tool call was rejected by the user")
);

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
