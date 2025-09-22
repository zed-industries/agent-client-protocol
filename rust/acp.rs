//! [![Agent Client Protocol](https://zed.dev/img/acp/banner-dark.webp)](https://agentclientprotocol.com/)
//!
//! # Agent Client Protocol (ACP)
//!
//! The Agent Client Protocol standardizes communication between code editors
//! (IDEs, text-editors, etc.) and coding agents (programs that use generative AI
//! to autonomously modify code).
//!
//! ## Protocol & Transport
//!
//! ACP is a JSON-RPC based protocol. While clients typically start agents as
//! subprocesses and communicate over stdio (stdin/stdout), this crate is
//! transport-agnostic.
//!
//! You can use any bidirectional stream that implements `AsyncRead` and `AsyncWrite`.
//!
//! ## Core Components
//!
//! - **Agent**: Programs that use generative AI to autonomously modify code
//!   - See: [Agent](https://agentclientprotocol.com/protocol/overview#agent)
//! - **Client**: Code editors that provide the interface between users and agents
//!   - See: [Client](https://agentclientprotocol.com/protocol/overview#client)
//!
//! ## Getting Started
//!
//! To understand the protocol, start by exploring the [`Agent`] and [`Client`] traits,
//! which define the core methods and capabilities of each side of the connection.
//!
//! To see working examples of these traits in action, check out the
//! [agent](https://github.com/zed-industries/agent-client-protocol/blob/main/rust/examples/agent.rs)
//! and
//! [client](https://github.com/zed-industries/agent-client-protocol/blob/main/rust/examples/client.rs)
//! example binaries included with this crate.
//!
//! ### Implementation Pattern
//!
//! ACP uses a symmetric design where each participant implements one trait and
//! creates a connection that provides the complementary trait:
//!
//! - **Agent builders** implement the [`Agent`] trait to handle client requests
//!   (like initialization, authentication, and prompts). They pass this implementation
//!   to [`AgentSideConnection::new`], which returns a connection providing [`Client`]
//!   methods for requesting permissions and accessing the file system.
//!
//! - **Client builders** implement the [`Client`] trait to handle agent requests
//!   (like file system operations and permission checks). They pass this implementation
//!   to [`ClientSideConnection::new`], which returns a connection providing [`Agent`]
//!   methods for managing sessions and sending prompts.
//!
//! For the complete protocol specification and documentation, visit:
//! [https://agentclientprotocol.com](https://agentclientprotocol.com)

mod agent;
mod client;
mod content;
mod error;
mod ext;
mod plan;
mod rpc;
#[cfg(test)]
mod rpc_tests;
mod stream_broadcast;
mod tool_call;
mod version;

pub use agent::*;
pub use client::*;
pub use content::*;
pub use error::*;
pub use ext::*;
pub use plan::*;
pub use serde_json::value::RawValue;
pub use stream_broadcast::{
    StreamMessage, StreamMessageContent, StreamMessageDirection, StreamReceiver,
};
pub use tool_call::*;
pub use version::*;

use anyhow::Result;
use futures::{AsyncRead, AsyncWrite, Future, future::LocalBoxFuture};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

use crate::rpc::{MessageHandler, RpcConnection, Side};

/// A unique identifier for a conversation session between a client and agent.
///
/// Sessions maintain their own context, conversation history, and state,
/// allowing multiple independent interactions with the same agent.
///
/// # Example
///
/// ```
/// use agent_client_protocol::SessionId;
/// use std::sync::Arc;
///
/// let session_id = SessionId(Arc::from("sess_abc123def456"));
/// ```
///
/// See protocol docs: [Session ID](https://agentclientprotocol.com/protocol/session-setup#session-id)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SessionId(pub Arc<str>);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Client to Agent

/// A client-side connection to an agent.
///
/// This struct provides the client's view of an ACP connection, allowing
/// clients (such as code editors) to communicate with agents. It implements
/// the [`Agent`] trait to provide methods for initializing sessions, sending
/// prompts, and managing the agent lifecycle.
///
/// See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
pub struct ClientSideConnection {
    conn: RpcConnection<ClientSide, AgentSide>,
}

