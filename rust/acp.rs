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
pub use rpc::*;
pub use tool_call::*;
pub use version::*;

use anyhow::Result;
use futures::{AsyncRead, AsyncWrite, Future, future::LocalBoxFuture};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::{cell::RefCell, fmt, rc::Rc, sync::Arc};

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

type SessionUpdateCallback = Rc<RefCell<Option<Box<dyn Fn(SessionNotification)>>>>;

impl AgentConnection {
    pub fn new(
        client: impl Client + 'static,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let on_session_update = Rc::new(RefCell::new(None));
        let handler = ClientMessageHandler {
            client,
            on_session_update: on_session_update.clone(),
        };

        let (conn, io_task) = RpcConnection::new(
            ClientMessageDecoder,
            handler,
            outgoing_bytes,
            incoming_bytes,
            spawn,
        );
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
        F: Fn(SessionNotification) + 'static,
    {
        self.on_session_update.replace(Some(Box::new(callback)));
    }

    pub async fn initialize(
        &self,
        arguments: InitializeRequest,
    ) -> Result<InitializeResponse, Error> {
        self.conn
            .request(
                INITIALIZE_METHOD_NAME,
                Some(AgentRequest::InitializeRequest(arguments)),
            )
            .await
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

struct AgentMessageDecoder;

impl MessageDecoder<AgentSide, ClientSide> for AgentMessageDecoder {
    fn decode_request(
        &self,
        method: &str,
        params: Option<&RawValue>,
    ) -> Result<AgentRequest, Error> {
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
        &self,
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

struct AgentMessageHandler<D: Agent> {
    agent: D,
    on_cancel: SessionCancelCallback,
}

type SessionCancelCallback = Rc<RefCell<Option<Box<dyn Fn(SessionId)>>>>;

impl<D: Agent> MessageHandler<AgentSide, ClientSide> for AgentMessageHandler<D> {
    async fn handle_request(&self, request: AgentRequest) -> Result<AgentResponse, Error> {
        match request {
            AgentRequest::InitializeRequest(args) => {
                let response = self.agent.initialize(args).await?;
                Ok(AgentResponse::InitializeResponse(response))
            }
            AgentRequest::AuthenticateRequest(args) => {
                self.agent.authenticate(args).await?;
                Ok(AgentResponse::AuthenticateResponse)
            }
            AgentRequest::NewSessionRequest(args) => {
                let response = self.agent.new_session(args).await?;
                Ok(AgentResponse::NewSessionResponse(response))
            }
            AgentRequest::LoadSessionRequest(args) => {
                let response = self.agent.load_session(args).await?;
                Ok(AgentResponse::LoadSessionResponse(response))
            }
            AgentRequest::PromptRequest(args) => {
                self.agent.prompt(args).await?;
                Ok(AgentResponse::PromptResponse)
            }
        }
    }

    async fn handle_notification(&self, notification: ClientNotification) -> Result<(), Error> {
        match notification {
            ClientNotification::CancelledNotification(notification) => {
                if let Some(callback) = &*self.on_cancel.borrow() {
                    callback(notification.session_id);
                }
            }
        }
        Ok(())
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
        let on_cancel = Rc::new(RefCell::new(None));
        let handler = AgentMessageHandler {
            agent,
            on_cancel: on_cancel.clone(),
        };

        let (conn, io_task) = RpcConnection::new(
            AgentMessageDecoder,
            handler,
            outgoing_bytes,
            incoming_bytes,
            spawn,
        );
        (Self { conn, on_cancel }, io_task)
    }

    pub fn on_cancel(&self, callback: impl Fn(SessionId) + 'static) {
        self.on_cancel.replace(Some(Box::new(callback)));
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
    async fn handle_request(&self, request: ClientRequest) -> Result<ClientResponse, Error> {
        match request {
            ClientRequest::RequestPermissionRequest(args) => {
                let response = self.client.request_permission(args).await?;
                Ok(ClientResponse::RequestPermissionResponse(response))
            }
            ClientRequest::WriteTextFileRequest(args) => {
                self.client.write_text_file(args).await?;
                Ok(ClientResponse::WriteTextFileResponse)
            }
            ClientRequest::ReadTextFileRequest(args) => {
                let response = self.client.read_text_file(args).await?;
                Ok(ClientResponse::ReadTextFileResponse(response))
            }
        }
    }

    async fn handle_notification(&self, notification: AgentNotification) -> Result<(), Error> {
        match notification {
            AgentNotification::SessionNotification(session_notification) => {
                if let Some(callback) = &*self.on_session_update.borrow() {
                    callback(session_notification);
                }
            }
        }
        Ok(())
    }
}
