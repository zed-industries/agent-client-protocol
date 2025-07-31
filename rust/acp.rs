mod agent;
mod client;
mod content;
mod error;
mod plan;
pub mod rpc;
mod tool_call;

pub use agent::*;
pub use client::*;
pub use content::*;
pub use error::*;
use futures::{
    AsyncRead, AsyncWrite, Future, FutureExt,
    channel::mpsc::{self, UnboundedSender},
    future::LocalBoxFuture,
};
pub use plan::*;
pub use tool_call::*;

use anyhow::Result;
use parking_lot::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

use crate::rpc::{Dispatcher, OutgoingMessage, RpcConnection, RpcSide};

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
            delegate: client,
            spawn: Box::new(spawn),
            outgoing_tx: outgoing_tx.clone(),
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

    pub async fn new_session(
        &self,
        arguments: NewSessionArguments,
    ) -> Result<NewSessionOutput, Error> {
        self.conn
            .request(
                NEW_SESSION_METHOD_NAME,
                Some(AgentRequest::NewSession(arguments)),
            )
            .await
    }

    pub async fn load_session(
        &self,
        arguments: LoadSessionArguments,
    ) -> Result<LoadSessionOutput, Error> {
        self.conn
            .request(
                LOAD_SESSION_METHOD_NAME,
                Some(AgentRequest::LoadSession(arguments)),
            )
            .await
    }

    pub async fn prompt(&self, arguments: PromptArguments) -> Result<(), Error> {
        self.conn
            .request(PROMPT_METHOD_NAME, Some(AgentRequest::Prompt(arguments)))
            .await
    }

    pub fn cancel(&self, request_id: u64) -> Result<(), Error> {
        self.conn
            .notify(ClientNotification::Cancelled { request_id })
    }
}

pub struct ClientConnection {
    conn: RpcConnection<AgentSide, ClientSide>,
    on_cancel: Arc<Mutex<Option<Box<dyn Fn(u64) + Send + Sync>>>>,
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
            delegate: agent,
            spawn: Box::new(spawn),
            outgoing_tx: outgoing_tx.clone(),
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
        F: Fn(u64) + Send + Sync + 'static,
    {
        *self.on_cancel.lock() = Some(Box::new(callback));
    }

    pub async fn request_permission(
        &self,
        arguments: RequestPermissionArguments,
    ) -> Result<RequestPermissionOutput, Error> {
        self.conn
            .request(
                REQUEST_PERMISSION_METHOD_NAME,
                Some(ClientRequest::RequestPermission(arguments)),
            )
            .await
    }

    pub async fn write_text_file(&self, arguments: WriteTextFileArguments) -> Result<(), Error> {
        self.conn
            .request(
                WRITE_TEXT_FILE_METHOD_NAME,
                Some(ClientRequest::WriteTextFile(arguments)),
            )
            .await
    }

    pub async fn read_text_file(
        &self,
        arguments: ReadTextFileArguments,
    ) -> Result<ReadTextFileOutput, Error> {
        self.conn
            .request(
                READ_TEXT_FILE_METHOD_NAME,
                Some(ClientRequest::ReadTextFile(arguments)),
            )
            .await
    }

    pub fn send_session_update(
        &self,
        session_id: SessionId,
        update: SessionUpdate,
    ) -> Result<(), Error> {
        self.conn
            .notify(AgentNotification::SessionUpdate(SessionNotification {
                session_id,
                update,
            }))
    }
}

