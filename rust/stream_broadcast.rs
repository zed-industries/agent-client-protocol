use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;
use serde_json::value::RawValue;

use crate::{
    Error,
    rpc::{OutgoingMessage, ResponseResult, Side},
};

#[derive(Clone)]
pub struct StreamMessage {
    pub direction: StreamMessageDirection,
    pub message: StreamMessageContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMessageDirection {
    Incoming,
    Outgoing,
}

#[derive(Clone)]
pub enum StreamMessageContent {
    Request {
        id: i32,
        method: Arc<str>,
        params: Option<serde_json::Value>,
    },
    Response {
        id: i32,
        result: Result<Option<serde_json::Value>, Error>,
    },
    Notification {
        method: Arc<str>,
        params: Option<serde_json::Value>,
    },
}

pub struct StreamReceiver(async_broadcast::Receiver<StreamMessage>);

impl StreamReceiver {
    pub async fn recv(&mut self) -> Result<StreamMessage> {
        Ok(self.0.recv().await?)
    }
}

pub struct StreamSender(async_broadcast::Sender<StreamMessage>);

impl StreamSender {
    pub fn outgoing<L: Side, R: Side>(&self, message: &OutgoingMessage<L, R>) {
        if self.0.receiver_count() == 0 {
            return;
        }

        let message = StreamMessage {
            direction: StreamMessageDirection::Outgoing,
            message: match message {
                OutgoingMessage::Request { id, method, params } => StreamMessageContent::Request {
                    id: *id,
                    method: (*method).into(),
                    params: serde_json::to_value(params).ok(),
                },
                OutgoingMessage::Response { id, result } => StreamMessageContent::Response {
                    id: *id,
                    result: match result {
                        ResponseResult::Result(value) => Ok(serde_json::to_value(value).ok()),
                        ResponseResult::Error(error) => Err(error.clone()),
                    },
                },
                OutgoingMessage::Notification { method, params } => {
                    StreamMessageContent::Notification {
                        method: (*method).into(),
                        params: serde_json::to_value(params).ok(),
                    }
                }
            },
        };

        self.0.try_broadcast(message).ok();
    }

    pub fn incoming_request(&self, id: i32, method: impl Into<Arc<str>>, params: &impl Serialize) {
        if self.0.receiver_count() == 0 {
            return;
        }

        let message = StreamMessage {
            direction: StreamMessageDirection::Incoming,
            message: StreamMessageContent::Request {
                id,
                method: method.into(),
                params: serde_json::to_value(params).ok(),
            },
        };

        self.0.try_broadcast(message).ok();
    }

    pub fn incoming_response(&self, id: i32, result: Result<Option<&RawValue>, &Error>) {
        if self.0.receiver_count() == 0 {
            return;
        }

        let result = match result {
            Ok(Some(value)) => Ok(serde_json::from_str(value.get()).ok()),
            Ok(None) => Ok(None),
            Err(err) => Err(err.clone()),
        };

        let message = StreamMessage {
            direction: StreamMessageDirection::Incoming,
            message: StreamMessageContent::Response { id, result },
        };

        self.0.try_broadcast(message).ok();
    }

    pub fn incoming_notification(&self, method: impl Into<Arc<str>>, params: &impl Serialize) {
        if self.0.receiver_count() == 0 {
            return;
        }

        let message = StreamMessage {
            direction: StreamMessageDirection::Incoming,
            message: StreamMessageContent::Notification {
                method: method.into(),
                params: serde_json::to_value(params).ok(),
            },
        };

        self.0.try_broadcast(message).ok();
    }
}

pub struct StreamBroadcast {
    receiver: async_broadcast::InactiveReceiver<StreamMessage>,
}

impl StreamBroadcast {
    pub fn new() -> (StreamSender, Self) {
        let (sender, receiver) = async_broadcast::broadcast(1);
        (
            StreamSender(sender),
            Self {
                receiver: receiver.deactivate(),
            },
        )
    }

    pub fn receiver(&self) -> StreamReceiver {
        let was_empty = self.receiver.receiver_count() == 0;
        let mut new_receiver = self.receiver.activate_cloned();
        if was_empty {
            // Grow capacity once we actually have a receiver
            new_receiver.set_capacity(64);
        }
        StreamReceiver(new_receiver)
    }
}

impl<Local: Side, Remote: Side> From<OutgoingMessage<Local, Remote>> for StreamMessage {
    fn from(message: OutgoingMessage<Local, Remote>) -> Self {
        Self {
            direction: StreamMessageDirection::Outgoing,
            message: match message {
                OutgoingMessage::Request { id, method, params } => StreamMessageContent::Request {
                    id,
                    method: method.into(),
                    params: serde_json::to_value(params).ok(),
                },
                OutgoingMessage::Response { id, result } => StreamMessageContent::Response {
                    id,
                    result: match result {
                        ResponseResult::Result(value) => Ok(serde_json::to_value(value).ok()),
                        ResponseResult::Error(error) => Err(error),
                    },
                },
                OutgoingMessage::Notification { method, params } => {
                    StreamMessageContent::Notification {
                        method: method.into(),
                        params: serde_json::to_value(params).ok(),
                    }
                }
            },
        }
    }
}
