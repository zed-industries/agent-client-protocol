// To parse the JSON, install kotlin's serialization plugin and do:
//
// val json         = Json { allowStructuredMapKeys = true }
// val aCPAggregate = json.parse(ACPAggregate.serializer(), jsonString)

package schema

import kotlinx.serialization.*
import kotlinx.serialization.json.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*

@Serializable
data class ACPAggregate (
    @SerialName("AgentCapabilities")
    val agentCapabilities: AgentCapabilities? = null,

    @SerialName("AgentNotification")
    val agentNotification: SessionNotification? = null,

    @SerialName("AgentRequest")
    val agentRequest: AgentRequest? = null,

    @SerialName("AgentResponse")
    val agentResponse: AgentResponseClass? = null,

    @SerialName("Annotations")
    val annotations: AnnotationsClass? = null,

    @SerialName("AudioContent")
    val audioContent: AudioContent? = null,

    @SerialName("AuthenticateRequest")
    val authenticateRequest: AuthenticateRequest? = null,

    @SerialName("AuthMethod")
    val authMethod: AuthMethodElement? = null,

    @SerialName("AuthMethodId")
    val authMethodID: String? = null,

    @SerialName("AvailableCommand")
    val availableCommand: AvailableCommandElement? = null,

    @SerialName("AvailableCommandInput")
    val availableCommandInput: UnstructuredCommandInput? = null,

    @SerialName("BlobResourceContents")
    val blobResourceContents: BlobResourceContents? = null,

    @SerialName("CancelNotification")
    val cancelNotification: CancelNotification? = null,

    @SerialName("ClientCapabilities")
    val clientCapabilities: ClientCapabilities? = null,

    @SerialName("ClientNotification")
    val clientNotification: CancelNotification? = null,

    @SerialName("ClientRequest")
    val clientRequest: ClientRequest? = null,

    @SerialName("ClientResponse")
    val clientResponse: ClientResponseClass? = null,

    @SerialName("ContentBlock")
    val contentBlock: ContentBlockElement? = null,

    @SerialName("CreateTerminalRequest")
    val createTerminalRequest: CreateTerminalRequest? = null,

    @SerialName("CreateTerminalResponse")
    val createTerminalResponse: CreateTerminalResponse? = null,

    @SerialName("EmbeddedResource")
    val embeddedResource: EmbeddedResource? = null,

    @SerialName("EmbeddedResourceResource")
    val embeddedResourceResource: Resource? = null,

    @SerialName("EnvVariable")
    val envVariable: EnvVariableElement? = null,

    @SerialName("FileSystemCapability")
    val fileSystemCapability: FS? = null,

    @SerialName("ImageContent")
    val imageContent: ImageContent? = null,

    @SerialName("InitializeRequest")
    val initializeRequest: InitializeRequest? = null,

    @SerialName("InitializeResponse")
    val initializeResponse: InitializeResponse? = null,

    @SerialName("KillTerminalRequest")
    val killTerminalRequest: KillTerminalRequest? = null,

    @SerialName("LoadSessionRequest")
    val loadSessionRequest: LoadSessionRequest? = null,

    @SerialName("McpServer")
    val mcpServer: MCPServerElement? = null,

    @SerialName("NewSessionRequest")
    val newSessionRequest: NewSessionRequest? = null,

    @SerialName("NewSessionResponse")
    val newSessionResponse: NewSessionResponse? = null,

    @SerialName("PermissionOption")
    val permissionOption: PermissionOptionElement? = null,

    @SerialName("PermissionOptionId")
    val permissionOptionID: String? = null,

    @SerialName("PermissionOptionKind")
    val permissionOptionKind: PermissionOptionKindEnum? = null,

    @SerialName("Plan")
    val plan: Plan? = null,

    @SerialName("PlanEntry")
    val planEntry: PlanEntryElement? = null,

    @SerialName("PlanEntryPriority")
    val planEntryPriority: Priority? = null,

    @SerialName("PlanEntryStatus")
    val planEntryStatus: PlanEntryStatusEnum? = null,

    @SerialName("PromptCapabilities")
    val promptCapabilities: PromptCapabilities? = null,

    @SerialName("PromptRequest")
    val promptRequest: PromptRequest? = null,

    @SerialName("PromptResponse")
    val promptResponse: PromptResponse? = null,

    @SerialName("ProtocolVersion")
    val protocolVersion: Long? = null,

    @SerialName("ReadTextFileRequest")
    val readTextFileRequest: ReadTextFileRequest? = null,

    @SerialName("ReadTextFileResponse")
    val readTextFileResponse: ReadTextFileResponse? = null,

    @SerialName("ReleaseTerminalRequest")
    val releaseTerminalRequest: ReleaseTerminalRequest? = null,

    @SerialName("RequestPermissionOutcome")
    val requestPermissionOutcome: RequestPermissionOutcomeClass? = null,

    @SerialName("RequestPermissionRequest")
    val requestPermissionRequest: RequestPermissionRequest? = null,

    @SerialName("RequestPermissionResponse")
    val requestPermissionResponse: RequestPermissionResponse? = null,

    @SerialName("ResourceLink")
    val resourceLink: ResourceLink? = null,

    @SerialName("Role")
    val role: RoleElement? = null,

    @SerialName("SessionId")
    val sessionID: String? = null,

    @SerialName("SessionNotification")
    val sessionNotification: SessionNotification? = null,

    @SerialName("SessionUpdate")
    val sessionUpdate: Update? = null,

    @SerialName("StopReason")
    val stopReason: StopReason? = null,

    @SerialName("TerminalExitStatus")
    val terminalExitStatus: TerminalExitStatusClass? = null,

    @SerialName("TerminalOutputRequest")
    val terminalOutputRequest: TerminalOutputRequest? = null,

    @SerialName("TerminalOutputResponse")
    val terminalOutputResponse: TerminalOutputResponse? = null,

    @SerialName("TextContent")
    val textContent: TextContent? = null,

    @SerialName("TextResourceContents")
    val textResourceContents: TextResourceContents? = null,

    @SerialName("ToolCall")
    val toolCall: ToolCallClass? = null,

    @SerialName("ToolCallContent")
    val toolCallContent: ToolCallContentElement? = null,

    @SerialName("ToolCallId")
    val toolCallID: String? = null,

    @SerialName("ToolCallLocation")
    val toolCallLocation: ToolCallLocationElement? = null,

    @SerialName("ToolCallStatus")
    val toolCallStatus: ToolCallStatusEnum? = null,

    @SerialName("ToolCallUpdate")
    val toolCallUpdate: ToolCallUpdateClass? = null,

    @SerialName("ToolKind")
    val toolKind: ToolKindEnum? = null,

    @SerialName("WaitForTerminalExitRequest")
    val waitForTerminalExitRequest: WaitForTerminalExitRequest? = null,

    @SerialName("WaitForTerminalExitResponse")
    val waitForTerminalExitResponse: WaitForTerminalExitResponse? = null,

    @SerialName("WriteTextFileRequest")
    val writeTextFileRequest: WriteTextFileRequest? = null
)

