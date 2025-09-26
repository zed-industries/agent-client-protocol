@file:Suppress("unused")
@file:OptIn(ExperimentalSerializationApi::class)

package io.agentclientprotocol.model

import kotlinx.serialization.EncodeDefault
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.Serializable

/**
 * File system capabilities that a client may support.
 *
 * See protocol docs: [FileSystem](https://agentclientprotocol.com/protocol/initialization#filesystem)
 */
@Serializable
public data class FileSystemCapability(
    @EncodeDefault val readTextFile: Boolean = false,
    @EncodeDefault val writeTextFile: Boolean = false
)

/**
 * Prompt capabilities supported by the agent in `session/prompt` requests.
 *
 * Baseline agent functionality requires support for text and resource links in prompt requests.
 * Other variants must be explicitly opted in to.
 *
 * See protocol docs: [Prompt Capabilities](https://agentclientprotocol.com/protocol/initialization#prompt-capabilities)
 */
@Serializable
public data class PromptCapabilities(
    @EncodeDefault val audio: Boolean = false,
    @EncodeDefault val image: Boolean = false,
    @EncodeDefault val embeddedContext: Boolean = false
)

/**
 * Capabilities supported by the client.
 *
 * Advertised during initialization to inform the agent about
 * available features and methods.
 *
 * See protocol docs: [Client Capabilities](https://agentclientprotocol.com/protocol/initialization#client-capabilities)
 */
@Serializable
public data class ClientCapabilities(
    @EncodeDefault val fs: FileSystemCapability = FileSystemCapability(),
    @EncodeDefault val terminal: Boolean = false
)

/**
 * MCP capabilities supported by the agent
 */
@Serializable
public data class McpCapabilities(
    @EncodeDefault val http: Boolean = false,
    @EncodeDefault val sse: Boolean = false
)

/**
 * Capabilities supported by the agent.
 *
 * Advertised during initialization to inform the client about
 * available features and content types.
 *
 * See protocol docs: [Agent Capabilities](https://agentclientprotocol.com/protocol/initialization#agent-capabilities)
 */
@Serializable
public data class AgentCapabilities(
    @EncodeDefault val loadSession: Boolean = false,
    @EncodeDefault val promptCapabilities: PromptCapabilities = PromptCapabilities(),
    @EncodeDefault val mcpCapabilities: McpCapabilities = McpCapabilities()
)