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
use futures::{AsyncRead, AsyncWrite, channel::mpsc, future::LocalBoxFuture};
use parking_lot::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::{fmt, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SessionId(pub Arc<str>);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct AgentSide;

impl RpcSide for AgentSide {
    type Request = AgentRequest;
    type Response = AgentResponse;
    type Notification = AgentNotification;
}

pub struct ClientSide;

impl RpcSide for ClientSide {
    type Request = ClientRequest;
    type Response = ClientResponse;
    type Notification = ClientNotification;
}

pub struct AgentConnection {
    conn: RpcConnection<ClientSide, AgentSide>,
    on_session_update: Arc<Mutex<Option<Box<dyn Fn(SessionNotification) + Send + Sync>>>>,
}

impl AgentConnection {
    pub fn new(
        client: impl Client + 'static,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded();
        let on_session_update = Arc::new(Mutex::new(None));
        let dispatcher = ClientDispatcher {
            base: BaseDispatcher {
                delegate: client,
                spawn: Box::new(spawn),
                outgoing_tx: outgoing_tx.clone(),
            },
            on_session_update: on_session_update.clone(),
        };

        let (conn, io_task) = RpcConnection::new(
            dispatcher,
            outgoing_tx,
            outgoing_rx,
            outgoing_bytes,
            incoming_bytes,
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

pub struct ClientConnection {
    conn: RpcConnection<AgentSide, ClientSide>,
    on_cancel: Arc<Mutex<Option<Box<dyn Fn(SessionId) + Send + Sync>>>>,
}

impl ClientConnection {
    pub fn new(
        agent: impl Agent + 'static,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded();
        let on_cancel = Arc::new(Mutex::new(None));
        let dispatcher = AgentDispatcher {
            base: BaseDispatcher {
                delegate: agent,
                spawn: Box::new(spawn),
                outgoing_tx: outgoing_tx.clone(),
            },
            on_cancel: on_cancel.clone(),
        };

        let (conn, io_task) = RpcConnection::new(
            dispatcher,
            outgoing_tx,
            outgoing_rx,
            outgoing_bytes,
            incoming_bytes,
        );
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

struct AgentDispatcher<D: Agent> {
    base: BaseDispatcher<D, AgentSide, ClientSide>,
    on_cancel: Arc<Mutex<Option<Box<dyn Fn(SessionId) + Send + Sync>>>>,
}

impl<D: Agent> Dispatcher for AgentDispatcher<D> {
    type Notification = ClientNotification;

    fn request(
        &self,
        id: i32,
        method: &str,
        params: Option<&serde_json::value::RawValue>,
    ) -> Result<(), Error> {
        match method {
            AUTHENTICATE_METHOD_NAME => dispatch_request!(
                &self.base,
                id,
                params,
                AuthenticateRequest,
                |delegate: &D, args| delegate.authenticate(args),
                |_| AgentResponse::AuthenticateResponse
            ),
            SESSION_NEW_METHOD_NAME => dispatch_request!(
                &self.base,
                id,
                params,
                NewSessionRequest,
                |delegate: &D, args| delegate.new_session(args),
                AgentResponse::NewSessionResponse
            ),
            SESSION_LOAD_METHOD_NAME => dispatch_request!(
                &self.base,
                id,
                params,
                LoadSessionRequest,
                |delegate: &D, args| delegate.load_session(args),
                AgentResponse::LoadSessionResponse
            ),
            SESSION_PROMPT_METHOD_NAME => dispatch_request!(
                &self.base,
                id,
                params,
                PromptRequest,
                |delegate: &D, args| delegate.prompt(args),
                |_| AgentResponse::PromptResponse
            ),
            _ => Err(Error::method_not_found()),
        }
    }

    fn notification(&self, method: &str, params: Option<&RawValue>) -> Result<(), Error> {
        match method {
            SESSION_CANCELLED_METHOD_NAME => {
                dispatch_notification!(
                    method,
                    params,
                    client::CancelledNotification,
                    |params: client::CancelledNotification| {
                        if let Some(callback) = &*self.on_cancel.lock() {
                            callback(params.session_id);
                        }
                        Ok(())
                    }
                )
            }
            _ => Err(Error::method_not_found()),
        }
    }
}

struct ClientDispatcher<D: Client> {
    base: BaseDispatcher<D, ClientSide, AgentSide>,
    on_session_update: Arc<Mutex<Option<Box<dyn Fn(SessionNotification) + Send + Sync>>>>,
}

impl<D: Client> Dispatcher for ClientDispatcher<D> {
    type Notification = AgentNotification;

    fn request(
        &self,
        id: i32,
        method: &str,
        params: Option<&serde_json::value::RawValue>,
    ) -> Result<(), Error> {
        match method {
            SESSION_REQUEST_PERMISSION_METHOD_NAME => dispatch_request!(
                &self.base,
                id,
                params,
                RequestPermissionRequest,
                |delegate: &D, args| delegate.request_permission(args),
                ClientResponse::RequestPermissionResponse
            ),
            FS_WRITE_TEXT_FILE_METHOD_NAME => dispatch_request!(
                &self.base,
                id,
                params,
                WriteTextFileRequest,
                |delegate: &D, args| delegate.write_text_file(args),
                |_| ClientResponse::WriteTextFileResponse
            ),
            FS_READ_TEXT_FILE_METHOD_NAME => dispatch_request!(
                &self.base,
                id,
                params,
                ReadTextFileRequest,
                |delegate: &D, args| delegate.read_text_file(args),
                ClientResponse::ReadTextFileResponse
            ),
            _ => Err(Error::method_not_found()),
        }
    }

    fn notification(&self, method: &str, params: Option<&RawValue>) -> Result<(), Error> {
        match method {
            SESSION_UPDATE_NOTIFICATION => {
                dispatch_notification!(
                    method,
                    params,
                    SessionNotification,
                    |notification: SessionNotification| {
                        if let Some(callback) = &*self.on_session_update.lock() {
                            callback(notification);
                        }
                        Ok(())
                    }
                )
            }
            _ => Err(Error::method_not_found()),
        }
    }
}
