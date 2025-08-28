@file:Suppress("unused")

package io.agentclientprotocol.model

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.*
import kotlin.jvm.JvmInline

/**
 * Protocol version identifier.
 * 
 * This version is only bumped for breaking changes.
 * Non-breaking changes should be introduced via capabilities.
 */
public typealias ProtocolVersion = UShort

/**
 * The latest protocol version supported.
 */
public const val LATEST_PROTOCOL_VERSION: ProtocolVersion = 1u

/**
 * All supported protocol versions.
 */
public val SUPPORTED_PROTOCOL_VERSIONS: Array<ProtocolVersion> = arrayOf(LATEST_PROTOCOL_VERSION)

/**
 * A unique identifier for a conversation session between a client and agent.
 *
 * Sessions maintain their own context, conversation history, and state,
 * allowing multiple independent interactions with the same agent.
 */
@JvmInline
@Serializable
public value class SessionId(public val value: String) {
    override fun toString(): String = value
}

/**
 * Unique identifier for a tool call within a session.
 */
@JvmInline
@Serializable
public value class ToolCallId(public val value: String) {
    override fun toString(): String = value
}

/**
 * Unique identifier for an authentication method.
 */
@JvmInline
@Serializable
public value class AuthMethodId(public val value: String) {
    override fun toString(): String = value
}

/**
 * Unique identifier for a permission option.
 */
@JvmInline
@Serializable
public value class PermissionOptionId(public val value: String) {
    override fun toString(): String = value
}

/**
 * Unique identifier for a Session Mode.
 */
@JvmInline
@Serializable
public value class SessionModeId(public val value: String) {
    override fun toString(): String = value
}

/**
 * **UNSTABLE**
 *
 * This capability is not part of the spec yet, and may be removed or changed at any point.
 *
 * A unique identifier for a model.
 */
@JvmInline
@Serializable
public value class ModelId(public val value: String) {
    override fun toString(): String = value
}

/**
 * The sender or recipient of messages and data in a conversation.
 */
@Serializable
public enum class Role {
    @SerialName("assistant") ASSISTANT,
    @SerialName("user") USER
}

/**
 * Optional annotations for the client. The client can use annotations to inform how objects are used or displayed.
 */
@Serializable
public data class Annotations(
    val audience: List<Role>? = null,
    val priority: Double? = null,
    val lastModified: String? = null
)

@OptIn(ExperimentalSerializationApi::class)
public val ACPJson: Json by lazy {
    Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
        isLenient = true
        classDiscriminatorMode = ClassDiscriminatorMode.POLYMORPHIC
        explicitNulls = false
    }
}