use std::cell::Cell;

use agent_client_protocol::{self as acp, Client, SessionNotification};
use anyhow::bail;
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};
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

impl acp::Agent for ExampleAgent {
    async fn initialize(
        &self,
        _arguments: acp::InitializeRequest,
    ) -> Result<acp::InitializeResponse, acp::Error> {
        log::info!("Received initialize request");
        Ok(acp::InitializeResponse {
            protocol_version: acp::V1,
            agent_capabilities: acp::AgentCapabilities::default(),
            auth_methods: Vec::new(),
        })
    }

    async fn authenticate(&self, _arguments: acp::AuthenticateRequest) -> Result<(), acp::Error> {
        log::info!("Received authenticate request");
        Ok(())
    }

    async fn new_session(
        &self,
        _arguments: acp::NewSessionRequest,
    ) -> Result<acp::NewSessionResponse, acp::Error> {
        log::info!("Received new session request");
        let session_id = self.next_session_id.get();
        self.next_session_id.set(session_id + 1);
        Ok(acp::NewSessionResponse {
            session_id: acp::SessionId(session_id.to_string().into()),
        })
    }

    async fn load_session(&self, _arguments: acp::LoadSessionRequest) -> Result<(), acp::Error> {
        log::info!("Received load session request");
        Err(acp::Error::method_not_found())
    }

    async fn prompt(
        &self,
        arguments: acp::PromptRequest,
    ) -> Result<acp::PromptResponse, acp::Error> {
        log::info!("Received prompt request");
        for content in ["Client sent: ".into()].into_iter().chain(arguments.prompt) {
            let (tx, rx) = oneshot::channel();
            self.session_update_tx
                .send((
                    SessionNotification {
                        session_id: arguments.session_id.clone(),
                        update: acp::SessionUpdate::AgentMessageChunk { content },
                    },
                    tx,
                ))
                .map_err(|_| acp::Error::internal_error())?;
            rx.await.map_err(|_| acp::Error::internal_error())?;
        }
        Ok(acp::PromptResponse {
            stop_reason: acp::StopReason::EndTurn,
        })
    }

    async fn cancel(&self, _args: acp::CancelNotification) -> Result<(), acp::Error> {
        log::info!("Received cancel request");
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let local_set = tokio::task::LocalSet::new();

    let (outgoing, incoming) = match std::env::args().collect::<Vec<_>>().as_slice() {
        [_, addr] => {
            let listener = TcpListener::bind(addr).await?;
            log::info!("Listening on {}", listener.local_addr()?);
            let (stream, _) = listener.accept().await?;
            let (incoming, outgoing) = stream.into_split();
            (outgoing.compat_write(), incoming.compat())
        }
        _ => bail!("Unexpected arguments"),
    };

    // The AgentSideConnection will spawn futures onto our Tokio runtime.
    let spawn = |fut| {
        tokio::task::spawn_local(fut);
    };
    local_set
        .run_until(async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            // Start up the ExampleServer connected to stdio.
            let (conn, handle_io) =
                acp::AgentSideConnection::new(ExampleAgent::new(tx), outgoing, incoming, spawn);
            // Kick off a background task to send the ExampleServer's session notifications to the client.
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
            // Run until input/output connections are closed.
            handle_io.await
        })
        .await
}
