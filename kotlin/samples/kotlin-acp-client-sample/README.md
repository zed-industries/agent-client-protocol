# Kotlin ACP Sample

This sample demonstrates how to create both ACP agents and clients in Kotlin.

## Features

### Agent Samples

1. **SimpleAgent** - A complete implementation of the ACP Agent interface that:
   - Handles initialization and capability negotiation
   - Manages sessions (create, load, cancel)
   - Processes prompts and returns responses
   - Reports tool calls and execution plans
   - Sends real-time session updates to the client

2. **AgentSample** - Runner that demonstrates the agent implementation with STDIO transport

### Client Samples

1. **SimpleClient** - An implementation of the ACP Client interface that:
   - Handles file read/write requests from the agent
   - Manages permission requests with automatic approval for read operations
   - Displays session updates and messages from the agent

2. **GeminiClientApp** - An interactive console chat application that:
   - Starts a Gemini agent process with `gemini --experimental-acp`
   - Initializes the ACP connection
   - Creates a session in the current working directory
   - Provides an **interactive console-based conversation** interface
   - Handles real-time agent responses and session updates
   - Supports graceful exit with cleanup

3. **ProcessTransportUtils** - Utility functions to create StdioTransport from external processes

## Usage

### Prerequisites

1. Ensure you have `gemini` command available with ACP support:
   ```bash
   gemini --experimental-acp --help
   ```

2. Build the project:
   ```bash
   ./gradlew :samples:kotlin-acp-client-sample:build
   ```

### Running the Samples

#### Run the Agent Sample
```bash
./gradlew :samples:kotlin-acp-client-sample:run -PmainClass=io.agentclientprotocol.samples.agent.AgentSampleKt
```

#### Run the Client Sample (Default)
```bash
./gradlew :samples:kotlin-acp-client-sample:run
```
or explicitly:
```bash
./gradlew :samples:kotlin-acp-client-sample:run -PmainClass=io.agentclientprotocol.samples.client.ClientSampleKt
```

#### Run the Gemini Interactive Chat Client
```bash
./gradlew :samples:kotlin-acp-client-sample:run -PmainClass=io.agentclientprotocol.samples.client.GeminiClientAppKt
```

### Chat Commands

Once the interactive chat starts, you can:

- **Type any message**: Just type your question or request and press Enter
- **Exit the chat**: Type `exit`, `quit`, or `bye` to end the conversation
- **Empty messages**: Press Enter with no input to skip (won't send anything to the agent)

**Example conversation:**
```
You: Hello! What can you help me with?
Agent: Hello! I'm Gemini, and I can help you with a wide variety of tasks...

You: Can you list the files in this directory?
Agent: I'll help you list the files in the current directory...
[Agent may request permission to read directory]
[Permission automatically granted by client]
Agent: Here are the files in the current directory: ...

You: exit
=== Goodbye! ===
```

### Expected Flow

When you run the Gemini client app, you should see:

1. **Process Start**: The app starts the `gemini --experimental-acp` process
2. **Connection**: Establishes ACP connection and initializes with protocol version
3. **Session Creation**: Creates a new session in the current working directory
4. **Interactive Chat**: 
   - You'll see a prompt `You: ` where you can type your messages
   - The agent responds in real-time with `Agent: ` prefix
   - Session updates (tool calls, thoughts, etc.) are displayed as they occur
   - Type `exit`, `quit`, or `bye` to end the conversation
5. **Real-time Interaction**: 
   - The agent can ask for permissions to access files
   - File operations are handled automatically by the client
   - All conversation history is maintained within the session
6. **Cleanup**: The process is terminated gracefully when you exit

### Customization

You can easily modify the `GeminiClientApp.kt` to:
- Use different commands (replace `listOf("gemini", "--experimental-acp")` in createProcessStdioTransport)
- Add custom input validation or preprocessing
- Implement different conversation modes (single-shot vs. continuous)
- Add authentication if required
- Use different client capabilities
- Customize the chat interface (colors, formatting, etc.)

## Architecture

### Agent Architecture
```
┌─────────────────┐
│   SimpleAgent   │ Implements Agent interface
└────────┬────────┘
         │
┌────────▼────────┐
│ AgentSideConn-  │ Handles protocol messages
│    ection       │
└────────┬────────┘
         │
┌────────▼────────┐
│ StdioTransport  │ JSON-RPC over STDIO
└─────────────────┘
```

### Client Architecture  
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  GeminiClient   │───▶│ ProcessTransport │───▶│ gemini process  │
│      App        │    │      Utils       │    │ --experimental- │
└─────────────────┘    └──────────────────┘    │      acp        │
          │                       │             └─────────────────┘
          │                       │                       │
          ▼                       ▼                       │
┌─────────────────┐    ┌──────────────────┐              │
│  SimpleClient   │    │ ClientSideConn-  │◀─────────────┘
│                 │◀───│     ection       │ JSON-RPC over STDIO
└─────────────────┘    └──────────────────┘
```

The client apps use ProcessTransportUtils to start external agent processes and create StdioTransport instances, while ClientSideConnection handles the ACP protocol communication. The SimpleClient implementation responds to requests from agents.

## Key Implementation Details

### ContentBlock Serialization
The `ContentBlock` sealed class uses `@JsonClassDiscriminator("type")` to ensure proper JSON serialization with the required `type` field for ACP protocol compatibility.

### Session Management
Both agent and client samples demonstrate proper session lifecycle:
- Session creation with working directory and MCP servers
- Session updates via notifications
- Session cancellation handling

### Real-time Communication
The samples show bidirectional real-time communication:
- Agents send session updates during prompt processing
- Clients receive and display updates immediately
- Tool calls and execution plans are reported in real-time