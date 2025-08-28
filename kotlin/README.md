# ACP Kotlin SDK

[![Kotlin Multiplatform](https://img.shields.io/badge/Kotlin-Multiplatform-blueviolet?logo=kotlin)](https://kotlinlang.org/docs/multiplatform.html)
[![JVM](https://img.shields.io/badge/Platform-JVM-orange?logo=kotlin)](https://kotlinlang.org/docs/multiplatform.html)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Kotlin implementation of the [Agent Client Protocol](https://agentclientprotocol.com) (ACP), providing both client and agent capabilities for integrating with AI agents across various platforms.

## Overview

The Agent Client Protocol allows applications to provide a standardized interface for AI agents, enabling seamless communication between clients (like code editors) and agents (AI assistants). This SDK implements the ACP specification for Kotlin, currently targeting JVM with future multiplatform support planned.

- Build ACP clients that can connect to any ACP agent
- Create ACP agents that expose capabilities to clients  
- Use STDIO transport for process communication
- Handle all ACP protocol messages and lifecycle events
- Full support for sessions, tool calls, permissions, and file system operations

## Project Structure

- **kotlin-acp**: Single module containing all ACP functionality
  - `agent/`: Agent-side implementation (`Agent.kt`, `AgentSideConnection.kt`)
  - `client/`: Client-side implementation (`Client.kt`, `ClientSideConnection.kt`)
  - `model/`: Protocol data models and types
  - `protocol/`: Core protocol handling
  - `rpc/`: JSON-RPC implementation
  - `transport/`: Transport layer implementation (STDIO)
- **samples/**: Example implementations demonstrating usage

## Installation

Add the repository to your build file:

```kotlin
repositories {
    mavenCentral()
}
```

Add the dependency:

```kotlin
dependencies {
    implementation("io.agentclientprotocol:kotlin-acp:0.1.0-SNAPSHOT")
}
```

## Quick Start

### Creating an Agent

```kotlin
import io.agentclientprotocol.agent.*
import io.agentclientprotocol.model.*
import io.agentclientprotocol.transport.StdioTransport
import kotlinx.coroutines.runBlocking
import kotlinx.io.*

class MyAgent : Agent {
    override suspend fun initialize(request: InitializeRequest): InitializeResponse {
        return InitializeResponse(
            protocolVersion = LATEST_PROTOCOL_VERSION,
            agentCapabilities = AgentCapabilities(
                promptCapabilities = PromptCapabilities(
                    image = true,
                    embeddedContext = true
                )
            )
        )
    }
    
    override suspend fun newSession(request: NewSessionRequest): NewSessionResponse {
        val sessionId = SessionId("session-${System.currentTimeMillis()}")
        return NewSessionResponse(sessionId)
    }
    
    override suspend fun prompt(request: PromptRequest): PromptResponse {
        // Process the user's prompt and send updates via client connection
        return PromptResponse(StopReason.END_TURN)
    }
    
    // Implement other required methods...
}

// Set up agent with STDIO transport
fun main() = runBlocking {
    val agent = MyAgent()
    val transport = StdioTransport(System.`in`.asSource(), System.out.asSink())
    val connection = AgentSideConnection(agent)
    
    connection.connect(transport)
    transport.start()
}
```

### Creating a Client

```kotlin
import io.agentclientprotocol.client.*
import io.agentclientprotocol.model.*
import io.agentclientprotocol.transport.StdioTransport
import kotlinx.coroutines.runBlocking
import kotlinx.io.*
import java.io.File

class MyClient : Client {
    override suspend fun readTextFile(request: ReadTextFileRequest): ReadTextFileResponse {
        val content = File(request.path).readText()
        return ReadTextFileResponse(content)
    }
    
    override suspend fun writeTextFile(request: WriteTextFileRequest) {
        File(request.path).writeText(request.content)
    }
    
    override suspend fun requestPermission(request: RequestPermissionRequest): RequestPermissionResponse {
        // Present options to user and return their choice
        val selectedOption = request.options.first()
        return RequestPermissionResponse(
            RequestPermissionOutcome.Selected(selectedOption.optionId)
        )
    }
    
    override suspend fun sessionUpdate(notification: SessionNotification) {
        // Handle real-time updates from agent
        println("Agent update: ${notification.update}")
    }
}

// Set up client
fun main() = runBlocking {
    val client = MyClient()
    // Create transport from external process (agent)
    val agentProcess = ProcessBuilder("gemini", "--experimental-acp").start()
    val transport = StdioTransport(
        agentProcess.inputStream.asSource(),
        agentProcess.outputStream.asSink()
    )
    val connection = ClientSideConnection(client)
    
    connection.connect(transport)
    transport.start()
    
    // Initialize agent
    val initResponse = connection.initialize(InitializeRequest(
        protocolVersion = LATEST_PROTOCOL_VERSION,
        clientCapabilities = ClientCapabilities(
            fs = FileSystemCapability(
                readTextFile = true,
                writeTextFile = true
            )
        )
    ))
    
    // Create session and send prompts
    val sessionResponse = connection.newSession(NewSessionRequest(
        cwd = "/path/to/working/directory",
        mcpServers = emptyList()
    ))
    
    val promptResponse = connection.prompt(PromptRequest(
        sessionId = sessionResponse.sessionId,
        prompt = listOf(ContentBlock.Text("Hello, agent!"))
    ))
}
```

## Samples

The `samples/` directory contains complete working examples:

- **kotlin-acp-client-sample**: Contains both agent and client examples
  - `SimpleAgent.kt`: A basic agent implementation that echoes messages
  - `SimpleClient.kt`: A client implementation with file operations and permissions
  - `GeminiClientApp.kt`: Interactive chat application that connects to Gemini agent
  - `AgentSample.kt` & `ClientSample.kt`: Sample runners

Run the samples:

```bash
# Run the default client sample
./gradlew :samples:kotlin-acp-client-sample:run

# Run the Gemini interactive chat client
./gradlew :samples:kotlin-acp-client-sample:run -PmainClass=io.agentclientprotocol.samples.client.GeminiClientAppKt
```

## Features

### Protocol Support
- ✅ Full ACP v1 protocol implementation
- ✅ JSON-RPC message handling with request/response correlation
- ✅ Support for all ACP message types (requests, responses, notifications)

### Agent Features
- ✅ Agent initialization and capability negotiation
- ✅ Session management (create, load, cancel)
- ✅ Prompt processing with real-time updates
- ✅ Tool call reporting and progress updates
- ✅ Execution plan reporting
- ✅ File system operations (via client)
- ✅ Permission requests

### Client Features  
- ✅ Client initialization and capability advertising
- ✅ File system operations (read/write text files)
- ✅ Permission handling and user prompts
- ✅ Real-time session update processing
- ✅ Agent lifecycle management

### Transport Support
- ✅ STDIO transport for process communication

### Multiplatform Support
- ✅ JVM target
- 🚧 JavaScript/Node.js (planned)
- 🚧 Native targets (planned)
- 🚧 WebAssembly (planned)

## Architecture

The SDK follows a clean architecture with clear separation of concerns:

```
┌─────────────────┐    ┌─────────────────┐
│   Agent App     │    │   Client App    │
├─────────────────┤    ├─────────────────┤
│ AgentSideConn   │    │ ClientSideConn  │
├─────────────────┤    ├─────────────────┤
│    Protocol     │    │    Protocol     │
├─────────────────┤    ├─────────────────┤
│   Transport     │    │   Transport     │
│     (STDIO)     │◄──►│     (STDIO)     │
└─────────────────┘    └─────────────────┘
```

- **Transport Layer**: Handles raw message transmission via STDIO
- **Protocol Layer**: Manages JSON-RPC framing, request correlation, and error handling
- **Connection Layer**: Provides type-safe ACP method calls and handles serialization
- **Application Layer**: Your agent or client implementation