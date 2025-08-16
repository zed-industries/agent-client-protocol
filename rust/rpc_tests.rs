use anyhow::Result;
use std::sync::{Arc, Mutex};

use crate::*;

#[derive(Clone)]
struct TestClient {
    permission_responses: Arc<Mutex<Vec<RequestPermissionOutcome>>>,
    file_contents: Arc<Mutex<std::collections::HashMap<std::path::PathBuf, String>>>,
    written_files: Arc<Mutex<Vec<(std::path::PathBuf, String)>>>,
    session_notifications: Arc<Mutex<Vec<SessionNotification>>>,
}

impl TestClient {
    fn new() -> Self {
        Self {
            permission_responses: Arc::new(Mutex::new(vec![])),
            file_contents: Arc::new(Mutex::new(std::collections::HashMap::new())),
            written_files: Arc::new(Mutex::new(vec![])),
            session_notifications: Arc::new(Mutex::new(vec![])),
        }
    }

    fn add_permission_response(&self, outcome: RequestPermissionOutcome) {
        self.permission_responses.lock().unwrap().push(outcome);
    }

    fn add_file_content(&self, path: std::path::PathBuf, content: String) {
        self.file_contents.lock().unwrap().insert(path, content);
    }
}

impl Client for TestClient {
    async fn request_permission(
        &self,
        _arguments: RequestPermissionRequest,
    ) -> Result<RequestPermissionResponse, Error> {
        let responses = self.permission_responses.clone();
        let mut responses = responses.lock().unwrap();
        let outcome = responses
            .pop()
            .unwrap_or(RequestPermissionOutcome::Cancelled);
        Ok(RequestPermissionResponse { outcome })
    }

    async fn write_text_file(&self, arguments: WriteTextFileRequest) -> Result<(), Error> {
        self.written_files
            .lock()
            .unwrap()
            .push((arguments.path, arguments.content));
        Ok(())
    }

    async fn read_text_file(
        &self,
        arguments: ReadTextFileRequest,
    ) -> Result<ReadTextFileResponse, Error> {
        let contents = self.file_contents.lock().unwrap();
        let content = contents
            .get(&arguments.path)
            .cloned()
            .unwrap_or_else(|| "default content".to_string());
        Ok(ReadTextFileResponse { content })
    }

    async fn session_notification(&self, args: SessionNotification) -> Result<(), Error> {
        self.session_notifications.lock().unwrap().push(args);
        Ok(())
    }
}

#[derive(Clone)]
struct TestAgent {
    sessions: Arc<Mutex<std::collections::HashSet<SessionId>>>,
    prompts_received: Arc<Mutex<Vec<PromptReceived>>>,
    cancellations_received: Arc<Mutex<Vec<SessionId>>>,
}

type PromptReceived = (SessionId, Vec<ContentBlock>);

impl TestAgent {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashSet::new())),
            prompts_received: Arc::new(Mutex::new(vec![])),
            cancellations_received: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl Agent for TestAgent {
    async fn initialize(&self, arguments: InitializeRequest) -> Result<InitializeResponse, Error> {
        Ok(InitializeResponse {
            protocol_version: arguments.protocol_version,
            agent_capabilities: Default::default(),
            auth_methods: vec![],
        })
    }

    async fn authenticate(&self, _arguments: AuthenticateRequest) -> Result<(), Error> {
        Ok(())
    }

    async fn new_session(
        &self,
        _arguments: NewSessionRequest,
    ) -> Result<NewSessionResponse, Error> {
        let session_id = SessionId(Arc::from("test-session-123"));
        self.sessions.lock().unwrap().insert(session_id.clone());
        Ok(NewSessionResponse { session_id })
    }

    async fn load_session(&self, _: LoadSessionRequest) -> Result<(), Error> {
        Ok(())
    }

    async fn prompt(&self, arguments: PromptRequest) -> Result<PromptResponse, Error> {
        self.prompts_received
            .lock()
            .unwrap()
            .push((arguments.session_id, arguments.prompt));
        Ok(PromptResponse {
            stop_reason: StopReason::EndTurn,
        })
    }

    async fn cancel(&self, args: CancelNotification) -> Result<(), Error> {
        self.cancellations_received
            .lock()
            .unwrap()
            .push(args.session_id);
        Ok(())
    }
}

