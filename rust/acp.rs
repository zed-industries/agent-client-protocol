#[cfg(test)]
mod acp_tests;
mod schema;

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
pub use schema::*;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering::SeqCst},
    },
};

/// A connection to a separate agent process over the ACP protocol.
pub struct AgentConnection(Connection<AnyClientRequest, AnyAgentRequest>);

/// A connection to a separate client process over the ACP protocol.
pub struct ClientConnection(Connection<AnyAgentRequest, AnyClientRequest>);

impl AgentConnection {
    /// Connect to an agent process, handling any incoming requests
    /// using the given handler.
    pub fn connect_to_agent<H: 'static + Client>(
        handler: H,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<(), Error>>) {
        let handler = Arc::new(handler);
        let (connection, io_task) = Connection::new(
            Box::new(move |request| {
                let handler = handler.clone();
                async move { handler.call(request).await }.boxed_local()
            }),
            outgoing_bytes,
            incoming_bytes,
            spawn,
        );
        (Self(connection), io_task)
    }

    /// Send a request to the agent and wait for a response.
    pub fn request<R: AgentRequest + 'static>(
        &self,
        params: R,
    ) -> impl Future<Output = Result<R::Response, Error>> {
        let params = params.into_any();
        let result = self.0.request(params.method_name(), params);
        async move {
            let result = result.await?;
            R::response_from_any(result)
        }
    }
}

impl ClientConnection {
    pub fn connect_to_client<H: 'static + Agent>(
        handler: H,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<(), Error>>) {
        let handler = Arc::new(handler);
        let (connection, io_task) = Connection::new(
            Box::new(move |request| {
                let handler = handler.clone();
                async move { handler.call(request).await }.boxed_local()
            }),
            outgoing_bytes,
            incoming_bytes,
            spawn,
        );
        (Self(connection), io_task)
    }

    pub fn request<R: ClientRequest>(
        &self,
        params: R,
    ) -> impl use<R> + Future<Output = Result<R::Response, Error>> {
        let params = params.into_any();
        let result = self.0.request(params.method_name(), params);
        async move {
            let result = result.await?;
            R::response_from_any(result)
        }
    }
}

struct Connection<In, Out>
where
    In: AnyRequest,
    Out: AnyRequest,
{
    outgoing_tx: UnboundedSender<OutgoingMessage<Out, In::Response>>,
    response_senders: ResponseSenders<Out::Response>,
    next_id: AtomicI32,
}

