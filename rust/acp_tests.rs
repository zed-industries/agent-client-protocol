use super::*;
use tokio::task::LocalSet;
use tokio::time::{Duration, timeout};

pub struct TestClient;
pub struct TestAgent;

impl Agent for TestAgent {
    async fn initialize(&self, request: InitializeParams) -> Result<InitializeResponse, Error> {
        Ok(InitializeResponse {
            protocol_version: request.protocol_version,
            is_authenticated: true,
        })
    }

    async fn authenticate(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn send_user_message(&self, _request: SendUserMessageParams) -> Result<(), Error> {
        Ok(())
    }

    async fn cancel_send_message(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl Client for TestClient {
    async fn stream_assistant_message_chunk(
        &self,
        _request: StreamAssistantMessageChunkParams,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn request_tool_call_confirmation(
        &self,
        _request: RequestToolCallConfirmationParams,
    ) -> Result<RequestToolCallConfirmationResponse, Error> {
        Ok(RequestToolCallConfirmationResponse {
            id: ToolCallId(0),
            outcome: ToolCallConfirmationOutcome::Allow,
        })
    }

    async fn push_tool_call(
        &self,
        _request: PushToolCallParams,
    ) -> Result<PushToolCallResponse, Error> {
        Ok(PushToolCallResponse { id: ToolCallId(0) })
    }

    async fn update_tool_call(&self, _request: UpdateToolCallParams) -> Result<(), Error> {
        Ok(())
    }

    async fn write_text_file(&self, _request: WriteTextFileParams) -> Result<(), Error> {
        Ok(())
    }

    async fn read_text_file(
        &self,
        _request: ReadTextFileParams,
    ) -> Result<ReadTextFileResponse, Error> {
        Ok(ReadTextFileResponse {
            content: String::new(),
        })
    }
}

#[tokio::test]
async fn test_client_agent_communication() {
    env_logger::init();

    let local = LocalSet::new();
    local
        .run_until(async move {
            let client = TestClient;
            let agent = TestAgent;

            let (client_to_agent_tx, client_to_agent_rx) = async_pipe::pipe();
            let (agent_to_client_tx, agent_to_client_rx) = async_pipe::pipe();

            let (client_connection, client_io_task) = AgentConnection::connect_to_agent(
                client,
                client_to_agent_tx,
                agent_to_client_rx,
                |fut| {
                    tokio::task::spawn_local(fut);
                },
            );
            let (agent_connection, agent_io_task) = ClientConnection::connect_to_client(
                agent,
                agent_to_client_tx,
                client_to_agent_rx,
                |fut| {
                    tokio::task::spawn_local(fut);
                },
            );

            let _task = tokio::spawn(client_io_task);
            let _task = tokio::spawn(agent_io_task);

            let response = agent_connection.request(PushToolCallParams {
                label: "test".into(),
                icon: Icon::FileSearch,
                content: None,
                locations: Vec::default(),
            });
            let response = timeout(Duration::from_secs(2), response)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(response.id, ToolCallId(0));

            let response = client_connection.request(InitializeParams {
                protocol_version: ProtocolVersion::latest(),
            });
            let response = timeout(Duration::from_secs(2), response)
                .await
                .unwrap()
                .unwrap();
            assert!(response.is_authenticated);
        })
        .await
}
