use std::{
    any::Any,
    collections::HashMap,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    },
};

use anyhow::Result;
use futures::{
    AsyncBufReadExt as _, AsyncRead, AsyncWrite, AsyncWriteExt as _, FutureExt as _,
    StreamExt as _,
    channel::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    future::LocalBoxFuture,
    io::BufReader,
    select_biased,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::value::RawValue;

use crate::stream_broadcast::{StreamBroadcast, StreamSender};
use crate::{Error, StreamReceiver};

pub struct RpcConnection<Local: Side, Remote: Side> {
    outgoing_tx: UnboundedSender<OutgoingMessage<Local, Remote>>,
    pending_responses: Arc<Mutex<HashMap<i32, PendingResponse>>>,
    next_id: AtomicI32,
    broadcast: StreamBroadcast,
}

struct PendingResponse {
    deserialize: fn(&serde_json::value::RawValue) -> Result<Box<dyn Any + Send>, Error>,
    respond: oneshot::Sender<Result<Box<dyn Any + Send>, Error>>,
}

impl<Local, Remote> RpcConnection<Local, Remote>
where
    Local: Side + 'static,
    Remote: Side + 'static,
{
    pub fn new<Handler>(
        handler: Handler,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl futures::Future<Output = Result<()>>)
    where
        Handler: MessageHandler<Local> + 'static,
    {
        let (incoming_tx, incoming_rx) = mpsc::unbounded();
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded();

        let pending_responses = Arc::new(Mutex::new(HashMap::default()));
        let (broadcast_tx, broadcast) = StreamBroadcast::new();

        let io_task = {
            let pending_responses = pending_responses.clone();
            async move {
                let result = Self::handle_io(
                    incoming_tx,
                    outgoing_rx,
                    outgoing_bytes,
                    incoming_bytes,
                    pending_responses.clone(),
                    broadcast_tx,
                )
                .await;
                pending_responses.lock().clear();
                result
            }
        };

        Self::handle_incoming(outgoing_tx.clone(), incoming_rx, handler, spawn);

        let this = Self {
            outgoing_tx,
            pending_responses,
            next_id: AtomicI32::new(0),
            broadcast,
        };

        (this, io_task)
    }

    pub fn subscribe(&self) -> StreamReceiver {
        self.broadcast.receiver()
    }

    pub fn notify(
        &self,
        method: impl Into<Arc<str>>,
        params: Option<Remote::InNotification>,
    ) -> Result<(), Error> {
        self.outgoing_tx
            .unbounded_send(OutgoingMessage::Notification {
                method: method.into(),
                params,
            })
            .map_err(|_| Error::internal_error().with_data("failed to send notification"))
    }

    pub fn request<Out: DeserializeOwned + Send + 'static>(
        &self,
        method: impl Into<Arc<str>>,
        params: Option<Remote::InRequest>,
    ) -> impl Future<Output = Result<Out, Error>> {
        let (tx, rx) = oneshot::channel();
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.pending_responses.lock().insert(
            id,
            PendingResponse {
                deserialize: |value| {
                    serde_json::from_str::<Out>(value.get())
                        .map(|out| Box::new(out) as _)
                        .map_err(|_| {
                            Error::internal_error().with_data("failed to deserialize response")
                        })
                },
                respond: tx,
            },
        );

        if self
            .outgoing_tx
            .unbounded_send(OutgoingMessage::Request {
                id,
                method: method.into(),
                params,
            })
            .is_err()
        {
            self.pending_responses.lock().remove(&id);
        }
        async move {
            let result = rx
                .await
                .map_err(|_| Error::internal_error().with_data("server shut down unexpectedly"))??
                .downcast::<Out>()
                .map_err(|_| Error::internal_error().with_data("failed to deserialize response"))?;

            Ok(*result)
        }
    }

    async fn handle_io(
        incoming_tx: UnboundedSender<IncomingMessage<Local>>,
        mut outgoing_rx: UnboundedReceiver<OutgoingMessage<Local, Remote>>,
        mut outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        pending_responses: Arc<Mutex<HashMap<i32, PendingResponse>>>,
        broadcast: StreamSender,
    ) -> Result<()> {
        // TODO: Create nicer abstraction for broadcast
        let mut input_reader = BufReader::new(incoming_bytes);
        let mut outgoing_line = Vec::new();
        let mut incoming_line = String::new();
        loop {
            select_biased! {
                message = outgoing_rx.next() => {
                    if let Some(message) = message {
                        outgoing_line.clear();
                        serde_json::to_writer(&mut outgoing_line, &JsonRpcMessage::wrap(&message)).map_err(Error::into_internal_error)?;
                        log::trace!("send: {}", String::from_utf8_lossy(&outgoing_line));
                        outgoing_line.push(b'\n');
                        outgoing_bytes.write_all(&outgoing_line).await.ok();
                        broadcast.outgoing(&message);
                    } else {
                        break;
                    }
                }
                bytes_read = input_reader.read_line(&mut incoming_line).fuse() => {
                    if bytes_read.map_err(Error::into_internal_error)? == 0 {
                        break
                    }
                    log::trace!("recv: {}", &incoming_line);

                    match serde_json::from_str::<RawIncomingMessage>(&incoming_line) {
                        Ok(message) => {
                            if let Some(id) = message.id {
                                if let Some(method) = message.method {
                                    // Request
                                    match Local::decode_request(method, message.params) {
                                        Ok(request) => {
                                            broadcast.incoming_request(id, method, &request);
                                            incoming_tx.unbounded_send(IncomingMessage::Request { id, request }).ok();
                                        }
                                        Err(err) => {
                                            outgoing_line.clear();
                                            let error_response = OutgoingMessage::<Local, Remote>::Response {
                                                id,
                                                result: ResponseResult::Error(err),
                                            };

                                            serde_json::to_writer(&mut outgoing_line, &JsonRpcMessage::wrap(&error_response))?;
                                            log::trace!("send: {}", String::from_utf8_lossy(&outgoing_line));
                                            outgoing_line.push(b'\n');
                                            outgoing_bytes.write_all(&outgoing_line).await.ok();
                                            broadcast.outgoing(&error_response);
                                        }
                                    }
                                } else if let Some(pending_response) = pending_responses.lock().remove(&id) {
                                    // Response
                                    if let Some(result_value) = message.result {
                                        broadcast.incoming_response(id, Ok(Some(result_value)));

                                        let result = (pending_response.deserialize)(result_value);
                                        pending_response.respond.send(result).ok();
                                    } else if let Some(error) = message.error {
                                        broadcast.incoming_response(id, Err(&error));

                                        pending_response.respond.send(Err(error)).ok();
                                    } else {
                                        broadcast.incoming_response(id, Ok(None));

                                        let result = (pending_response.deserialize)(&RawValue::from_string("null".into()).unwrap());
                                        pending_response.respond.send(result).ok();
                                    }
                                } else {
                                    log::error!("received response for unknown request id: {id}");
                                }
                            } else if let Some(method) = message.method {
                                // Notification
                                match Local::decode_notification(method, message.params) {
                                    Ok(notification) => {
                                        broadcast.incoming_notification(method, &notification);
                                        incoming_tx.unbounded_send(IncomingMessage::Notification { notification }).ok();
                                    }
                                    Err(err) => {
                                        log::error!("failed to decode {:?}: {err}", message.params);
                                    }
                                }
                            } else {
                                log::error!("received message with neither id nor method");
                            }
                        }
                        Err(error) => {
                            log::error!("failed to parse incoming message: {error}. Raw: {incoming_line}");
                        }
                    }
                    incoming_line.clear();
                }
            }
        }
        Ok(())
    }

    fn handle_incoming<Handler: MessageHandler<Local> + 'static>(
        outgoing_tx: UnboundedSender<OutgoingMessage<Local, Remote>>,
        mut incoming_rx: UnboundedReceiver<IncomingMessage<Local>>,
        handler: Handler,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) {
        let spawn = Rc::new(spawn);
        let handler = Rc::new(handler);
        spawn({
            let spawn = spawn.clone();
            async move {
                while let Some(message) = incoming_rx.next().await {
                    match message {
                        IncomingMessage::Request { id, request } => {
                            let outgoing_tx = outgoing_tx.clone();
                            let handler = handler.clone();
                            spawn(
                                async move {
                                    let result = handler.handle_request(request).await.into();
                                    outgoing_tx
                                        .unbounded_send(OutgoingMessage::Response { id, result })
                                        .ok();
                                }
                                .boxed_local(),
                            );
                        }
                        IncomingMessage::Notification { notification } => {
                            let handler = handler.clone();
                            spawn(
                                async move {
                                    if let Err(err) =
                                        handler.handle_notification(notification).await
                                    {
                                        log::error!("failed to handle notification: {err:?}");
                                    }
                                }
                                .boxed_local(),
                            );
                        }
                    }
                }
            }
            .boxed_local()
        });
    }
}

