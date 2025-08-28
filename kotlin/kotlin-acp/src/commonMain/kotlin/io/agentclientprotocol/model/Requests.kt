@file:Suppress("unused")

package io.agentclientprotocol.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * Describes an available authentication method.
 */
@Serializable
public data class AuthMethod(
    val id: AuthMethodId,
    val name: String,
    val description: String? = null
)

/**
 * An environment variable to set when launching an MCP server.
 */
@Serializable
public data class EnvVariable(
    val name: String,
    val value: String
)

/**
 * An HTTP header to set when making requests to the MCP server.
 */
@Serializable
public data class HttpHeader(
    val name: String,
    val value: String
)

/**
 * Configuration for connecting to an MCP (Model Context Protocol) server.
 *
 * MCP servers provide tools and context that the agent can use when
 * processing prompts.
 *
 * See protocol docs: [MCP Servers](https://agentclientprotocol.com/protocol/session-setup#mcp-servers)
 */
@Serializable
public sealed class McpServer {
    public abstract val name: String
    
    /**
     * Stdio transport configuration
     *
     * All Agents MUST support this transport.
     */
    @Serializable
    @SerialName("stdio")
    public data class Stdio(
        override val name: String,
        val command: String,
        val args: List<String>,
        val env: List<EnvVariable>
    ) : McpServer()
    
    /**
     * HTTP transport configuration
     *
     * Only available when the Agent capabilities indicate `mcp_capabilities.http` is `true`.
     */
    @Serializable
    @SerialName("http")
    public data class Http(
        override val name: String,
        val url: String,
        val headers: List<HttpHeader>
    ) : McpServer()
    
    /**
     * SSE transport configuration
     *
     * Only available when the Agent capabilities indicate `mcp_capabilities.sse` is `true`.
     */
    @Serializable
    @SerialName("sse")
    public data class Sse(
        override val name: String,
        val url: String,
        val headers: List<HttpHeader>
    ) : McpServer()
}

/**
 * Reasons why an agent stops processing a prompt turn.
 *
 * See protocol docs: [Stop Reasons](https://agentclientprotocol.com/protocol/prompt-turn#stop-reasons)
 */
@Serializable
public enum class StopReason {
    @SerialName("end_turn") END_TURN,
    @SerialName("max_tokens") MAX_TOKENS,
    @SerialName("max_turn_requests") MAX_TURN_REQUESTS,
    @SerialName("refusal") REFUSAL,
    @SerialName("cancelled") CANCELLED
}

/**
 * The type of permission option being presented to the user.
 *
 * Helps clients choose appropriate icons and UI treatment.
 */
@Serializable
public enum class PermissionOptionKind {
    @SerialName("allow_once") ALLOW_ONCE,
    @SerialName("allow_always") ALLOW_ALWAYS,
    @SerialName("reject_once") REJECT_ONCE,
    @SerialName("reject_always") REJECT_ALWAYS
}

/**
 * An option presented to the user when requesting permission.
 */
@Serializable
public data class PermissionOption(
    val optionId: PermissionOptionId,
    val name: String,
    val kind: PermissionOptionKind
)

/**
 * The outcome of a permission request.
 */
@Serializable
public sealed class RequestPermissionOutcome {
    /**
     * The prompt turn was cancelled before the user responded.
     */
    @Serializable
    @SerialName("cancelled")
    public object Cancelled : RequestPermissionOutcome()

    /**
     * The user selected one of the provided options.
     */
    @Serializable
    @SerialName("selected")
    public data class Selected(
        val optionId: PermissionOptionId
    ) : RequestPermissionOutcome()
}

// === Request Types ===

/**
 * Request parameters for the initialize method.
 *
 * Sent by the client to establish connection and negotiate capabilities.
 *
 * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
 */
@Serializable
public data class InitializeRequest(
    val protocolVersion: ProtocolVersion,
    val clientCapabilities: ClientCapabilities = ClientCapabilities()
)

/**
 * Request parameters for the authenticate method.
 *
 * Specifies which authentication method to use.
 */
@Serializable
public data class AuthenticateRequest(
    val methodId: AuthMethodId
)

/**
 * Request parameters for creating a new session.
 *
 * See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 */
@Serializable
public data class NewSessionRequest(
    val cwd: String,
    val mcpServers: List<McpServer>
)

/**
 * Request parameters for loading an existing session.
 *
 * Only available if the agent supports the `loadSession` capability.
 *
 * See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
 */
@Serializable
public data class LoadSessionRequest(
    val sessionId: SessionId,
    val cwd: String,
    val mcpServers: List<McpServer>
)

/**
 * Request parameters for sending a user prompt to the agent.
 *
 * Contains the user's message and any additional context.
 *
 * See protocol docs: [User Message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
 */
