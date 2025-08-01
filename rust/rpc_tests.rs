use anyhow::Result;
use futures::future::LocalBoxFuture;
use std::sync::{Arc, Mutex};

use crate::*;

#[derive(Clone)]
struct TestClient {
    permission_responses: Arc<Mutex<Vec<RequestPermissionOutcome>>>,
    file_contents: Arc<Mutex<std::collections::HashMap<std::path::PathBuf, String>>>,
    written_files: Arc<Mutex<Vec<(std::path::PathBuf, String)>>>,
}

impl TestClient {
    fn new() -> Self {
        Self {
            permission_responses: Arc::new(Mutex::new(vec![])),
            file_contents: Arc::new(Mutex::new(std::collections::HashMap::new())),
            written_files: Arc::new(Mutex::new(vec![])),
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
    fn request_permission(
        &self,
        _arguments: RequestPermissionRequest,
    ) -> LocalBoxFuture<'static, Result<RequestPermissionResponse, Error>> {
        let responses = self.permission_responses.clone();
        Box::pin(async move {
            let mut responses = responses.lock().unwrap();
            let outcome = responses
                .pop()
                .unwrap_or(RequestPermissionOutcome::Cancelled);
            Ok(RequestPermissionResponse { outcome })
        })
    }

    fn write_text_file(
        &self,
        arguments: WriteTextFileRequest,
    ) -> LocalBoxFuture<'static, Result<(), Error>> {
        let written_files = self.written_files.clone();
        Box::pin(async move {
            written_files
                .lock()
                .unwrap()
                .push((arguments.path, arguments.content));
            Ok(())
        })
    }

    fn read_text_file(
        &self,
        arguments: ReadTextFileRequest,
    ) -> LocalBoxFuture<'static, Result<ReadTextFileResponse, Error>> {
        let file_contents = self.file_contents.clone();
        Box::pin(async move {
            let contents = file_contents.lock().unwrap();
            let content = contents
                .get(&arguments.path)
                .cloned()
                .unwrap_or_else(|| "default content".to_string());
            Ok(ReadTextFileResponse { content })
        })
    }
}

#[derive(Clone)]
struct TestAgent {
    sessions: Arc<Mutex<std::collections::HashSet<SessionId>>>,
    prompts_received: Arc<Mutex<Vec<(SessionId, Vec<ContentBlock>)>>>,
}

impl TestAgent {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashSet::new())),
            prompts_received: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl Agent for TestAgent {
    fn initialize(
        &self,
        arguments: InitializeRequest,
    ) -> LocalBoxFuture<'static, Result<InitializeResponse, Error>> {
        Box::pin(async move {
            Ok(InitializeResponse {
                protocol_version: arguments.protocol_version,
                agent_capabilities: Default::default(),
                auth_methods: vec![],
            })
        })
    }

    fn authenticate(
        &self,
        _arguments: AuthenticateRequest,
    ) -> LocalBoxFuture<'static, Result<(), Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn new_session(
        &self,
        _arguments: NewSessionRequest,
    ) -> LocalBoxFuture<'static, Result<NewSessionResponse, Error>> {
        let sessions = self.sessions.clone();
        Box::pin(async move {
            let session_id = SessionId(Arc::from("test-session-123"));
            sessions.lock().unwrap().insert(session_id.clone());
            Ok(NewSessionResponse {
                session_id: Some(session_id),
            })
        })
    }

    fn load_session(
        &self,
        arguments: LoadSessionRequest,
    ) -> LocalBoxFuture<'static, Result<LoadSessionResponse, Error>> {
        let sessions = self.sessions.clone();
        Box::pin(async move {
            let has_session = sessions.lock().unwrap().contains(&arguments.session_id);
            Ok(LoadSessionResponse {
                auth_required: !has_session,
                auth_methods: vec![],
            })
        })
    }

    fn prompt(&self, arguments: PromptRequest) -> LocalBoxFuture<'static, Result<(), Error>> {
        let prompts_received = self.prompts_received.clone();
        Box::pin(async move {
            prompts_received
                .lock()
                .unwrap()
                .push((arguments.session_id, arguments.prompt));
            Ok(())
        })
    }
}

