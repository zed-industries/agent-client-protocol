package io.agentclientprotocol.samples.agent

import io.agentclientprotocol.agent.Agent
import io.agentclientprotocol.model.AgentCapabilities
import io.agentclientprotocol.model.AuthenticateRequest
import io.agentclientprotocol.model.CancelNotification
import io.agentclientprotocol.client.Client
import io.agentclientprotocol.model.AuthenticateResponse
import io.agentclientprotocol.model.ClientCapabilities
import io.agentclientprotocol.model.ContentBlock
import io.agentclientprotocol.model.InitializeRequest
import io.agentclientprotocol.model.InitializeResponse
import io.agentclientprotocol.model.LATEST_PROTOCOL_VERSION
import io.agentclientprotocol.model.LoadSessionRequest
import io.agentclientprotocol.model.LoadSessionResponse
import io.agentclientprotocol.model.McpServer
import io.agentclientprotocol.model.NewSessionRequest
import io.agentclientprotocol.model.NewSessionResponse
import io.agentclientprotocol.model.Plan
import io.agentclientprotocol.model.PlanEntry
import io.agentclientprotocol.model.PlanEntryPriority
import io.agentclientprotocol.model.PlanEntryStatus
import io.agentclientprotocol.model.PromptCapabilities
import io.agentclientprotocol.model.PromptRequest
import io.agentclientprotocol.model.PromptResponse
import io.agentclientprotocol.model.SessionId
import io.agentclientprotocol.model.SessionNotification
import io.agentclientprotocol.model.SessionUpdate
import io.agentclientprotocol.model.SetSessionModeRequest
import io.agentclientprotocol.model.SetSessionModeResponse
import io.agentclientprotocol.model.StopReason
import io.agentclientprotocol.model.ToolCallContent
import io.agentclientprotocol.model.ToolCallId
import io.agentclientprotocol.model.ToolCallLocation
import io.agentclientprotocol.model.ToolCallStatus
import io.agentclientprotocol.model.ToolKind

import io.github.oshai.kotlinlogging.KotlinLogging
import kotlinx.coroutines.delay
import java.util.concurrent.ConcurrentHashMap

private val logger = KotlinLogging.logger {}

/**
 * Simple example agent implementation.
 *
 * This agent demonstrates basic ACP functionality including:
 * - Session management
 * - Content processing
 * - Tool call simulation
 * - Plan reporting
 *
 * Note: This agent needs a way to send updates back to the client.
 * Use [withClient] to create a wrapper that can send updates.
 */
class SimpleAgent : Agent {
    private val sessions = ConcurrentHashMap<SessionId, SessionContext>()
    private var clientCapabilities: ClientCapabilities? = null

    // Callback for sending session updates - set by the connection wrapper
    internal var onSessionUpdate: (suspend (SessionNotification) -> Unit)? = null

    data class SessionContext(
        val sessionId: SessionId,
        val cwd: String,
        val mcpServers: List<McpServer>,
        var cancelled: Boolean = false
    )
    
    override suspend fun initialize(request: InitializeRequest): InitializeResponse {
        logger.info { "Initializing agent with protocol version ${request.protocolVersion}" }
        clientCapabilities = request.clientCapabilities
        
        return InitializeResponse(
            protocolVersion = LATEST_PROTOCOL_VERSION,
            agentCapabilities = AgentCapabilities(
                loadSession = true,
                promptCapabilities = PromptCapabilities(
                    audio = false,
                    image = true,
                    embeddedContext = true
                )
            ),
            authMethods = emptyList() // No authentication required
        )
    }
    
    override suspend fun authenticate(request: AuthenticateRequest): AuthenticateResponse {
        logger.info { "Authentication requested with method: ${request.methodId}" }
        // No-op: this agent doesn't require authentication
        return AuthenticateResponse()
    }
    
    override suspend fun sessionNew(request: NewSessionRequest): NewSessionResponse {
        val sessionId = SessionId("session-${System.currentTimeMillis()}")
        val context = SessionContext(
            sessionId = sessionId,
            cwd = request.cwd,
            mcpServers = request.mcpServers
        )
        sessions[sessionId] = context
        
        logger.info { "Created new session: $sessionId in directory: ${request.cwd}" }
        return NewSessionResponse(sessionId)
    }
    
