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
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::{
    collections::HashMap,
    fmt::Display,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering::SeqCst},
    },
};

struct Connection<In, Out> {
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
        #[serde(skip_serializing_if = "is_none_or_null")]
        params: Option<Req>,
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

fn is_none_or_null<T: Serialize>(opt: &Option<T>) -> bool {
    match opt {
        None => true,
        Some(value) => {
            matches!(serde_json::to_value(value), Ok(serde_json::Value::Null))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum JsonSchemaVersion {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Serialize)]
struct OutJsonRpcMessage<Req, Resp> {
    jsonrpc: JsonSchemaVersion,
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
                params: Some(params),
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
                        let message = OutJsonRpcMessage {
                            jsonrpc: JsonSchemaVersion::V2,
                            message,
                        };
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
                        Ok(IncomingMessage { id, method, params, result, error }) => {
                            if let Some(method) = method {
                                match In::from_method_and_params(method, params.unwrap_or(RawValue::NULL)) {
                                    Ok(params) => {
                                        incoming_tx.unbounded_send((id, params)).ok();
                                    }
                                    Err(error) => {
                                        log::error!("failed to parse incoming {method} message params: {error}. Raw: {incoming_line}");
                                    }
                                }
                            } else if let Some(error) = error {
                                if let Some((_, tx)) = response_senders.lock().remove(&id) {
                                    tx.send(Err(error)).ok();
                                }
                            } else {
                                let result = result.unwrap_or(RawValue::NULL);
                                if let Some((method, tx)) = response_senders.lock().remove(&id) {
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Error {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Error {
    pub fn new(code: impl Into<(i32, String)>) -> Self {
        let (code, message) = code.into();
        Error {
            code,
            message,
            data: None,
        }
    }

    pub fn with_data(mut self, data: impl Into<serde_json::Value>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
    pub fn parse_error() -> Self {
        Error::new(ErrorCode::PARSE_ERROR)
    }

    /// The JSON sent is not a valid Request object.
    pub fn invalid_request() -> Self {
        Error::new(ErrorCode::INVALID_REQUEST)
    }

    /// The method does not exist / is not available.
    pub fn method_not_found() -> Self {
        Error::new(ErrorCode::METHOD_NOT_FOUND)
    }

    /// Invalid method parameter(s).
    pub fn invalid_params() -> Self {
        Error::new(ErrorCode::INVALID_PARAMS)
    }

    /// Internal JSON-RPC error.
    pub fn internal_error() -> Self {
        Error::new(ErrorCode::INTERNAL_ERROR)
    }

    pub fn into_internal_error(err: impl std::error::Error) -> Self {
        Error::internal_error().with_data(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorCode {
    code: i32,
    message: &'static str,
}

impl ErrorCode {
    pub const PARSE_ERROR: ErrorCode = ErrorCode {
        code: -32700,
        message: "Parse error",
    };

    pub const INVALID_REQUEST: ErrorCode = ErrorCode {
        code: -32600,
        message: "Invalid Request",
    };

    pub const METHOD_NOT_FOUND: ErrorCode = ErrorCode {
        code: -32601,
        message: "Method not found",
    };

    pub const INVALID_PARAMS: ErrorCode = ErrorCode {
        code: -32602,
        message: "Invalid params",
    };

    pub const INTERNAL_ERROR: ErrorCode = ErrorCode {
        code: -32603,
        message: "Internal error",
    };
}

impl From<ErrorCode> for (i32, String) {
    fn from(error_code: ErrorCode) -> Self {
        (error_code.code, error_code.message.to_string())
    }
}

impl From<ErrorCode> for Error {
    fn from(error_code: ErrorCode) -> Self {
        Error::new(error_code)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.message.is_empty() {
            write!(f, "{}", self.code)?;
        } else {
            write!(f, "{}", self.message)?;
        }

        if let Some(data) = &self.data {
            write!(f, ": {data}")?;
        }

        Ok(())
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::into_internal_error(error.deref())
    }
}
