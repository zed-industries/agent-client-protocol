use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
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
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::Error;

pub trait Side {
    type Request: Serialize + DeserializeOwned + 'static;
    type Response: Serialize + DeserializeOwned + 'static;
    type Notification: Serialize + DeserializeOwned + 'static;
}

pub trait Local: Side {
    type Peer: Side;

    fn handle_request<'a>(
        &'a self,
        request: Self::Request,
    ) -> LocalBoxFuture<'a, Result<Self::Response, Error>>;

    fn handle_notification<'a>(
        &'a self,
        notification: <Self::Peer as Side>::Notification,
    ) -> LocalBoxFuture<'a, ()>;
}

pub struct RpcConnection<L: Local> {
    outgoing_tx: UnboundedSender<OutgoingMessage<L>>,
    pending_responses: PendingResponses<L>,
    next_id: AtomicI32,
}

type PendingResponses<L> =
    Rc<RefCell<HashMap<i32, oneshot::Sender<Result<ResponseFromPeer<L>, Error>>>>>;

impl<L> RpcConnection<L>
where
    L: Local + 'static,
{
    pub fn new(
        local: L,
        outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded();
        let (incoming_tx, incoming_rx) = mpsc::unbounded();

        let io_task = Self::handle_io(outgoing_rx, incoming_tx, outgoing_bytes, incoming_bytes);

        let pending_responses = Rc::new(RefCell::new(HashMap::default()));
        Self::handle_incoming(
            outgoing_tx.clone(),
            incoming_rx,
            local,
            pending_responses.clone(),
            spawn,
        );

        let this = Self {
            outgoing_tx,
            pending_responses,
            next_id: AtomicI32::new(0),
        };

        (this, io_task)
    }

    fn request(
        &self,
        request: RequestToPeer<L>,
    ) -> impl use<L> + Future<Output = Result<ResponseFromPeer<L>, Error>> {
        let (tx, rx) = oneshot::channel();
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.pending_responses.borrow_mut().insert(id, tx);
        if self
            .outgoing_tx
            .unbounded_send(Message::Request { id, request })
            .is_err()
        {
            self.pending_responses.borrow_mut().remove(&id);
        }
        async move {
            rx.await
                .map_err(|e| Error::internal_error().with_data(e.to_string()))?
        }
    }

    async fn handle_io(
        mut outgoing_rx: UnboundedReceiver<OutgoingMessage<L>>,
        incoming_tx: UnboundedSender<IncomingMessage<L>>,
        mut outgoing_bytes: impl Unpin + AsyncWrite,
        incoming_bytes: impl Unpin + AsyncRead,
    ) -> Result<()> {
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
                    match serde_json::from_str::<IncomingMessage<L>>(&incoming_line) {
                        Ok(message) => {
                            incoming_tx.unbounded_send(message).ok();
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

    fn handle_incoming(
        outgoing_tx: UnboundedSender<OutgoingMessage<L>>,
        mut incoming_rx: UnboundedReceiver<IncomingMessage<L>>,
        local: L,
        pending_responses: PendingResponses<L>,
        spawn: impl Fn(LocalBoxFuture<'static, ()>) + 'static,
    ) {
        let spawn = Rc::new(spawn);
        let local_peer = Rc::new(local);

        spawn({
            let spawn = spawn.clone();
            async move {
                while let Some(message) = incoming_rx.next().await {
                    let outgoing_tx = outgoing_tx.clone();
                    let local_peer = local_peer.clone();
                    let pending_responses = pending_responses.clone();
                    spawn(
                        async move {
                            match message {
                                Message::Request { id, request } => {
                                    let result = match local_peer.handle_request(request).await {
                                        Ok(result) => ResponseResult::Result(result),
                                        Err(error) => ResponseResult::Error(error),
                                    };

                                    outgoing_tx
                                        .unbounded_send(Message::Response { id, result })
                                        .ok();
                                }
                                Message::Notification { notification } => {
                                    local_peer.handle_notification(notification).await;
                                }
                                Message::Response { id, result } => {
                                    if let Some(pending_response) =
                                        pending_responses.borrow_mut().remove(&id)
                                    {
                                        match result {
                                            ResponseResult::Result(result) => {
                                                pending_response.send(Ok(result)).ok();
                                            }
                                            ResponseResult::Error(error) => {
                                                pending_response.send(Err(error)).ok();
                                            }
                                        }
                                    } else {
                                        log::error!(
                                            "Received response for unknown request ID: {}",
                                            id
                                        );
                                    }
                                }
                            }
                        }
                        .boxed_local(),
                    )
                }
            }
            .boxed_local()
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message<Req, Res, N> {
    Request {
        id: i32,
        #[serde(flatten)]
        request: Req,
    },
    Notification {
        #[serde(flatten)]
        notification: N,
    },
    Response {
        id: i32,
        #[serde(flatten)]
        result: ResponseResult<Res>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ResponseResult<Res> {
    Result(Res),
    Error(Error),
}

type RequestToPeer<L> = <<L as Local>::Peer as Side>::Request;
type RequestFromPeer<L> = <L as Side>::Request;
type ResponseToPeer<L> = <L as Side>::Response;
type ResponseFromPeer<L> = <<L as Local>::Peer as Side>::Response;
type NotificationToPeer<L> = <L as Side>::Notification;
type NotificationFromPeer<L> = <<L as Local>::Peer as Side>::Notification;

type OutgoingMessage<L> = Message<RequestToPeer<L>, ResponseToPeer<L>, NotificationToPeer<L>>;
type IncomingMessage<L> = Message<RequestFromPeer<L>, ResponseFromPeer<L>, NotificationFromPeer<L>>;

// mod tests {
//     use crate::{ClientRequest, LoadSessionOutput, SessionId};

//     use super::*;
//     use serde_json::json;

//     use std::path::Path;

//     #[test]
//     fn test_deserialize_request_message() {
//         let message: Message<ClientRequest, ()> = serde_json::from_value(json!({
//             "id": 0,
//             "method": "writeTextFile",
//             "params": { "sessionId": "1234", "path": "foo.txt", "content": "hello" }
//         }))
//         .unwrap();

//         let Message::Request {
//             id,
//             request: ClientRequest::WriteTextFile(args),
//         } = message
//         else {
//             panic!("Got: {:?}", message);
//         };

//         assert_eq!(id, 0);
//         assert_eq!(args.session_id, SessionId("1234".into()));
//         assert_eq!(args.path, Path::new("foo.txt"));
//         assert_eq!(args.content, "hello");
//     }

//     #[test]
//     fn test_deserialize_response_ok() {
//         let message: Message<ClientRequest, serde_json::Value> = serde_json::from_value(json!({
//             "id": 42,
//             "result": {
//                 "authRequired": false,
//                 "authMethods": []
//             }
//         }))
//         .unwrap();

//         let Message::ResponseOk { id, result } = message else {
//             panic!("Got: {:?}", message);
//         };

//         let output: LoadSessionOutput = serde_json::from_value(result).unwrap();

//         assert_eq!(id, 42);
//         assert_eq!(output.auth_required, false);
//         assert_eq!(output.auth_methods.len(), 0);
//     }

//     #[test]
//     fn test_deserialize_response_err() {
//         let message: Message<ClientRequest, ()> = serde_json::from_value(json!({
//             "id": 123,
//             "error": {
//                 "code": -32602,
//                 "message": "Invalid params",
//                 "data": "Missing required field"
//             }
//         }))
//         .unwrap();

//         let Message::ResponseErr { id, error } = message else {
//             panic!("Got: {:?}", message);
//         };

//         assert_eq!(id, 123);
//         assert_eq!(error.code, -32602);
//         assert_eq!(error.message, "Invalid params");
//         assert_eq!(error.data, Some(json!("Missing required field")));
//     }
// }
