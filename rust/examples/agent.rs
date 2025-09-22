//! A simple ACP agent server for educational purposes.
//!
//! The agent communicates with clients over stdio. To run it with logging:
//!
//! ```bash
//! RUST_LOG=info cargo run --example agent
//! ```
//!
//! To connect it to the example client from this crate:
//!
//! ```bash
//! cargo build --example agent && cargo run --example client -- target/debug/examples/agent
//! ```

use std::cell::Cell;

use agent_client_protocol::{
    self as acp, AuthenticateResponse, Client, ExtNotification, ExtRequest, ExtResponse,
    SessionNotification, SetSessionModeResponse,
};
use serde_json::json;
use tokio::sync::{mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt as _, TokioAsyncWriteCompatExt as _};

struct ExampleAgent {
    session_update_tx: mpsc::UnboundedSender<(acp::SessionNotification, oneshot::Sender<()>)>,
    next_session_id: Cell<u64>,
}

impl ExampleAgent {
    fn new(
        session_update_tx: mpsc::UnboundedSender<(acp::SessionNotification, oneshot::Sender<()>)>,
    ) -> Self {
        Self {
            session_update_tx,
            next_session_id: Cell::new(0),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl acp::Agent for ExampleAgent {
    async fn initialize(
        &self,
        arguments: acp::InitializeRequest,
    ) -> Result<acp::InitializeResponse, acp::Error> {
        log::info!("Received initialize request {arguments:?}");
        Ok(acp::InitializeResponse {
            protocol_version: acp::V1,
            agent_capabilities: acp::AgentCapabilities::default(),
            auth_methods: Vec::new(),
            meta: None,
        })
    }

    async fn authenticate(
        &self,
        arguments: acp::AuthenticateRequest,
    ) -> Result<AuthenticateResponse, acp::Error> {
        log::info!("Received authenticate request {arguments:?}");
        Ok(AuthenticateResponse::default())
    }

    async fn new_session(
        &self,
        arguments: acp::NewSessionRequest,
    ) -> Result<acp::NewSessionResponse, acp::Error> {
        log::info!("Received new session request {arguments:?}");
        let session_id = self.next_session_id.get();
        self.next_session_id.set(session_id + 1);
        Ok(acp::NewSessionResponse {
            session_id: acp::SessionId(session_id.to_string().into()),
            modes: None,
            #[cfg(feature = "unstable")]
            models: None,
            meta: None,
        })
    }

    async fn load_session(
        &self,
        arguments: acp::LoadSessionRequest,
    ) -> Result<acp::LoadSessionResponse, acp::Error> {
        log::info!("Received load session request {arguments:?}");
        Ok(acp::LoadSessionResponse {
            modes: None,
            #[cfg(feature = "unstable")]
            models: None,
            meta: None,
        })
    }

    async fn prompt(
        &self,
        arguments: acp::PromptRequest,
    ) -> Result<acp::PromptResponse, acp::Error> {
        log::info!("Received prompt request {arguments:?}");
        for content in ["Client sent: ".into()].into_iter().chain(arguments.prompt) {
            let (tx, rx) = oneshot::channel();
            self.session_update_tx
                .send((
                    SessionNotification {
                        session_id: arguments.session_id.clone(),
                        update: acp::SessionUpdate::AgentMessageChunk { content },
                        meta: None,
                    },
                    tx,
                ))
                .map_err(|_| acp::Error::internal_error())?;
            rx.await.map_err(|_| acp::Error::internal_error())?;
        }
        Ok(acp::PromptResponse {
            stop_reason: acp::StopReason::EndTurn,
            meta: None,
        })
    }

    async fn cancel(&self, args: acp::CancelNotification) -> Result<(), acp::Error> {
        log::info!("Received cancel request {args:?}");
        Ok(())
    }

    async fn set_session_mode(
        &self,
        args: acp::SetSessionModeRequest,
    ) -> Result<acp::SetSessionModeResponse, acp::Error> {
        log::info!("Received set session mode request {args:?}");
        Ok(SetSessionModeResponse::default())
    }

    #[cfg(feature = "unstable")]
    async fn set_session_model(
        &self,
        args: acp::SetSessionModelRequest,
    ) -> Result<acp::SetSessionModelResponse, acp::Error> {
        log::info!("Received select model request {args:?}");
        Ok(acp::SetSessionModelResponse::default())
    }

    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, acp::Error> {
        log::info!(
            "Received extension method call: method={}, params={:?}",
            args.method,
            args.params
        );
        Ok(serde_json::value::to_raw_value(&json!({"example": "response"}))?.into())
    }

    async fn ext_notification(&self, args: ExtNotification) -> Result<(), acp::Error> {
        log::info!(
            "Received extension notification: method={}, params={:?}",
            args.method,
            args.params
        );
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let outgoing = tokio::io::stdout().compat_write();
    let incoming = tokio::io::stdin().compat();

    // The AgentSideConnection will spawn futures onto our Tokio runtime.
    // LocalSet and spawn_local are used because the futures from the
    // agent-client-protocol crate are not Send.
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            // Start up the ExampleAgent connected to stdio.
            let (conn, handle_io) =
                acp::AgentSideConnection::new(ExampleAgent::new(tx), outgoing, incoming, |fut| {
                    tokio::task::spawn_local(fut);
                });
            // Kick off a background task to send the ExampleAgent's session notifications to the client.
            tokio::task::spawn_local(async move {
                while let Some((session_notification, tx)) = rx.recv().await {
                    let result = conn.session_notification(session_notification).await;
                    if let Err(e) = result {
                        log::error!("{e}");
                        break;
                    }
                    tx.send(()).ok();
                }
            });
            // Run until stdin/stdout are closed.
            handle_io.await
        })
        .await
}