@Serializable
public data class PromptRequest(
    val sessionId: SessionId,
    val prompt: List<ContentBlock>
)

/**
 * Request to read content from a text file.
 *
 * Only available if the client supports the `fs.readTextFile` capability.
 */
@Serializable
public data class ReadTextFileRequest(
    val sessionId: SessionId,
    val path: String,
    val line: UInt? = null,
    val limit: UInt? = null
)

/**
 * Request to write content to a text file.
 *
 * Only available if the client supports the `fs.writeTextFile` capability.
 */
@Serializable
public data class WriteTextFileRequest(
    val sessionId: SessionId,
    val path: String,
    val content: String
)

/**
 * Request for user permission to execute a tool call.
 *
 * Sent when the agent needs authorization before performing a sensitive operation.
 *
 * See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
 */
@Serializable
public data class RequestPermissionRequest(
    val sessionId: SessionId,
    val toolCall: ToolCallUpdate,
    val options: List<PermissionOption>
)

/**
 * Request parameters for setting a session mode.
 */
@Serializable
public data class SetSessionModeRequest(
    val sessionId: SessionId,
    val modeId: SessionModeId
)

/**
 * **UNSTABLE**
 *
 * This capability is not part of the spec yet, and may be removed or changed at any point.
 *
 * Request parameters for setting a session model.
 */
@Serializable
public data class SetSessionModelRequest(
    val sessionId: SessionId,
    val modelId: ModelId
)

// === Response Types ===

/**
 * Response from the initialize method.
 *
 * Contains the negotiated protocol version and agent capabilities.
 *
 * See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
 */
@Serializable
public data class InitializeResponse(
    val protocolVersion: ProtocolVersion,
    val agentCapabilities: AgentCapabilities = AgentCapabilities(),
    val authMethods: List<AuthMethod> = emptyList()
)

/**
 * A mode the agent can operate in.
 *
 * See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
 */
@Serializable
public data class SessionMode(
    val id: SessionModeId,
    val name: String,
    val description: String? = null
)

/**
 * The set of modes and the one currently active.
 */
@Serializable
public data class SessionModeState(
    val currentModeId: SessionModeId,
    val availableModes: List<SessionMode>
)

/**
 * **UNSTABLE**
 *
 * This capability is not part of the spec yet, and may be removed or changed at any point.
 *
 * Information about a selectable model.
 */
@Serializable
public data class ModelInfo(
    val modelId: ModelId,
    val name: String,
    val description: String? = null
)

/**
 * **UNSTABLE**
 *
 * This capability is not part of the spec yet, and may be removed or changed at any point.
 *
 * The set of models and the one currently active.
 */
@Serializable
public data class SessionModelState(
    val currentModelId: ModelId,
    val availableModels: List<ModelInfo>
)

/**
 * Response from creating a new session.
 *
 * See protocol docs: [Creating a Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 */
@Serializable
public data class NewSessionResponse(
    val sessionId: SessionId,
    val modes: SessionModeState? = null,
    val models: SessionModelState? = null
)

/**
 * Response from processing a user prompt.
 *
 * See protocol docs: [Check for Completion](https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion)
 */
@Serializable
public data class PromptResponse(
    val stopReason: StopReason
)

/**
 * Response from loading an existing session.
 */
@Serializable
public data class LoadSessionResponse(
    val modes: SessionModeState? = null,
    val models: SessionModelState? = null
)

/**
 * Response containing the contents of a text file.
 */
@Serializable
public data class ReadTextFileResponse(
    val content: String
)

/**
 * Response to a permission request.
 */
@Serializable
public data class RequestPermissionResponse(
    val outcome: RequestPermissionOutcome
)

/**
 * Response to authenticate method
 */
@Serializable
public class AuthenticateResponse

/**
 * Response to `session/set_mode` method.
 */
@Serializable
public class SetSessionModeResponse

/**
 * **UNSTABLE**
 *
 * This capability is not part of the spec yet, and may be removed or changed at any point.
 *
 * Response to `session/set_model` method.
 */
@Serializable
public class SetSessionModelResponse

/**
 * Response to `fs/write_text_file`
 */
@Serializable
public class WriteTextFileResponse

/**
 * Response to terminal/release method
 */
@Serializable
public class ReleaseTerminalResponse

// === Notification Types ===

/**
 * Notification to cancel ongoing operations for a session.
 *
 * See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
 */
@Serializable
public data class CancelNotification(
    val sessionId: SessionId
)

/**
 * Notification containing a session update from the agent.
 *
 * Used to stream real-time progress and results during prompt processing.
 *
 * See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
 */
@Serializable
public data class SessionNotification(
    val sessionId: SessionId,
    val update: SessionUpdate
)