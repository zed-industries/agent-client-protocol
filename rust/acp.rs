mod agent;
mod client;
mod content;
mod error;
mod plan;
mod rpc;
#[cfg(test)]
mod rpc_tests;
mod tool_call;
mod version;

pub use agent::*;
pub use client::*;
pub use content::*;
pub use error::*;
pub use plan::*;
pub use tool_call::*;
pub use version::*;

use anyhow::Result;
use futures::{AsyncRead, AsyncWrite, Future, future::LocalBoxFuture};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::{fmt, sync::Arc};

use crate::rpc::{MessageDecoder, MessageHandler, RpcConnection, Side};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SessionId(pub Arc<str>);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Client to Agent

pub struct ClientSide;

impl Side for ClientSide {
    type Request = ClientRequest;
    type Response = ClientResponse;
    type Notification = ClientNotification;
}

pub struct AgentConnection {
    conn: RpcConnection<ClientSide, AgentSide>,
}

impl AgentConnection {
    pub fn new(
        client: impl MessageHandler<ClientSide, AgentSide> + 'static,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let (conn, io_task) = RpcConnection::new(client, outgoing_bytes, incoming_bytes, spawn);
        (Self { conn }, io_task)
    }
}

impl Agent for AgentConnection {
    async fn initialize(&self, arguments: InitializeRequest) -> Result<InitializeResponse, Error> {
        self.conn
            .request(
                INITIALIZE_METHOD_NAME,
                Some(AgentRequest::InitializeRequest(arguments)),
            )
            .await
    }

    async fn authenticate(&self, arguments: AuthenticateRequest) -> Result<(), Error> {
        self.conn
            .request(
                AUTHENTICATE_METHOD_NAME,
                Some(AgentRequest::AuthenticateRequest(arguments)),
            )
            .await
    }

    async fn new_session(&self, arguments: NewSessionRequest) -> Result<NewSessionResponse, Error> {
        self.conn
            .request(
                SESSION_NEW_METHOD_NAME,
                Some(AgentRequest::NewSessionRequest(arguments)),
            )
            .await
    }

    async fn load_session(
        &self,
        arguments: LoadSessionRequest,
    ) -> Result<LoadSessionResponse, Error> {
        self.conn
            .request(
                SESSION_LOAD_METHOD_NAME,
                Some(AgentRequest::LoadSessionRequest(arguments)),
            )
            .await
    }

    async fn prompt(&self, arguments: PromptRequest) -> Result<(), Error> {
        self.conn
            .request(
                SESSION_PROMPT_METHOD_NAME,
                Some(AgentRequest::PromptRequest(arguments)),
            )
            .await
    }

    async fn cancelled(&self, notification: CancelledNotification) -> Result<(), Error> {
        self.conn.notify(
            SESSION_CANCELLED_METHOD_NAME,
            Some(ClientNotification::CancelledNotification(notification)),
        )
    }
}

impl MessageDecoder<AgentSide, ClientSide> for AgentSide {
    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<AgentRequest, Error> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            INITIALIZE_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::InitializeRequest)
                .map_err(Into::into),
            AUTHENTICATE_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::AuthenticateRequest)
                .map_err(Into::into),
            SESSION_NEW_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::NewSessionRequest)
                .map_err(Into::into),
            SESSION_LOAD_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::LoadSessionRequest)
                .map_err(Into::into),
            SESSION_PROMPT_METHOD_NAME => serde_json::from_str(params.get())
                .map(AgentRequest::PromptRequest)
                .map_err(Into::into),
            _ => Err(Error::method_not_found()),
        }
    }

    fn decode_notification(
        method: &str,
        params: Option<&RawValue>,
    ) -> Result<ClientNotification, Error> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            SESSION_CANCELLED_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientNotification::CancelledNotification)
                .map_err(Into::into),
            _ => Err(Error::method_not_found()),
        }
    }
}

