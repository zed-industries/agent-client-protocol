package io.agentclientprotocol.samples.client

import io.agentclientprotocol.model.ClientCapabilities
import io.agentclientprotocol.model.ContentBlock
import io.agentclientprotocol.model.FileSystemCapability
import io.agentclientprotocol.model.InitializeRequest
import io.agentclientprotocol.model.LATEST_PROTOCOL_VERSION
import io.agentclientprotocol.model.NewSessionRequest
import io.agentclientprotocol.model.PromptRequest
import io.agentclientprotocol.client.ClientSideConnection
import io.agentclientprotocol.transport.StdioTransport
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.runBlocking
import kotlinx.io.asSource
import kotlinx.io.asSink
import kotlinx.io.buffered
import java.io.File

private val logger = KotlinLogging.logger {}

/**
 * Sample ACP client using STDIO transport.
 * 
 * This demonstrates how to create and run an ACP client that communicates
 * with an agent via standard input/output streams.
 * 
 * Usage:
 * ```
 * ./gradlew :samples:kotlin-acp-client-sample:run
 * ```
 */
fun main() = runBlocking {
    logger.info { "Starting ACP Client Sample" }
    
    try {
        // Create the client implementation
        val client = SimpleClient(File("."))
        
        // Create STDIO transport (for subprocess communication)
        // In a real scenario, this would connect to an agent process
        val transport = StdioTransport(
            input = System.`in`.asSource().buffered(),
            output = System.out.asSink().buffered()
        )
        
        // Create client-side connection
        val connection = ClientSideConnection(client)
        
        // Connect to agent
        connection.start(transport)
        
        // Initialize the agent
        val initResponse = connection.initialize(
            InitializeRequest(
                protocolVersion = LATEST_PROTOCOL_VERSION,
                clientCapabilities = ClientCapabilities(
                    fs = FileSystemCapability(
                        readTextFile = true,
                        writeTextFile = true
                    )
                )
            )
        )
        
        println("Connected to agent:")
        println("  Protocol version: ${initResponse.protocolVersion}")
        println("  Agent capabilities: ${initResponse.agentCapabilities}")
        println("  Auth methods: ${initResponse.authMethods}")
        
        // Create a session
        val sessionResponse = connection.newSession(
            NewSessionRequest(
                cwd = System.getProperty("user.dir"),
                mcpServers = emptyList()
            )
        )
        
        println("Created session: ${sessionResponse.sessionId}")
        
        // Send a test prompt
        val promptResponse = connection.prompt(
            PromptRequest(
                sessionId = sessionResponse.sessionId,
                prompt = listOf(
                    ContentBlock.Text("Hello, I'm testing the ACP Kotlin SDK!")
                )
            )
        )
        
        println("Prompt completed with stop reason: ${promptResponse.stopReason}")
        
        // Keep client running for a bit to receive any final updates
        kotlinx.coroutines.delay(2000)
        
    } catch (e: Exception) {
        logger.error(e) { "Client error" }
    } finally {
        logger.info { "Client shutting down" }
    }
}