struct AgentDispatcher<D: Agent> {
    delegate: D,
    spawn: Box<dyn Fn(LocalBoxFuture<'static, ()>) + 'static>,
    outgoing_tx: UnboundedSender<OutgoingMessage<AgentSide, ClientSide>>,
    on_cancel: Arc<Mutex<Option<Box<dyn Fn(u64) + Send + Sync>>>>,
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
            NEW_SESSION_METHOD_NAME => {
                let Some(params) = params else {
                    return Err(Error::invalid_params());
                };

                match serde_json::from_str::<NewSessionArguments>(params.get()) {
                    Ok(arguments) => {
                        let fut = self.delegate.new_session(arguments);
                        let outgoing_tx = self.outgoing_tx.clone();
                        (self.spawn)(
                            async move {
                                outgoing_tx
                                    .unbounded_send(OutgoingMessage::Response {
                                        id,
                                        result: fut.await.map(AgentResponse::NewSession).into(),
                                    })
                                    .ok();
                            }
                            .boxed_local(),
                        );

                        Ok(())
                    }
                    Err(err) => Err(Error::invalid_params().with_data(err.to_string())),
                }
            }
            LOAD_SESSION_METHOD_NAME => {
                let Some(params) = params else {
                    return Err(Error::invalid_params());
                };

                match serde_json::from_str::<LoadSessionArguments>(params.get()) {
                    Ok(arguments) => {
                        let fut = self.delegate.load_session(arguments);
                        let outgoing_tx = self.outgoing_tx.clone();
                        (self.spawn)(
                            async move {
                                outgoing_tx
                                    .unbounded_send(OutgoingMessage::Response {
                                        id,
                                        result: fut.await.map(AgentResponse::LoadSession).into(),
                                    })
                                    .ok();
                            }
                            .boxed_local(),
                        );

                        Ok(())
                    }
                    Err(err) => Err(Error::invalid_params().with_data(err.to_string())),
                }
            }
            PROMPT_METHOD_NAME => {
                let Some(params) = params else {
                    return Err(Error::invalid_params());
                };

                match serde_json::from_str::<PromptArguments>(params.get()) {
                    Ok(arguments) => {
                        let fut = self.delegate.prompt(arguments);
                        let outgoing_tx = self.outgoing_tx.clone();
                        (self.spawn)(
                            async move {
                                outgoing_tx
                                    .unbounded_send(OutgoingMessage::Response {
                                        id,
                                        result: fut.await.map(|_| AgentResponse::Prompt).into(),
                                    })
                                    .ok();
                            }
                            .boxed_local(),
                        );

                        Ok(())
                    }
                    Err(err) => Err(Error::invalid_params().with_data(err.to_string())),
                }
            }
            _ => Err(Error::method_not_found()),
        }
    }

    fn notification(&self, notification: Self::Notification) -> Result<(), Error> {
        match notification {
            ClientNotification::Cancelled { request_id } => {
                if let Some(callback) = &*self.on_cancel.lock() {
                    callback(request_id);
                }
                Ok(())
            }
        }
    }
}

struct ClientDispatcher<D: Client> {
    delegate: D,
    spawn: Box<dyn Fn(LocalBoxFuture<'static, ()>) + 'static>,
    outgoing_tx: UnboundedSender<OutgoingMessage<ClientSide, AgentSide>>,
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
            REQUEST_PERMISSION_METHOD_NAME => {
                let Some(params) = params else {
                    return Err(Error::invalid_params());
                };

                match serde_json::from_str::<RequestPermissionArguments>(params.get()) {
                    Ok(arguments) => {
                        let fut = self.delegate.request_permission(arguments);
                        let outgoing_tx = self.outgoing_tx.clone();
                        (self.spawn)(
                            async move {
                                outgoing_tx
                                    .unbounded_send(OutgoingMessage::Response {
                                        id,
                                        result: fut
                                            .await
                                            .map(ClientResponse::RequestPermission)
                                            .into(),
                                    })
                                    .ok();
                            }
                            .boxed_local(),
                        );

                        Ok(())
                    }
                    Err(err) => Err(Error::invalid_params().with_data(err.to_string())),
                }
            }
            WRITE_TEXT_FILE_METHOD_NAME => {
                let Some(params) = params else {
                    return Err(Error::invalid_params());
                };

                match serde_json::from_str::<WriteTextFileArguments>(params.get()) {
                    Ok(arguments) => {
                        let fut = self.delegate.write_text_file(arguments);
                        let outgoing_tx = self.outgoing_tx.clone();
                        (self.spawn)(
                            async move {
                                outgoing_tx
                                    .unbounded_send(OutgoingMessage::Response {
                                        id,
                                        result: fut
                                            .await
                                            .map(|_| ClientResponse::WriteTextFile)
                                            .into(),
                                    })
                                    .ok();
                            }
                            .boxed_local(),
                        );

                        Ok(())
                    }
                    Err(err) => Err(Error::invalid_params().with_data(err.to_string())),
                }
            }
            READ_TEXT_FILE_METHOD_NAME => {
                let Some(params) = params else {
                    return Err(Error::invalid_params());
                };

                match serde_json::from_str::<ReadTextFileArguments>(params.get()) {
                    Ok(arguments) => {
                        let fut = self.delegate.read_text_file(arguments);
                        let outgoing_tx = self.outgoing_tx.clone();
                        (self.spawn)(
                            async move {
                                outgoing_tx
                                    .unbounded_send(OutgoingMessage::Response {
                                        id,
                                        result: fut.await.map(ClientResponse::ReadTextFile).into(),
                                    })
                                    .ok();
                            }
                            .boxed_local(),
                        );

                        Ok(())
                    }
                    Err(err) => Err(Error::invalid_params().with_data(err.to_string())),
                }
            }
            _ => Err(Error::method_not_found()),
        }
    }

    fn notification(&self, notification: Self::Notification) -> Result<(), Error> {
        match notification {
            AgentNotification::SessionUpdate(session_notification) => {
                if let Some(callback) = &*self.on_session_update.lock() {
                    callback(session_notification);
                }
                Ok(())
            }
        }
    }
}