impl ClientSideConnection {
    /// Creates a new client-side connection to an agent.
    ///
    /// This establishes the communication channel between a client and agent
    /// following the ACP specification.
    ///
    /// # Arguments
    ///
    /// * `client` - A handler that implements the [`Client`] trait to process incoming agent requests
    /// * `outgoing_bytes` - The stream for sending data to the agent (typically stdout)
    /// * `incoming_bytes` - The stream for receiving data from the agent (typically stdin)
    /// * `spawn` - A function to spawn async tasks (e.g., `tokio::spawn`)
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The connection instance for making requests to the agent
    /// - An I/O future that must be spawned to handle the underlying communication
    ///
    /// See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)
    pub fn new(
        client: impl MessageHandler<ClientSide> + 'static,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let (conn, io_task) = RpcConnection::new(client, outgoing_bytes, incoming_bytes, spawn);
        (Self { conn }, io_task)
    }

    /// Subscribe to receive stream updates from the agent.
    ///
    /// This allows the client to receive real-time notifications about
    /// agent activities, such as tool calls, content updates, and progress reports.
    ///
    /// # Returns
    ///
    /// A [`StreamReceiver`] that can be used to receive stream messages.
    pub fn subscribe(&self) -> StreamReceiver {
        self.conn.subscribe()
    }
}

#[async_trait::async_trait(?Send)]
impl Agent for ClientSideConnection {
    async fn initialize(&self, args: InitializeRequest) -> Result<InitializeResponse, Error> {
        self.conn
            .request(
                INITIALIZE_METHOD_NAME,
                Some(ClientRequest::InitializeRequest(args)),
            )
            .await
    }

    async fn authenticate(&self, args: AuthenticateRequest) -> Result<AuthenticateResponse, Error> {
        self.conn
            .request::<Option<_>>(
                AUTHENTICATE_METHOD_NAME,
                Some(ClientRequest::AuthenticateRequest(args)),
            )
            .await
            .map(Option::unwrap_or_default)
    }

    async fn new_session(&self, args: NewSessionRequest) -> Result<NewSessionResponse, Error> {
        self.conn
            .request(
                SESSION_NEW_METHOD_NAME,
                Some(ClientRequest::NewSessionRequest(args)),
            )
            .await
    }

    async fn load_session(&self, args: LoadSessionRequest) -> Result<LoadSessionResponse, Error> {
        self.conn
            .request::<Option<_>>(
                SESSION_LOAD_METHOD_NAME,
                Some(ClientRequest::LoadSessionRequest(args)),
            )
            .await
            .map(Option::unwrap_or_default)
    }

    async fn set_session_mode(
        &self,
        args: SetSessionModeRequest,
    ) -> Result<SetSessionModeResponse, Error> {
        self.conn
            .request(
                SESSION_SET_MODE_METHOD_NAME,
                Some(ClientRequest::SetSessionModeRequest(args)),
            )
            .await
    }

    async fn prompt(&self, args: PromptRequest) -> Result<PromptResponse, Error> {
        self.conn
            .request(
                SESSION_PROMPT_METHOD_NAME,
                Some(ClientRequest::PromptRequest(args)),
            )
            .await
    }

    async fn cancel(&self, args: CancelNotification) -> Result<(), Error> {
        self.conn.notify(
            SESSION_CANCEL_METHOD_NAME,
            Some(ClientNotification::CancelNotification(args)),
        )
    }

    #[cfg(feature = "unstable")]
    async fn set_session_model(
        &self,
        args: SetSessionModelRequest,
    ) -> Result<SetSessionModelResponse, Error> {
        self.conn
            .request(
                SESSION_SET_MODEL_METHOD_NAME,
                Some(ClientRequest::SetSessionModelRequest(args)),
            )
            .await
    }

    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, Error> {
        self.conn
            .request(
                format!("_{}", args.method),
                Some(ClientRequest::ExtMethodRequest(args)),
            )
            .await
    }

    async fn ext_notification(&self, args: ExtNotification) -> Result<(), Error> {
        self.conn.notify(
            format!("_{}", args.method),
            Some(ClientNotification::ExtNotification(args)),
        )
    }
}

/// Marker type representing the client side of an ACP connection.
///
/// This type is used by the RPC layer to determine which messages
/// are incoming vs outgoing from the client's perspective.
///
/// See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)
#[derive(Clone)]
pub struct ClientSide;

