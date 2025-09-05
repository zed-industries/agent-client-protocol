package client

import schema.AuthenticateRequest
import schema.CancelNotification
import schema.InitializeRequest
import schema.InitializeResponse
import schema.LoadSessionRequest
import schema.NewSessionRequest
import schema.NewSessionResponse
import schema.PromptRequest
import schema.PromptResponse

/**
 * The Agent interface defines the interface that all ACP-compliant agents must implement.
 *
 * Agents are programs that use generative AI to autonomously modify code. They handle
 * requests from clients and execute tasks using language models and tools.
 */
interface Agent {
    /**
     * Establishes the connection with a client and negotiates protocol capabilities.
     *
     * This method is called once at the beginning of the connection to:
     * - Negotiate the protocol version to use
     * - Exchange capability information between client and agent
     * - Determine available authentication methods
     *
     * The agent should respond with its supported protocol version and capabilities.
     *
     * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
     */
    suspend fun initialize(params: InitializeRequest): InitializeResponse

    /**
     * Creates a new conversation session with the agent.
     *
     * Sessions represent independent conversation contexts with their own history and state.
     *
     * The agent should:
     * - Create a new session context
     * - Connect to any specified MCP servers
     * - Return a unique session ID for future requests
     *
     * May return an `auth_required` error if the agent requires authentication.
     *
     * See protocol docs: [Session Setup](https://agentclientprotocol.com/protocol/session-setup)
     */
    suspend fun newSession(params: NewSessionRequest): NewSessionResponse

    /**
     * Loads an existing session to resume a previous conversation.
     *
     * This method is only available if the agent advertises the `loadSession` capability.
     *
     * The agent should:
     * - Restore the session context and conversation history
     * - Connect to the specified MCP servers
     * - Stream the entire conversation history back to the client via notifications
     *
     * See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
     */
    suspend fun loadSession(params: LoadSessionRequest) {} // optional

    /**
     * Authenticates the client using the specified authentication method.
     *
     * Called when the agent requires authentication before allowing session creation.
     * The client provides the authentication method ID that was advertised during initialization.
     *
     * After successful authentication, the client can proceed to create sessions with
     * `newSession` without receiving an `auth_required` error.
     *
     * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
     */
    suspend fun authenticate(params: AuthenticateRequest)

    /**
     * Processes a user prompt within a session.
     *
     * This method handles the whole lifecycle of a prompt:
     * - Receives user messages with optional context (files, images, etc.)
     * - Processes the prompt using language models
     * - Reports language model content and tool calls to the Clients
     * - Requests permission to run tools
     * - Executes any requested tool calls
     * - Returns when the turn is complete with a stop reason
     *
     * See protocol docs: [Prompt Turn](https://agentclientprotocol.com/protocol/prompt-turn)
     */
    suspend fun prompt(params: PromptRequest): PromptResponse

    /**
     * Cancels ongoing operations for a session.
     *
     * This is a notification sent by the client to cancel an ongoing prompt turn.
     *
     * Upon receiving this notification, the Agent SHOULD:
     * - Stop all language model requests as soon as possible
     * - Abort all tool call invocations in progress
     * - Send any pending `session/update` notifications
     * - Respond to the original `session/prompt` request with `StopReason::Cancelled`
     *
     * See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
     */
    suspend fun cancel(params: CancelNotification)
}