/**
 * Capabilities supported by the agent.
 *
 * Advertised during initialization to inform the client about
 * available features and content types.
 *
 * See protocol docs: [Agent
 * Capabilities](https://agentclientprotocol.com/protocol/initialization#agent-capabilities)
 *
 * Capabilities supported by the agent.
 */
@Serializable
data class AgentCapabilities (
    /**
     * Whether the agent supports `session/load`.
     */
    val loadSession: Boolean? = null,

    /**
     * Prompt capabilities supported by the agent.
     */
    val promptCapabilities: PromptCapabilities? = null
)

/**
 * Prompt capabilities supported by the agent.
 *
 * Prompt capabilities supported by the agent in `session/prompt` requests.
 *
 * Baseline agent functionality requires support for [`ContentBlock::Text`]
 * and [`ContentBlock::ResourceLink`] in prompt requests.
 *
 * Other variants must be explicitly opted in to.
 * Capabilities for different types of content in prompt requests.
 *
 * Indicates which content types beyond the baseline (text and resource links)
 * the agent can process.
 *
 * See protocol docs: [Prompt
 * Capabilities](https://agentclientprotocol.com/protocol/initialization#prompt-capabilities)
 */
@Serializable
data class PromptCapabilities (
    /**
     * Agent supports [`ContentBlock::Audio`].
     */
    val audio: Boolean? = null,

    /**
     * Agent supports embedded context in `session/prompt` requests.
     *
     * When enabled, the Client is allowed to include [`ContentBlock::Resource`]
     * in prompt requests for pieces of context that are referenced in the message.
     */
    val embeddedContext: Boolean? = null,

    /**
     * Agent supports [`ContentBlock::Image`].
     */
    val image: Boolean? = null
)

/**
 * All possible notifications that an agent can send to a client.
 *
 * This enum is used internally for routing RPC notifications. You typically won't need
 * to use this directly - use the notification methods on the [`Client`] trait instead.
 *
 * Notifications do not expect a response.
 *
 * Notification containing a session update from the agent.
 *
 * Used to stream real-time progress and results during prompt processing.
 *
 * See protocol docs: [Agent Reports
 * Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
 */
@Serializable
data class SessionNotification (
    /**
     * The ID of the session this update pertains to.
     */
    @SerialName("sessionId")
    val sessionID: String,

    /**
     * The actual update content.
     */
    val update: Update
)

/**
 * The actual update content.
 *
 * Different types of updates that can be sent during session processing.
 *
 * These updates provide real-time feedback about the agent's progress.
 *
 * See protocol docs: [Agent Reports
 * Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
 *
 * A chunk of the user's message being streamed.
 *
 * A chunk of the agent's response being streamed.
 *
 * A chunk of the agent's internal reasoning being streamed.
 *
 * Notification that a new tool call has been initiated.
 *
 * Update on the status or results of a tool call.
 *
 * The agent's execution plan for complex tasks.
 * See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
 *
 * Available commands are ready or have changed
 */
@Serializable
data class Update (
    /**
     * Content produced by the tool call.
     *
     * Replace the content collection.
     */
    val content: ContentUnion? = null,

    val sessionUpdate: SessionUpdate,

    /**
     * The category of tool being invoked.
     * Helps clients choose appropriate icons and UI treatment.
     *
     * Update the tool kind.
     */
    val kind: ToolKindEnum? = null,

    /**
     * File locations affected by this tool call.
     * Enables "follow-along" features in clients.
     *
     * Replace the locations collection.
     */
    val locations: List<ToolCallLocationElement>? = null,

    /**
     * Raw input parameters sent to the tool.
     *
     * Update the raw input.
     */
    val rawInput: JsonElement? = null,

    /**
     * Raw output returned by the tool.
     *
     * Update the raw output.
     */
    val rawOutput: JsonElement? = null,

    /**
     * Current execution status of the tool call.
     *
     * Update the execution status.
     */
    val status: ToolCallStatusEnum? = null,

    /**
     * Human-readable title describing what the tool is doing.
     *
     * Update the human-readable title.
     */
    val title: String? = null,

    /**
     * Unique identifier for this tool call within the session.
     *
     * The ID of the tool call being updated.
     */
    @SerialName("toolCallId")
    val toolCallID: String? = null,

    /**
     * The list of tasks to be accomplished.
     *
     * When updating a plan, the agent must send a complete list of all entries
     * with their current status. The client replaces the entire plan with each update.
     */
    val entries: List<PlanEntryElement>? = null,

    val availableCommands: List<AvailableCommandElement>? = null
)