// Helper function to create a bidirectional connection
async fn create_connection_pair(
    client: TestClient,
    agent: TestAgent,
) -> (ClientSideConnection, AgentSideConnection) {
    let (client_to_agent_tx, client_to_agent_rx) = async_pipe::pipe();
    let (agent_to_client_tx, agent_to_client_rx) = async_pipe::pipe();

    let (agent_conn, agent_io_task) = ClientSideConnection::new(
        client.clone(),
        client_to_agent_tx,
        agent_to_client_rx,
        |fut| {
            tokio::task::spawn_local(fut);
        },
    );

    let (client_conn, client_io_task) = AgentSideConnection::new(
        agent.clone(),
        agent_to_client_tx,
        client_to_agent_rx,
        |fut| {
            tokio::task::spawn_local(fut);
        },
    );

    // Spawn the IO tasks
    tokio::task::spawn_local(agent_io_task);
    tokio::task::spawn_local(client_io_task);

    (agent_conn, client_conn)
}

#[tokio::test]
async fn test_initialize() {
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let client = TestClient::new();
            let agent = TestAgent::new();

            let (agent_conn, _client_conn) = create_connection_pair(client, agent).await;

            let result = agent_conn
                .initialize(InitializeRequest {
                    protocol_version: VERSION,
                    client_capabilities: Default::default(),
                })
                .await;

            assert!(result.is_ok());
            let response = result.unwrap();
            assert_eq!(response.protocol_version, VERSION);
        })
        .await;
}

#[tokio::test]
async fn test_basic_session_creation() {
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let client = TestClient::new();
            let agent = TestAgent::new();

            let (agent_conn, _client_conn) = create_connection_pair(client, agent).await;

            agent_conn
                .new_session(NewSessionRequest {
                    mcp_servers: vec![],
                    cwd: std::path::PathBuf::from("/test"),
                })
                .await
                .expect("new_session failed");
        })
        .await;
}

#[tokio::test]
async fn test_bidirectional_file_operations() {
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let client = TestClient::new();
            let agent = TestAgent::new();

            // Add test file content
            let test_path = std::path::PathBuf::from("/test/file.txt");
            client.add_file_content(test_path.clone(), "Hello, World!".to_string());

            let (_agent_conn, client_conn) = create_connection_pair(client.clone(), agent).await;

            // Test reading a file
            let session_id = SessionId(Arc::from("test-session"));
            let read_result = client_conn
                .read_text_file(ReadTextFileRequest {
                    session_id: session_id.clone(),
                    path: test_path.clone(),
                    line: None,
                    limit: None,
                })
                .await
                .expect("read_text_file failed");

            assert_eq!(read_result.content, "Hello, World!");

            // Test writing a file
            let write_result = client_conn
                .write_text_file(WriteTextFileRequest {
                    session_id: session_id.clone(),
                    path: test_path.clone(),
                    content: "Updated content".to_string(),
                })
                .await;

            assert!(write_result.is_ok());
        })
        .await;
}

#[tokio::test]
async fn test_session_notifications() {
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let client = TestClient::new();
            let agent = TestAgent::new();

            let (_agent_conn, client_conn) = create_connection_pair(client.clone(), agent).await;

            let session_id = SessionId(Arc::from("test-session"));
            // Send various session updates
            client_conn
                .session_notification(SessionNotification {
                    session_id: session_id.clone(),
                    update: SessionUpdate::UserMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Hello from user".to_string(),
                        }),
                    },
                })
                .await
                .expect("session_notification failed");

            client_conn
                .session_notification(SessionNotification {
                    session_id: session_id.clone(),
                    update: SessionUpdate::AgentMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Hello from agent".to_string(),
                        }),
                    },
                })
                .await
                .expect("session_notification failed");

            tokio::task::yield_now().await;

            let notifications = client.session_notifications.lock().unwrap();
            assert_eq!(notifications.len(), 2);
            assert_eq!(notifications[0].session_id, session_id);
            assert_eq!(notifications[1].session_id, session_id);
        })
        .await;
}

#[tokio::test]
async fn test_cancel_notification() {
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let client = TestClient::new();
            let agent = TestAgent::new();

            let (agent_conn, _client_conn) = create_connection_pair(client, agent.clone()).await;

            let session_id = SessionId(Arc::from("test-session"));
            // Send cancel notification
            agent_conn
                .cancel(CancelNotification {
                    session_id: session_id.clone(),
                })
                .await
                .expect("cancel failed");

            tokio::task::yield_now().await;

            let cancelled = agent.cancellations_received.lock().unwrap();
            assert_eq!(cancelled.len(), 1);
            assert_eq!(cancelled[0], session_id);
        })
        .await;
}

