use anyhow::Result;
use serde_json::json;
use std::sync::{Arc, Mutex};

use crate::*;

#[derive(Clone)]
struct TestClient {
    permission_responses: Arc<Mutex<Vec<RequestPermissionOutcome>>>,
    file_contents: Arc<Mutex<std::collections::HashMap<std::path::PathBuf, String>>>,
    written_files: Arc<Mutex<Vec<(std::path::PathBuf, String)>>>,
    session_notifications: Arc<Mutex<Vec<SessionNotification>>>,
    extension_notifications: Arc<Mutex<Vec<(String, ExtNotification)>>>,
}

impl TestClient {
    fn new() -> Self {
        Self {
            permission_responses: Arc::new(Mutex::new(Vec::new())),
            file_contents: Arc::new(Mutex::new(std::collections::HashMap::new())),
            written_files: Arc::new(Mutex::new(Vec::new())),
            session_notifications: Arc::new(Mutex::new(Vec::new())),
            extension_notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_permission_response(&self, outcome: RequestPermissionOutcome) {
        self.permission_responses.lock().unwrap().push(outcome);
    }

    fn add_file_content(&self, path: std::path::PathBuf, content: String) {
        self.file_contents.lock().unwrap().insert(path, content);
    }
}

macro_rules! raw_json {
    ($($json:tt)+) => {{
        let response = serde_json::json!($($json)+);
        serde_json::value::to_raw_value(&response).unwrap().into()
    }};
}

#[async_trait::async_trait(?Send)]
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
        Ok(RequestPermissionResponse {
            outcome,
            meta: None,
        })
    }

    async fn write_text_file(
        &self,
        arguments: WriteTextFileRequest,
    ) -> Result<WriteTextFileResponse, Error> {
        self.written_files
            .lock()
            .unwrap()
            .push((arguments.path, arguments.content));
        Ok(WriteTextFileResponse::default())
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
        Ok(ReadTextFileResponse {
            content,
            meta: None,
        })
    }

    async fn session_notification(&self, args: SessionNotification) -> Result<(), Error> {
        self.session_notifications.lock().unwrap().push(args);
        Ok(())
    }

    async fn create_terminal(
        &self,
        _args: CreateTerminalRequest,
    ) -> Result<CreateTerminalResponse, Error> {
        unimplemented!()
    }

    async fn terminal_output(
        &self,
        _args: TerminalOutputRequest,
    ) -> Result<TerminalOutputResponse, Error> {
        unimplemented!()
    }

    async fn kill_terminal_command(
        &self,
        _args: KillTerminalCommandRequest,
    ) -> Result<KillTerminalCommandResponse, Error> {
        unimplemented!()
    }

    async fn release_terminal(
        &self,
        _args: ReleaseTerminalRequest,
    ) -> Result<ReleaseTerminalResponse, Error> {
        unimplemented!()
    }

    async fn wait_for_terminal_exit(
        &self,
        _args: WaitForTerminalExitRequest,
    ) -> Result<WaitForTerminalExitResponse, Error> {
        unimplemented!()
    }

    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, Error> {
        match dbg!(args.method.as_ref()) {
            "example.com/ping" => Ok(raw_json!({
                "response": "pong",
                "params": args.params
            })),
            _ => Err(Error::method_not_found()),
        }
    }

    async fn ext_notification(&self, args: ExtNotification) -> Result<(), Error> {
        self.extension_notifications
            .lock()
            .unwrap()
            .push((args.method.to_string(), args));
        Ok(())
    }
}

#[derive(Clone)]
struct TestAgent {
    sessions: Arc<Mutex<std::collections::HashSet<SessionId>>>,
    prompts_received: Arc<Mutex<Vec<PromptReceived>>>,
    cancellations_received: Arc<Mutex<Vec<SessionId>>>,
    extension_notifications: Arc<Mutex<Vec<(String, ExtNotification)>>>,
}

type PromptReceived = (SessionId, Vec<ContentBlock>);