impl Side for ClientSide {
    type InNotification = AgentNotification;
    type InRequest = AgentRequest;
    type OutResponse = ClientResponse;

    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<AgentRequest, Error> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            SESSION_REQUEST_PERMISSION_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::RequestPermissionRequest)
                .map_err(Into::into),
            FS_WRITE_TEXT_FILE_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::WriteTextFileRequest)
                .map_err(Into::into),
            FS_READ_TEXT_FILE_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::ReadTextFileRequest)
                .map_err(Into::into),
            TERMINAL_CREATE_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::CreateTerminalRequest)
                .map_err(Into::into),
            TERMINAL_OUTPUT_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::TerminalOutputRequest)
                .map_err(Into::into),
            TERMINAL_KILL_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::KillTerminalCommandRequest)
                .map_err(Into::into),
            TERMINAL_RELEASE_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::ReleaseTerminalRequest)
                .map_err(Into::into),
            TERMINAL_WAIT_FOR_EXIT_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::WaitForTerminalExitRequest)
                .map_err(Into::into),
            _ => {
                if let Some(custom_method) = method.strip_prefix('_') {
                    Ok(AgentRequest::ExtMethodRequest(ExtRequest {
                        method: custom_method.into(),
                        params: RawValue::from_string(params.get().to_string())?.into(),
                    }))
                } else {
                    Err(Error::method_not_found())
                }
            }
        }
    }

    fn decode_notification(
        method: &str,
        params: Option<&RawValue>,
    ) -> Result<AgentNotification, Error> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            SESSION_UPDATE_NOTIFICATION => serde_json::from_str(params.get())
                .map(AgentNotification::SessionNotification)
                .map_err(Into::into),
            _ => {
                if let Some(custom_method) = method.strip_prefix('_') {
                    Ok(AgentNotification::ExtNotification(ExtNotification {
                        method: custom_method.into(),
                        params: RawValue::from_string(params.get().to_string())?.into(),
                    }))
                } else {
                    Err(Error::method_not_found())
                }
            }
        }
    }
}

impl<T: Client> MessageHandler<ClientSide> for T {
    async fn handle_request(&self, request: AgentRequest) -> Result<ClientResponse, Error> {
        match request {
            AgentRequest::RequestPermissionRequest(args) => {
                let response = self.request_permission(args).await?;
                Ok(ClientResponse::RequestPermissionResponse(response))
            }
            AgentRequest::WriteTextFileRequest(args) => {
                let response = self.write_text_file(args).await?;
                Ok(ClientResponse::WriteTextFileResponse(response))
            }
            AgentRequest::ReadTextFileRequest(args) => {
                let response = self.read_text_file(args).await?;
                Ok(ClientResponse::ReadTextFileResponse(response))
            }
            AgentRequest::CreateTerminalRequest(args) => {
                let response = self.create_terminal(args).await?;
                Ok(ClientResponse::CreateTerminalResponse(response))
            }
            AgentRequest::TerminalOutputRequest(args) => {
                let response = self.terminal_output(args).await?;
                Ok(ClientResponse::TerminalOutputResponse(response))
            }
            AgentRequest::ReleaseTerminalRequest(args) => {
                let response = self.release_terminal(args).await?;
                Ok(ClientResponse::ReleaseTerminalResponse(response))
            }
            AgentRequest::WaitForTerminalExitRequest(args) => {
                let response = self.wait_for_terminal_exit(args).await?;
                Ok(ClientResponse::WaitForTerminalExitResponse(response))
            }
            AgentRequest::KillTerminalCommandRequest(args) => {
                let response = self.kill_terminal_command(args).await?;
                Ok(ClientResponse::KillTerminalResponse(response))
            }
            AgentRequest::ExtMethodRequest(args) => {
                let response = self.ext_method(args).await?;
                Ok(ClientResponse::ExtMethodResponse(response))
            }
        }
    }

    async fn handle_notification(&self, notification: AgentNotification) -> Result<(), Error> {
        match notification {
            AgentNotification::SessionNotification(args) => {
                self.session_notification(args).await?;
            }
            AgentNotification::ExtNotification(args) => {
                self.ext_notification(args).await?;
            }
        }
        Ok(())
    }
}

// Agent to Client

/// An agent-side connection to a client.
///
/// This struct provides the agent's view of an ACP connection, allowing
/// agents to communicate with clients. It implements the [`Client`] trait
/// to provide methods for requesting permissions, accessing the file system,
/// and sending session updates.
///
/// See protocol docs: [Agent](https://agentclientprotocol.com/protocol/overview#agent)
pub struct AgentSideConnection {
    conn: RpcConnection<AgentSide, ClientSide>,
}