/**
 * Information about a command.
 */
@Serializable
data class AvailableCommandElement (
    /**
     * Human-readable description of what the command does.
     */
    val description: String,

    /**
     * Input for the command if required
     */
    val input: UnstructuredCommandInput? = null,

    /**
     * Command name (e.g., "create_plan", "research_codebase").
     */
    val name: String
)

/**
 * All text that was typed after the command name is provided as input.
 */
@Serializable
data class UnstructuredCommandInput (
    /**
     * A brief description of the expected input
     */
    val hint: String
)

@Serializable
sealed class ContentUnion {
    class ContentBlockElementValue(val value: ContentBlockElement)                  : ContentUnion()
    class ToolCallContentElementArrayValue(val value: List<ToolCallContentElement>) : ContentUnion()
    class NullValue()                                                               : ContentUnion()
}

/**
 * Content produced by a tool call.
 *
 * Tool calls can produce different types of content including
 * standard content blocks (text, images) or file diffs.
 *
 * See protocol docs:
 * [Content](https://agentclientprotocol.com/protocol/tool-calls#content)
 *
 * Standard content block (text, images, resources).
 *
 * File modification shown as a diff.
 */
@Serializable
data class ToolCallContentElement (
    /**
     * The actual content block.
     */
    val content: ContentBlockElement? = null,

    val type: ToolCallContentType,

    /**
     * The new content after modification.
     */
    val newText: String? = null,

    /**
     * The original content (None for new files).
     */
    val oldText: String? = null,

    /**
     * The file path being modified.
     */
    val path: String? = null,

    @SerialName("terminalId")
    val terminalID: String? = null
)

/**
 * Content blocks represent displayable information in the Agent Client Protocol.
 *
 * They provide a structured way to handle various types of user-facing content—whether
 * it's text from language models, images for analysis, or embedded resources for context.
 *
 * Content blocks appear in:
 * - User prompts sent via `session/prompt`
 * - Language model output streamed through `session/update` notifications
 * - Progress updates and results from tool calls
 *
 * This structure is compatible with the Model Context Protocol (MCP), enabling
 * agents to seamlessly forward content from MCP tool outputs without transformation.
 *
 * See protocol docs: [Content](https://agentclientprotocol.com/protocol/content)
 *
 * The actual content block.
 *
 * Plain text content
 *
 * All agents MUST support text content blocks in prompts.
 *
 * Images for visual context or analysis.
 *
 * Requires the `image` prompt capability when included in prompts.
 *
 * Audio data for transcription or analysis.
 *
 * Requires the `audio` prompt capability when included in prompts.
 *
 * References to resources that the agent can access.
 *
 * All agents MUST support resource links in prompts.
 *
 * Complete resource contents embedded directly in the message.
 *
 * Preferred for including context as it avoids extra round-trips.
 *
 * Requires the `embeddedContext` prompt capability when included in prompts.
 */
@Serializable
data class ContentBlockElement (
    val annotations: AnnotationsClass? = null,
    val text: String? = null,
    val type: ContentBlockType,
    val data: String? = null,
    val mimeType: String? = null,
    val uri: String? = null,
    val description: String? = null,
    val name: String? = null,
    val size: Long? = null,
    val title: String? = null,
    val resource: Resource? = null
)

/**
 * Optional annotations for the client. The client can use annotations to inform how objects
 * are used or displayed
 */
@Serializable
data class AnnotationsClass (
    val audience: List<RoleElement>? = null,
    val lastModified: String? = null,
    val priority: Double? = null
)

/**
 * The sender or recipient of messages and data in a conversation.
 */
@Serializable
enum class RoleElement(val value: String) {
    @SerialName("assistant") Assistant("assistant"),
    @SerialName("user") User("user");
}

/**
 * Resource content that can be embedded in a message.
 *
 * Text-based resource contents.
 *
 * Binary resource contents.
 */
@Serializable
data class Resource (
    val mimeType: String? = null,
    val text: String? = null,
    val uri: String,
    val blob: String? = null
)

@Serializable
enum class ContentBlockType(val value: String) {
    @SerialName("audio") Audio("audio"),
    @SerialName("image") Image("image"),
    @SerialName("resource") Resource("resource"),
    @SerialName("resource_link") ResourceLink("resource_link"),
    @SerialName("text") Text("text");
}

@Serializable
enum class ToolCallContentType(val value: String) {
    @SerialName("content") Content("content"),
    @SerialName("diff") Diff("diff"),
    @SerialName("terminal") Terminal("terminal");
}

/**
 * A single entry in the execution plan.
 *
 * Represents a task or goal that the assistant intends to accomplish
 * as part of fulfilling the user's request.
 * See protocol docs: [Plan
 * Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
 */
@Serializable
data class PlanEntryElement (
    /**
     * Human-readable description of what this task aims to accomplish.
     */
    val content: String,

    /**
     * The relative importance of this task.
     * Used to indicate which tasks are most critical to the overall goal.
     */
    val priority: Priority,

    /**
     * Current execution status of this task.
     */
    val status: PlanEntryStatusEnum
)