    override suspend fun sessionLoad(request: LoadSessionRequest): LoadSessionResponse {
        val context = SessionContext(
            sessionId = request.sessionId,
            cwd = request.cwd,
            mcpServers = request.mcpServers
        )
        sessions[request.sessionId] = context

        logger.info { "Loaded session: ${request.sessionId}" }
        return LoadSessionResponse()
    }

    override suspend fun sessionSetMode(request: SetSessionModeRequest): SetSessionModeResponse {
        logger.info { "Session mode change requested for session ${request.sessionId} to mode ${request.modeId}" }
        // This simple agent doesn't support multiple modes, so just acknowledge
        return SetSessionModeResponse()
    }

    override suspend fun sessionPrompt(request: PromptRequest): PromptResponse {
        val context = sessions[request.sessionId]
            ?: throw IllegalArgumentException("Unknown session: ${request.sessionId}")
            
        if (context.cancelled) {
            return PromptResponse(StopReason.CANCELLED)
        }
        
        logger.info { "Processing sessionPrompt for session ${request.sessionId}" }
        
        try {
            // Send initial plan
            sendPlan(request.sessionId)
            
            // Echo the user's message
            for (block in request.prompt) {
                onSessionUpdate?.invoke(
                    SessionNotification(
                        sessionId = request.sessionId,
                        update = SessionUpdate.UserMessageChunk(block)
                    )
                )
                delay(100) // Simulate processing time
            }

            // Send agent response
            val responseText = "I received your message: ${
                request.prompt.filterIsInstance<ContentBlock.Text>()
                    .joinToString(" ") { it.text }
            }"

            onSessionUpdate?.invoke(
                SessionNotification(
                    sessionId = request.sessionId,
                    update = SessionUpdate.AgentMessageChunk(
                        ContentBlock.Text(responseText)
                    )
                )
            )
            
            // Simulate a tool call if client supports file operations
            if (clientCapabilities?.fs?.readTextFile == true) {
                simulateToolCall(request.sessionId)
            }
            
            return PromptResponse(StopReason.END_TURN)
            
        } catch (e: Exception) {
            logger.error(e) { "Error processing sessionPrompt" }
            return PromptResponse(StopReason.REFUSAL)
        }
    }
    
    override suspend fun sessionCancel(notification: CancelNotification) {
        logger.info { "Cancellation requested for session: ${notification.sessionId}" }
        sessions[notification.sessionId]?.cancelled = true
    }
    
    private suspend fun sendPlan(sessionId: SessionId) {
        val plan = Plan(
            listOf(
                PlanEntry("Process user input", PlanEntryPriority.HIGH, PlanEntryStatus.IN_PROGRESS),
                PlanEntry("Generate response", PlanEntryPriority.HIGH, PlanEntryStatus.PENDING),
                PlanEntry("Execute tools if needed", PlanEntryPriority.MEDIUM, PlanEntryStatus.PENDING)
            )
        )
        
        onSessionUpdate?.invoke(
            SessionNotification(
                sessionId = sessionId,
                update = SessionUpdate.PlanUpdate(plan.entries)
            )
        )
    }
    
    private suspend fun simulateToolCall(sessionId: SessionId) {
        val toolCallId = ToolCallId("tool-${System.currentTimeMillis()}")

        // Start tool call
        onSessionUpdate?.invoke(
            SessionNotification(
                sessionId = sessionId,
                update = SessionUpdate.ToolCallUpdate(
                    toolCallId = toolCallId,
                    title = "Reading current directory",
                    kind = ToolKind.READ,
                    status = ToolCallStatus.PENDING,
                    locations = listOf(ToolCallLocation(".")),
                    content = emptyList()
                )
            )
        )

        delay(500) // Simulate work

        // Update to in progress
        onSessionUpdate?.invoke(
            SessionNotification(
                sessionId = sessionId,
                update = SessionUpdate.ToolCallUpdate(
                    toolCallId = toolCallId,
                    status = ToolCallStatus.IN_PROGRESS
                )
            )
        )

        delay(500) // Simulate more work

        // Complete the tool call
        onSessionUpdate?.invoke(
            SessionNotification(
                sessionId = sessionId,
                update = SessionUpdate.ToolCallUpdate(
                    toolCallId = toolCallId,
                    status = ToolCallStatus.COMPLETED,
                    content = listOf(
                        ToolCallContent.Content(
                            ContentBlock.Text("Directory listing completed successfully")
                        )
                    )
                )
            )
        )
    }
}