impl AgentSideConnection {
    /// Creates a new agent-side connection to a client.
    ///
    /// This establishes the communication channel from the agent's perspective
    /// following the ACP specification.
    ///
    /// # Arguments
    ///
    /// * `agent` - A handler that implements the [`Agent`] trait to process incoming client requests
    /// * `outgoing_bytes` - The stream for sending data to the client (typically stdout)
    /// * `incoming_bytes` - The stream for receiving data from the client (typically stdin)
    /// * `spawn` - A function to spawn async tasks (e.g., `tokio::spawn`)
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The connection instance for making requests to the client
    /// - An I/O future that must be spawned to handle the underlying communication
    ///
    /// See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)
    pub fn new(
        agent: impl MessageHandler<AgentSide> + 'static,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let (conn, io_task) = RpcConnection::new(agent, outgoing_bytes, incoming_bytes, spawn);
        (Self { conn }, io_task)
    }

    /// Subscribe to receive stream updates from the client.
    ///
    /// This allows the agent to receive real-time notifications about
    /// client activities and cancellation requests.
    ///
    /// # Returns
    ///
    /// A [`StreamReceiver`] that can be used to receive stream messages.
    pub fn subscribe(&self) -> StreamReceiver {
        self.conn.subscribe()
    }
}

#[async_trait::async_trait(?Send)]
impl Client for AgentSideConnection {
    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> Result<RequestPermissionResponse, Error> {
        self.conn
            .request(
                SESSION_REQUEST_PERMISSION_METHOD_NAME,
                Some(AgentRequest::RequestPermissionRequest(args)),
            )
            .await
    }

    async fn write_text_file(
        &self,
        args: WriteTextFileRequest,
    ) -> Result<WriteTextFileResponse, Error> {
        self.conn
            .request::<Option<_>>(
                FS_WRITE_TEXT_FILE_METHOD_NAME,
                Some(AgentRequest::WriteTextFileRequest(args)),
            )
            .await
            .map(Option::unwrap_or_default)
    }

    async fn read_text_file(
        &self,
        args: ReadTextFileRequest,
    ) -> Result<ReadTextFileResponse, Error> {
        self.conn
            .request(
                FS_READ_TEXT_FILE_METHOD_NAME,
                Some(AgentRequest::ReadTextFileRequest(args)),
            )
            .await
    }

    async fn create_terminal(
        &self,
        args: CreateTerminalRequest,
    ) -> Result<CreateTerminalResponse, Error> {
        self.conn
            .request(
                TERMINAL_CREATE_METHOD_NAME,
                Some(AgentRequest::CreateTerminalRequest(args)),
            )
            .await
    }

    async fn terminal_output(
        &self,
        args: TerminalOutputRequest,
    ) -> Result<TerminalOutputResponse, Error> {
        self.conn
            .request(
                TERMINAL_OUTPUT_METHOD_NAME,
                Some(AgentRequest::TerminalOutputRequest(args)),
            )
            .await
    }

    async fn release_terminal(
        &self,
        args: ReleaseTerminalRequest,
    ) -> Result<ReleaseTerminalResponse, Error> {
        self.conn
            .request::<Option<_>>(
                TERMINAL_RELEASE_METHOD_NAME,
                Some(AgentRequest::ReleaseTerminalRequest(args)),
            )
            .await
            .map(Option::unwrap_or_default)
    }

    async fn wait_for_terminal_exit(
        &self,
        args: WaitForTerminalExitRequest,
    ) -> Result<WaitForTerminalExitResponse, Error> {
        self.conn
            .request(
                TERMINAL_WAIT_FOR_EXIT_METHOD_NAME,
                Some(AgentRequest::WaitForTerminalExitRequest(args)),
            )
            .await
    }

    async fn kill_terminal_command(
        &self,
        args: KillTerminalCommandRequest,
    ) -> Result<KillTerminalCommandResponse, Error> {
        self.conn
            .request::<Option<_>>(
                TERMINAL_KILL_METHOD_NAME,
                Some(AgentRequest::KillTerminalCommandRequest(args)),
            )
            .await
            .map(Option::unwrap_or_default)
    }

    async fn session_notification(&self, args: SessionNotification) -> Result<(), Error> {
        self.conn.notify(
            SESSION_UPDATE_NOTIFICATION,
            Some(AgentNotification::SessionNotification(args)),
        )
    }

    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, Error> {
        self.conn
            .request(
                format!("_{}", args.method),
                Some(AgentRequest::ExtMethodRequest(args)),
            )
            .await
    }

