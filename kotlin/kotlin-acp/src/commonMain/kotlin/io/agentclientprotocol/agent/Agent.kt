package io.agentclientprotocol.agent

import io.agentclientprotocol.model.AuthenticateRequest
import io.agentclientprotocol.model.AuthenticateResponse
import io.agentclientprotocol.model.CancelNotification
import io.agentclientprotocol.model.InitializeRequest
import io.agentclientprotocol.model.InitializeResponse
import io.agentclientprotocol.model.LoadSessionRequest
import io.agentclientprotocol.model.LoadSessionResponse
import io.agentclientprotocol.model.NewSessionRequest
import io.agentclientprotocol.model.NewSessionResponse
import io.agentclientprotocol.model.PromptRequest
import io.agentclientprotocol.model.PromptResponse
import io.agentclientprotocol.model.SetSessionModeRequest
import io.agentclientprotocol.model.SetSessionModeResponse

/**
 * Interface that agents must implement to handle client requests.
 *
 * This interface defines the contract for agent implementations,
 * covering the full agent lifecycle from initialization through
 * session management and prompt processing.
 *
 * See protocol docs: [Agent](https://agentclientprotocol.com/protocol/overview#agent)
 */
public interface Agent {
    /**
     * Initialize the agent with client capabilities and protocol version.
     *
     * This is the first method called when a client connects to the agent.
     * The agent should validate the protocol version and store client capabilities.
     *
     * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
     */
    public suspend fun initialize(request: InitializeRequest): InitializeResponse

    /**
     * Authenticate using the specified authentication method.
     *
     * Called when the agent requires authentication before allowing session creation.
     * The client provides the authentication method ID that was advertised during initialization.
     * After successful authentication, the client can proceed to create sessions without
     * receiving an `auth_required` error.
     *
     * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
     *
     * @param request The authentication request containing the method ID
     * @return Authentication response
     */
    public suspend fun authenticate(request: AuthenticateRequest): AuthenticateResponse

    /**
     * Create a new conversation session.
     *
     * Sessions represent independent conversation contexts with their own history and state.
     * The agent should create a new session context, connect to any specified MCP servers,
     * and return a unique session ID for future requests.
     *
     * May return an `auth_required` error if the agent requires authentication.
     *
     * See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
     *
     * @param request The session creation request with working directory and MCP servers
     * @return The session ID and optional mode/model state
     */
    public suspend fun sessionNew(request: NewSessionRequest): NewSessionResponse

    /**
     * Load an existing conversation session to resume a previous conversation.
     *
     * Only called if the agent advertises the `loadSession` capability.
     * The agent should restore the session context and conversation history, connect to
     * the specified MCP servers, and stream the entire conversation history back to the
     * client via notifications.
     *
     * See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
     *
     * @param request The session load request with session ID, working directory, and MCP servers
     * @return Optional mode/model state for the loaded session
     */
    public suspend fun sessionLoad(request: LoadSessionRequest): LoadSessionResponse

    /**
     * Sets the operational mode for a session.
     *
     * Allows switching between different agent modes (e.g., "ask", "architect", "code")
     * that affect system prompts, tool availability, and permission behaviors. The mode
     * must be one of the modes advertised in `availableModes` during session creation or loading.
     *
     * This method can be called at any time during a session, whether the agent is idle or
     * actively generating a turn. Agents may also change modes autonomously and notify the
     * client via `current_mode_update` notifications.
     *
     * See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
     *
     * @param request The set mode request with session ID and mode ID
     * @return Set session mode response
     */
    public suspend fun sessionSetMode(request: SetSessionModeRequest): SetSessionModeResponse

    /**
     * Process a user prompt within a session.
     *
     * This method handles the full lifecycle of a prompt turn:
     * 1. Receives user messages with optional context (files, images, etc.)
     * 2. Processes the prompt using language models
     * 3. Reports language model content and tool calls to the client via [sessionUpdate]
     * 4. Requests permission to run tools via client's [sessionRequestPermission]
     * 5. Executes approved tool calls
     * 6. Returns when the turn is complete with a stop reason
     *
     * See protocol docs: [Prompt Turn](https://agentclientprotocol.com/protocol/prompt-turn)
     *
     * @param request The prompt request with session ID and content blocks
     * @return The stop reason (end_turn, max_tokens, max_turn_requests, refusal, or cancelled)
     */
    public suspend fun sessionPrompt(request: PromptRequest): PromptResponse

    /**
     * Cancel ongoing operations for a session.
     *
     * This is a notification sent by the client to cancel an ongoing prompt turn.
     * Upon receiving this notification, the agent should:
     * - Stop all language model requests as soon as possible
     * - Abort all tool call invocations in progress
     * - Send any pending session update notifications
     * - Respond to the original [sessionPrompt] request with `StopReason.CANCELLED`
     *
     * See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
     *
     * @param notification The cancellation notification with session ID
     */
    public suspend fun sessionCancel(notification: CancelNotification)
}