/**
 * The relative importance of this task.
 * Used to indicate which tasks are most critical to the overall goal.
 *
 * Priority levels for plan entries.
 *
 * Used to indicate the relative importance or urgency of different
 * tasks in the execution plan.
 * See protocol docs: [Plan
 * Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
 *
 * High priority task - critical to the overall goal.
 *
 * Medium priority task - important but not critical.
 *
 * Low priority task - nice to have but not essential.
 */
@Serializable
enum class Priority(val value: String) {
    @SerialName("high") High("high"),
    @SerialName("low") Low("low"),
    @SerialName("medium") Medium("medium");
}

/**
 * Current execution status of this task.
 *
 * Status of a plan entry in the execution flow.
 *
 * Tracks the lifecycle of each task from planning through completion.
 * See protocol docs: [Plan
 * Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
 *
 * The task has not started yet.
 *
 * The task is currently being worked on.
 *
 * The task has been successfully completed.
 */
@Serializable
enum class PlanEntryStatusEnum(val value: String) {
    @SerialName("completed") Completed("completed"),
    @SerialName("in_progress") InProgress("in_progress"),
    @SerialName("pending") Pending("pending");
}

/**
 * The category of tool being invoked.
 * Helps clients choose appropriate icons and UI treatment.
 *
 * Categories of tools that can be invoked.
 *
 * Tool kinds help clients choose appropriate icons and optimize how they
 * display tool execution progress.
 *
 * See protocol docs:
 * [Creating](https://agentclientprotocol.com/protocol/tool-calls#creating)
 *
 * Reading files or data.
 *
 * Modifying files or content.
 *
 * Removing files or data.
 *
 * Moving or renaming files.
 *
 * Searching for information.
 *
 * Running commands or code.
 *
 * Internal reasoning or planning.
 *
 * Retrieving external data.
 *
 * Other tool types (default).
 */
@Serializable
enum class ToolKindEnum(val value: String) {
    @SerialName("delete") Delete("delete"),
    @SerialName("edit") Edit("edit"),
    @SerialName("execute") Execute("execute"),
    @SerialName("fetch") Fetch("fetch"),
    @SerialName("move") Move("move"),
    @SerialName("other") Other("other"),
    @SerialName("read") Read("read"),
    @SerialName("search") Search("search"),
    @SerialName("think") Think("think");
}

/**
 * A file location being accessed or modified by a tool.
 *
 * Enables clients to implement "follow-along" features that track
 * which files the agent is working with in real-time.
 *
 * See protocol docs: [Following the
 * Agent](https://agentclientprotocol.com/protocol/tool-calls#following-the-agent)
 */
@Serializable
data class ToolCallLocationElement (
    /**
     * Optional line number within the file.
     */
    val line: Long? = null,

    /**
     * The file path being accessed or modified.
     */
    val path: String
)

@Serializable
enum class SessionUpdate(val value: String) {
    @SerialName("agent_message_chunk") AgentMessageChunk("agent_message_chunk"),
    @SerialName("agent_thought_chunk") AgentThoughtChunk("agent_thought_chunk"),
    @SerialName("available_commands_update") AvailableCommandsUpdate("available_commands_update"),
    @SerialName("plan") Plan("plan"),
    @SerialName("tool_call") ToolCall("tool_call"),
    @SerialName("tool_call_update") ToolCallUpdate("tool_call_update"),
    @SerialName("user_message_chunk") UserMessageChunk("user_message_chunk");
}

/**
 * Current execution status of the tool call.
 *
 * Execution status of a tool call.
 *
 * Tool calls progress through different statuses during their lifecycle.
 *
 * See protocol docs: [Status](https://agentclientprotocol.com/protocol/tool-calls#status)
 *
 * The tool call hasn't started running yet because the input is either
 * streaming or we're awaiting approval.
 *
 * The tool call is currently running.
 *
 * The tool call completed successfully.
 *
 * The tool call failed with an error.
 */
@Serializable
enum class ToolCallStatusEnum(val value: String) {
    @SerialName("completed") Completed("completed"),
    @SerialName("failed") Failed("failed"),
    @SerialName("in_progress") InProgress("in_progress"),
    @SerialName("pending") Pending("pending");
}

/**
 * All possible requests that an agent can send to a client.
 *
 * This enum is used internally for routing RPC requests. You typically won't need
 * to use this directly - instead, use the methods on the [`Client`] trait.
 *
 * This enum encompasses all method calls from agent to client.
 *
 * Request to write content to a text file.
 *
 * Only available if the client supports the `fs.writeTextFile` capability.
 *
 * Request to read content from a text file.
 *
 * Only available if the client supports the `fs.readTextFile` capability.
 *
 * Request for user permission to execute a tool call.
 *
 * Sent when the agent needs authorization before performing a sensitive operation.
 *
 * See protocol docs: [Requesting
 * Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
 */
@Serializable
data class AgentRequest (
    /**
     * The text content to write to the file.
     */
    val content: String? = null,

    /**
     * Absolute path to the file to write.
     *
     * Absolute path to the file to read.
     */
    val path: String? = null,

    /**
     * The session ID for this request.
     */
    @SerialName("sessionId")
    val sessionID: String,

    /**
     * Optional maximum number of lines to read.
     */
    val limit: Long? = null,

    /**
     * Optional line number to start reading from (1-based).
     */
    val line: Long? = null,

    /**
     * Available permission options for the user to choose from.
     */
    val options: List<PermissionOptionElement>? = null,

    /**
     * Details about the tool call requiring permission.
     */
    val toolCall: ToolCallUpdateClass? = null,

    val args: List<String>? = null,
    val command: String? = null,
    val cwd: String? = null,
    val env: List<EnvVariableElement>? = null,
    val outputByteLimit: Long? = null,

    @SerialName("terminalId")
    val terminalID: String? = null
)