impl TestAgent {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(std::collections::HashSet::new())),
            prompts_received: Arc::new(Mutex::new(Vec::new())),
            cancellations_received: Arc::new(Mutex::new(Vec::new())),
            extension_notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Agent for TestAgent {
    async fn initialize(&self, arguments: InitializeRequest) -> Result<InitializeResponse, Error> {
        Ok(InitializeResponse {
            protocol_version: arguments.protocol_version,
            agent_capabilities: AgentCapabilities::default(),
            auth_methods: vec![],
            meta: None,
        })
    }

    async fn authenticate(
        &self,
        _arguments: AuthenticateRequest,
    ) -> Result<AuthenticateResponse, Error> {
        Ok(AuthenticateResponse::default())
    }

    async fn new_session(
        &self,
        _arguments: NewSessionRequest,
    ) -> Result<NewSessionResponse, Error> {
        let session_id = SessionId(Arc::from("test-session-123"));
        self.sessions.lock().unwrap().insert(session_id.clone());
        Ok(NewSessionResponse {
            session_id,
            modes: None,
            #[cfg(feature = "unstable")]
            models: None,
            meta: None,
        })
    }

    async fn load_session(&self, _: LoadSessionRequest) -> Result<LoadSessionResponse, Error> {
        Ok(LoadSessionResponse {
            modes: None,
            #[cfg(feature = "unstable")]
            models: None,
            meta: None,
        })
    }

    async fn set_session_mode(
        &self,
        _arguments: SetSessionModeRequest,
    ) -> Result<SetSessionModeResponse, Error> {
        Ok(SetSessionModeResponse { meta: None })
    }

    async fn prompt(&self, arguments: PromptRequest) -> Result<PromptResponse, Error> {
        self.prompts_received
            .lock()
            .unwrap()
            .push((arguments.session_id, arguments.prompt));
        Ok(PromptResponse {
            stop_reason: StopReason::EndTurn,
            meta: None,
        })
    }

    async fn cancel(&self, args: CancelNotification) -> Result<(), Error> {
        self.cancellations_received
            .lock()
            .unwrap()
            .push(args.session_id);
        Ok(())
    }

    #[cfg(feature = "unstable")]
    async fn set_session_model(
        &self,
        args: SetSessionModelRequest,
    ) -> Result<SetSessionModelResponse, Error> {
        log::info!("Received select model request {args:?}");
        Ok(SetSessionModelResponse::default())
    }

    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, Error> {
        dbg!();
        match dbg!(args.method.as_ref()) {
            "example.com/echo" => {
                let response = serde_json::json!({
                    "echo": args.params
                });
                Ok(serde_json::value::to_raw_value(&response)?.into())
            }
            _ => Err(Error::method_not_found()),
        }
    }

    async fn ext_notification(&self, args: ExtNotification) -> Result<(), Error> {
        self.extension_notifications
            .lock()
            .unwrap()
            .push((args.method.to_string(), args));
        Ok(())
    }
}