#[tokio::test]
async fn test_concurrent_operations() {
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let client = TestClient::new();
            let agent = TestAgent::new();

            // Add multiple file contents
            for i in 0..5 {
                let path = std::path::PathBuf::from(format!("/test/file{i}.txt"));
                client.add_file_content(path, format!("Content {i}"));
            }

            let (_agent_conn, client_conn) = create_connection_pair(client.clone(), agent).await;

            let session_id = SessionId(Arc::from("test-session"));

            // Launch multiple concurrent read operations
            let mut read_futures = vec![];
            for i in 0..5 {
                let path = std::path::PathBuf::from(format!("/test/file{i}.txt"));
                let future = client_conn.read_text_file(ReadTextFileRequest {
                    session_id: session_id.clone(),
                    path,
                    line: None,
                    limit: None,
                });
                read_futures.push(future);
            }

            // Wait for all reads to complete
            let results = futures::future::join_all(read_futures).await;

            // Verify all reads succeeded
            for (i, result) in results.into_iter().enumerate() {
                let output = result.expect("read failed");
                assert_eq!(output.content, format!("Content {i}"));
            }
        })
        .await;
}

#[tokio::test]
async fn test_full_conversation_flow() {
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let client = TestClient::new();
            let agent = TestAgent::new();

            // Set up permission to approve the tool call
            client.add_permission_response(RequestPermissionOutcome::Selected {
                option_id: PermissionOptionId(Arc::from("allow-once")),
            });

            let (agent_conn, client_conn) = create_connection_pair(client.clone(), agent).await;
            // 1. Start new session
            let new_session_result = agent_conn
                .new_session(NewSessionRequest {
                    mcp_servers: vec![],
                    cwd: std::path::PathBuf::from("/test"),
                })
                .await
                .expect("new_session failed");

            let session_id = new_session_result.session_id;

            // 2. Send user message
            let user_prompt = vec![ContentBlock::Text(TextContent {
                annotations: None,
                text: "Please analyze the file and summarize it".to_string(),
            })];

            agent_conn
                .prompt(PromptRequest {
                    session_id: session_id.clone(),
                    prompt: user_prompt,
                })
                .await
                .expect("prompt failed");

            // 3. Agent starts responding
            client_conn
                .session_notification(SessionNotification {
                    session_id: session_id.clone(),
                    update: SessionUpdate::AgentMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "I'll analyze the file for you. ".to_string(),
                        }),
                    },
                })
                .await
                .expect("session_notification failed");

            // 4. Agent creates a tool call
            let tool_call_id = ToolCallId(Arc::from("read-file-001"));
            client_conn
                .session_notification(SessionNotification {
                    session_id: session_id.clone(),
                    update: SessionUpdate::ToolCall(ToolCall {
                        id: tool_call_id.clone(),
                        title: "Reading file".to_string(),
                        kind: ToolKind::Read,
                        status: ToolCallStatus::Pending,
                        content: vec![],
                        locations: vec![ToolCallLocation {
                            path: std::path::PathBuf::from("/test/data.txt"),
                            line: None,
                        }],
                        raw_input: None,
                        raw_output: None,
                    }),
                })
                .await
                .expect("session_notification failed");

            // 5. Agent requests permission for the tool call
            let permission_result = client_conn
                .request_permission(RequestPermissionRequest {
                    session_id: session_id.clone(),
                    tool_call: ToolCallRef::Id(tool_call_id.clone()),
                    options: vec![
                        PermissionOption {
                            id: PermissionOptionId(Arc::from("allow-once")),
                            name: "Allow once".to_string(),
                            kind: PermissionOptionKind::AllowOnce,
                        },
                        PermissionOption {
                            id: PermissionOptionId(Arc::from("reject-once")),
                            name: "Reject".to_string(),
                            kind: PermissionOptionKind::RejectOnce,
                        },
                    ],
                })
                .await
                .expect("request_permission failed");

            // Verify permission was granted
            match permission_result.outcome {
                RequestPermissionOutcome::Selected { option_id } => {
                    assert_eq!(option_id.0.as_ref(), "allow-once");
                }
                _ => panic!("Expected permission to be granted"),
            }

            // 6. Update tool call status
            client_conn
                .session_notification(SessionNotification {
                    session_id: session_id.clone(),
                    update: SessionUpdate::ToolCallUpdate(ToolCallUpdate {
                        id: tool_call_id.clone(),
                        fields: ToolCallUpdateFields {
                            status: Some(ToolCallStatus::InProgress),
                            ..Default::default()
                        },
                    }),
                })
                .await
                .expect("session_notification failed");

            // 7. Tool call completes with content
            client_conn
                .session_notification(SessionNotification {
                    session_id: session_id.clone(),
                    update: SessionUpdate::ToolCallUpdate(ToolCallUpdate {
                        id: tool_call_id.clone(),
                        fields: ToolCallUpdateFields {
                            status: Some(ToolCallStatus::Completed),
                            content: Some(vec![ToolCallContent::Content {
                                content: ContentBlock::Text(TextContent {
                                    annotations: None,
                                    text: "File contents: Lorem ipsum dolor sit amet".to_string(),
                                }),
                            }]),
                            ..Default::default()
                        },
                    }),
                })
                .await
                .expect("session_notification failed");

            // 8. Agent sends more text after tool completion
            client_conn
                .session_notification(SessionNotification {
                    session_id: session_id.clone(),
                    update: SessionUpdate::AgentMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Based on the file contents, here's my summary: The file contains placeholder text commonly used in the printing industry.".to_string(),
                        }),
                    },
                })
                .await
                .expect("session_notification failed");

            for _ in 0..10 {
                tokio::task::yield_now().await;
            }

            // Verify we received all the updates
            let updates = client.session_notifications.lock().unwrap();
            assert!(updates.len() >= 5); // At least 5 updates sent

            // Verify the sequence of updates
            let mut found_agent_message = false;
            let mut found_tool_call = false;
            let mut found_tool_update = false;
            let mut found_final_message = false;

            for notification in updates.iter() {
                match &notification.update {
                    SessionUpdate::AgentMessageChunk { content : ContentBlock::Text(text)} => {
                        if text.text.contains("I'll analyze") {
                            found_agent_message = true;
                        } else if text.text.contains("Based on the file") {
                            found_final_message = true;
                        }
                    }
                    SessionUpdate::ToolCall(_) => {
                        found_tool_call = true;
                    }
                    SessionUpdate::ToolCallUpdate(update) => {
                        if let Some(ToolCallStatus::Completed) = update.fields.status {
                            found_tool_update = true;
                        }
                    }
                    _ => {}
                }
            }

            assert!(found_agent_message, "Should have initial agent message");
            assert!(found_tool_call, "Should have tool call");
            assert!(found_tool_update, "Should have tool call completion");
            assert!(found_final_message, "Should have final agent message");
        })
        .await;
}