#[derive(Deserialize)]
struct RawIncomingMessage<'a> {
    id: Option<i32>,
    method: Option<&'a str>,
    params: Option<&'a RawValue>,
    result: Option<&'a RawValue>,
    error: Option<Error>,
}

enum IncomingMessage<Local: Side> {
    Request { id: i32, request: Local::InRequest },
    Notification { notification: Local::InNotification },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum OutgoingMessage<Local: Side, Remote: Side> {
    Request {
        id: i32,
        method: Arc<str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Remote::InRequest>,
    },
    Response {
        id: i32,
        #[serde(flatten)]
        result: ResponseResult<Local::OutResponse>,
    },
    Notification {
        method: Arc<str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Remote::InNotification>,
    },
}

/// Either [`OutgoingMessage`] or [`IncomingMessage`] with `"jsonrpc": "2.0"` specified as
/// [required by JSON-RPC 2.0 Specification][1].
///
/// [1]: https://www.jsonrpc.org/specification#compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcMessage<M> {
    jsonrpc: &'static str,
    #[serde(flatten)]
    message: M,
}

impl<M> JsonRpcMessage<M> {
    /// Used version of [JSON-RPC protocol].
    ///
    /// [JSON-RPC]: https://www.jsonrpc.org
    pub const VERSION: &'static str = "2.0";