    async fn ext_notification(&self, args: ExtNotification) -> Result<(), Error> {
        self.conn.notify(
            format!("_{}", args.method),
            Some(AgentNotification::ExtNotification(args)),
        )
    }
}

/// Marker type representing the agent side of an ACP connection.
///
/// This type is used by the RPC layer to determine which messages
/// are incoming vs outgoing from the agent's perspective.
///
/// See protocol docs: [Communication Model](https://agentclientprotocol.com/protocol/overview#communication-model)
#[derive(Clone)]
pub struct AgentSide;

impl Side for AgentSide {
    type InRequest = ClientRequest;
    type InNotification = ClientNotification;
    type OutResponse = AgentResponse;

    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<ClientRequest, Error> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            INITIALIZE_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::InitializeRequest)
                .map_err(Into::into),
            AUTHENTICATE_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::AuthenticateRequest)
                .map_err(Into::into),
            SESSION_NEW_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::NewSessionRequest)
                .map_err(Into::into),
            SESSION_LOAD_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::LoadSessionRequest)
                .map_err(Into::into),
            SESSION_SET_MODE_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::SetSessionModeRequest)
                .map_err(Into::into),
            #[cfg(feature = "unstable")]
            SESSION_SET_MODEL_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::SetSessionModelRequest)
                .map_err(Into::into),
            SESSION_PROMPT_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::PromptRequest)
                .map_err(Into::into),
            _ => {
                if let Some(custom_method) = method.strip_prefix('_') {
                    Ok(ClientRequest::ExtMethodRequest(ExtRequest {
                        method: custom_method.into(),
                        params: RawValue::from_string(params.get().to_string())?.into(),
                    }))
                } else {
                    Err(Error::method_not_found())
                }
            }
        }
    }

    fn decode_notification(
        method: &str,
        params: Option<&RawValue>,
    ) -> Result<ClientNotification, Error> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            SESSION_CANCEL_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientNotification::CancelNotification)
                .map_err(Into::into),
            _ => {
                if let Some(custom_method) = method.strip_prefix('_') {
                    Ok(ClientNotification::ExtNotification(ExtNotification {
                        method: custom_method.into(),
                        params: RawValue::from_string(params.get().to_string())?.into(),
                    }))
                } else {
                    Err(Error::method_not_found())
                }
            }
        }
    }
}

impl<T: Agent> MessageHandler<AgentSide> for T {
    async fn handle_request(&self, request: ClientRequest) -> Result<AgentResponse, Error> {
        match request {
            ClientRequest::InitializeRequest(args) => {
                let response = self.initialize(args).await?;
                Ok(AgentResponse::InitializeResponse(response))
            }
            ClientRequest::AuthenticateRequest(args) => {
                let response = self.authenticate(args).await?;
                Ok(AgentResponse::AuthenticateResponse(response))
            }
            ClientRequest::NewSessionRequest(args) => {
                let response = self.new_session(args).await?;
                Ok(AgentResponse::NewSessionResponse(response))
            }
            ClientRequest::LoadSessionRequest(args) => {
                let response = self.load_session(args).await?;
                Ok(AgentResponse::LoadSessionResponse(response))
            }
            ClientRequest::PromptRequest(args) => {
                let response = self.prompt(args).await?;
                Ok(AgentResponse::PromptResponse(response))
            }
            ClientRequest::SetSessionModeRequest(args) => {
                let response = self.set_session_mode(args).await?;
                Ok(AgentResponse::SetSessionModeResponse(response))
            }
            #[cfg(feature = "unstable")]
            ClientRequest::SetSessionModelRequest(args) => {
                let response = self.set_session_model(args).await?;
                Ok(AgentResponse::SetSessionModelResponse(response))
            }
            ClientRequest::ExtMethodRequest(args) => {
                let response = self.ext_method(args).await?;
                Ok(AgentResponse::ExtMethodResponse(response))
            }
        }
    }

    async fn handle_notification(&self, notification: ClientNotification) -> Result<(), Error> {
        match notification {
            ClientNotification::CancelNotification(args) => {
                self.cancel(args).await?;
            }
            ClientNotification::ExtNotification(args) => {
                self.ext_notification(args).await?;
            }
        }
        Ok(())
    }
}