#[tokio::test]
async fn test_notification_wire_format() {
    use crate::{
        AgentNotification, AgentSide, CancelNotification, ClientNotification, ClientSide,
        ContentBlock, SessionNotification, SessionUpdate, TextContent, rpc::OutgoingMessage,
    };
    use serde_json::{Value, json};

    // Test client -> agent notification wire format
    let outgoing_msg = OutgoingMessage::<ClientSide, AgentSide>::Notification {
        method: "cancel",
        params: Some(ClientNotification::CancelNotification(CancelNotification {
            session_id: SessionId("test-123".into()),
        })),
    };

    let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
    assert_eq!(
        serialized,
        json!({
            "method": "cancel",
            "params": {
                "sessionId": "test-123"
            }
        })
    );

    // Test agent -> client notification wire format
    let outgoing_msg = OutgoingMessage::<AgentSide, ClientSide>::Notification {
        method: "sessionUpdate",
        params: Some(AgentNotification::SessionNotification(
            SessionNotification {
                session_id: SessionId("test-456".into()),
                update: SessionUpdate::AgentMessageChunk {
                    content: ContentBlock::Text(TextContent {
                        annotations: None,
                        text: "Hello".to_string(),
                    }),
                },
            },
        )),
    };

    let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
    assert_eq!(
        serialized,
        json!({
            "method": "sessionUpdate",
            "params": {
                "sessionId": "test-456",
                "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": {
                        "type": "text",
                        "text": "Hello"
                    }
                }
            }
        })
    );
}
