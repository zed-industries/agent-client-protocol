//! A simple ACP client for educational purposes.
//!
//! The client starts an agent as a subprocess and communicates with it over stdio. Run the client like this:
//!
//! ```bash
//! cargo run --example client -- path/to/agent --agent-arg
//! ```
//!
//! To connect it to the example agent from this crate:
//!
//! ```bash
//! cargo build --example agent && cargo run --example client -- target/debug/examples/agent
//! ```

use agent_client_protocol::{
    self as acp, Agent, ExtNotification, ExtRequest, ExtResponse, KillTerminalCommandResponse,
};
use anyhow::bail;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

struct ExampleClient {}

#[async_trait::async_trait(?Send)]
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
    ) -> anyhow::Result<acp::WriteTextFileResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn read_text_file(
        &self,
        _args: acp::ReadTextFileRequest,
    ) -> anyhow::Result<acp::ReadTextFileResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn create_terminal(
        &self,
        _args: acp::CreateTerminalRequest,
    ) -> Result<acp::CreateTerminalResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn terminal_output(
        &self,
        _args: acp::TerminalOutputRequest,
    ) -> anyhow::Result<acp::TerminalOutputResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn release_terminal(
        &self,
        _args: acp::ReleaseTerminalRequest,
    ) -> anyhow::Result<acp::ReleaseTerminalResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn wait_for_terminal_exit(
        &self,
        _args: acp::WaitForTerminalExitRequest,
    ) -> anyhow::Result<acp::WaitForTerminalExitResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn kill_terminal_command(
        &self,
        _args: acp::KillTerminalCommandRequest,
    ) -> anyhow::Result<KillTerminalCommandResponse, acp::Error> {
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
                println!("| Agent: {text}");
            }
            acp::SessionUpdate::UserMessageChunk { .. }
            | acp::SessionUpdate::AgentThoughtChunk { .. }
            | acp::SessionUpdate::ToolCall(_)
            | acp::SessionUpdate::ToolCallUpdate(_)
            | acp::SessionUpdate::Plan(_)
            | acp::SessionUpdate::CurrentModeUpdate { .. }
            | acp::SessionUpdate::AvailableCommandsUpdate { .. } => {}
        }
        Ok(())
    }

    async fn ext_method(&self, _args: ExtRequest) -> Result<ExtResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, _args: ExtNotification) -> Result<(), acp::Error> {
        Err(acp::Error::method_not_found())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let command = std::env::args().collect::<Vec<_>>();
    let (outgoing, incoming, child) = match command.as_slice() {
        [_, program, args @ ..] => {
            let mut child = tokio::process::Command::new(program)
                .args(args.iter())
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .kill_on_drop(true)
                .spawn()?;
            (
                child.stdin.take().unwrap().compat_write(),
                child.stdout.take().unwrap().compat(),
                child,
            )
        }
        _ => bail!("Usage: client AGENT_PROGRAM AGENT_ARG..."),
    };

    // The ClientSideConnection will spawn futures onto our Tokio runtime.
    // LocalSet and spawn_local are used because the futures from the
    // agent-client-protocol crate are not Send.
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async move {
            // Set up the ExampleClient connected to the agent's stdio.
            let (conn, handle_io) =
                acp::ClientSideConnection::new(ExampleClient {}, outgoing, incoming, |fut| {
                    tokio::task::spawn_local(fut);
                });

            // Handle I/O in the background.
            tokio::task::spawn_local(handle_io);

            // Connect to the agent and set up a session.
            conn.initialize(acp::InitializeRequest {
                protocol_version: acp::V1,
                client_capabilities: acp::ClientCapabilities::default(),
                meta: None,
            })
            .await?;
            let response = conn
                .new_session(acp::NewSessionRequest {
                    mcp_servers: Vec::new(),
                    cwd: std::env::current_dir()?,
                    meta: None,
                })
                .await?;

            // Send prompts to the agent until stdin is closed.
            let mut rl = rustyline::DefaultEditor::new()?;
            while let Ok(line) = rl.readline("> ") {
                let result = conn
                    .prompt(acp::PromptRequest {
                        session_id: response.session_id.clone(),
                        prompt: vec![line.into()],
                        meta: None,
                    })
                    .await;
                if let Err(e) = result {
                    log::error!("{e}");
                }
            }

            drop(child);
            Ok(())
        })
        .await
}