    /// Wraps the provided [`OutgoingMessage`] or [`IncomingMessage`] into a versioned
    /// [`JsonRpcMessage`].
    #[must_use]
    pub fn wrap(message: M) -> Self {
        Self {
            jsonrpc: Self::VERSION,
            message,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ResponseResult<Res> {
    Result(Res),
    Error(Error),
}

impl<T> From<Result<T, Error>> for ResponseResult<T> {
    fn from(result: Result<T, Error>) -> Self {
        match result {
            Ok(value) => ResponseResult::Result(value),
            Err(error) => ResponseResult::Error(error),
        }
    }
}

pub trait Side: Clone {
    type InRequest: Clone + Serialize + DeserializeOwned + 'static;
    type OutResponse: Clone + Serialize + DeserializeOwned + 'static;
    type InNotification: Clone + Serialize + DeserializeOwned + 'static;

    fn decode_request(method: &str, params: Option<&RawValue>) -> Result<Self::InRequest, Error>;

    fn decode_notification(
        method: &str,
        params: Option<&RawValue>,
    ) -> Result<Self::InNotification, Error>;
}

pub trait MessageHandler<Local: Side> {
    fn handle_request(
        &self,
        request: Local::InRequest,
    ) -> impl Future<Output = Result<Local::OutResponse, Error>>;

    fn handle_notification(
        &self,
        notification: Local::InNotification,
    ) -> impl Future<Output = Result<(), Error>>;
}
