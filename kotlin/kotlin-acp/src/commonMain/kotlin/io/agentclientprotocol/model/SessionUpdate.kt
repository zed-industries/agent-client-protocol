@file:Suppress("unused")
@file:OptIn(ExperimentalSerializationApi::class)

package io.agentclientprotocol.model

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonClassDiscriminator
import kotlinx.serialization.json.JsonElement

/**
 * Input specification for a command.
 */
@Serializable
public sealed class AvailableCommandInput {
    /**
     * All text that was typed after the command name is provided as input.
     */
    @Serializable
    @SerialName("UnstructuredCommandInput")
    public data class UnstructuredCommandInput(
        val hint: String
    ) : AvailableCommandInput()
}

/**
 * Information about a command.
 */
@Serializable
public data class AvailableCommand(
    val name: String,
    val description: String,
    val input: AvailableCommandInput? = null
)

/**
 * Different types of updates that can be sent during session processing.
 *
 * These updates provide real-time feedback about the agent's progress.
 *
 * See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
 */
@Serializable
@JsonClassDiscriminator("sessionUpdate")
public sealed class SessionUpdate {
    /**
     * A chunk of the user's message being streamed.
     */
    @Serializable
    @SerialName("user_message_chunk")
    public data class UserMessageChunk(
        val content: ContentBlock
    ) : SessionUpdate()

    /**
     * A chunk of the agent's response being streamed.
     */
    @Serializable
    @SerialName("agent_message_chunk")
    public data class AgentMessageChunk(
        val content: ContentBlock
    ) : SessionUpdate()

    /**
     * A chunk of the agent's internal reasoning being streamed.
     */
    @Serializable
    @SerialName("agent_thought_chunk")
    public data class AgentThoughtChunk(
        val content: ContentBlock
    ) : SessionUpdate()

    /**
     * Notification that a new tool call has been initiated.
     */
    @Serializable
    @SerialName("tool_call")
    public data class ToolCall(
        val toolCallId: ToolCallId,
        val title: String,
        val kind: ToolKind? = null,
        val status: ToolCallStatus? = null,
        val content: List<ToolCallContent> = emptyList(),
        val locations: List<ToolCallLocation> = emptyList(),
        val rawInput: JsonElement? = null,
        val rawOutput: JsonElement? = null
    ) : SessionUpdate()

    /**
     * Update on the status or results of a tool call.
     */
    @Serializable
    @SerialName("tool_call_update")
    public data class ToolCallUpdate(
        val toolCallId: ToolCallId,
        val title: String? = null,
        val kind: ToolKind? = null,
        val status: ToolCallStatus? = null,
        val content: List<ToolCallContent>? = null,
        val locations: List<ToolCallLocation>? = null,
        val rawInput: JsonElement? = null,
        val rawOutput: JsonElement? = null
    ) : SessionUpdate()

    /**
     * The agent's execution plan for complex tasks.
     *
     * See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
     */
    @Serializable
    @SerialName("plan")
    public data class PlanUpdate(
        val entries: List<PlanEntry>
    ) : SessionUpdate()

    /**
     * Available commands are ready or have changed
     */
    @Serializable
    @SerialName("available_commands_update")
    public data class AvailableCommandsUpdate(
        val availableCommands: List<AvailableCommand>
    ) : SessionUpdate()

    /**
     * The current mode of the session has changed
     *
     * See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
     */
    @Serializable
    @SerialName("current_mode_update")
    public data class CurrentModeUpdate(
        val currentModeId: SessionModeId
    ) : SessionUpdate()
}