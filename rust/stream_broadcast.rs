//! JSON-RPC Stream broadcasting for debugging and monitoring communication.
//!
//! This module provides functionality to observe the JSON-RPC message stream between
//! clients and agents. It's primarily used for debugging, logging, and building
//! development tools that need to monitor the protocol communication.

use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;
use serde_json::value::RawValue;

use crate::{
    Error,
    rpc::{OutgoingMessage, ResponseResult, Side},
};

/// A message that flows through the RPC stream.
///
/// This represents any JSON-RPC message (request, response, or notification)
/// along with its direction (incoming or outgoing).
///
/// Stream messages are used for observing and debugging the protocol communication
/// without interfering with the actual message handling.
#[derive(Debug, Clone)]
pub struct StreamMessage {
    /// The direction of the message relative to this side of the connection.
    pub direction: StreamMessageDirection,
    /// The actual content of the message.
    pub message: StreamMessageContent,
}

/// The direction of a message in the RPC stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMessageDirection {
    /// A message received from the other side of the connection.
    Incoming,
    /// A message sent to the other side of the connection.
    Outgoing,
}

/// The content of a stream message.
///
/// This enum represents the three types of JSON-RPC messages:
/// - Requests: Method calls that expect a response
/// - Responses: Replies to previous requests
/// - Notifications: One-way messages that don't expect a response
#[derive(Debug, Clone)]
pub enum StreamMessageContent {
    /// A JSON-RPC request message.
    Request {
        /// The unique identifier for this request.
        id: i32,
        /// The name of the method being called.
        method: Arc<str>,
        /// Optional parameters for the method.
        params: Option<serde_json::Value>,
    },
    /// A JSON-RPC response message.
    Response {
        /// The ID of the request this response is for.
        id: i32,
        /// The result of the request (success or error).
        result: Result<Option<serde_json::Value>, Error>,
    },
    /// A JSON-RPC notification message.
    Notification {
        /// The name of the notification method.
        method: Arc<str>,
        /// Optional parameters for the notification.
        params: Option<serde_json::Value>,
    },
}

/// A receiver for observing the message stream.
///
/// This allows you to receive copies of all messages flowing through the connection,
/// useful for debugging, logging, or building development tools.
///
/// # Example
///
/// ```no_run
/// use agent_client_protocol::{StreamReceiver, StreamMessageDirection};
///
/// async fn monitor_messages(mut receiver: StreamReceiver) {
///     while let Ok(message) = receiver.recv().await {
///         match message.direction {
///             StreamMessageDirection::Incoming => println!("← Received: {:?}", message.message),
///             StreamMessageDirection::Outgoing => println!("→ Sent: {:?}", message.message),
///         }
///     }
/// }
/// ```
pub struct StreamReceiver(async_broadcast::Receiver<StreamMessage>);

impl StreamReceiver {
    /// Receives the next message from the stream.
    ///
    /// This method will wait until a message is available or the sender is dropped.
    ///
    /// # Returns
    ///
    /// - `Ok(StreamMessage)` when a message is received
    /// - `Err` when the sender is dropped or the receiver is lagged
    pub async fn recv(&mut self) -> Result<StreamMessage> {
        Ok(self.0.recv().await?)
    }
}

/// Internal sender for broadcasting stream messages.
///
/// This is used internally by the RPC system to broadcast messages to all receivers.
/// You typically won't interact with this directly.
pub(crate) struct StreamSender(async_broadcast::Sender<StreamMessage>);

impl StreamSender {
    /// Broadcasts an outgoing message to all receivers.
    pub(crate) fn outgoing<L: Side, R: Side>(&self, message: &OutgoingMessage<L, R>) {
        if self.0.receiver_count() == 0 {
            return;
        }

        let message = StreamMessage {
            direction: StreamMessageDirection::Outgoing,
            message: match message {
                OutgoingMessage::Request { id, method, params } => StreamMessageContent::Request {
                    id: *id,
                    method: method.clone(),
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
                        method: method.clone(),
                        params: serde_json::to_value(params).ok(),
                    }
                }
            },
        };

        self.0.try_broadcast(message).ok();
    }

    /// Broadcasts an incoming request to all receivers.
    pub(crate) fn incoming_request(
        &self,
        id: i32,
        method: impl Into<Arc<str>>,
        params: &impl Serialize,
    ) {
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

    /// Broadcasts an incoming response to all receivers.
    pub(crate) fn incoming_response(&self, id: i32, result: Result<Option<&RawValue>, &Error>) {
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

    /// Broadcasts an incoming notification to all receivers.
    pub(crate) fn incoming_notification(
        &self,
        method: impl Into<Arc<str>>,
        params: &impl Serialize,
    ) {
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

/// A broadcast for observing RPC message streams.
///
/// This is used internally by the RPC connection to allow multiple receivers
/// to observe the message stream.
pub(crate) struct StreamBroadcast {
    receiver: async_broadcast::InactiveReceiver<StreamMessage>,
}

impl StreamBroadcast {
    /// Creates a new broadcast.
    ///
    /// Returns a sender for broadcasting messages and the broadcast instance
    /// for creating receivers.
    pub(crate) fn new() -> (StreamSender, Self) {
        let (sender, receiver) = async_broadcast::broadcast(1);
        (
            StreamSender(sender),
            Self {
                receiver: receiver.deactivate(),
            },
        )
    }

    /// Creates a new receiver for observing the message stream.
    ///
    /// Each receiver will get its own copy of every message.
    pub(crate) fn receiver(&self) -> StreamReceiver {
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
                    method,
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
                        method,
                        params: serde_json::to_value(params).ok(),
                    }
                }
            },
        }
    }
}
