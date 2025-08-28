@file:Suppress("unused")

package io.agentclientprotocol.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

/**
 * Categories of tools that can be invoked.
 *
 * Tool kinds help clients choose appropriate icons and optimize how they
 * display tool execution progress.
 *
 * See protocol docs: [Creating](https://agentclientprotocol.com/protocol/tool-calls#creating)
 */
@Serializable
public enum class ToolKind {
    @SerialName("read") READ,
    @SerialName("edit") EDIT,
    @SerialName("delete") DELETE,
    @SerialName("move") MOVE,
    @SerialName("search") SEARCH,
    @SerialName("execute") EXECUTE,
    @SerialName("think") THINK,
    @SerialName("fetch") FETCH,
    @SerialName("switch_mode") SWITCH_MODE,
    @SerialName("other") OTHER
}

/**
 * Execution status of a tool call.
 *
 * Tool calls progress through different statuses during their lifecycle.
 *
 * See protocol docs: [Status](https://agentclientprotocol.com/protocol/tool-calls#status)
 */
@Serializable
public enum class ToolCallStatus {
    @SerialName("pending") PENDING,
    @SerialName("in_progress") IN_PROGRESS,
    @SerialName("completed") COMPLETED,
    @SerialName("failed") FAILED
}

/**
 * A file location being accessed or modified by a tool.
 *
 * Enables clients to implement "follow-along" features that track
 * which files the agent is working with in real-time.
 *
 * See protocol docs: [Following the Agent](https://agentclientprotocol.com/protocol/tool-calls#following-the-agent)
 */
@Serializable
public data class ToolCallLocation(
    val path: String,
    val line: UInt? = null
)

/**
 * Content produced by a tool call.
 *
 * Tool calls can produce different types of content including
 * standard content blocks (text, images) or file diffs.
 *
 * See protocol docs: [Content](https://agentclientprotocol.com/protocol/tool-calls#content)
 */
@Serializable
public sealed class ToolCallContent {
    /**
     * Standard content block (text, images, resources).
     */
    @Serializable
    @SerialName("content")
    public data class Content(
        val content: ContentBlock
    ) : ToolCallContent()

    /**
     * File modification shown as a diff.
     */
    @Serializable
    @SerialName("diff")
    public data class Diff(
        val path: String,
        val newText: String,
        val oldText: String? = null
    ) : ToolCallContent()

    /**
     * Terminal output reference.
     */
    @Serializable
    @SerialName("terminal")
    public data class Terminal(
        val terminalId: String
    ) : ToolCallContent()
}

/**
 * Represents a tool call that the language model has requested.
 *
 * Tool calls are actions that the agent executes on behalf of the language model,
 * such as reading files, executing code, or fetching data from external sources.
 *
 * See protocol docs: [Tool Calls](https://agentclientprotocol.com/protocol/tool-calls)
 */
@Serializable
public data class ToolCall(
    val toolCallId: ToolCallId,
    val title: String,
    val kind: ToolKind? = null,
    val status: ToolCallStatus? = null,
    val content: List<ToolCallContent> = emptyList(),
    val locations: List<ToolCallLocation> = emptyList(),
    val rawInput: JsonElement? = null,
    val rawOutput: JsonElement? = null
)

/**
 * An update to an existing tool call.
 *
 * Used to report progress and results as tools execute. All fields except
 * the tool call ID are optional - only changed fields need to be included.
 *
 * See protocol docs: [Updating](https://agentclientprotocol.com/protocol/tool-calls#updating)
 */
@Serializable
public data class ToolCallUpdate(
    val toolCallId: ToolCallId,
    val title: String? = null,
    val kind: ToolKind? = null,
    val status: ToolCallStatus? = null,
    val content: List<ToolCallContent>? = null,
    val locations: List<ToolCallLocation>? = null,
    val rawInput: JsonElement? = null,
    val rawOutput: JsonElement? = null
)