/**
 * An environment variable to set when launching an MCP server.
 */
@Serializable
data class EnvVariableElement (
    /**
     * The name of the environment variable.
     */
    val name: String,

    /**
     * The value to set for the environment variable.
     */
    val value: String
)

/**
 * An option presented to the user when requesting permission.
 */
@Serializable
data class PermissionOptionElement (
    /**
     * Hint about the nature of this permission option.
     */
    val kind: PermissionOptionKindEnum,

    /**
     * Human-readable label to display to the user.
     */
    val name: String,

    /**
     * Unique identifier for this permission option.
     */
    @SerialName("optionId")
    val optionID: String
)

/**
 * Hint about the nature of this permission option.
 *
 * The type of permission option being presented to the user.
 *
 * Helps clients choose appropriate icons and UI treatment.
 *
 * Allow this operation only this time.
 *
 * Allow this operation and remember the choice.
 *
 * Reject this operation only this time.
 *
 * Reject this operation and remember the choice.
 */
@Serializable
enum class PermissionOptionKindEnum(val value: String) {
    @SerialName("allow_always") AllowAlways("allow_always"),
    @SerialName("allow_once") AllowOnce("allow_once"),
    @SerialName("reject_always") RejectAlways("reject_always"),
    @SerialName("reject_once") RejectOnce("reject_once");
}

/**
 * Details about the tool call requiring permission.
 *
 * An update to an existing tool call.
 *
 * Used to report progress and results as tools execute. All fields except
 * the tool call ID are optional - only changed fields need to be included.
 *
 * See protocol docs:
 * [Updating](https://agentclientprotocol.com/protocol/tool-calls#updating)
 */
@Serializable
data class ToolCallUpdateClass (
    /**
     * Replace the content collection.
     */
    val content: List<ToolCallContentElement>? = null,

    /**
     * Update the tool kind.
     */
    val kind: ToolKindEnum? = null,

    /**
     * Replace the locations collection.
     */
    val locations: List<ToolCallLocationElement>? = null,

    /**
     * Update the raw input.
     */
    val rawInput: JsonElement? = null,

    /**
     * Update the raw output.
     */
    val rawOutput: JsonElement? = null,

    /**
     * Update the execution status.
     */
    val status: ToolCallStatusEnum? = null,

    /**
     * Update the human-readable title.
     */
    val title: String? = null,

    /**
     * The ID of the tool call being updated.
     */
    @SerialName("toolCallId")
    val toolCallID: String
)

/**
 * Response from the initialize method.
 *
 * Contains the negotiated protocol version and agent capabilities.
 *
 * See protocol docs:
 * [Initialization](https://agentclientprotocol.com/protocol/initialization)
 *
 * Response from creating a new session.
 *
 * See protocol docs: [Creating a
 * Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 *
 * Response from processing a user prompt.
 *
 * See protocol docs: [Check for
 * Completion](https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion)
 */
@Serializable
data class AgentResponseClass (
    /**
     * Capabilities supported by the agent.
     */
    val agentCapabilities: AgentCapabilities? = null,

    /**
     * Authentication methods supported by the agent.
     */
    val authMethods: List<AuthMethodElement>? = null,

    /**
     * The protocol version the client specified if supported by the agent,
     * or the latest protocol version supported by the agent.
     *
     * The client should disconnect, if it doesn't support this version.
     */
    val protocolVersion: Long? = null,

    /**
     * Unique identifier for the created session.
     *
     * Used in all subsequent requests for this conversation.
     */
    @SerialName("sessionId")
    val sessionID: String? = null,

    /**
     * Indicates why the agent stopped processing the turn.
     */
    val stopReason: StopReason? = null
)

/**
 * Describes an available authentication method.
 */
@Serializable
data class AuthMethodElement (
    /**
     * Optional description providing more details about this authentication method.
     */
    val description: String? = null,

    /**
     * Unique identifier for this authentication method.
     */
    val id: String,

    /**
     * Human-readable name of the authentication method.
     */
    val name: String
)

/**
 * Indicates why the agent stopped processing the turn.
 *
 * Reasons why an agent stops processing a prompt turn.
 *
 * See protocol docs: [Stop
 * Reasons](https://agentclientprotocol.com/protocol/prompt-turn#stop-reasons)
 *
 * The turn ended successfully.
 *
 * The turn ended because the agent reached the maximum number of tokens.
 *
 * The turn ended because the agent reached the maximum number of allowed
 * agent requests between user turns.
 *
 * The turn ended because the agent refused to continue. The user prompt
 * and everything that comes after it won't be included in the next
 * prompt, so this should be reflected in the UI.
 *
 * The turn was cancelled by the client via `session/cancel`.
 *
 * This stop reason MUST be returned when the client sends a `session/cancel`
 * notification, even if the cancellation causes exceptions in underlying operations.
 * Agents should catch these exceptions and return this semantically meaningful
 * response to confirm successful cancellation.
 */
@Serializable
enum class StopReason(val value: String) {
    @SerialName("cancelled") Cancelled("cancelled"),
    @SerialName("end_turn") EndTurn("end_turn"),
    @SerialName("max_tokens") MaxTokens("max_tokens"),
    @SerialName("max_turn_requests") MaxTurnRequests("max_turn_requests"),
    @SerialName("refusal") Refusal("refusal");
}

/**
 * Audio provided to or from an LLM.
 */
@Serializable
data class AudioContent (
    val annotations: AnnotationsClass? = null,
    val data: String,
    val mimeType: String
)