type ResponseSenders<T> =
    Arc<Mutex<HashMap<i32, (&'static str, oneshot::Sender<Result<T, Error>>)>>>;

#[derive(Debug, Deserialize)]
struct IncomingMessage<'a> {
    id: i32,
    method: Option<&'a str>,
    params: Option<&'a RawValue>,
    result: Option<&'a RawValue>,
    error: Option<Error>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum OutgoingMessage<Req, Resp> {
    Request {
        id: i32,
        method: Box<str>,
        params: Req,
    },
    OkResponse {
        id: i32,
        result: Resp,
    },
    ErrorResponse {
        id: i32,
        error: Error,
    },
}

#[derive(Serialize)]
pub struct JsonRpcMessage<Req, Resp> {
    pub jsonrpc: &'static str,
    #[serde(flatten)]
    message: OutgoingMessage<Req, Resp>,
}

type ResponseHandler<In, Resp> =
    Box<dyn 'static + Fn(In) -> LocalBoxFuture<'static, Result<Resp, Error>>>;

impl<In, Out> Connection<In, Out>
where
    In: AnyRequest,
    Out: AnyRequest,
{
    fn new(
        request_handler: ResponseHandler<In, In::Response>,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<(), Error>>) {
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded();
        let (incoming_tx, incoming_rx) = mpsc::unbounded();
        let this = Self {
            response_senders: ResponseSenders::default(),
            outgoing_tx: outgoing_tx.clone(),
            next_id: AtomicI32::new(0),
        };
        Self::handle_incoming(outgoing_tx, incoming_rx, request_handler, spawn);
        let io_task = Self::handle_io(
            outgoing_rx,
            incoming_tx,
            this.response_senders.clone(),
            outgoing_bytes,
            incoming_bytes,
        );
        (this, io_task)
    }

    fn request(
        &self,
        method: &'static str,
        params: Out,
    ) -> impl use<In, Out> + Future<Output = Result<Out::Response, Error>> {
        let (tx, rx) = oneshot::channel();
        let id = self.next_id.fetch_add(1, SeqCst);
        self.response_senders.lock().insert(id, (method, tx));
        if self
            .outgoing_tx
            .unbounded_send(OutgoingMessage::Request {
                id,
                method: method.into(),
                params,
            })
            .is_err()
        {
            self.response_senders.lock().remove(&id);
        }
        async move {
            rx.await
                .map_err(|e| Error::internal_error().with_data(e.to_string()))?
        }
    }

    async fn handle_io(
        mut outgoing_rx: UnboundedReceiver<OutgoingMessage<Out, In::Response>>,
        incoming_tx: UnboundedSender<(i32, In)>,
        response_senders: ResponseSenders<Out::Response>,
        mut outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
    ) -> Result<(), Error> {
        let mut output_reader = BufReader::new(incoming_bytes);
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
                bytes_read = output_reader.read_line(&mut incoming_line).fuse() => {
                    if bytes_read.map_err(Error::into_internal_error)? == 0 {
                        break
                    }
                    log::trace!("recv: {}", &incoming_line);
                    match serde_json::from_str::<IncomingMessage>(&incoming_line) {
                        Ok(message) => {
                            if let Some(method) = message.method {
                                match In::from_method_and_params(method, message.params.unwrap_or(RawValue::NULL)) {
                                    Ok(params) => {
                                        incoming_tx.unbounded_send((message.id, params)).ok();
                                    }
                                    Err(error) => {
                                        log::error!("failed to parse incoming {method} message params: {error}. Raw: {incoming_line}");
                                    }
                                }
                            } else if let Some(error) = message.error {
                                if let Some((_, tx)) = response_senders.lock().remove(&message.id) {
                                    tx.send(Err(error)).ok();
                                }
                            } else {
                                let result = message.result.unwrap_or(RawValue::NULL);
                                if let Some((method, tx)) = response_senders.lock().remove(&message.id) {
                                    match Out::response_from_method_and_result(method, result) {
                                        Ok(result) => {
                                            tx.send(Ok(result)).ok();
                                        }
                                        Err(error) => {
                                            log::error!("failed to parse {method} message result: {error}. Raw: {result}");
                                        }
                                    }
                                }
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
        response_senders.lock().clear();
        Ok(())
    }

    fn handle_incoming(
        outgoing_tx: UnboundedSender<OutgoingMessage<Out, In::Response>>,
        mut incoming_rx: UnboundedReceiver<(i32, In)>,
        incoming_handler: ResponseHandler<In, In::Response>,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) {
        let spawn = Rc::new(spawn);
        let spawn2 = spawn.clone();
        spawn(
            async move {
                while let Some((id, params)) = incoming_rx.next().await {
                    let result = incoming_handler(params);
                    let outgoing_tx = outgoing_tx.clone();
                    spawn2(
                        async move {
                            let result = result.await;
                            match result {
                                Ok(result) => {
                                    outgoing_tx
                                        .unbounded_send(OutgoingMessage::OkResponse { id, result })
                                        .ok();
                                }
                                Err(error) => {
                                    outgoing_tx
                                        .unbounded_send(OutgoingMessage::ErrorResponse {
                                            id,
                                            error: Error::into_internal_error(error),
                                        })
                                        .ok();
                                }
                            }
                        }
                        .boxed_local(),
                    )
                }
            }
            .boxed_local(),
        )
    }
}
