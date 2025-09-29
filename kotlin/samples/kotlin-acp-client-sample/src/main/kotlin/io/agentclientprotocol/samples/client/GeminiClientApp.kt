package io.agentclientprotocol.samples.client

import io.agentclientprotocol.model.ClientCapabilities
import io.agentclientprotocol.model.ContentBlock
import io.agentclientprotocol.model.FileSystemCapability
import io.agentclientprotocol.model.InitializeRequest
import io.agentclientprotocol.model.LATEST_PROTOCOL_VERSION
import io.agentclientprotocol.model.NewSessionRequest
import io.agentclientprotocol.model.PromptRequest
import io.agentclientprotocol.model.StopReason
import io.agentclientprotocol.client.ClientSideConnection
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.runBlocking
import java.io.File

private val logger = KotlinLogging.logger {}

/**
 * Interactive console chat app that communicates with a Gemini agent via ACP.
 * 
 * This demonstrates how to:
 * 1. Start an external agent process (`gemini --experimental-acp`)
 * 2. Create a client transport to communicate with the process
 * 3. Support interactive console-based conversation with the agent
 * 4. Handle real-time session updates and agent responses
 * 
 * Usage:
 * ```
 * ./gradlew :samples:kotlin-acp-client-sample:run -PmainClass=io.agentclientprotocol.samples.client.GeminiClientAppKt
 * ```
 */
suspend fun main() = coroutineScope {
    logger.info { "Starting Gemini ACP Client App" }
    
    try {
        // Create the client implementation
        val client = SimpleClient(File("."))
        
        // Create process transport to start Gemini agent
        val transport = createProcessStdioTransport(this, "gemini", "--experimental-acp")
        
        // Create client-side connection
        val connection = ClientSideConnection(this, transport, client)
        
        logger.info { "Starting Gemini agent process..." }
        
        // Connect to agent and start transport
        connection.start()
        
        logger.info { "Connected to Gemini agent, initializing..." }
        
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
        
        println("=== Successfully connected to Gemini agent ===")
        println("Protocol version: ${initResponse.protocolVersion}")
        println("Agent capabilities: ${initResponse.agentCapabilities}")
        if (initResponse.authMethods.isNotEmpty()) {
            println("Auth methods: ${initResponse.authMethods}")
        }
        println()
        
        // Create a session
        val sessionResponse = connection.newSession(
            NewSessionRequest(
                cwd = System.getProperty("user.dir"),
                mcpServers = emptyList()
            )
        )
        
        println("=== Session created: ${sessionResponse.sessionId} ===")
        println("Type your messages below. Use 'exit', 'quit', or Ctrl+C to stop.")
        println("=".repeat(60))
        println()
        
        // Start interactive chat loop
        while (true) {
            print("You: ")
            val userInput = readLine()
            
            // Check for exit conditions
            if (userInput == null || userInput.lowercase() in listOf("exit", "quit", "bye")) {
                println("\n=== Goodbye! ===")
                break
            }
            
            // Skip empty inputs
            if (userInput.isBlank()) {
                continue
            }
            
            try {
                print("Agent: ")
                
                // Send user input to agent
                val promptResponse = connection.prompt(
                    PromptRequest(
                        sessionId = sessionResponse.sessionId,
                        prompt = listOf(
                            ContentBlock.Text(userInput.trim())
                        )
                    )
                )
                
                // Give a moment for any final session updates to be processed
                kotlinx.coroutines.delay(500)
                
                when (promptResponse.stopReason) {
                    StopReason.END_TURN -> {
                        // Normal completion - no action needed
                    }
                    StopReason.MAX_TOKENS -> {
                        println("\n[Response truncated due to token limit]")
                    }
                    StopReason.MAX_TURN_REQUESTS -> {
                        println("\n[Turn limit reached]")
                    }
                    StopReason.REFUSAL -> {
                        println("\n[Agent declined to respond]")
                    }
                    StopReason.CANCELLED -> {
                        println("\n[Response was cancelled]")
                    }
                }
                
                println() // Extra newline for readability
                
            } catch (e: Exception) {
                println("\n[Error: ${e.message}]")
                logger.error(e) { "Error during chat interaction" }
                println()
            }
        }
        
    } catch (e: Exception) {
        logger.error(e) { "Client error occurred" }
        println("Error: ${e.message}")
        e.printStackTrace()
    } finally {
        logger.info { "Gemini ACP client shutting down" }
    }
}