// Helper function to create a bidirectional connection
fn create_connection_pair(
    client: &TestClient,
    agent: &TestAgent,
) -> (ClientSideConnection, AgentSideConnection) {
    let (client_to_agent_rx, client_to_agent_tx) = piper::pipe(1024);
    let (agent_to_client_rx, agent_to_client_tx) = piper::pipe(1024);

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

            let (agent_conn, _client_conn) = create_connection_pair(&client, &agent);

            let result = agent_conn
                .initialize(InitializeRequest {
                    protocol_version: VERSION,
                    client_capabilities: ClientCapabilities::default(),
                    meta: None,
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

            let (agent_conn, _client_conn) = create_connection_pair(&client, &agent);

            agent_conn
                .new_session(NewSessionRequest {
                    mcp_servers: vec![],
                    cwd: std::path::PathBuf::from("/test"),
                    meta: None,
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

            let (_agent_conn, client_conn) = create_connection_pair(&client, &agent);

            // Test reading a file
            let session_id = SessionId(Arc::from("test-session"));
            let read_result = client_conn
                .read_text_file(ReadTextFileRequest {
                    session_id: session_id.clone(),
                    path: test_path.clone(),
                    line: None,
                    limit: None,
                    meta: None,
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
                    meta: None,
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

            let (_agent_conn, client_conn) = create_connection_pair(&client, &agent);

            let session_id = SessionId(Arc::from("test-session"));
            // Send various session updates
            client_conn
                .session_notification(SessionNotification {
                    session_id: session_id.clone(),
                    update: SessionUpdate::UserMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Hello from user".to_string(),
                            meta: None,
                        }),
                    },
                    meta: None,
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
                            meta: None,
                        }),
                    },
                    meta: None,
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

            let (agent_conn, _client_conn) = create_connection_pair(&client, &agent);

            let session_id = SessionId(Arc::from("test-session"));
            // Send cancel notification
            agent_conn
                .cancel(CancelNotification {
                    session_id: session_id.clone(),
                    meta: None,
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

            let (_agent_conn, client_conn) = create_connection_pair(&client, &agent);

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
                    meta: None,
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

            let (agent_conn, client_conn) = create_connection_pair(&client, &agent);
            // 1. Start new session
            let new_session_result = agent_conn
                .new_session(NewSessionRequest {
                    mcp_servers: vec![],
                    cwd: std::path::PathBuf::from("/test"),
                    meta: None,
                })
                .await
                .expect("new_session failed");

            let session_id = new_session_result.session_id;

            // 2. Send user message
            let user_prompt = vec![ContentBlock::Text(TextContent {
                annotations: None,
                text: "Please analyze the file and summarize it".to_string(),
                meta: None,
            })];

            agent_conn
                .prompt(PromptRequest {
                    session_id: session_id.clone(),
                    prompt: user_prompt,
                    meta: None,
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
                            meta: None,
                        }),
                    },
                    meta: None,
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
                            meta: None,
                        }],
                        raw_input: None,
                        raw_output: None,
                        meta: None,
                    }),
                    meta: None,
                })
                .await
                .expect("session_notification failed");

            // 5. Agent requests permission for the tool call
            let permission_result = client_conn
                .request_permission(RequestPermissionRequest {
                    session_id: session_id.clone(),
                    tool_call: ToolCallUpdate {
                        id: tool_call_id.clone(),
                        fields: ToolCallUpdateFields {
                            title: Some("Read /test/data.txt".to_string()),
                            locations: Some(vec![ToolCallLocation {
                                path: std::path::PathBuf::from("/test/data.txt"),
                                line: None,
                                meta: None,
                            }]),
                            ..Default::default()
                        },
                        meta: None,
                    },
                    options: vec![
                        PermissionOption {
                            id: PermissionOptionId(Arc::from("allow-once")),
                            name: "Allow once".to_string(),
                            kind: PermissionOptionKind::AllowOnce,
                            meta: None,
                        },
                        PermissionOption {
                            id: PermissionOptionId(Arc::from("reject-once")),
                            name: "Reject".to_string(),
                            kind: PermissionOptionKind::RejectOnce,
                            meta: None,
                        },
                    ],
                    meta: None,
                })
                .await
                .expect("request_permission failed");

            // Verify permission was granted
            match permission_result.outcome {
                RequestPermissionOutcome::Selected { option_id } => {
                    assert_eq!(option_id.0.as_ref(), "allow-once");
                }
                RequestPermissionOutcome::Cancelled => panic!("Expected permission to be granted"),
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
                        meta: None,
                    }),
                    meta: None,
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
                                    meta: None,
                                }),
                            }]),
                            ..Default::default()
                        },
                        meta: None,
                    }),
                    meta: None,
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
                            meta: None,
                        }),
                    },
                    meta: None,
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
        ContentBlock, SessionNotification, SessionUpdate, TextContent,
        rpc::{JsonRpcMessage, OutgoingMessage},
    };
    use serde_json::{Value, json};

    // Test client -> agent notification wire format
    let outgoing_msg =
        JsonRpcMessage::wrap(OutgoingMessage::<ClientSide, AgentSide>::Notification {
            method: "cancel".into(),
            params: Some(ClientNotification::CancelNotification(CancelNotification {
                session_id: SessionId("test-123".into()),
                meta: None,
            })),
        });

    let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
    assert_eq!(
        serialized,
        json!({
            "jsonrpc": "2.0",
            "method": "cancel",
            "params": {
                "sessionId": "test-123"
            },
        })
    );

    // Test agent -> client notification wire format
    let outgoing_msg =
        JsonRpcMessage::wrap(OutgoingMessage::<AgentSide, ClientSide>::Notification {
            method: "sessionUpdate".into(),
            params: Some(AgentNotification::SessionNotification(
                SessionNotification {
                    session_id: SessionId("test-456".into()),
                    update: SessionUpdate::AgentMessageChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Hello".to_string(),
                            meta: None,
                        }),
                    },
                    meta: None,
                },
            )),
        });

    let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
    assert_eq!(
        serialized,
        json!({
            "jsonrpc": "2.0",
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

#[tokio::test]
async fn test_extension_methods_and_notifications() {
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async {
            let client = TestClient::new();
            let agent = TestAgent::new();

            // Store references to the client and agent to check notifications later
            let client_ref = client.clone();
            let agent_ref = agent.clone();

            let (client_conn, agent_conn) = create_connection_pair(&client, &agent);

            // Test agent calling client extension method
            let client_response = agent_conn
                .ext_method(ExtRequest {
                    method: "example.com/ping".into(),
                    params: raw_json!({"data": "test"}),
                })
                .await
                .unwrap();

            assert_eq!(
                serde_json::to_value(client_response).unwrap(),
                serde_json::json!({
                    "response": "pong",
                    "params": {"data": "test"}
                })
            );

            // Test client calling agent extension method
            let agent_response = client_conn
                .ext_method(ExtRequest {
                    method: "example.com/echo".into(),
                    params: raw_json!({"message": "hello"}),
                })
                .await
                .unwrap();

            assert_eq!(
                serde_json::to_value(agent_response).unwrap(),
                serde_json::json!({
                    "echo": {"message": "hello"}
                })
            );

            // Test extension notifications
            agent_conn
                .ext_notification(ExtNotification {
                    method: "example.com/client/notify".into(),
                    params: raw_json!({"info": "client notification"}),
                })
                .await
                .unwrap();

            client_conn
                .ext_notification(ExtNotification {
                    method: "example.com/agent/notify".into(),
                    params: raw_json!({"info": "agent notification"}),
                })
                .await
                .unwrap();

            // Yield to allow notifications to be processed
            tokio::task::yield_now().await;

            // Verify client received the notification
            let client_notifications = client_ref.extension_notifications.lock().unwrap();
            assert_eq!(client_notifications.len(), 1);
            assert_eq!(client_notifications[0].0, "example.com/client/notify");
            assert_eq!(
                serde_json::to_value(&client_notifications[0].1).unwrap(),
                serde_json::json!({"info": "client notification"})
            );

            // Verify agent received the notification
            let agent_notifications = agent_ref.extension_notifications.lock().unwrap();
            assert_eq!(agent_notifications.len(), 1);
            assert_eq!(agent_notifications[0].0, "example.com/agent/notify");
            assert_eq!(
                serde_json::to_value(&agent_notifications[0].1).unwrap(),
                serde_json::json!({"info": "agent notification"})
            );
        })
        .await;
}
