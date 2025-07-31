# Agent-Client Protocol (ACP) Rust Implementation

This module provides a Rust implementation of the Agent-Client Protocol, enabling bidirectional communication between agents and clients over JSON-RPC.

## Overview

The implementation consists of two main connection types:

- **`AgentConnection`**: Used by clients to connect to agents
- **`ClientConnection`**: Used by agents to connect to clients

Both connections support:
- Request/response communication
- Asynchronous notifications
- Full type safety with Rust's type system

## Core Components

### Method Constants

All RPC method names are defined as constants for type safety:

**Agent Methods:**
- `NEW_SESSION_METHOD_NAME`: "newSession"
- `LOAD_SESSION_METHOD_NAME`: "loadSession"
- `PROMPT_METHOD_NAME`: "prompt"

**Client Methods:**
- `REQUEST_PERMISSION_METHOD_NAME`: "request_permission"
- `WRITE_TEXT_FILE_METHOD_NAME`: "write_text_file"
- `READ_TEXT_FILE_METHOD_NAME`: "read_text_file"

### AgentConnection

Used by clients to communicate with agents:

```rust
let (agent_conn, io_task) = AgentConnection::new(
    client_impl,        // Your Client trait implementation
    outgoing_bytes,     // AsyncWrite stream
    incoming_bytes,     // AsyncRead stream
    |fut| { /* spawn future */ }
);

// Available methods:
agent_conn.new_session(arguments).await?;
agent_conn.load_session(arguments).await?;
agent_conn.prompt(arguments).await?;
agent_conn.cancel(request_id)?;  // Send cancellation notification

// Set up callback for incoming session updates
agent_conn.on_session_update(|notification| {
    println!("Session {}: {:?}", notification.session_id, notification.update);
});
```

### ClientConnection

Used by agents to communicate with clients:

```rust
let (client_conn, io_task) = ClientConnection::new(
    agent_impl,         // Your Agent trait implementation
    outgoing_bytes,     // AsyncWrite stream
    incoming_bytes,     // AsyncRead stream
    |fut| { /* spawn future */ }
);

// Available methods:
client_conn.request_permission(arguments).await?;
client_conn.write_text_file(arguments).await?;
client_conn.read_text_file(arguments).await?;
client_conn.send_session_update(session_id, update)?;  // Send session update notification

// Set up callback for incoming cancellation requests
client_conn.on_cancel(|request_id| {
    println!("Request {} was cancelled", request_id);
});
```

### Dispatchers

The implementation uses two internal dispatchers:

- **`AgentDispatcher`**: Handles incoming requests to the agent (newSession, loadSession, prompt) and notifications from the client (cancelled)
- **`ClientDispatcher`**: Handles incoming requests to the client (request_permission, write_text_file, read_text_file) and notifications from the agent (sessionUpdate)

### Notifications

**Client → Agent:**
- `ClientNotification::Cancelled { request_id }`: Notify agent that a request was cancelled

**Agent → Client:**
- `AgentNotification::SessionUpdate(SessionNotification)`: Send session updates including message chunks, tool calls, and plans

### Callbacks

Both connection types support callbacks for handling incoming notifications:

- **`AgentConnection::on_session_update`**: Called when the agent sends a session update notification
- **`ClientConnection::on_cancel`**: Called when the client sends a cancellation notification

These callbacks are optional and can be set at any time after creating the connection.

## Usage Example

```rust
use agent_client_protocol::{
    Agent, AgentConnection, Client, ClientConnection,
    NewSessionArguments, SessionId, SessionUpdate, ContentBlock,
};

// Implement the Agent trait
struct MyAgent;
impl Agent for MyAgent {
    // ... implement required methods
}

// Implement the Client trait
struct MyClient;
impl Client for MyClient {
    // ... implement required methods
}

// Create connections
let (client_read, client_write) = create_transport();
let (agent_read, agent_write) = create_transport();

// Client side - connects to agent
let (agent_conn, agent_io) = AgentConnection::new(
    MyClient,
    client_write,
    agent_read,
    |fut| tokio::spawn(fut)
);

// Agent side - connects to client
let (client_conn, client_io) = ClientConnection::new(
    MyAgent,
    agent_write,
    client_read,
    |fut| tokio::spawn(fut)
);

// Run IO tasks
tokio::select! {
    _ = agent_io => {}
    _ = client_io => {}
}
```

## Implementation Details

### Type Safety

The implementation leverages Rust's type system to ensure:
- Correct request/response pairing
- Proper notification types for each direction
- Compile-time verification of method parameters

### Error Handling

All methods return `Result<T, Error>` where `Error` follows JSON-RPC 2.0 error codes:
- Parse errors (-32700)
- Invalid requests (-32600)
- Method not found (-32601)
- Invalid parameters (-32602)
- Internal errors (-32603)

### Async Support

The implementation is fully asynchronous using:
- `futures` for async traits and channels
- `LocalBoxFuture` for trait methods
- Configurable spawn function for executor flexibility

### Transport Agnostic

The connections work with any `AsyncRead + AsyncWrite` streams, allowing use with:
- TCP sockets
- Unix domain sockets
- In-memory channels
- WebSocket connections
- Any other byte stream transport