/**
 * Request parameters for the authenticate method.
 *
 * Specifies which authentication method to use.
 */
@Serializable
data class AuthenticateRequest (
    /**
     * The ID of the authentication method to use.
     * Must be one of the methods advertised in the initialize response.
     */
    @SerialName("methodId")
    val methodID: String
)

/**
 * Binary resource contents.
 */
@Serializable
data class BlobResourceContents (
    val blob: String,
    val mimeType: String? = null,
    val uri: String
)

/**
 * Notification to cancel ongoing operations for a session.
 *
 * See protocol docs:
 * [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
 *
 * All possible notifications that a client can send to an agent.
 *
 * This enum is used internally for routing RPC notifications. You typically won't need
 * to use this directly - use the notification methods on the [`Agent`] trait instead.
 *
 * Notifications do not expect a response.
 */
@Serializable
data class CancelNotification (
    /**
     * The ID of the session to cancel operations for.
     */
    @SerialName("sessionId")
    val sessionID: String
)

/**
 * Capabilities supported by the client.
 *
 * Advertised during initialization to inform the agent about
 * available features and methods.
 *
 * See protocol docs: [Client
 * Capabilities](https://agentclientprotocol.com/protocol/initialization#client-capabilities)
 *
 * Capabilities supported by the client.
 */
@Serializable
data class ClientCapabilities (
    /**
     * File system capabilities supported by the client.
     * Determines which file operations the agent can request.
     */
    val fs: FS? = null,

    /**
     * **UNSTABLE**
     *
     * This capability is not part of the spec yet, and may be removed or changed at any point.
     */
    val terminal: Boolean? = null
)

/**
 * File system capabilities supported by the client.
 * Determines which file operations the agent can request.
 *
 * File system capabilities that a client may support.
 *
 * See protocol docs:
 * [FileSystem](https://agentclientprotocol.com/protocol/initialization#filesystem)
 */
@Serializable
data class FS (
    /**
     * Whether the Client supports `fs/read_text_file` requests.
     */
    val readTextFile: Boolean? = null,

    /**
     * Whether the Client supports `fs/write_text_file` requests.
     */
    val writeTextFile: Boolean? = null
)

/**
 * All possible requests that a client can send to an agent.
 *
 * This enum is used internally for routing RPC requests. You typically won't need
 * to use this directly - instead, use the methods on the [`Agent`] trait.
 *
 * This enum encompasses all method calls from client to agent.
 *
 * Request parameters for the initialize method.
 *
 * Sent by the client to establish connection and negotiate capabilities.
 *
 * See protocol docs:
 * [Initialization](https://agentclientprotocol.com/protocol/initialization)
 *
 * Request parameters for the authenticate method.
 *
 * Specifies which authentication method to use.
 *
 * Request parameters for creating a new session.
 *
 * See protocol docs: [Creating a
 * Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 *
 * Request parameters for loading an existing session.
 *
 * Only available if the agent supports the `loadSession` capability.
 *
 * See protocol docs: [Loading
 * Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
 *
 * Request parameters for sending a user prompt to the agent.
 *
 * Contains the user's message and any additional context.
 *
 * See protocol docs: [User
 * Message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
 */
@Serializable
data class ClientRequest (
    /**
     * Capabilities supported by the client.
     */
    val clientCapabilities: ClientCapabilities? = null,

    /**
     * The latest protocol version supported by the client.
     */
    val protocolVersion: Long? = null,

    /**
     * The ID of the authentication method to use.
     * Must be one of the methods advertised in the initialize response.
     */
    @SerialName("methodId")
    val methodID: String? = null,

    /**
     * The working directory for this session. Must be an absolute path.
     *
     * The working directory for this session.
     */
    val cwd: String? = null,

    /**
     * List of MCP (Model Context Protocol) servers the agent should connect to.
     *
     * List of MCP servers to connect to for this session.
     */
    val mcpServers: List<MCPServerElement>? = null,

    /**
     * The ID of the session to load.
     *
     * The ID of the session to send this user message to
     */
    @SerialName("sessionId")
    val sessionID: String? = null,

    /**
     * The blocks of content that compose the user's message.
     *
     * As a baseline, the Agent MUST support [`ContentBlock::Text`] and
     * [`ContentBlock::ResourceLink`],
     * while other variants are optionally enabled via [`PromptCapabilities`].
     *
     * The Client MUST adapt its interface according to [`PromptCapabilities`].
     *
     * The client MAY include referenced pieces of context as either
     * [`ContentBlock::Resource`] or [`ContentBlock::ResourceLink`].
     *
     * When available, [`ContentBlock::Resource`] is preferred
     * as it avoids extra round-trips and allows the message to include
     * pieces of context from sources the agent may not have access to.
     */
    val prompt: List<ContentBlockElement>? = null
)

/**
 * Configuration for connecting to an MCP (Model Context Protocol) server.
 *
 * MCP servers provide tools and context that the agent can use when
 * processing prompts.
 *
 * See protocol docs: [MCP
 * Servers](https://agentclientprotocol.com/protocol/session-setup#mcp-servers)
 */
@Serializable
data class MCPServerElement (
    /**
     * Command-line arguments to pass to the MCP server.
     */
    val args: List<String>,

    /**
     * Path to the MCP server executable.
     */
    val command: String,

    /**
     * Environment variables to set when launching the MCP server.
     */
    val env: List<EnvVariableElement>,

    /**
     * Human-readable name identifying this MCP server.
     */
    val name: String
)

/**
 * Response containing the contents of a text file.
 *
 * Response to a permission request.
 */
