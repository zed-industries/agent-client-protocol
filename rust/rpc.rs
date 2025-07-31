use std::{
    any::Any,
    collections::HashMap,
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
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    future::LocalBoxFuture,
    io::BufReader,
    select_biased,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::value::RawValue;

use crate::Error;

pub trait RpcSide {
    type Request: Serialize + DeserializeOwned + 'static;
    type Response: Serialize + DeserializeOwned + 'static;
    type Notification: Serialize + DeserializeOwned + 'static;
}

pub struct RpcConnection<Local: RpcSide, Remote: RpcSide> {
    outgoing_tx: UnboundedSender<OutgoingMessage<Local, Remote>>,
    pending_responses: Arc<Mutex<HashMap<i32, PendingResponse>>>,
    next_id: AtomicI32,
}

struct PendingResponse {
    deserialize: fn(&serde_json::value::RawValue) -> Result<Box<dyn Any>, Error>,
    respond: oneshot::Sender<Result<Box<dyn Any>, Error>>,
}

impl<Local: RpcSide, Remote: RpcSide> RpcConnection<Local, Remote> {
    pub fn new<D>(
        dispatcher: D,
        outgoing_tx: UnboundedSender<OutgoingMessage<Local, Remote>>,
        outgoing_rx: UnboundedReceiver<OutgoingMessage<Local, Remote>>,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
    ) -> (Self, impl futures::Future<Output = Result<()>>)
    where
        D: Dispatcher<Notification = Remote::Notification> + 'static,
    {
        let pending_responses = Arc::new(Mutex::new(HashMap::default()));

        let io_task = Self::handle_io(
            outgoing_rx,
            outgoing_bytes,
            incoming_bytes,
            dispatcher,
            pending_responses.clone(),
        );

        let this = Self {
            outgoing_tx,
            pending_responses,
            next_id: AtomicI32::new(0),
        };

        (this, io_task)
    }

    pub(crate) fn notify(&self, notification: Local::Notification) -> Result<(), Error> {
        self.outgoing_tx
            .unbounded_send(OutgoingMessage::Notification { notification })
            .map_err(|_| Error::internal_error().with_data("failed to send notification"))
    }

    pub(crate) fn request<Out: DeserializeOwned + 'static>(
        &self,
        method: &'static str,
        params: Option<Remote::Request>,
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
            .unbounded_send(OutgoingMessage::Request { id, method, params })
            .is_err()
        {
            self.pending_responses.lock().remove(&id);
        }
        async move {
            let result = rx
                .await
                .map_err(|e| Error::internal_error().with_data(e.to_string()))??
                .downcast::<Out>()
                .map_err(|_| Error::internal_error().with_data("failed to deserialize response"))?;

            Ok(*result)
        }
    }

    async fn handle_io<D>(
        mut outgoing_rx: UnboundedReceiver<OutgoingMessage<Local, Remote>>,
        mut outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        dispatcher: D,
        pending_responses: Arc<Mutex<HashMap<i32, PendingResponse>>>,
    ) -> Result<()>
    where
        D: Dispatcher<Notification = Remote::Notification>,
    {
        let mut input_reader = BufReader::new(incoming_bytes);
        let mut outgoing_line = Vec::new();
        let mut incoming_line = String::new();
        loop {
            select_biased! {
                message = outgoing_rx.next() => {
                    if let Some(message) = message {
                        outgoing_line.clear();
                        serde_json::to_writer(&mut outgoing_line, &message).map_err(Error::into_internal_error)?;
                        log::trace!("send: {}", String::from_utf8_lossy(&outgoing_line));
                        outgoing_line.push(b'\n');
                        outgoing_bytes.write_all(&outgoing_line).await.ok();
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
                                    if let Err(error) = dispatcher.request(id, method, message.params) {
                                        // Send error response for failed request handling
                                        outgoing_line.clear();
                                        let error_response = OutgoingMessage::<Local, Remote>::Response {
                                            id,
                                            result: ResponseResult::Error(error),
                                        };
                                        if let Err(e) = serde_json::to_writer(&mut outgoing_line, &error_response) {
                                            log::error!("failed to serialize error response: {e}");
                                        } else {
                                            log::trace!("send: {}", String::from_utf8_lossy(&outgoing_line));
                                            outgoing_line.push(b'\n');
                                            outgoing_bytes.write_all(&outgoing_line).await.ok();
                                        }
                                    }
                                } else {
                                    if let Some(pending_response) = pending_responses.lock().remove(&id) {
                                        if let Some(result) = message.result {
                                            let result = (pending_response.deserialize)(result);
                                            pending_response.respond.send(result).ok();
                                        } else if let Some(error) = message.error {
                                            pending_response.respond.send(Err(error)).ok();
                                        } else {
                                            let result = (pending_response.deserialize)(&RawValue::from_string("null".into()).unwrap());
                                            pending_response.respond.send(result).ok();
                                        }

                                    } else {
                                        log::error!("received response for unknown request id: {id}");
                                    }
                                }
                            } else if let Some(method) = message.method {
                                if let Err(e) = dispatcher.notification(method, message.params) {
                                    log::error!("failed to handle notification '{}': {}", method, e);
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
}

#[derive(Deserialize)]
struct RawIncomingMessage<'a> {
    id: Option<i32>,
    method: Option<&'a str>,
    params: Option<&'a RawValue>,
    result: Option<&'a RawValue>,
    error: Option<Error>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutgoingMessage<Local: RpcSide, Remote: RpcSide> {
    Request {
        id: i32,
        method: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<Remote::Request>,
    },
    Response {
        id: i32,
        #[serde(flatten)]
        result: ResponseResult<Local::Response>,
    },
    Notification {
        #[serde(flatten)]
        notification: Local::Notification,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

pub trait Dispatcher {
    type Notification: DeserializeOwned;

    fn request(&self, id: i32, method: &str, params: Option<&RawValue>) -> Result<(), Error>;
    fn notification(&self, method: &str, params: Option<&RawValue>) -> Result<(), Error>;
}

#[macro_export]
macro_rules! dispatch_request {
    ($base:expr, $id:expr, $params:expr, $request_type:ty, $method:expr, $response_wrapper:expr) => {{
        let Some(params) = $params else {
            return Err($crate::Error::invalid_params());
        };

        match serde_json::from_str::<$request_type>(params.get()) {
            Ok(arguments) => {
                let fut = $method(&$base.delegate, arguments);
                let outgoing_tx = $base.outgoing_tx.clone();
                ($base.spawn)(::futures::FutureExt::boxed_local(async move {
                    outgoing_tx
                        .unbounded_send($crate::rpc::OutgoingMessage::Response {
                            id: $id,
                            result: fut.await.map($response_wrapper).into(),
                        })
                        .ok();
                }));

                Ok(())
            }
            Err(err) => Err($crate::Error::invalid_params().with_data(err.to_string())),
        }
    }};
}

#[macro_export]
macro_rules! dispatch_notification {
    ($method:expr, $params:expr, $params_type:ty, $handler:expr) => {{
        let Some(params) = $params else {
            return Err($crate::Error::invalid_params());
        };

        match serde_json::from_str::<$params_type>(params.get()) {
            Ok(arguments) => $handler(arguments),
            Err(err) => Err($crate::Error::invalid_params().with_data(err.to_string())),
        }
    }};
}

pub struct BaseDispatcher<D, Local: RpcSide, Remote: RpcSide> {
    pub delegate: D,
    pub spawn: Box<dyn Fn(LocalBoxFuture<'static, ()>) + 'static>,
    pub outgoing_tx: UnboundedSender<OutgoingMessage<Local, Remote>>,
}
