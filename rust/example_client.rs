use agent_client_protocol::{self as acp, Agent};
use anyhow::bail;
use tokio::net::TcpStream;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

struct ExampleClient {}

impl acp::Client for ExampleClient {
    async fn request_permission(
        &self,
        _args: acp::RequestPermissionRequest,
    ) -> anyhow::Result<acp::RequestPermissionResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn write_text_file(
        &self,
        _args: acp::WriteTextFileRequest,
    ) -> anyhow::Result<(), acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn read_text_file(
        &self,
        _args: acp::ReadTextFileRequest,
    ) -> anyhow::Result<acp::ReadTextFileResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn session_notification(
        &self,
        args: acp::SessionNotification,
    ) -> anyhow::Result<(), acp::Error> {
        match args.update {
            acp::SessionUpdate::AgentMessageChunk { content } => {
                let text = match content {
                    acp::ContentBlock::Text(text_content) => text_content.text,
                    acp::ContentBlock::Image(_) => "<image>".into(),
                    acp::ContentBlock::Audio(_) => "<audio>".into(),
                    acp::ContentBlock::ResourceLink(resource_link) => resource_link.uri,
                    acp::ContentBlock::Resource(_) => "<resource>".into(),
                };
                println!("| Server: {text}");
            }
            acp::SessionUpdate::UserMessageChunk { .. }
            | acp::SessionUpdate::AgentThoughtChunk { .. }
            | acp::SessionUpdate::ToolCall(_)
            | acp::SessionUpdate::ToolCallUpdate(_)
            | acp::SessionUpdate::Plan(_) => {}
        }
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let local_set = tokio::task::LocalSet::new();

    let (outgoing, incoming) = match std::env::args().collect::<Vec<_>>().as_slice() {
        [_, addr] => {
            let stream = TcpStream::connect(addr).await?;
            let (incoming, outgoing) = stream.into_split();
            (outgoing.compat_write(), incoming.compat())
        }
        _ => bail!("Unexpected arguments"),
    };

    // The ClientSideConnection will spawn futures onto our Tokio runtime.
    let spawn = |fut| {
        tokio::task::spawn_local(fut);
    };
    local_set
        .run_until(async move {
            // Set up the ExampleClient connected to stdio.
            let (conn, handle_io) =
                acp::ClientSideConnection::new(ExampleClient {}, outgoing, incoming, spawn);

            tokio::task::spawn_local(handle_io);

            // Connect to the agent and set up a session.
            conn.initialize(acp::InitializeRequest {
                protocol_version: acp::V1,
                client_capabilities: acp::ClientCapabilities::default(),
            })
            .await?;
            let response = conn
                .new_session(acp::NewSessionRequest {
                    mcp_servers: Vec::new(),
                    cwd: std::env::current_dir()?,
                })
                .await?;

            // Send prompts to the server until stdin is closed.
            let mut rl = rustyline::DefaultEditor::new()?;
            while let Some(line) = rl.readline("> ").ok() {
                let result = conn
                    .prompt(acp::PromptRequest {
                        session_id: response.session_id.clone(),
                        prompt: vec![line.into()],
                    })
                    .await;
                if let Err(e) = result {
                    log::error!("{e}");
                }
            }

            Ok(())
        })
        .await
}
