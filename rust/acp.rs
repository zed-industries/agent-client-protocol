mod agent;
mod client;
mod content;
mod error;
mod plan;
mod rpc;
#[cfg(test)]
mod rpc_tests;
mod tool_call;

pub use agent::*;
pub use client::*;
pub use content::*;
pub use error::*;
pub use plan::*;
pub use rpc::*;
pub use tool_call::*;

use anyhow::Result;
use futures::{AsyncRead, AsyncWrite, Future, FutureExt, future::LocalBoxFuture};
use parking_lot::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::{fmt, sync::Arc};

// Type aliases to reduce complexity
type SessionUpdateCallback = Arc<Mutex<Option<Box<dyn Fn(SessionNotification) + Send + Sync>>>>;
type SessionCancelCallback = Arc<Mutex<Option<Box<dyn Fn(SessionId) + Send + Sync>>>>;

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

impl RpcSide for ClientSide {
    type Request = ClientRequest;
    type Response = ClientResponse;
    type Notification = ClientNotification;
}

pub struct AgentConnection {
    conn: RpcConnection<ClientSide, AgentSide>,
    on_session_update: SessionUpdateCallback,
}

impl AgentConnection {
    pub fn new(
        client: impl Client + 'static,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let on_session_update = Arc::new(Mutex::new(None));
        let decoder = ClientMessageDecoder;
        let handler = ClientMessageHandler {
            client,
            on_session_update: on_session_update.clone(),
        };

        let (conn, io_task) =
            RpcConnection::new(decoder, handler, outgoing_bytes, incoming_bytes, spawn);
        (
            Self {
                conn,
                on_session_update,
            },
            io_task,
        )
    }

    pub fn on_session_update<F>(&self, callback: F)
    where
        F: Fn(SessionNotification) + Send + Sync + 'static,
    {
        *self.on_session_update.lock() = Some(Box::new(callback));
    }

    pub async fn authenticate(&self, arguments: AuthenticateRequest) -> Result<(), Error> {
        self.conn
            .request(
                AUTHENTICATE_METHOD_NAME,
                Some(AgentRequest::AuthenticateRequest(arguments)),
            )
            .await
    }

    pub async fn new_session(
        &self,
        arguments: NewSessionRequest,
    ) -> Result<NewSessionResponse, Error> {
        self.conn
            .request(
                SESSION_NEW_METHOD_NAME,
                Some(AgentRequest::NewSessionRequest(arguments)),
            )
            .await
    }

    pub async fn load_session(
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

    pub async fn prompt(&self, arguments: PromptRequest) -> Result<(), Error> {
        self.conn
            .request(
                SESSION_PROMPT_METHOD_NAME,
                Some(AgentRequest::PromptRequest(arguments)),
            )
            .await
    }

    pub fn cancel_generation(&self, session_id: SessionId) -> Result<(), Error> {
        self.conn.notify(
            SESSION_CANCELLED_METHOD_NAME,
            Some(ClientNotification::CancelledNotification(
                CancelledNotification { session_id },
            )),
        )
    }
}

struct AgentMessageDecoder {
    on_cancel: SessionCancelCallback,
}

impl MessageDecoder<AgentSide, ClientSide> for AgentMessageDecoder {
    fn decode_request(
        &self,
        method: &str,
        params: Option<&RawValue>,
    ) -> Result<AgentRequest, Error> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
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
        &self,
        method: &str,
        params: Option<&RawValue>,
    ) -> Result<ClientNotification, Error> {
        let params = params.ok_or_else(Error::invalid_params)?;

        match method {
            SESSION_CANCELLED_METHOD_NAME => {
                let notification: client::CancelledNotification =
                    serde_json::from_str(params.get())?;

                if let Some(callback) = &*self.on_cancel.lock() {
                    callback(notification.session_id.clone());
                }

                Ok(ClientNotification::CancelledNotification(notification))
            }
            _ => Err(Error::method_not_found()),
        }
    }
}

struct AgentMessageHandler<D: Agent> {
    agent: D,
}