@Serializable
data class ClientResponseClass (
    val content: String? = null,

    /**
     * The user's decision on the permission request.
     */
    val outcome: RequestPermissionOutcomeClass? = null,

    @SerialName("terminalId")
    val terminalID: String? = null,

    val exitStatus: TerminalExitStatusClass? = null,
    val output: String? = null,
    val truncated: Boolean? = null,
    val exitCode: Long? = null,
    val signal: String? = null
)

@Serializable
data class TerminalExitStatusClass (
    val exitCode: Long? = null,
    val signal: String? = null
)

/**
 * The user's decision on the permission request.
 *
 * The outcome of a permission request.
 *
 * The prompt turn was cancelled before the user responded.
 *
 * When a client sends a `session/cancel` notification to cancel an ongoing
 * prompt turn, it MUST respond to all pending `session/request_permission`
 * requests with this `Cancelled` outcome.
 *
 * See protocol docs:
 * [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
 *
 * The user selected one of the provided options.
 */
@Serializable
data class RequestPermissionOutcomeClass (
    val outcome: OutcomeEnum,

    /**
     * The ID of the option the user selected.
     */
    @SerialName("optionId")
    val optionID: String? = null
)

@Serializable
enum class OutcomeEnum(val value: String) {
    @SerialName("cancelled") Cancelled("cancelled"),
    @SerialName("selected") Selected("selected");
}

@Serializable
data class CreateTerminalRequest (
    val args: List<String>? = null,
    val command: String,
    val cwd: String? = null,
    val env: List<EnvVariableElement>? = null,
    val outputByteLimit: Long? = null,

    @SerialName("sessionId")
    val sessionID: String
)

@Serializable
data class CreateTerminalResponse (
    @SerialName("terminalId")
    val terminalID: String
)

/**
 * The contents of a resource, embedded into a prompt or tool call result.
 */
@Serializable
data class EmbeddedResource (
    val annotations: AnnotationsClass? = null,
    val resource: Resource
)

/**
 * An image provided to or from an LLM.
 */
@Serializable
data class ImageContent (
    val annotations: AnnotationsClass? = null,
    val data: String,
    val mimeType: String,
    val uri: String? = null
)

/**
 * Request parameters for the initialize method.
 *
 * Sent by the client to establish connection and negotiate capabilities.
 *
 * See protocol docs:
 * [Initialization](https://agentclientprotocol.com/protocol/initialization)
 */
@Serializable
data class InitializeRequest (
    /**
     * Capabilities supported by the client.
     */
    val clientCapabilities: ClientCapabilities? = null,

    /**
     * The latest protocol version supported by the client.
     */
    val protocolVersion: Long
)

/**
 * Response from the initialize method.
 *
 * Contains the negotiated protocol version and agent capabilities.
 *
 * See protocol docs:
 * [Initialization](https://agentclientprotocol.com/protocol/initialization)
 */
@Serializable
data class InitializeResponse (
    /**
     * Capabilities supported by the agent.
     */
    val agentCapabilities: AgentCapabilities? = null,

    /**
     * Authentication methods supported by the agent.
     */
    val authMethods: List<AuthMethodElement>? = null,

    /**
     * The protocol version the client specified if supported by the agent,
     * or the latest protocol version supported by the agent.
     *
     * The client should disconnect, if it doesn't support this version.
     */
    val protocolVersion: Long
)

@Serializable
data class KillTerminalRequest (
    @SerialName("sessionId")
    val sessionID: String,

    @SerialName("terminalId")
    val terminalID: String
)

/**
 * Request parameters for loading an existing session.
 *
 * Only available if the agent supports the `loadSession` capability.
 *
 * See protocol docs: [Loading
 * Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
 */
@Serializable
data class LoadSessionRequest (
    /**
     * The working directory for this session.
     */
    val cwd: String,

    /**
     * List of MCP servers to connect to for this session.
     */
    val mcpServers: List<MCPServerElement>,

    /**
     * The ID of the session to load.
     */
    @SerialName("sessionId")
    val sessionID: String
)

/**
 * Request parameters for creating a new session.
 *
 * See protocol docs: [Creating a
 * Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 */
@Serializable
data class NewSessionRequest (
    /**
     * The working directory for this session. Must be an absolute path.
     */
    val cwd: String,

    /**
     * List of MCP (Model Context Protocol) servers the agent should connect to.
     */
    val mcpServers: List<MCPServerElement>
)

/**
 * Response from creating a new session.
 *
 * See protocol docs: [Creating a
 * Session](https://agentclientprotocol.com/protocol/session-setup#creating-a-session)
 */
@Serializable
data class NewSessionResponse (
    /**
     * Unique identifier for the created session.
     *
     * Used in all subsequent requests for this conversation.
     */
    @SerialName("sessionId")
    val sessionID: String
)

/**
 * An execution plan for accomplishing complex tasks.
 *
 * Plans consist of multiple entries representing individual tasks or goals.
 * Agents report plans to clients to provide visibility into their execution strategy.
 * Plans can evolve during execution as the agent discovers new requirements or completes
 * tasks.
 *
 * See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
 */
@Serializable
data class Plan (
    /**
     * The list of tasks to be accomplished.
     *
     * When updating a plan, the agent must send a complete list of all entries
     * with their current status. The client replaces the entire plan with each update.
     */
    val entries: List<PlanEntryElement>
)

/**
 * Request parameters for sending a user prompt to the agent.
 *
 * Contains the user's message and any additional context.
 *
 * See protocol docs: [User
 * Message](https://agentclientprotocol.com/protocol/prompt-turn#1-user-message)
 */
