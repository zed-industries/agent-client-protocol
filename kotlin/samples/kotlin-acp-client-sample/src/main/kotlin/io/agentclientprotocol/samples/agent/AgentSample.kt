package io.agentclientprotocol.samples.agent

import io.agentclientprotocol.agent.AgentSideConnection
import io.agentclientprotocol.transport.StdioTransport
import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.io.asSource
import kotlinx.io.asSink
import kotlinx.io.buffered

private val logger = KotlinLogging.logger {}

/**
 * Sample ACP agent using STDIO transport.
 * 
 * This demonstrates how to create and run an ACP agent that communicates
 * via standard input/output streams.
 * 
 * Usage:
 * ```
 * ./gradlew :samples:kotlin-acp-agent-sample:run
 * ```
 */
suspend fun main() = coroutineScope {
    logger.info { "Starting ACP Agent Sample" }

    try {
        // Create STDIO transport
        val transport = StdioTransport(
            parentScope = this,
            input = System.`in`.asSource().buffered(),
            output = System.out.asSink().buffered()
        )

        // Create agent
        val agent = SimpleAgent()

        // Create agent-side connection - this implements Client interface
        val connection = AgentSideConnection(this, agent, transport)

        // Wire up the agent to send updates through the connection
        agent.onSessionUpdate = { notification ->
            connection.sessionUpdate(notification)
        }

        // Connect and start processing
        connection.start()
        
        logger.info { "Agent connected and ready" }
        
        // Keep the agent running
        // In a real implementation, you might want to handle shutdown signals
        while (transport.isConnected) {
            delay(1000)
        }
        
    } catch (e: Exception) {
        logger.error(e) { "Agent error" }
    } finally {
        logger.info { "Agent shutting down" }
    }
}