impl<D: Agent> MessageHandler<AgentSide, ClientSide> for AgentMessageHandler<D> {
    fn handle_request(
        &self,
        request: AgentRequest,
    ) -> LocalBoxFuture<'static, Result<AgentResponse, Error>> {
        match request {
            AgentRequest::AuthenticateRequest(args) => {
                let fut = self.agent.authenticate(args);
                async move {
                    fut.await?;
                    Ok(AgentResponse::AuthenticateResponse)
                }
                .boxed_local()
            }
            AgentRequest::NewSessionRequest(args) => {
                let fut = self.agent.new_session(args);
                async move { Ok(AgentResponse::NewSessionResponse(fut.await?)) }.boxed_local()
            }
            AgentRequest::LoadSessionRequest(args) => {
                let fut = self.agent.load_session(args);
                async move { Ok(AgentResponse::LoadSessionResponse(fut.await?)) }.boxed_local()
            }
            AgentRequest::PromptRequest(args) => {
                let fut = self.agent.prompt(args);
                async move {
                    fut.await?;
                    Ok(AgentResponse::PromptResponse)
                }
                .boxed_local()
            }
        }
    }

    fn handle_notification(
        &self,
        _notification: ClientNotification,
    ) -> LocalBoxFuture<'static, Result<(), Error>> {
        // Agent doesn't handle client notifications in the handler
        async { Ok(()) }.boxed_local()
    }
}

// Agent to Client

pub struct AgentSide;

impl RpcSide for AgentSide {
    type Request = AgentRequest;
    type Response = AgentResponse;
    type Notification = AgentNotification;
}

pub struct ClientConnection {
    conn: RpcConnection<AgentSide, ClientSide>,
    on_cancel: SessionCancelCallback,
}

impl ClientConnection {
    pub fn new(
        agent: impl Agent + 'static,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let on_cancel = Arc::new(Mutex::new(None));
        let decoder = AgentMessageDecoder {
            on_cancel: on_cancel.clone(),
        };
        let handler = AgentMessageHandler { agent };

        let (conn, io_task) =
            RpcConnection::new(decoder, handler, outgoing_bytes, incoming_bytes, spawn);
        (Self { conn, on_cancel }, io_task)
    }

    pub fn on_cancel<F>(&self, callback: F)
    where
        F: Fn(SessionId) + Send + Sync + 'static,
    {
        *self.on_cancel.lock() = Some(Box::new(callback));
    }

    pub async fn request_permission(
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

    pub async fn write_text_file(&self, arguments: WriteTextFileRequest) -> Result<(), Error> {
        self.conn
            .request(
                FS_WRITE_TEXT_FILE_METHOD_NAME,
                Some(ClientRequest::WriteTextFileRequest(arguments)),
            )
            .await
    }

    pub async fn read_text_file(
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

    pub fn send_session_update(
        &self,
        session_id: SessionId,
        update: SessionUpdate,
    ) -> Result<(), Error> {
        self.conn.notify(
            SESSION_UPDATE_NOTIFICATION,
            Some(AgentNotification::SessionNotification(
                SessionNotification { session_id, update },
            )),
        )
    }
}

struct ClientMessageDecoder;

impl MessageDecoder<ClientSide, AgentSide> for ClientMessageDecoder {
    fn decode_request(
        &self,
        method: &str,
        params: Option<&RawValue>,
    ) -> Result<ClientRequest, Error> {
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
        &self,
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

struct ClientMessageHandler<D: Client> {
    client: D,
    on_session_update: SessionUpdateCallback,
}

impl<D: Client> MessageHandler<ClientSide, AgentSide> for ClientMessageHandler<D> {
    fn handle_request(
        &self,
        request: ClientRequest,
    ) -> LocalBoxFuture<'static, Result<ClientResponse, Error>> {
        match request {
            ClientRequest::RequestPermissionRequest(args) => {
                let fut = self.client.request_permission(args);
                async move { Ok(ClientResponse::RequestPermissionResponse(fut.await?)) }
                    .boxed_local()
            }
            ClientRequest::WriteTextFileRequest(args) => {
                let fut = self.client.write_text_file(args);
                async move {
                    fut.await?;
                    Ok(ClientResponse::WriteTextFileResponse)
                }
                .boxed_local()
            }
            ClientRequest::ReadTextFileRequest(args) => {
                let fut = self.client.read_text_file(args);
                async move { Ok(ClientResponse::ReadTextFileResponse(fut.await?)) }.boxed_local()
            }
        }
    }

    fn handle_notification(
        &self,
        notification: AgentNotification,
    ) -> LocalBoxFuture<'static, Result<(), Error>> {
        match notification {
            AgentNotification::SessionNotification(session_notification) => {
                if let Some(callback) = &*self.on_session_update.lock() {
                    callback(session_notification);
                }
            }
        }
        async { Ok(()) }.boxed_local()
    }
}
