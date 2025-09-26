package io.agentclientprotocol.agent

import io.agentclientprotocol.model.AuthenticateRequest
import io.agentclientprotocol.model.CancelNotification
import io.agentclientprotocol.model.InitializeRequest
import io.agentclientprotocol.model.InitializeResponse
import io.agentclientprotocol.model.LoadSessionRequest
import io.agentclientprotocol.model.NewSessionRequest
import io.agentclientprotocol.model.NewSessionResponse
import io.agentclientprotocol.model.PromptRequest
import io.agentclientprotocol.model.PromptResponse

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
     * Called when the client wants to authenticate with a specific method
     * that was advertised in the initialize response.
     *
     * @param request The authentication request containing the method ID
     * @return null (authentication is just a handshake)
     */
    public suspend fun authenticate(request: AuthenticateRequest)

    /**
     * Create a new conversation session.
     *
     * Sessions maintain their own context and allow multiple independent
     * conversations with the same agent.
     *
     * See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
     */
    public suspend fun newSession(request: NewSessionRequest): NewSessionResponse

    /**
     * Load an existing conversation session.
     *
     * Only called if the agent advertises the `loadSession` capability.
     * Allows resuming previous conversations.
     *
     * See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
     */
    public suspend fun loadSession(request: LoadSessionRequest)

    /**
     * Process a user prompt within a session.
     *
     * This is the main method for handling user interactions. The agent should:
     * 1. Process the prompt content
     * 2. Send session updates via the client connection
     * 3. Execute any necessary tool calls (with permission if needed)
     * 4. Return a final response with the stop reason
     *
     * See protocol docs: [User Message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
     */
    public suspend fun prompt(request: PromptRequest): PromptResponse

    /**
     * Cancel ongoing operations for a session.
     *
     * The client sends this notification to request cancellation of the current
     * prompt turn. The agent should stop processing and return a cancelled response.
     *
     * See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
     */
    public suspend fun cancel(notification: CancelNotification)
}