impl<T: Agent> MessageHandler<AgentSide, ClientSide> for T {
    async fn handle_request(&self, request: AgentRequest) -> Result<AgentResponse, Error> {
        match request {
            AgentRequest::InitializeRequest(args) => {
                let response = self.initialize(args).await?;
                Ok(AgentResponse::InitializeResponse(response))
            }
            AgentRequest::AuthenticateRequest(args) => {
                self.authenticate(args).await?;
                Ok(AgentResponse::AuthenticateResponse)
            }
            AgentRequest::NewSessionRequest(args) => {
                let response = self.new_session(args).await?;
                Ok(AgentResponse::NewSessionResponse(response))
            }
            AgentRequest::LoadSessionRequest(args) => {
                let response = self.load_session(args).await?;
                Ok(AgentResponse::LoadSessionResponse(response))
            }
            AgentRequest::PromptRequest(args) => {
                self.prompt(args).await?;
                Ok(AgentResponse::PromptResponse)
            }
        }
    }

    async fn handle_notification(&self, notification: ClientNotification) -> Result<(), Error> {
        match notification {
            ClientNotification::CancelledNotification(notification) => {
                self.cancelled(notification).await?;
            }
        }
        Ok(())
    }
}

// Agent to Client

pub struct AgentSide;

impl Side for AgentSide {
    type Request = AgentRequest;
    type Response = AgentResponse;
    type Notification = AgentNotification;
}

pub struct ClientConnection {
    conn: RpcConnection<AgentSide, ClientSide>,
}

impl ClientConnection {
    pub fn new(
        agent: impl MessageHandler<AgentSide, ClientSide> + 'static,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let (conn, io_task) = RpcConnection::new(agent, outgoing_bytes, incoming_bytes, spawn);
        (Self { conn }, io_task)
    }
}

impl Client for ClientConnection {
    async fn request_permission(
        &self,
        arguments: RequestPermissionRequest,
    ) -> Result<RequestPermissionResponse, Error> {
        self.conn
            .request(
                SESSION_REQUEST_PERMISSION_METHOD_NAME,
                Some(ClientRequest::RequestPermissionRequest(arguments)),
            )
            .await
    }

    async fn write_text_file(&self, arguments: WriteTextFileRequest) -> Result<(), Error> {
        self.conn
            .request(
                FS_WRITE_TEXT_FILE_METHOD_NAME,
                Some(ClientRequest::WriteTextFileRequest(arguments)),
            )
            .await
    }

    async fn read_text_file(
        &self,
        arguments: ReadTextFileRequest,
    ) -> Result<ReadTextFileResponse, Error> {
        self.conn
            .request(
                FS_READ_TEXT_FILE_METHOD_NAME,
                Some(ClientRequest::ReadTextFileRequest(arguments)),
            )
            .await
    }

    async fn session_notification(&self, notification: SessionNotification) -> Result<(), Error> {
        self.conn.notify(
            SESSION_UPDATE_NOTIFICATION,
            Some(AgentNotification::SessionNotification(notification)),
        )
    }
}

impl MessageDecoder<ClientSide, AgentSide> for ClientSide {
    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<ClientRequest, Error> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            SESSION_REQUEST_PERMISSION_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::RequestPermissionRequest)
                .map_err(Into::into),
            FS_WRITE_TEXT_FILE_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::WriteTextFileRequest)
                .map_err(Into::into),
            FS_READ_TEXT_FILE_METHOD_NAME => serde_json::from_str(params.get())
                .map(ClientRequest::ReadTextFileRequest)
                .map_err(Into::into),
            _ => Err(Error::method_not_found()),
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
            _ => Err(Error::method_not_found()),
        }
    }
}

impl<T: Client> MessageHandler<ClientSide, AgentSide> for T {
    async fn handle_request(&self, request: ClientRequest) -> Result<ClientResponse, Error> {
        match request {
            ClientRequest::RequestPermissionRequest(args) => {
                let response = self.request_permission(args).await?;
                Ok(ClientResponse::RequestPermissionResponse(response))
            }
            ClientRequest::WriteTextFileRequest(args) => {
                self.write_text_file(args).await?;
                Ok(ClientResponse::WriteTextFileResponse)
            }
            ClientRequest::ReadTextFileRequest(args) => {
                let response = self.read_text_file(args).await?;
                Ok(ClientResponse::ReadTextFileResponse(response))
            }
        }
    }

    async fn handle_notification(&self, notification: AgentNotification) -> Result<(), Error> {
        match notification {
            AgentNotification::SessionNotification(notification) => {
                self.session_notification(notification).await?;
            }
        }
        Ok(())
    }
}