@Serializable
data class PromptRequest (
    /**
     * The blocks of content that compose the user's message.
     *
     * As a baseline, the Agent MUST support [`ContentBlock::Text`] and
     * [`ContentBlock::ResourceLink`],
     * while other variants are optionally enabled via [`PromptCapabilities`].
     *
     * The Client MUST adapt its interface according to [`PromptCapabilities`].
     *
     * The client MAY include referenced pieces of context as either
     * [`ContentBlock::Resource`] or [`ContentBlock::ResourceLink`].
     *
     * When available, [`ContentBlock::Resource`] is preferred
     * as it avoids extra round-trips and allows the message to include
     * pieces of context from sources the agent may not have access to.
     */
    val prompt: List<ContentBlockElement>,

    /**
     * The ID of the session to send this user message to
     */
    @SerialName("sessionId")
    val sessionID: String
)

/**
 * Response from processing a user prompt.
 *
 * See protocol docs: [Check for
 * Completion](https://agentclientprotocol.com/protocol/prompt-turn#4-check-for-completion)
 */
@Serializable
data class PromptResponse (
    /**
     * Indicates why the agent stopped processing the turn.
     */
    val stopReason: StopReason
)

/**
 * Request to read content from a text file.
 *
 * Only available if the client supports the `fs.readTextFile` capability.
 */
@Serializable
data class ReadTextFileRequest (
    /**
     * Optional maximum number of lines to read.
     */
    val limit: Long? = null,

    /**
     * Optional line number to start reading from (1-based).
     */
    val line: Long? = null,

    /**
     * Absolute path to the file to read.
     */
    val path: String,

    /**
     * The session ID for this request.
     */
    @SerialName("sessionId")
    val sessionID: String
)

/**
 * Response containing the contents of a text file.
 */
@Serializable
data class ReadTextFileResponse (
    val content: String
)

@Serializable
data class ReleaseTerminalRequest (
    @SerialName("sessionId")
    val sessionID: String,

    @SerialName("terminalId")
    val terminalID: String
)

/**
 * Request for user permission to execute a tool call.
 *
 * Sent when the agent needs authorization before performing a sensitive operation.
 *
 * See protocol docs: [Requesting
 * Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
 */
@Serializable
data class RequestPermissionRequest (
    /**
     * Available permission options for the user to choose from.
     */
    val options: List<PermissionOptionElement>,

    /**
     * The session ID for this request.
     */
    @SerialName("sessionId")
    val sessionID: String,

    /**
     * Details about the tool call requiring permission.
     */
    val toolCall: ToolCallUpdateClass
)

/**
 * Response to a permission request.
 */
@Serializable
data class RequestPermissionResponse (
    /**
     * The user's decision on the permission request.
     */
    val outcome: RequestPermissionOutcomeClass
)

/**
 * A resource that the server is capable of reading, included in a prompt or tool call
 * result.
 */
@Serializable
data class ResourceLink (
    val annotations: AnnotationsClass? = null,
    val description: String? = null,
    val mimeType: String? = null,
    val name: String,
    val size: Long? = null,
    val title: String? = null,
    val uri: String
)

@Serializable
data class TerminalOutputRequest (
    @SerialName("sessionId")
    val sessionID: String,

    @SerialName("terminalId")
    val terminalID: String
)

@Serializable
data class TerminalOutputResponse (
    val exitStatus: TerminalExitStatusClass? = null,
    val output: String,
    val truncated: Boolean
)

/**
 * Text provided to or from an LLM.
 */
@Serializable
data class TextContent (
    val annotations: AnnotationsClass? = null,
    val text: String
)

/**
 * Text-based resource contents.
 */
@Serializable
data class TextResourceContents (
    val mimeType: String? = null,
    val text: String,
    val uri: String
)

/**
 * Represents a tool call that the language model has requested.
 *
 * Tool calls are actions that the agent executes on behalf of the language model,
 * such as reading files, executing code, or fetching data from external sources.
 *
 * See protocol docs: [Tool Calls](https://agentclientprotocol.com/protocol/tool-calls)
 */
@Serializable
data class ToolCallClass (
    /**
     * Content produced by the tool call.
     */
    val content: List<ToolCallContentElement>? = null,

    /**
     * The category of tool being invoked.
     * Helps clients choose appropriate icons and UI treatment.
     */
    val kind: ToolKindEnum? = null,

    /**
     * File locations affected by this tool call.
     * Enables "follow-along" features in clients.
     */
    val locations: List<ToolCallLocationElement>? = null,

    /**
     * Raw input parameters sent to the tool.
     */
    val rawInput: JsonElement? = null,

    /**
     * Raw output returned by the tool.
     */
    val rawOutput: JsonElement? = null,

    /**
     * Current execution status of the tool call.
     */
    val status: ToolCallStatusEnum? = null,

    /**
     * Human-readable title describing what the tool is doing.
     */
    val title: String,

    /**
     * Unique identifier for this tool call within the session.
     */
    @SerialName("toolCallId")
    val toolCallID: String
)

@Serializable
data class WaitForTerminalExitRequest (
    @SerialName("sessionId")
    val sessionID: String,

    @SerialName("terminalId")
    val terminalID: String
)

@Serializable
data class WaitForTerminalExitResponse (
    val exitCode: Long? = null,
    val signal: String? = null
)

/**
 * Request to write content to a text file.
 *
 * Only available if the client supports the `fs.writeTextFile` capability.
 */
@Serializable
data class WriteTextFileRequest (
    /**
     * The text content to write to the file.
     */
    val content: String,

    /**
     * Absolute path to the file to write.
     */
    val path: String,

    /**
     * The session ID for this request.
     */
    @SerialName("sessionId")
    val sessionID: String
)