// Helper function to create a bidirectional connection
async fn create_connection_pair(
    client: TestClient,
    agent: TestAgent,
) -> (AgentConnection, ClientConnection) {
    let (client_to_agent_tx, client_to_agent_rx) = async_pipe::pipe();
    let (agent_to_client_tx, agent_to_client_rx) = async_pipe::pipe();

    let (agent_conn, agent_io_task) = AgentConnection::new(
        client.clone(),
        client_to_agent_tx,
        agent_to_client_rx,
        |fut| {
            tokio::task::spawn_local(fut);
        },
    );

    let (client_conn, client_io_task) = ClientConnection::new(
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

            let result = agent_conn
                .new_session(NewSessionRequest {
                    mcp_servers: vec![],
                    cwd: std::path::PathBuf::from("/test"),
                })
                .await
                .expect("new_session failed");

            assert!(result.session_id.is_some());
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

            let (agent_conn, client_conn) = create_connection_pair(client, agent).await;

            // Set up notification handler
            let notifications_received = Arc::new(Mutex::new(Vec::new()));
            let notifications_clone = notifications_received.clone();
            agent_conn.on_session_update(move |notification| {
                notifications_clone.lock().unwrap().push(notification);
            });

            let session_id = SessionId(Arc::from("test-session"));
            // Send various session updates
            client_conn
                .send_session_update(
                    session_id.clone(),
                    SessionUpdate::UserMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Hello from user".to_string(),
                        }),
                    },
                )
                .expect("send_session_update failed");

            client_conn
                .send_session_update(
                    session_id.clone(),
                    SessionUpdate::AgentMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Hello from agent".to_string(),
                        }),
                    },
                )
                .expect("send_session_update failed");

            tokio::task::yield_now().await;

            let notifications = notifications_received.lock().unwrap();
            assert!(!notifications.is_empty());
            assert_eq!(notifications[0].session_id, session_id);
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

            let (agent_conn, client_conn) = create_connection_pair(client, agent).await;

            // Set up cancel handler
            let cancelled_sessions = Arc::new(Mutex::new(Vec::new()));
            let cancelled_clone = cancelled_sessions.clone();
            client_conn.on_cancel(move |session_id| {
                cancelled_clone.lock().unwrap().push(session_id);
            });

            let session_id = SessionId(Arc::from("test-session"));
            // Send cancel notification
            agent_conn
                .cancel_generation(session_id.clone())
                .expect("cancel failed");

            tokio::task::yield_now().await;

            let cancelled = cancelled_sessions.lock().unwrap();
            assert!(!cancelled.is_empty());
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
                let path = std::path::PathBuf::from(format!("/test/file{}.txt", i));
                client.add_file_content(path, format!("Content {}", i));
            }

            let (_agent_conn, client_conn) = create_connection_pair(client.clone(), agent).await;

            let session_id = SessionId(Arc::from("test-session"));

            // Launch multiple concurrent read operations
            let mut read_futures = vec![];
            for i in 0..5 {
                let path = std::path::PathBuf::from(format!("/test/file{}.txt", i));
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
                assert_eq!(output.content, format!("Content {}", i));
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

            let (agent_conn, client_conn) = create_connection_pair(client, agent).await;

            // Track all session updates
            let all_updates = Arc::new(Mutex::new(Vec::new()));
            let updates_clone = all_updates.clone();
            agent_conn.on_session_update(move |notification| {
                updates_clone.lock().unwrap().push(notification);
            });
            // 1. Start new session
            let new_session_result = agent_conn
                .new_session(NewSessionRequest {
                    mcp_servers: vec![],
                    cwd: std::path::PathBuf::from("/test"),
                })
                .await
                .expect("new_session failed");

            let session_id = new_session_result.session_id.unwrap();

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
                .send_session_update(
                    session_id.clone(),
                    SessionUpdate::AgentMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "I'll analyze the file for you. ".to_string(),
                        }),
                    },
                )
                .expect("send_session_update failed");

            // 4. Agent creates a tool call
            let tool_call_id = ToolCallId(Arc::from("read-file-001"));
            client_conn
                .send_session_update(
                    session_id.clone(),
                    SessionUpdate::ToolCall(ToolCall {
                        id: tool_call_id.clone(),
                        label: "Reading file".to_string(),
                        kind: ToolKind::Read,
                        status: ToolCallStatus::Pending,
                        content: vec![],
                        locations: vec![ToolCallLocation {
                            path: std::path::PathBuf::from("/test/data.txt"),
                            line: None,
                        }],
                        raw_input: None,
                    }),
                )
                .expect("send_session_update failed");

            // 5. Agent requests permission for the tool call
            let permission_result = client_conn
                .request_permission(RequestPermissionRequest {
                    session_id: session_id.clone(),
                    tool_call: ToolCall {
                        id: tool_call_id.clone(),
                        label: "Read /test/data.txt".to_string(),
                        kind: ToolKind::Read,
                        status: ToolCallStatus::Pending,
                        content: vec![],
                        locations: vec![ToolCallLocation {
                            path: std::path::PathBuf::from("/test/data.txt"),
                            line: None,
                        }],
                        raw_input: None,
                    },
                    options: vec![
                        PermissionOption {
                            id: PermissionOptionId(Arc::from("allow-once")),
                            label: "Allow once".to_string(),
                            kind: PermissionOptionKind::AllowOnce,
                        },
                        PermissionOption {
                            id: PermissionOptionId(Arc::from("reject-once")),
                            label: "Reject".to_string(),
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
                .send_session_update(
                    session_id.clone(),
                    SessionUpdate::ToolCallUpdate(ToolCallUpdate {
                        id: tool_call_id.clone(),
                        fields: ToolCallUpdateFields {
                            status: Some(ToolCallStatus::InProgress),
                            ..Default::default()
                        },
                    }),
                )
                .expect("send_session_update failed");

            // 7. Tool call completes with content
            client_conn
                .send_session_update(
                    session_id.clone(),
                    SessionUpdate::ToolCallUpdate(ToolCallUpdate {
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
                )
                .expect("send_session_update failed");

            // 8. Agent sends more text after tool completion
            client_conn
                .send_session_update(
                    session_id.clone(),
                    SessionUpdate::AgentMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Based on the file contents, here's my summary: The file contains placeholder text commonly used in the printing industry.".to_string(),
                        }),
                    },
                )
                .expect("send_session_update failed");

            for _ in 0..10 {
                tokio::task::yield_now().await;
            }

            // Verify we received all the updates
            let updates = all_updates.lock().unwrap();
            assert!(updates.len() >= 5); // At least 5 updates sent

            // Verify the sequence of updates
            let mut found_agent_message = false;
            let mut found_tool_call = false;
            let mut found_tool_update = false;
            let mut found_final_message = false;

            for update in updates.iter() {
                match &update.update {
                    SessionUpdate::AgentMessageChunk { content } => {
                        if let ContentBlock::Text(text) = content {
                            if text.text.contains("I'll analyze") {
                                found_agent_message = true;
                            } else if text.text.contains("Based on the file") {
                                found_final_message = true;
                            }
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
        AgentNotification, AgentSide, CancelledNotification, ClientNotification, ClientSide,
        ContentBlock, SessionNotification, SessionUpdate, TextContent, rpc::OutgoingMessage,
    };
    use serde_json::{Value, json};

    // Test client -> agent notification wire format
    let outgoing_msg = OutgoingMessage::<ClientSide, AgentSide>::Notification {
        method: "cancelled",
        params: Some(ClientNotification::CancelledNotification(
            CancelledNotification {
                session_id: SessionId("test-123".into()),
            },
        )),
    };

    let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
    assert_eq!(
        serialized,
        json!({
            "method": "cancelled",
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
                "sessionUpdate": "agentMessageChunk",
                "content": {
                    "type": "text",
                    "text": "Hello"
                }
            }
        })
    );
}
